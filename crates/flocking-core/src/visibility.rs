use serde::{Deserialize, Serialize};

use crate::{
    ContentTarget, Error, Evaluation, EvaluationInput, Faculty, PublicKey, Target, evaluate,
    evaluate::effective_value,
};

/// One authored contribution or newly signed revision to test for eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    pub author: PublicKey,
    pub target: ContentTarget,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<u64>,
}

/// Inputs to cross-faculty content eligibility.
#[derive(Debug, Clone, Copy)]
pub struct VisibilityInput<'a> {
    pub evaluation: EvaluationInput<'a>,
    pub contribution: &'a Contribution,
}

/// The faculty that presently excludes a contribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Exclusion {
    Block,
    Silence {
        cutoff: u64,
        local_timing_evidence: bool,
    },
    Hide,
    CommunityRemoval,
}

/// Eligibility plus each independent faculty result used to derive it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visibility {
    pub eligible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusion: Option<Exclusion>,
    pub block: Evaluation,
    pub silence: Evaluation,
    pub hide: Evaluation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub membership: Option<Evaluation>,
}

/// Composes block, silence, hide, and contextual membership without erasing state.
///
/// # Errors
///
/// Returns an error when any input or composed faculty question is invalid.
pub fn evaluate_visibility(input: VisibilityInput<'_>) -> Result<Visibility, Error> {
    let person = Target::Person(input.contribution.author.clone());
    let content = Target::Content(input.contribution.target.clone());
    let block = evaluate(input.evaluation, Faculty::Block, &person)?;
    let silence = evaluate(input.evaluation, Faculty::Silence, &person)?;
    let hide = evaluate(input.evaluation, Faculty::Hide, &content)?;
    let membership = if input.evaluation.context.topic.is_some() {
        Some(evaluate(
            input.evaluation,
            Faculty::CommunityMembership,
            &content,
        )?)
    } else {
        None
    };

    let indeterminate = [&block, &silence, &hide]
        .into_iter()
        .chain(membership.iter())
        .any(|result| matches!(result, Evaluation::Indeterminate { .. }));
    let exclusion = if effective_value(&block) == Some(true) {
        Some(Exclusion::Block)
    } else if let Some(cutoff) = silence_cutoff(&silence) {
        let signed_future = input.contribution.created_at >= cutoff;
        let local_backdating = input.contribution.created_at < cutoff
            && input
                .contribution
                .first_seen
                .is_some_and(|first_seen| first_seen >= cutoff);
        (signed_future || local_backdating).then_some(Exclusion::Silence {
            cutoff,
            local_timing_evidence: local_backdating,
        })
    } else if effective_value(&hide) == Some(true) {
        Some(Exclusion::Hide)
    } else if membership.as_ref().and_then(effective_value) == Some(true) {
        Some(Exclusion::CommunityRemoval)
    } else {
        None
    };
    let eligible = if exclusion.is_some() {
        Some(false)
    } else if indeterminate {
        None
    } else {
        Some(true)
    };
    Ok(Visibility {
        eligible,
        exclusion,
        block,
        silence,
        hide,
        membership,
    })
}

fn silence_cutoff(evaluation: &Evaluation) -> Option<u64> {
    match evaluation {
        Evaluation::Determinate {
            effective: Some(effective),
            ..
        } if effective.value => effective.evidence.since,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{
        Action, CONFIG_VERSION, Completeness, Config, Context, EventId, FacultyGrant, Judgment,
        Scope, Source, SourceState, judgment::JudgmentEvidence,
    };

    fn key(character: char) -> PublicKey {
        PublicKey::parse(character.to_string().repeat(64)).unwrap()
    }

    fn content(character: char) -> ContentTarget {
        ContentTarget::Event(EventId::parse(character.to_string().repeat(64)).unwrap())
    }

    fn config() -> Config {
        Config {
            version: CONFIG_VERSION.to_owned(),
            persona: key('0'),
            sources: vec![Source {
                pubkey: key('1'),
                grants: [Faculty::Block, Faculty::Silence, Faculty::Hide]
                    .into_iter()
                    .enumerate()
                    .map(|(index, faculty)| FacultyGrant {
                        faculty,
                        global: true,
                        topics: BTreeSet::new(),
                        rank: Some(u32::try_from(index + 1).unwrap()),
                    })
                    .collect(),
                reverse_blocks: None,
            }],
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: Vec::new(),
        }
    }

    fn state(faculty: Faculty) -> SourceState {
        SourceState {
            source: key('1'),
            faculty,
            scope: Scope::Global,
            completeness: Completeness::Complete,
        }
    }

    fn judgment(faculty: Faculty, action: Action, since: Option<u64>) -> Judgment {
        Judgment {
            author: key('1'),
            faculty,
            scope: Scope::Global,
            target: if matches!(faculty, Faculty::Block | Faculty::Silence) {
                Target::Person(key('9'))
            } else {
                Target::Content(content('a'))
            },
            action,
            created_at: 10,
            event_id: None,
            since,
            reason: None,
            evidence: JudgmentEvidence::Local,
        }
    }

    fn visible(judgments: &[Judgment], contribution: &Contribution) -> Visibility {
        let states = [
            state(Faculty::Block),
            state(Faculty::Silence),
            state(Faculty::Hide),
        ];
        evaluate_visibility(VisibilityInput {
            evaluation: EvaluationInput {
                config: &config(),
                judgments,
                source_states: &states,
                context: &Context::default(),
            },
            contribution,
        })
        .unwrap()
    }

    #[test]
    fn block_dominates_without_erasing_silence() {
        let result = visible(
            &[
                judgment(Faculty::Block, Action::Block, None),
                judgment(Faculty::Silence, Action::Silence, Some(10)),
            ],
            &Contribution {
                author: key('9'),
                target: content('a'),
                created_at: 11,
                first_seen: None,
            },
        );
        assert_eq!(result.exclusion, Some(Exclusion::Block));
        assert_eq!(effective_value(&result.silence), Some(true));
    }

    #[test]
    fn silence_preserves_past_and_excludes_new_revision() {
        let silence = judgment(Faculty::Silence, Action::Silence, Some(10));
        let old = visible(
            std::slice::from_ref(&silence),
            &Contribution {
                author: key('9'),
                target: content('a'),
                created_at: 9,
                first_seen: Some(9),
            },
        );
        let revision = visible(
            &[silence],
            &Contribution {
                author: key('9'),
                target: content('a'),
                created_at: 11,
                first_seen: Some(11),
            },
        );
        assert_eq!(old.eligible, Some(true));
        assert!(matches!(
            revision.exclusion,
            Some(Exclusion::Silence { .. })
        ));
    }

    #[test]
    fn first_seen_can_expose_backdating_but_block_needs_no_time() {
        let result = visible(
            &[judgment(Faculty::Silence, Action::Silence, Some(10))],
            &Contribution {
                author: key('9'),
                target: content('a'),
                created_at: 1,
                first_seen: Some(12),
            },
        );
        assert_eq!(
            result.exclusion,
            Some(Exclusion::Silence {
                cutoff: 10,
                local_timing_evidence: true
            })
        );
    }

    #[test]
    fn descendant_by_other_author_is_not_blocked() {
        let result = visible(
            &[judgment(Faculty::Block, Action::Block, None)],
            &Contribution {
                author: key('8'),
                target: content('b'),
                created_at: 11,
                first_seen: None,
            },
        );
        assert_eq!(result.eligible, Some(true));
    }
}
