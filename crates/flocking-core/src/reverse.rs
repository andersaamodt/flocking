use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    Action, Completeness, Config, Error, Faculty, Judgment, PublicKey, Scope, SourceProvenance,
    SourceState, Target, canonical_current, judgment::JudgmentEvidence,
};

type SourceEvents = BTreeMap<PublicKey, Option<crate::EventId>>;
type ReverseAggregate = (SourceEvents, u64, Scope);

/// Inputs to Reverse Flocking discovery.
#[derive(Debug, Clone, Copy)]
pub struct ReverseInput<'a> {
    pub config: &'a Config,
    pub judgments: &'a [Judgment],
    pub source_states: &'a [SourceState],
    pub context: &'a crate::Context,
}

/// One deduplicated person discovered through selected sources' blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseTarget {
    pub target: PublicKey,
    pub source_count: usize,
    pub newest_support: u64,
    pub sources: Vec<PublicKey>,
    pub provenance: Vec<SourceProvenance>,
    pub discovery_scope: Scope,
}

/// Ordered Reverse-Flocking discovery with completeness evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseResult {
    pub targets: Vec<ReverseTarget>,
    pub incomplete_sources: Vec<PublicKey>,
    pub stale: bool,
}

/// The two direct actions in an explicit local Rescue transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rescue {
    pub follow: Judgment,
    pub unblock: Judgment,
}

/// Discovers people whom selected sources currently block.
///
/// # Errors
///
/// Returns an error when configuration, source state, or judgments are invalid.
#[allow(clippy::too_many_lines)]
pub fn evaluate_reverse(input: ReverseInput<'_>) -> Result<ReverseResult, Error> {
    input.config.validate()?;
    let current = canonical_current(input.judgments);
    let mut states = BTreeMap::new();
    for state in input.source_states {
        let valid = state.faculty == Faculty::Block
            && input
                .config
                .source(&state.source)
                .and_then(|source| source.reverse_blocks.as_ref())
                .is_some_and(|grant| grant.enables(&state.scope));
        if !valid {
            return Err(Error::InvalidSourceState);
        }
        if states
            .insert(
                (state.source.clone(), state.faculty, state.scope.clone()),
                state.completeness,
            )
            .is_some()
        {
            return Err(Error::DuplicateSourceState);
        }
    }
    let mut scopes = Vec::new();
    if let Some(topic) = &input.context.topic {
        scopes.push(Scope::Topic(topic.clone()));
    }
    scopes.push(Scope::Global);

    let mut discovered: BTreeMap<PublicKey, ReverseAggregate> = BTreeMap::new();
    let mut incomplete = BTreeSet::new();
    let mut stale = false;
    for source in &input.config.sources {
        let Some(grant) = &source.reverse_blocks else {
            continue;
        };
        for scope in scopes.iter().filter(|scope| grant.enables(scope)) {
            let completeness = states
                .get(&(source.pubkey.clone(), Faculty::Block, scope.clone()))
                .copied()
                .unwrap_or(Completeness::Unknown);
            match completeness {
                Completeness::Unknown => {
                    incomplete.insert(source.pubkey.clone());
                    continue;
                }
                Completeness::Stale => stale = true,
                Completeness::Complete => {}
            }
            for judgment in current.iter().filter(|judgment| {
                judgment.author == source.pubkey
                    && judgment.faculty == Faculty::Block
                    && &judgment.scope == scope
                    && judgment.action == Action::Block
            }) {
                let Target::Person(target) = &judgment.target else {
                    continue;
                };
                let entry = discovered
                    .entry(target.clone())
                    .or_insert_with(|| (BTreeMap::new(), judgment.created_at, scope.clone()));
                entry
                    .0
                    .insert(source.pubkey.clone(), judgment.event_id.clone());
                if judgment.created_at > entry.1 {
                    entry.1 = judgment.created_at;
                    entry.2 = scope.clone();
                }
            }
        }
    }
    let mut targets: Vec<_> = discovered
        .into_iter()
        .map(
            |(target, (source_events, newest_support, discovery_scope))| {
                let sources = source_events.keys().cloned().collect::<Vec<_>>();
                let provenance = source_events
                    .into_iter()
                    .map(|(source, event_id)| SourceProvenance { source, event_id })
                    .collect::<Vec<_>>();
                ReverseTarget {
                    target,
                    source_count: sources.len(),
                    newest_support,
                    sources,
                    provenance,
                    discovery_scope,
                }
            },
        )
        .collect();
    targets.sort_by(|left, right| {
        right
            .source_count
            .cmp(&left.source_count)
            .then_with(|| right.newest_support.cmp(&left.newest_support))
            .then_with(|| left.target.cmp(&right.target))
    });
    Ok(ReverseResult {
        targets,
        incomplete_sources: incomplete.into_iter().collect(),
        stale,
    })
}

