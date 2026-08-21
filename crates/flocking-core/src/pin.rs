use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    Certainty, Completeness, Config, ContentTarget, Error, Faculty, Judgment, PublicKey, Scope,
    SourceProvenance, SourceState, Target, Topic, canonical_current,
};

type SourceEvents = BTreeMap<PublicKey, Option<crate::EventId>>;
type PinAggregate = (SourceEvents, u64, bool);

/// Inputs to contextual pin aggregation.
#[derive(Debug, Clone, Copy)]
pub struct PinInput<'a> {
    pub config: &'a Config,
    pub judgments: &'a [Judgment],
    pub source_states: &'a [SourceState],
    pub topic: &'a Topic,
    pub ineligible: &'a BTreeSet<ContentTarget>,
}

/// One ordered direct or inherited pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinSupport {
    pub target: ContentTarget,
    pub direct: bool,
    pub source_count: usize,
    pub newest_support: u64,
    pub sources: Vec<PublicKey>,
    pub provenance: Vec<SourceProvenance>,
    pub certainty: Certainty,
}

/// Ordered pins plus evidence that support may be incomplete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinResult {
    pub pins: Vec<PinSupport>,
    pub incomplete_sources: Vec<PublicKey>,
    pub stale: bool,
}

/// Aggregates direct and flock-derived pins for one topic.
///
/// # Errors
///
/// Returns an error when configuration, metadata, judgments, or dismissals are invalid.
#[allow(clippy::too_many_lines)]
pub fn evaluate_pins(input: PinInput<'_>) -> Result<PinResult, Error> {
    input.config.validate()?;
    let current = canonical_current(input.judgments);
    for judgment in &current {
        judgment.validate()?;
    }
    let scope = Scope::Topic(input.topic.clone());
    let states = state_index(input.config, input.source_states)?;
    let dismissals: BTreeSet<_> = input
        .config
        .local_pin_dismissals
        .iter()
        .filter(|dismissal| &dismissal.topic == input.topic)
        .map(crate::LocalPinDismissal::content_target)
        .collect::<Result<_, _>>()?;

    let mut direct = Vec::new();
    let mut direct_targets = BTreeSet::new();
    for judgment in current.iter().filter(|judgment| {
        judgment.author == input.config.persona
            && judgment.faculty == Faculty::Pin
            && judgment.scope == scope
            && judgment.action == crate::Action::Pin
    }) {
        let Target::Content(target) = &judgment.target else {
            continue;
        };
        if input.ineligible.contains(target) {
            continue;
        }
        direct_targets.insert(target.clone());
        direct.push(PinSupport {
            target: target.clone(),
            direct: true,
            source_count: 1,
            newest_support: judgment.created_at,
            sources: vec![input.config.persona.clone()],
            provenance: vec![SourceProvenance {
                source: input.config.persona.clone(),
                event_id: judgment.event_id.clone(),
            }],
            certainty: Certainty::Current,
        });
    }
    direct.sort_by(|left, right| {
        right
            .newest_support
            .cmp(&left.newest_support)
            .then_with(|| left.target.cmp(&right.target))
    });

    let mut inherited: BTreeMap<ContentTarget, PinAggregate> = BTreeMap::new();
    let mut incomplete_sources = BTreeSet::new();
    let mut stale = false;
    for source in &input.config.sources {
        let Some(grant) = source
            .grants
            .iter()
            .find(|grant| grant.faculty == Faculty::Pin && grant.enables(&scope))
        else {
            continue;
        };
        debug_assert!(grant.rank.is_none());
        let completeness = states
            .get(&(source.pubkey.clone(), Faculty::Pin, scope.clone()))
            .copied()
            .unwrap_or(Completeness::Unknown);
        match completeness {
            Completeness::Unknown => {
                incomplete_sources.insert(source.pubkey.clone());
                continue;
            }
            Completeness::Stale => stale = true,
            Completeness::Complete => {}
        }
        for judgment in current.iter().filter(|judgment| {
            judgment.author == source.pubkey
                && judgment.faculty == Faculty::Pin
                && judgment.scope == scope
                && judgment.action == crate::Action::Pin
        }) {
            let Target::Content(target) = &judgment.target else {
                continue;
            };
            if direct_targets.contains(target)
                || dismissals.contains(target)
                || input.ineligible.contains(target)
            {
                continue;
            }
            let entry = inherited
                .entry(target.clone())
                .or_insert_with(|| (BTreeMap::new(), 0, false));
            entry
                .0
                .insert(source.pubkey.clone(), judgment.event_id.clone());
            entry.1 = entry.1.max(judgment.created_at);
            entry.2 |= completeness == Completeness::Stale;
        }
    }

    let mut inherited: Vec<_> = inherited
        .into_iter()
        .map(|(target, (source_events, newest_support, target_stale))| {
            let sources = source_events.keys().cloned().collect::<Vec<_>>();
            let provenance = source_events
                .into_iter()
                .map(|(source, event_id)| SourceProvenance { source, event_id })
                .collect::<Vec<_>>();
            PinSupport {
                target,
                direct: false,
                source_count: sources.len(),
                newest_support,
                sources,
                provenance,
                certainty: if target_stale {
                    Certainty::Stale
                } else {
                    Certainty::Current
                },
            }
        })
        .collect();
    inherited.sort_by(|left, right| {
        right
            .source_count
            .cmp(&left.source_count)
            .then_with(|| right.newest_support.cmp(&left.newest_support))
            .then_with(|| left.target.cmp(&right.target))
    });
    direct.extend(inherited);
    Ok(PinResult {
        pins: direct,
        incomplete_sources: incomplete_sources.into_iter().collect(),
        stale,
    })
}

