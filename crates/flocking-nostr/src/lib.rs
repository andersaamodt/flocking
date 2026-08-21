#![forbid(unsafe_code)]
//! Nostr event translation for Flocking v1.

use std::collections::BTreeSet;

use flocking_core::{
    Action, ContentTarget, EventId, Faculty, Judgment, JudgmentEvidence, PublicKey, Scope, Target,
    Topic,
};
use nostr::{Event, EventBuilder, Kind, Tag, Timestamp};
use thiserror::Error;

pub use flocking_core::{JUDGMENT_KIND, PROTOCOL_VERSION};

/// A malformed, unsupported, or unauthentic Nostr boundary value.
#[derive(Debug, Error)]
pub enum Error {
    #[error("event kind is not a Flocking judgment")]
    WrongKind,
    #[error("event signature or ID is invalid")]
    InvalidSignature,
    #[error("required tag {0} is missing")]
    MissingTag(&'static str),
    #[error("semantic tag {0} is duplicated")]
    DuplicateTag(&'static str),
    #[error("tag {0} has no value")]
    EmptyTag(&'static str),
    #[error("Flocking protocol version is not supported")]
    UnknownVersion,
    #[error("Flocking faculty is not supported")]
    UnknownFaculty,
    #[error("Flocking action is not supported")]
    UnknownAction,
    #[error("Flocking scope form is not supported")]
    UnknownScope,
    #[error("judgment contains an invalid number of semantic targets")]
    InvalidTargetCount,
    #[error("global judgment contains a topic tag")]
    GlobalTopic,
    #[error("topic judgment requires exactly one topic tag")]
    TopicCount,
    #[error("silence cutoff is not an integer Unix timestamp")]
    InvalidCutoff,
    #[error("Nostr tag construction failed")]
    TagConstruction,
    #[error(transparent)]
    Core(#[from] flocking_core::Error),
}

/// Parses and authenticates one published Flocking judgment.
///
/// # Errors
///
/// Returns an error for an invalid signature, unsupported semantic tag, malformed
/// target, invalid tuple, or mismatched stable address.
pub fn parse_judgment(event: &Event) -> Result<Judgment, Error> {
    if event.kind.as_u16() != JUDGMENT_KIND {
        return Err(Error::WrongKind);
    }
    event.verify().map_err(|_| Error::InvalidSignature)?;
    let version = one(event, "v")?;
    if version != PROTOCOL_VERSION {
        return Err(Error::UnknownVersion);
    }
    let faculty = one(event, "f")?
        .parse::<Faculty>()
        .map_err(|_| Error::UnknownFaculty)?;
    let action = one(event, "j")?
        .parse::<Action>()
        .map_err(|_| Error::UnknownAction)?;
    let scope = match one(event, "c")? {
        "global" => {
            if count(event, "t") != 0 {
                return Err(Error::GlobalTopic);
            }
            Scope::Global
        }
        "topic" => {
            if count(event, "t") != 1 {
                return Err(Error::TopicCount);
            }
            Scope::Topic(Topic::parse(one(event, "t")?)?)
        }
        _ => return Err(Error::UnknownScope),
    };
    let target_tags = ["p", "e", "a"]
        .into_iter()
        .filter(|kind| count(event, kind) > 0)
        .collect::<Vec<_>>();
    if target_tags.len() != 1 || count(event, target_tags.first().copied().unwrap_or("p")) != 1 {
        return Err(Error::InvalidTargetCount);
    }
    let target = match target_tags[0] {
        "p" => Target::Person(PublicKey::parse(one(event, "p")?)?),
        "e" => Target::Content(ContentTarget::Event(EventId::parse(one(event, "e")?)?)),
        "a" => Target::Content(ContentTarget::address(one(event, "a")?)?),
        _ => unreachable!(),
    };
    let since = match count(event, "since") {
        0 => None,
        1 => Some(
            one(event, "since")?
                .parse::<u64>()
                .map_err(|_| Error::InvalidCutoff)?,
        ),
        _ => return Err(Error::DuplicateTag("since")),
    };
    let judgment = Judgment {
        author: PublicKey::parse(event.pubkey.to_string())?,
        faculty,
        scope,
        target,
        action,
        created_at: event.created_at.as_secs(),
        event_id: Some(EventId::parse(event.id.to_string())?),
        since,
        reason: (!event.content.is_empty()).then(|| event.content.clone()),
        evidence: JudgmentEvidence::FlockingEvent,
    };
    judgment.validate()?;
    if one(event, "d")? != judgment.address() {
        return Err(flocking_core::Error::AddressMismatch.into());
    }
    Ok(judgment)
}

/// Builds the unsigned semantic event that a matching author may sign.
///
/// # Errors
///
/// Returns an error when the judgment or a generated Nostr tag is invalid.
pub fn judgment_event_builder(judgment: &Judgment) -> Result<EventBuilder, Error> {
    judgment.validate()?;
    let mut tags = vec![
        tag(["d", judgment.address().as_str()])?,
        tag(["v", PROTOCOL_VERSION])?,
        tag(["f", judgment.faculty.to_string().as_str()])?,
        tag(["j", judgment.action.to_string().as_str()])?,
    ];
    match &judgment.scope {
        Scope::Global => tags.push(tag(["c", "global"])?),
        Scope::Topic(topic) => {
            tags.push(tag(["c", "topic"])?);
            tags.push(tag(["t", topic.as_str()])?);
        }
    }
    match &judgment.target {
        Target::Person(pubkey) => tags.push(tag(["p", pubkey.as_str()])?),
        Target::Content(ContentTarget::Event(id)) => tags.push(tag(["e", id.as_str()])?),
        Target::Content(ContentTarget::Address(coordinate)) => {
            tags.push(tag(["a", coordinate.as_str()])?);
        }
    }
    if let Some(since) = judgment.since {
        tags.push(tag(["since", since.to_string().as_str()])?);
    }
    Ok(EventBuilder::new(
        Kind::Custom(JUDGMENT_KIND),
        judgment.reason.clone().unwrap_or_default(),
    )
    .tags(tags)
    .custom_created_at(Timestamp::from_secs(judgment.created_at))
    .allow_self_tagging())
}

/// Adapts membership in one current NIP-02 kind-3 list to fallback follows.
///
/// # Errors
///
/// Returns an error for the wrong kind, invalid signature, or malformed public key.
pub fn nip02_fallback(event: &Event) -> Result<Vec<Judgment>, Error> {
    fallback_people(
        event,
        3,
        Faculty::Follow,
        Action::Follow,
        JudgmentEvidence::Nip02,
    )
}

/// Adapts membership in one current NIP-51 kind-10000 mute list to fallback blocks.
///
/// # Errors
///
/// Returns an error for the wrong kind, invalid signature, or malformed public key.
pub fn nip51_block_fallback(event: &Event) -> Result<Vec<Judgment>, Error> {
    fallback_people(
        event,
        10_000,
        Faculty::Block,
        Action::Block,
        JudgmentEvidence::Nip51,
    )
}

/// Selects the newest valid replaceable-list event, with lower ID as the tie-breaker.
#[must_use]
pub fn select_current_list(events: &[Event], kind: u16) -> Option<&Event> {
    events
        .iter()
        .filter(|event| event.kind.as_u16() == kind && event.verify().is_ok())
        .max_by(|left, right| {
            left.created_at.cmp(&right.created_at).then_with(|| {
                // Lower event ID wins when timestamps are equal.
                right.id.cmp(&left.id)
            })
        })
}

/// Projects authored positive global follows into a Flocking-managed NIP-02 set.
#[must_use]
pub fn follow_mirror(judgments: &[Judgment], author: &PublicKey) -> BTreeSet<PublicKey> {
    mirror_people(judgments, author, Faculty::Follow, Action::Follow)
}

/// Projects authored positive global blocks into a Flocking-managed NIP-51 set.
#[must_use]
pub fn block_mirror(judgments: &[Judgment], author: &PublicKey) -> BTreeSet<PublicKey> {
    mirror_people(judgments, author, Faculty::Block, Action::Block)
}

fn one<'a>(event: &'a Event, kind: &'static str) -> Result<&'a str, Error> {
    match event
        .tags
        .iter()
        .filter(|tag| tag.as_slice().first().is_some_and(|value| value == kind))
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Err(Error::MissingTag(kind)),
        [tag] => tag
            .as_slice()
            .get(1)
            .map(String::as_str)
            .ok_or(Error::EmptyTag(kind)),
        _ => Err(Error::DuplicateTag(kind)),
    }
}

fn count(event: &Event, kind: &str) -> usize {
    event
        .tags
        .iter()
        .filter(|tag| tag.as_slice().first().is_some_and(|value| value == kind))
        .count()
}

fn tag<const N: usize>(values: [&str; N]) -> Result<Tag, Error> {
    Tag::parse(values).map_err(|_| Error::TagConstruction)
}

fn fallback_people(
    event: &Event,
    expected_kind: u16,
    faculty: Faculty,
    action: Action,
    evidence: JudgmentEvidence,
) -> Result<Vec<Judgment>, Error> {
    if event.kind.as_u16() != expected_kind {
        return Err(Error::WrongKind);
    }
    event.verify().map_err(|_| Error::InvalidSignature)?;
    let author = PublicKey::parse(event.pubkey.to_string())?;
    let event_id = EventId::parse(event.id.to_string())?;
    let mut targets = BTreeSet::new();
    for tag in event
        .tags
        .iter()
        .filter(|tag| tag.as_slice().first().is_some_and(|value| value == "p"))
    {
        let value = tag.as_slice().get(1).ok_or(Error::EmptyTag("p"))?;
        targets.insert(PublicKey::parse(value)?);
    }
    targets
        .into_iter()
        .map(|target| {
            let judgment = Judgment {
                author: author.clone(),
                faculty,
                scope: Scope::Global,
                target: Target::Person(target),
                action,
                created_at: event.created_at.as_secs(),
                event_id: Some(event_id.clone()),
                since: None,
                reason: None,
                evidence,
            };
            judgment.validate()?;
            Ok(judgment)
        })
        .collect()
}

fn mirror_people(
    judgments: &[Judgment],
    author: &PublicKey,
    faculty: Faculty,
    positive: Action,
) -> BTreeSet<PublicKey> {
    flocking_core::canonical_current(judgments)
        .into_iter()
        .filter_map(|judgment| {
            if &judgment.author != author
                || judgment.faculty != faculty
                || judgment.scope != Scope::Global
                || judgment.action != positive
            {
                return None;
            }
            match judgment.target {
                Target::Person(target) => Some(target),
                Target::Content(_) => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::{Keys, Tag};

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn signed(judgment: &Judgment, keys: &Keys) -> Event {
        judgment_event_builder(judgment)
            .unwrap()
            .sign_with_keys(keys)
            .unwrap()
    }

    #[test]
    fn judgment_round_trip_preserves_semantics() {
        let keys = Keys::generate();
        let author = PublicKey::parse(keys.public_key().to_string()).unwrap();
        let judgment = Judgment {
            author,
            faculty: Faculty::Silence,
            scope: Scope::Topic(Topic::parse("science").unwrap()),
            target: Target::Person(key('1')),
            action: Action::Silence,
            created_at: 10,
            event_id: None,
            since: Some(10),
            reason: Some("noise".to_owned()),
            evidence: JudgmentEvidence::Local,
        };
        let event = signed(&judgment, &keys);
        let parsed = parse_judgment(&event).unwrap();
        assert_eq!(parsed.author, judgment.author);
        assert_eq!(parsed.faculty, judgment.faculty);
        assert_eq!(parsed.scope, judgment.scope);
        assert_eq!(parsed.target, judgment.target);
        assert_eq!(parsed.action, judgment.action);
        assert_eq!(parsed.since, judgment.since);
        assert_eq!(parsed.reason, judgment.reason);
    }

    #[test]
    fn rejects_duplicate_semantic_tags() {
        let keys = Keys::generate();
        let author = PublicKey::parse(keys.public_key().to_string()).unwrap();
        let judgment = Judgment {
            author,
            faculty: Faculty::Block,
            scope: Scope::Global,
            target: Target::Person(key('1')),
            action: Action::Block,
            created_at: 10,
            event_id: None,
            since: None,
            reason: None,
            evidence: JudgmentEvidence::Local,
        };
        let event = judgment_event_builder(&judgment)
            .unwrap()
            .tag(Tag::parse(["j", "unblock"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        assert!(matches!(
            parse_judgment(&event),
            Err(Error::DuplicateTag("j"))
        ));
    }

    #[test]
    fn fallback_absence_creates_no_negative_judgment() {
        let keys = Keys::generate();
        let event = EventBuilder::new(Kind::Custom(3), "")
            .custom_created_at(Timestamp::from_secs(10))
            .sign_with_keys(&keys)
            .unwrap();
        assert!(nip02_fallback(&event).unwrap().is_empty());
    }

    #[test]
    fn fallback_deduplicates_people() {
        let keys = Keys::generate();
        let person = key('1');
        let event = EventBuilder::new(Kind::Custom(10_000), "")
            .tags([
                Tag::parse(["p", person.as_str()]).unwrap(),
                Tag::parse(["p", person.as_str(), "wss://relay.example"]).unwrap(),
            ])
            .custom_created_at(Timestamp::from_secs(10))
            .sign_with_keys(&keys)
            .unwrap();
        assert_eq!(nip51_block_fallback(&event).unwrap().len(), 1);
    }

    #[test]
    fn mirrors_only_current_positive_global_state() {
        let author = key('0');
        let mut follow = Judgment {
            author: author.clone(),
            faculty: Faculty::Follow,
            scope: Scope::Global,
            target: Target::Person(key('1')),
            action: Action::Follow,
            created_at: 1,
            event_id: None,
            since: None,
            reason: None,
            evidence: JudgmentEvidence::Local,
        };
        let mut unfollow = follow.clone();
        unfollow.action = Action::Unfollow;
        unfollow.created_at = 2;
        assert!(follow_mirror(&[follow.clone(), unfollow], &author).is_empty());
        follow.target = Target::Person(key('2'));
        assert_eq!(
            follow_mirror(&[follow], &author),
            BTreeSet::from([key('2')])
        );
    }

    #[test]
    fn rejects_a_validly_signed_event_with_wrong_stable_address() {
        let keys = Keys::generate();
        let event = EventBuilder::new(Kind::Custom(JUDGMENT_KIND), "")
            .tags([
                Tag::parse(["d", format!("flocking:v1:{}", "0".repeat(64)).as_str()]).unwrap(),
                Tag::parse(["v", PROTOCOL_VERSION]).unwrap(),
                Tag::parse(["f", "block"]).unwrap(),
                Tag::parse(["j", "block"]).unwrap(),
                Tag::parse(["c", "global"]).unwrap(),
                Tag::parse(["p", key('1').as_str()]).unwrap(),
            ])
            .custom_created_at(Timestamp::from_secs(10))
            .sign_with_keys(&keys)
            .unwrap();
        assert!(matches!(
            parse_judgment(&event),
            Err(Error::Core(flocking_core::Error::AddressMismatch))
        ));
    }

    #[test]
    fn unknown_nonsemantic_tag_does_not_change_meaning() {
        let keys = Keys::generate();
        let author = PublicKey::parse(keys.public_key().to_string()).unwrap();
        let judgment = Judgment {
            author,
            faculty: Faculty::Block,
            scope: Scope::Global,
            target: Target::Person(key('1')),
            action: Action::Block,
            created_at: 10,
            event_id: None,
            since: None,
            reason: None,
            evidence: JudgmentEvidence::Local,
        };
        let event = judgment_event_builder(&judgment)
            .unwrap()
            .tag(Tag::parse(["client", "hydra"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        assert_eq!(parse_judgment(&event).unwrap().action, Action::Block);
    }
}