/// Creates the direct follow and unblock positions that constitute Rescue.
#[must_use]
pub fn rescue(
    persona: &PublicKey,
    target: &PublicKey,
    discovery_scope: Scope,
    created_at: u64,
) -> Rescue {
    let common = |faculty, scope, action| Judgment {
        author: persona.clone(),
        faculty,
        scope,
        target: Target::Person(target.clone()),
        action,
        created_at,
        event_id: None,
        since: None,
        reason: None,
        evidence: JudgmentEvidence::Local,
    };
    Rescue {
        follow: common(Faculty::Follow, Scope::Global, Action::Follow),
        unblock: common(Faculty::Block, discovery_scope, Action::Unblock),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{CONFIG_VERSION, Context, EventId, ReverseBlockGrant, Source, Topic};

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn block(author: char, target: char, time: u64) -> Judgment {
        Judgment {
            author: key(author),
            faculty: Faculty::Block,
            scope: Scope::Global,
            target: Target::Person(key(target)),
            action: Action::Block,
            created_at: time,
            event_id: None,
            since: None,
            reason: None,
            evidence: JudgmentEvidence::Local,
        }
    }

    fn config() -> Config {
        Config {
            version: CONFIG_VERSION.to_owned(),
            persona: key('0'),
            sources: ['1', '2']
                .into_iter()
                .map(|character| Source {
                    pubkey: key(character),
                    grants: Vec::new(),
                    reverse_blocks: Some(ReverseBlockGrant {
                        global: true,
                        topics: BTreeSet::from([Topic::parse("science").unwrap()]),
                    }),
                })
                .collect(),
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: Vec::new(),
        }
    }

    #[test]
    fn deduplicates_targets_and_retains_all_sources() {
        let judgments = vec![block('1', '9', 1), block('2', '9', 2), block('1', '8', 3)];
        let states = ['1', '2']
            .into_iter()
            .map(|character| SourceState {
                source: key(character),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Complete,
            })
            .collect::<Vec<_>>();
        let result = evaluate_reverse(ReverseInput {
            config: &config(),
            judgments: &judgments,
            source_states: &states,
            context: &Context::default(),
        })
        .unwrap();
        assert_eq!(result.targets.len(), 2);
        assert_eq!(result.targets[0].target, key('9'));
        assert_eq!(result.targets[0].sources, vec![key('1'), key('2')]);
    }

    #[test]
    fn rescue_is_direct_and_does_not_modify_inherited_evidence() {
        let rescue = rescue(&key('0'), &key('9'), Scope::Global, 10);
        assert_eq!(rescue.follow.action, Action::Follow);
        assert_eq!(rescue.unblock.action, Action::Unblock);
        assert_eq!(rescue.follow.evidence, JudgmentEvidence::Local);
    }

    #[test]
    fn rejects_reverse_state_outside_the_granted_scope() {
        let result = evaluate_reverse(ReverseInput {
            config: &config(),
            judgments: &[],
            source_states: &[SourceState {
                source: key('1'),
                faculty: Faculty::Block,
                scope: Scope::Topic(Topic::parse("biology").unwrap()),
                completeness: Completeness::Complete,
            }],
            context: &Context::default(),
        });
        assert_eq!(result, Err(Error::InvalidSourceState));
    }

    #[test]
    fn reverse_result_retains_source_event_provenance() {
        let mut judgment = block('1', '9', 1);
        judgment.event_id = Some(EventId::parse("a".repeat(64)).unwrap());
        judgment.evidence = JudgmentEvidence::FlockingEvent;
        let result = evaluate_reverse(ReverseInput {
            config: &config(),
            judgments: &[judgment],
            source_states: &[SourceState {
                source: key('1'),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Complete,
            }],
            context: &Context::default(),
        })
        .unwrap();
        assert_eq!(
            result.targets[0].provenance[0].event_id,
            Some(EventId::parse("a".repeat(64)).unwrap())
        );
    }
}