fn state_index(
    config: &Config,
    states: &[SourceState],
) -> Result<BTreeMap<(PublicKey, Faculty, Scope), Completeness>, Error> {
    let mut index = BTreeMap::new();
    for state in states {
        let valid = state.faculty == Faculty::Pin
            && config
                .grant(&state.source, Faculty::Pin)
                .is_some_and(|grant| grant.enables(&state.scope));
        if !valid {
            return Err(Error::InvalidSourceState);
        }
        if index
            .insert(
                (state.source.clone(), state.faculty, state.scope.clone()),
                state.completeness,
            )
            .is_some()
        {
            return Err(Error::DuplicateSourceState);
        }
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{
        Action, CONFIG_VERSION, EventId, FacultyGrant, LocalPinDismissal, PinTargetType, Source,
        judgment::JudgmentEvidence,
    };

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn target(character: char) -> ContentTarget {
        ContentTarget::Event(EventId::parse(character.to_string().repeat(64)).unwrap())
    }

    fn pin(author: char, target: ContentTarget, time: u64) -> Judgment {
        Judgment {
            author: key(author),
            faculty: Faculty::Pin,
            scope: Scope::Topic(Topic::parse("science").unwrap()),
            target: Target::Content(target),
            action: Action::Pin,
            created_at: time,
            event_id: None,
            since: None,
            reason: None,
            evidence: JudgmentEvidence::Local,
        }
    }

    fn config(dismissals: Vec<LocalPinDismissal>) -> Config {
        Config {
            version: CONFIG_VERSION.to_owned(),
            persona: key('0'),
            sources: ['1', '2', '3']
                .into_iter()
                .map(|character| Source {
                    pubkey: key(character),
                    grants: vec![FacultyGrant {
                        faculty: Faculty::Pin,
                        global: false,
                        topics: BTreeSet::from([Topic::parse("science").unwrap()]),
                        rank: None,
                    }],
                    reverse_blocks: None,
                })
                .collect(),
            local_pin_dismissals: dismissals,
        }
    }

    fn states() -> Vec<SourceState> {
        ['1', '2', '3']
            .into_iter()
            .map(|character| SourceState {
                source: key(character),
                faculty: Faculty::Pin,
                scope: Scope::Topic(Topic::parse("science").unwrap()),
                completeness: Completeness::Complete,
            })
            .collect()
    }

    #[test]
    fn orders_support_count_then_recency_then_target() {
        let a = target('a');
        let b = target('b');
        let judgments = vec![
            pin('1', a.clone(), 2),
            pin('2', a.clone(), 3),
            pin('1', b.clone(), 9),
        ];
        let result = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &judgments,
            source_states: &states(),
            topic: &Topic::parse("science").unwrap(),
            ineligible: &BTreeSet::new(),
        })
        .unwrap();
        assert_eq!(result.pins[0].target, a);
        assert_eq!(result.pins[0].source_count, 2);
        assert_eq!(result.pins[1].target, b);
    }

    #[test]
    fn dismissal_hides_inherited_but_not_direct_pin() {
        let dismissed = target('a');
        let record = LocalPinDismissal {
            topic: Topic::parse("science").unwrap(),
            target_type: PinTargetType::Event,
            target: match &dismissed {
                ContentTarget::Event(id) => id.to_string(),
                ContentTarget::Address(_) => unreachable!(),
            },
        };
        let judgments = vec![
            pin('0', dismissed.clone(), 3),
            pin('1', dismissed.clone(), 2),
        ];
        let result = evaluate_pins(PinInput {
            config: &config(vec![record]),
            judgments: &judgments,
            source_states: &states(),
            topic: &Topic::parse("science").unwrap(),
            ineligible: &BTreeSet::new(),
        })
        .unwrap();
        assert_eq!(result.pins.len(), 1);
        assert!(result.pins[0].direct);
    }

    #[test]
    fn ineligible_pin_reactivates_without_changing_authored_state() {
        let content = target('a');
        let judgments = vec![pin('1', content.clone(), 2)];
        let topic = Topic::parse("science").unwrap();
        let hidden = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &judgments,
            source_states: &states(),
            topic: &topic,
            ineligible: &BTreeSet::from([content.clone()]),
        })
        .unwrap();
        let visible = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &judgments,
            source_states: &states(),
            topic: &topic,
            ineligible: &BTreeSet::new(),
        })
        .unwrap();
        assert!(hidden.pins.is_empty());
        assert_eq!(visible.pins[0].target, content);
    }

    #[test]
    fn rejects_source_state_for_an_unconfigured_person() {
        let topic = Topic::parse("science").unwrap();
        let invalid_state = SourceState {
            source: key('9'),
            faculty: Faculty::Pin,
            scope: Scope::Topic(topic.clone()),
            completeness: Completeness::Complete,
        };
        let result = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &[],
            source_states: &[invalid_state],
            topic: &topic,
            ineligible: &BTreeSet::new(),
        });
        assert_eq!(result, Err(Error::InvalidSourceState));
    }

    #[test]
    fn pin_result_retains_source_event_provenance() {
        let topic = Topic::parse("science").unwrap();
        let mut judgment = pin('1', target('a'), 2);
        judgment.event_id = Some(EventId::parse("e".repeat(64)).unwrap());
        judgment.evidence = JudgmentEvidence::FlockingEvent;
        let result = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &[judgment],
            source_states: &states(),
            topic: &topic,
            ineligible: &BTreeSet::new(),
        })
        .unwrap();
        assert_eq!(
            result.pins[0].provenance[0].event_id,
            Some(EventId::parse("e".repeat(64)).unwrap())
        );
    }

    #[test]
    fn direct_pin_orders_before_more_supported_inherited_pin() {
        let topic = Topic::parse("science").unwrap();
        let direct = pin('0', target('b'), 1);
        let inherited = vec![
            pin('1', target('a'), 2),
            pin('2', target('a'), 3),
            pin('3', target('a'), 4),
            direct,
        ];
        let result = evaluate_pins(PinInput {
            config: &config(Vec::new()),
            judgments: &inherited,
            source_states: &states(),
            topic: &topic,
            ineligible: &BTreeSet::new(),
        })
        .unwrap();
        assert!(result.pins[0].direct);
        assert_eq!(result.pins[0].target, target('b'));
        assert_eq!(result.pins[1].source_count, 3);
    }
}
