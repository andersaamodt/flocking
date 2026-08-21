use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    Action, Completeness, Config, Error, EventId, Faculty, Judgment, PublicKey, Scope, SourceState,
    Target, canonical_current,
    judgment::{JudgmentEvidence, validate_question},
};

/// Inputs to one ordinary-precedence evaluation.
#[derive(Debug, Clone, Copy)]
pub struct EvaluationInput<'a> {
    pub config: &'a Config,
    pub judgments: &'a [Judgment],
    pub source_states: &'a [SourceState],
    pub context: &'a crate::Context,
}

/// Whether the effective answer rests only on current-looking evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Certainty {
    Current,
    Stale,
}

/// How one applicable judgment entered the evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Direct,
    Flocked,
    Nip02Fallback,
    Nip51Fallback,
}

/// Inspectable evidence retained for a result or bypassed position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub author: PublicKey,
    pub faculty: Faculty,
    pub scope: Scope,
    pub action: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    pub completeness: Completeness,
}

/// A positive or affirmative-negative effective position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effective {
    pub value: bool,
    pub action: Action,
    pub faculty: Faculty,
    pub scope: Scope,
    pub certainty: Certainty,
    pub evidence: Evidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bypassed: Vec<Evidence>,
}

/// A determinate answer, no answer, or a result blocked by missing source state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Evaluation {
    Determinate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective: Option<Effective>,
        certainty: Certainty,
    },
    Indeterminate {
        unknown: Vec<SourceState>,
        stale: bool,
    },
}

/// Evaluates one faculty-target question using Flocking's ordinary precedence.
///
/// # Errors
///
/// Returns an error when configuration, metadata, judgments, or the question is invalid.
#[allow(clippy::too_many_lines)]
pub fn evaluate(
    input: EvaluationInput<'_>,
    faculty: Faculty,
    target: &Target,
) -> Result<Evaluation, Error> {
    if faculty == Faculty::Pin {
        return Err(Error::InvalidQuestion {
            faculty: faculty.to_string(),
            scope: "aggregation".to_owned(),
            target: target.to_string(),
        });
    }
    input.config.validate()?;
    let states = validate_states(input.config, input.source_states)?;
    let current = canonical_current(input.judgments);
    for judgment in &current {
        judgment.validate()?;
    }

    let primary_scope = match (&input.context.topic, faculty) {
        (_, Faculty::Follow) => Scope::Global,
        (Some(topic), _) => Scope::Topic(topic.clone()),
        (None, Faculty::CommunityMembership) => {
            return Err(Error::InvalidQuestion {
                faculty: faculty.to_string(),
                scope: "context_free".to_owned(),
                target: target.to_string(),
            });
        }
        (None, _) => Scope::Global,
    };
    validate_question(faculty, &primary_scope, target)?;

    let mut bypassed = Vec::new();
    let mut stale = false;
    if let Some(topic) = &input.context.topic {
        let scope = Scope::Topic(topic.clone());
        if let Some(judgment) = find(&current, &input.config.persona, faculty, &scope, target)
            && let Some(effective) = direct_effective(judgment, &mut bypassed, stale)
        {
            return Ok(Evaluation::Determinate {
                certainty: effective.certainty,
                effective: Some(effective),
            });
        }
    }
    let global = Scope::Global;
    if let Some(judgment) = find(&current, &input.config.persona, faculty, &global, target)
        && let Some(effective) = direct_effective(judgment, &mut bypassed, stale)
    {
        return Ok(Evaluation::Determinate {
            certainty: effective.certainty,
            effective: Some(effective),
        });
    }

    let mut scopes = Vec::new();
    if let Some(topic) = &input.context.topic {
        scopes.push(Scope::Topic(topic.clone()));
    }
    scopes.push(Scope::Global);

    for scope in scopes {
        let mut sources: Vec<_> = input
            .config
            .sources
            .iter()
            .filter_map(|source| {
                let grant = source
                    .grants
                    .iter()
                    .find(|grant| grant.faculty == faculty && grant.enables(&scope))?;
                grant.rank.map(|rank| (&source.pubkey, rank))
            })
            .collect();
        sources.sort_by_key(|(pubkey, rank)| (*rank, (*pubkey).clone()));

        for (source, rank) in sources {
            let state_record = states.get(&(source.clone(), faculty, scope.clone()));
            let completeness =
                state_record.map_or(Completeness::Unknown, |state| state.completeness);
            if completeness == Completeness::Unknown {
                let unknown = state_record
                    .map(|state| (*state).clone())
                    .unwrap_or(SourceState {
                        source: source.clone(),
                        faculty,
                        scope: scope.clone(),
                        completeness,
                    });
                return Ok(Evaluation::Indeterminate {
                    unknown: vec![unknown],
                    stale,
                });
            }
            stale |= completeness == Completeness::Stale;
            if let Some(judgment) = find(&current, source, faculty, &scope, target) {
                let evidence = evidence(judgment, EvidenceKind::Flocked, Some(rank), completeness);
                if judgment.action.is_withdrawn() {
                    bypassed.push(evidence);
                    continue;
                }
                let Some(value) = judgment.action.polarity() else {
                    continue;
                };
                let certainty = if stale {
                    Certainty::Stale
                } else {
                    Certainty::Current
                };
                let effective = Effective {
                    value,
                    action: judgment.action,
                    faculty,
                    scope: scope.clone(),
                    certainty,
                    evidence,
                    bypassed,
                };
                return Ok(Evaluation::Determinate {
                    effective: Some(effective),
                    certainty,
                });
            }
        }
    }

    Ok(Evaluation::Determinate {
        effective: None,
        certainty: if stale {
            Certainty::Stale
        } else {
            Certainty::Current
        },
    })
}

