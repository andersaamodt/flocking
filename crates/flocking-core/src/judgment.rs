use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Action, Error, EventId, Faculty, PublicKey, Scope, Target};

/// One validated authored Flocking position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Judgment {
    pub author: PublicKey,
    pub faculty: Faculty,
    pub scope: Scope,
    pub target: Target,
    pub action: Action,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub evidence: JudgmentEvidence,
}

/// The boundary that supplied a judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentEvidence {
    #[default]
    Local,
    FlockingEvent,
    Nip02,
    Nip51,
}

impl Judgment {
    /// Validates faculty, scope, target, action, cutoff, and public reason bounds.
    ///
    /// # Errors
    ///
    /// Returns an error for any noncanonical or unsupported v1 combination.
    pub fn validate(&self) -> Result<(), Error> {
        validate_question(self.faculty, &self.scope, &self.target)?;
        validate_action(self.faculty, self.action)?;
        match (self.action, self.since) {
            (Action::Silence, None) => return Err(Error::MissingSilenceCutoff),
            (Action::Silence, Some(since)) if since > self.created_at => {
                return Err(Error::FutureSilenceCutoff);
            }
            (Action::Silence, Some(_)) | (_, None) => {}
            (_, Some(_)) => return Err(Error::UnexpectedSilenceCutoff),
        }
        if self
            .reason
            .as_ref()
            .is_some_and(|reason| reason.len() > 500)
        {
            return Err(Error::ReasonTooLong);
        }
        if self.evidence != JudgmentEvidence::Local && self.event_id.is_none() {
            return Err(Error::MissingEventId);
        }
        Ok(())
    }

    /// Derives the stable address for the author/faculty/scope/target tuple.
    #[must_use]
    pub fn address(&self) -> String {
        address(self.faculty, &self.scope, &self.target)
    }

    #[must_use]
    pub fn tuple_key(&self) -> (PublicKey, Faculty, Scope, Target) {
        (
            self.author.clone(),
            self.faculty,
            self.scope.clone(),
            self.target.clone(),
        )
    }
}

/// Derives the stable `d` tag for a semantic tuple.
#[must_use]
pub fn address(faculty: Faculty, scope: &Scope, target: &Target) -> String {
    let material = [
        crate::PROTOCOL_VERSION,
        &faculty.to_string(),
        &scope.key(),
        &target.key(),
    ]
    .join("\0");
    let digest = Sha256::digest(material.as_bytes());
    format!("flocking:v1:{digest:x}")
}

/// Chooses the current valid judgment from events sharing one author and address.
#[must_use]
pub fn select_current<'a>(
    judgments: impl IntoIterator<Item = &'a Judgment>,
) -> Option<&'a Judgment> {
    judgments
        .into_iter()
        .filter(|judgment| judgment.validate().is_ok())
        .max_by(|left, right| {
            left.created_at.cmp(&right.created_at).then_with(|| {
                // The lexicographically lower event ID wins an equal timestamp.
                selection_key(right).cmp(&selection_key(left))
            })
        })
}

fn selection_key(judgment: &Judgment) -> String {
    judgment.event_id.as_ref().map_or_else(
        || {
            format!(
                "local:{}:{}:{}",
                judgment.action,
                judgment.since.unwrap_or_default(),
                judgment.reason.as_deref().unwrap_or_default()
            )
        },
        ToString::to_string,
    )
}

/// Resolves current judgments and gives canonical events authority over fallbacks.
#[must_use]
pub fn canonical_current(judgments: &[Judgment]) -> Vec<Judgment> {
    let mut groups: BTreeMap<(PublicKey, Faculty, Scope, Target), Vec<&Judgment>> = BTreeMap::new();
    for judgment in judgments {
        groups
            .entry(judgment.tuple_key())
            .or_default()
            .push(judgment);
    }
    let mut current = Vec::new();
    for events in groups.values() {
        let canonical: Vec<_> = events
            .iter()
            .copied()
            .filter(|judgment| {
                matches!(
                    judgment.evidence,
                    JudgmentEvidence::Local | JudgmentEvidence::FlockingEvent
                )
            })
            .collect();
        let selected = select_current(canonical).or_else(|| {
            select_current(events.iter().copied().filter(|judgment| {
                matches!(
                    judgment.evidence,
                    JudgmentEvidence::Nip02 | JudgmentEvidence::Nip51
                )
            }))
        });
        if let Some(selected) = selected {
            current.push(selected.clone());
        }
    }
    current.sort_by_key(Judgment::tuple_key);
    current
}