fn validate_states<'a>(
    config: &Config,
    states: &'a [SourceState],
) -> Result<BTreeMap<(PublicKey, Faculty, Scope), &'a SourceState>, Error> {
    let mut indexed = BTreeMap::new();
    for state in states {
        let Some(grant) = config.grant(&state.source, state.faculty) else {
            return Err(Error::InvalidSourceState);
        };
        if !grant.enables(&state.scope) {
            return Err(Error::InvalidSourceState);
        }
        if indexed
            .insert(
                (state.source.clone(), state.faculty, state.scope.clone()),
                state,
            )
            .is_some()
        {
            return Err(Error::DuplicateSourceState);
        }
    }
    Ok(indexed)
}

fn find<'a>(
    current: &'a [Judgment],
    author: &PublicKey,
    faculty: Faculty,
    scope: &Scope,
    target: &Target,
) -> Option<&'a Judgment> {
    current.iter().find(|judgment| {
        &judgment.author == author
            && judgment.faculty == faculty
            && &judgment.scope == scope
            && &judgment.target == target
    })
}

fn direct_effective(
    judgment: &Judgment,
    bypassed: &mut Vec<Evidence>,
    stale: bool,
) -> Option<Effective> {
    let evidence = evidence(judgment, EvidenceKind::Direct, None, Completeness::Complete);
    if judgment.action.is_withdrawn() {
        bypassed.push(evidence);
        return None;
    }
    let value = judgment.action.polarity()?;
    Some(Effective {
        value,
        action: judgment.action,
        faculty: judgment.faculty,
        scope: judgment.scope.clone(),
        certainty: if stale {
            Certainty::Stale
        } else {
            Certainty::Current
        },
        evidence,
        bypassed: std::mem::take(bypassed),
    })
}

fn evidence(
    judgment: &Judgment,
    ordinary_kind: EvidenceKind,
    rank: Option<u32>,
    completeness: Completeness,
) -> Evidence {
    let kind = match judgment.evidence {
        JudgmentEvidence::Nip02 => EvidenceKind::Nip02Fallback,
        JudgmentEvidence::Nip51 => EvidenceKind::Nip51Fallback,
        JudgmentEvidence::Local | JudgmentEvidence::FlockingEvent => ordinary_kind,
    };
    Evidence {
        kind,
        author: judgment.author.clone(),
        faculty: judgment.faculty,
        scope: judgment.scope.clone(),
        action: judgment.action,
        event_id: judgment.event_id.clone(),
        rank,
        since: judgment.since,
        completeness,
    }
}