pub(crate) fn validate_question(
    faculty: Faculty,
    scope: &Scope,
    target: &Target,
) -> Result<(), Error> {
    let valid = match faculty {
        Faculty::Follow => matches!((scope, target), (Scope::Global, Target::Person(_))),
        Faculty::Block | Faculty::Silence => matches!(target, Target::Person(_)),
        Faculty::Hide => matches!(target, Target::Content(_)),
        Faculty::CommunityMembership | Faculty::Pin => {
            matches!((scope, target), (Scope::Topic(_), Target::Content(_)))
        }
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidQuestion {
            faculty: faculty.to_string(),
            scope: scope.to_string(),
            target: target.to_string(),
        })
    }
}

fn validate_action(faculty: Faculty, action: Action) -> Result<(), Error> {
    let valid = match faculty {
        Faculty::Follow => matches!(action, Action::Follow | Action::Unfollow | Action::Withdraw),
        Faculty::Block => matches!(action, Action::Block | Action::Unblock | Action::Withdraw),
        Faculty::Silence => {
            matches!(
                action,
                Action::Silence | Action::Unsilence | Action::Withdraw
            )
        }
        Faculty::Hide => matches!(action, Action::Hide | Action::Unhide | Action::Withdraw),
        Faculty::CommunityMembership => {
            matches!(action, Action::Remove | Action::Restore | Action::Withdraw)
        }
        Faculty::Pin => matches!(action, Action::Pin | Action::Withdraw),
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidAction {
            faculty: faculty.to_string(),
            action: action.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Topic;

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn judgment(created_at: u64, event: char) -> Judgment {
        Judgment {
            author: key('1'),
            faculty: Faculty::Block,
            scope: Scope::Topic(Topic::parse("science").unwrap()),
            target: Target::Person(key('2')),
            action: Action::Block,
            created_at,
            event_id: Some(EventId::parse(event.to_string().repeat(64)).unwrap()),
            since: None,
            reason: None,
            evidence: JudgmentEvidence::FlockingEvent,
        }
    }

    #[test]
    fn address_does_not_depend_on_action_or_reason() {
        let mut first = judgment(10, '1');
        let expected = first.address();
        first.action = Action::Unblock;
        first.reason = Some("changed my mind".to_owned());
        assert_eq!(first.address(), expected);
    }

    #[test]
    fn lower_event_id_wins_equal_timestamp() {
        let lower = judgment(10, '1');
        let higher = judgment(10, '2');
        assert_eq!(select_current([&higher, &lower]), Some(&lower));
    }

    #[test]
    fn invalid_newer_event_does_not_displace_valid_event() {
        let valid = judgment(10, '1');
        let mut invalid = judgment(11, '2');
        invalid.action = Action::Follow;
        assert_eq!(select_current([&valid, &invalid]), Some(&valid));
    }

    #[test]
    fn canonical_withdrawal_suppresses_fallback() {
        let mut fallback = judgment(10, '1');
        fallback.scope = Scope::Global;
        fallback.evidence = JudgmentEvidence::Nip51;
        let mut withdrawal = fallback.clone();
        withdrawal.created_at = 11;
        withdrawal.event_id = Some(EventId::parse("2".repeat(64)).unwrap());
        withdrawal.action = Action::Withdraw;
        withdrawal.evidence = JudgmentEvidence::FlockingEvent;
        assert_eq!(
            canonical_current(&[fallback, withdrawal.clone()]),
            vec![withdrawal]
        );
    }

    #[test]
    fn invalid_canonical_event_does_not_suppress_valid_fallback() {
        let mut fallback = judgment(10, '1');
        fallback.scope = Scope::Global;
        fallback.evidence = JudgmentEvidence::Nip51;
        let mut invalid = fallback.clone();
        invalid.created_at = 11;
        invalid.event_id = Some(EventId::parse("2".repeat(64)).unwrap());
        invalid.action = Action::Follow;
        invalid.evidence = JudgmentEvidence::FlockingEvent;
        assert_eq!(
            canonical_current(&[fallback.clone(), invalid]),
            vec![fallback]
        );
    }

    #[test]
    fn local_equal_timestamp_selection_is_input_order_independent() {
        let mut block = judgment(10, '1');
        block.event_id = None;
        block.evidence = JudgmentEvidence::Local;
        let mut unblock = block.clone();
        unblock.action = Action::Unblock;
        assert_eq!(select_current([&block, &unblock]), Some(&block));
        assert_eq!(select_current([&unblock, &block]), Some(&block));
    }

    #[test]
    fn public_reason_limit_is_measured_in_utf8_bytes() {
        let mut value = judgment(10, '1');
        value.reason = Some("é".repeat(250));
        assert!(value.validate().is_ok());
        value.reason = Some("é".repeat(251));
        assert_eq!(value.validate(), Err(Error::ReasonTooLong));
    }
}