pub(crate) fn effective_value(evaluation: &Evaluation) -> Option<bool> {
    match evaluation {
        Evaluation::Determinate {
            effective: Some(effective),
            ..
        } => Some(effective.value),
        Evaluation::Determinate {
            effective: None, ..
        }
        | Evaluation::Indeterminate { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{CONFIG_VERSION, Context, FacultyGrant, Source, Topic, judgment::JudgmentEvidence};

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn judgment(author: char, action: Action, scope: Scope, created_at: u64) -> Judgment {
        Judgment {
            author: key(author),
            faculty: Faculty::Block,
            scope,
            target: Target::Person(key('9')),
            action,
            created_at,
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
            sources: vec![
                Source {
                    pubkey: key('1'),
                    grants: vec![FacultyGrant {
                        faculty: Faculty::Block,
                        global: true,
                        topics: BTreeSet::from([Topic::parse("science").unwrap()]),
                        rank: Some(1),
                    }],
                    reverse_blocks: None,
                },
                Source {
                    pubkey: key('2'),
                    grants: vec![FacultyGrant {
                        faculty: Faculty::Block,
                        global: true,
                        topics: BTreeSet::from([Topic::parse("science").unwrap()]),
                        rank: Some(2),
                    }],
                    reverse_blocks: None,
                },
            ],
            local_pin_dismissals: Vec::new(),
        }
    }

    #[test]
    fn direct_topic_beats_direct_global_and_flocked() {
        let topic = Topic::parse("science").unwrap();
        let context = Context {
            topic: Some(topic.clone()),
        };
        let judgments = vec![
            judgment('0', Action::Block, Scope::Global, 1),
            judgment('0', Action::Unblock, Scope::Topic(topic.clone()), 2),
            judgment('1', Action::Block, Scope::Topic(topic.clone()), 3),
        ];
        let states = vec![SourceState {
            source: key('1'),
            faculty: Faculty::Block,
            scope: Scope::Topic(topic),
            completeness: Completeness::Complete,
        }];
        let result = evaluate(
            EvaluationInput {
                config: &config(),
                judgments: &judgments,
                source_states: &states,
                context: &context,
            },
            Faculty::Block,
            &Target::Person(key('9')),
        )
        .unwrap();
        assert_eq!(effective_value(&result), Some(false));
    }

    #[test]
    fn withdrawal_exposes_lower_ranked_source() {
        let context = Context::default();
        let judgments = vec![
            judgment('1', Action::Withdraw, Scope::Global, 2),
            judgment('2', Action::Block, Scope::Global, 1),
        ];
        let states = vec![
            SourceState {
                source: key('1'),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Complete,
            },
            SourceState {
                source: key('2'),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Complete,
            },
        ];
        let result = evaluate(
            EvaluationInput {
                config: &config(),
                judgments: &judgments,
                source_states: &states,
                context: &context,
            },
            Faculty::Block,
            &Target::Person(key('9')),
        )
        .unwrap();
        assert_eq!(effective_value(&result), Some(true));
    }

    #[test]
    fn unknown_higher_rank_is_indeterminate() {
        let context = Context::default();
        let judgments = vec![judgment('2', Action::Block, Scope::Global, 1)];
        let states = vec![
            SourceState {
                source: key('1'),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Unknown,
            },
            SourceState {
                source: key('2'),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: Completeness::Complete,
            },
        ];
        let result = evaluate(
            EvaluationInput {
                config: &config(),
                judgments: &judgments,
                source_states: &states,
                context: &context,
            },
            Faculty::Block,
            &Target::Person(key('9')),
        )
        .unwrap();
        assert!(matches!(result, Evaluation::Indeterminate { .. }));
    }

    #[test]
    fn known_direct_answer_ignores_unknown_lower_input() {
        let context = Context::default();
        let judgments = vec![judgment('0', Action::Unblock, Scope::Global, 1)];
        let states = vec![SourceState {
            source: key('1'),
            faculty: Faculty::Block,
            scope: Scope::Global,
            completeness: Completeness::Unknown,
        }];
        let result = evaluate(
            EvaluationInput {
                config: &config(),
                judgments: &judgments,
                source_states: &states,
                context: &context,
            },
            Faculty::Block,
            &Target::Person(key('9')),
        )
        .unwrap();
        assert_eq!(effective_value(&result), Some(false));
    }

    #[test]
    fn follow_remains_global_inside_a_topic_view() {
        let mut config = config();
        config.sources.clear();
        let mut direct = judgment('0', Action::Block, Scope::Global, 1);
        direct.faculty = Faculty::Follow;
        direct.action = Action::Follow;
        let context = Context {
            topic: Some(Topic::parse("science").unwrap()),
        };
        let result = evaluate(
            EvaluationInput {
                config: &config,
                judgments: &[direct],
                source_states: &[],
                context: &context,
            },
            Faculty::Follow,
            &Target::Person(key('9')),
        )
        .unwrap();
        assert_eq!(effective_value(&result), Some(true));
    }
}
