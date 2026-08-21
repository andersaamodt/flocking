use std::collections::{BTreeMap, BTreeSet};

use flocking_core::{
    Action, CONFIG_VERSION, Certainty, Completeness, Config, ContentTarget, Context, Evaluation,
    EvaluationInput, EventId, Exclusion, Faculty, FacultyGrant, Judgment, JudgmentEvidence,
    LocalPinDismissal, PinInput, PinTargetType, PublicKey, Rescue, ReverseBlockGrant, ReverseInput,
    Scope, Source, SourceState, Target, Topic, VisibilityInput, address, evaluate, evaluate_pins,
    evaluate_reverse, evaluate_visibility, rescue, select_current,
};
use serde::Deserialize;

const VECTORS: &str = include_str!("../../../vectors/flocking-v1.json");
const CONFIG_SCHEMA: &str = include_str!("../../../schemas/flocking-config-1.schema.json");
const JUDGMENT_SCHEMA: &str = include_str!("../../../schemas/flocking-judgment-1.schema.json");
const EVENT_SCHEMA: &str = include_str!("../../../schemas/flocking-nostr-event-1.schema.json");
const VECTOR_SCHEMA: &str = include_str!("../../../schemas/flocking-vectors-1.schema.json");

#[derive(Debug, Deserialize)]
struct Suite {
    version: String,
    identities: BTreeMap<String, String>,
    content: BTreeMap<String, String>,
    topic_cases: Vec<TopicCase>,
    address_cases: Vec<AddressCase>,
    action_cases: Vec<ActionCase>,
    selection_cases: Vec<SelectionCase>,
    evaluation_cases: Vec<EvaluationCase>,
    visibility_cases: Vec<VisibilityCase>,
    pin_cases: Vec<PinCase>,
    reverse_cases: Vec<ReverseCase>,
    rescue: RescueCase,
    fallback_cases: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TopicCase {
    input: String,
    expected: Option<String>,
    #[serde(default)]
    error: bool,
}

#[derive(Debug, Deserialize)]
struct AddressCase {
    faculty: String,
    scope: String,
    target: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
struct ActionCase {
    faculty: String,
    scope: String,
    target: String,
    action: String,
    since: Option<u64>,
    valid: bool,
}

#[derive(Debug, Deserialize)]
struct SelectionCase {
    name: String,
    events: Vec<SelectionEvent>,
    expected: String,
}

#[derive(Debug, Deserialize)]
struct SelectionEvent {
    time: u64,
    id: String,
    action: String,
}

#[derive(Debug, Deserialize)]
struct EvaluationCase {
    name: String,
    context: Option<String>,
    faculty: String,
    target: String,
    #[serde(default)]
    sources: Vec<GrantInput>,
    #[serde(default)]
    states: Vec<StateInput>,
    #[serde(default)]
    judgments: Vec<JudgmentInput>,
    expected: ExpectedEvaluation,
}

#[derive(Debug, Deserialize)]
struct GrantInput {
    source: String,
    global: bool,
    topics: Vec<String>,
    rank: u32,
}

#[derive(Debug, Deserialize)]
struct StateInput {
    source: String,
    scope: String,
    completeness: String,
}

#[derive(Debug, Deserialize)]
struct JudgmentInput {
    author: String,
    scope: String,
    action: String,
    time: u64,
}

#[derive(Debug, Deserialize)]
struct ExpectedEvaluation {
    status: String,
    value: Option<bool>,
    action: Option<String>,
    certainty: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VisibilityCase {
    name: String,
    judgments: Vec<VisibilityJudgment>,
    #[serde(default = "bob")]
    author: String,
    created_at: u64,
    first_seen: Option<u64>,
    eligible: Option<bool>,
    exclusion: Option<String>,
    #[serde(default)]
    silence_effective: bool,
}

fn bob() -> String {
    "bob".to_owned()
}

#[derive(Debug, Deserialize)]
struct VisibilityJudgment {
    faculty: String,
    action: String,
    since: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PinCase {
    name: String,
    pins: Vec<SupportInput>,
    #[serde(default)]
    dismissed: Vec<String>,
    #[serde(default)]
    ineligible: Vec<String>,
    #[serde(default)]
    unknown: Vec<String>,
    expected: Vec<String>,
    counts: Vec<usize>,
    #[serde(default)]
    incomplete: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SupportInput {
    source: String,
    target: String,
    time: u64,
}

#[derive(Debug, Deserialize)]
struct ReverseCase {
    name: String,
    blocks: Vec<ReverseSupportInput>,
    #[serde(default)]
    unknown: Vec<String>,
    expected: Vec<String>,
    counts: Vec<usize>,
    #[serde(default)]
    incomplete: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReverseSupportInput {
    source: String,
    target: String,
    time: u64,
}

#[derive(Debug, Deserialize)]
struct RescueCase {
    persona: String,
    target: String,
    scope: String,
    follow_action: String,
    follow_scope: String,
    unblock_action: String,
    unblock_scope: String,
}

struct Resolver<'a> {
    suite: &'a Suite,
}

impl Resolver<'_> {
    fn person(&self, alias: &str) -> PublicKey {
        PublicKey::parse(self.suite.identities.get(alias).unwrap()).unwrap()
    }

    fn content(&self, alias: &str) -> ContentTarget {
        ContentTarget::Event(EventId::parse(self.suite.content.get(alias).unwrap()).unwrap())
    }

    fn scope(value: &str) -> Scope {
        if value == "global" {
            Scope::Global
        } else {
            Scope::Topic(Topic::parse(value.strip_prefix("topic:").unwrap()).unwrap())
        }
    }

    fn target(&self, value: &str) -> Target {
        let (kind, alias) = value.split_once(':').unwrap();
        match kind {
            "p" => Target::Person(self.person(alias)),
            "e" => Target::Content(self.content(alias)),
            _ => panic!("unknown vector target kind"),
        }
    }
}

fn suite() -> Suite {
    serde_json::from_str(VECTORS).unwrap()
}

#[test]
fn normative_schemas_are_valid_versioned_json() {
    for (name, schema) in [
        ("configuration", CONFIG_SCHEMA),
        ("judgment", JUDGMENT_SCHEMA),
        ("event", EVENT_SCHEMA),
        ("vectors", VECTOR_SCHEMA),
    ] {
        let parsed: serde_json::Value = serde_json::from_str(schema).unwrap();
        assert_eq!(
            parsed["$schema"], "https://json-schema.org/draft/2020-12/schema",
            "{name} schema"
        );
        assert!(parsed["$id"].as_str().is_some(), "{name} schema");
    }
}

#[allow(clippy::too_many_arguments)]
fn judgment(
    resolver: &Resolver<'_>,
    author: &str,
    faculty: Faculty,
    scope: &str,
    target: &str,
    action: &str,
    time: u64,
    since: Option<u64>,
) -> Judgment {
    Judgment {
        author: resolver.person(author),
        faculty,
        scope: Resolver::scope(scope),
        target: resolver.target(target),
        action: action.parse().unwrap(),
        created_at: time,
        event_id: None,
        since,
        reason: None,
        evidence: JudgmentEvidence::Local,
    }
}

#[test]
fn normative_topics_addresses_and_actions() {
    let suite = suite();
    assert_eq!(suite.version, "flocking-vectors/1");
    let resolver = Resolver { suite: &suite };
    for case in &suite.topic_cases {
        let parsed = Topic::parse(&case.input);
        if case.error {
            assert!(parsed.is_err(), "topic case accepted: {}", case.input);
        } else {
            assert_eq!(parsed.unwrap().as_str(), case.expected.as_deref().unwrap());
        }
    }
    for case in &suite.address_cases {
        assert_eq!(
            address(
                case.faculty.parse().unwrap(),
                &Resolver::scope(&case.scope),
                &resolver.target(&case.target),
            ),
            case.expected
        );
    }
    let actions: BTreeSet<_> = suite
        .action_cases
        .iter()
        .map(|case| case.action.as_str())
        .collect();
    assert!(
        [
            "follow",
            "unfollow",
            "block",
            "unblock",
            "silence",
            "unsilence",
            "hide",
            "unhide",
            "remove",
            "restore",
            "pin",
            "withdraw"
        ]
        .into_iter()
        .all(|action| actions.contains(action))
    );
    for case in &suite.action_cases {
        let value = judgment(
            &resolver,
            "alice",
            case.faculty.parse().unwrap(),
            &case.scope,
            &case.target,
            &case.action,
            10,
            case.since,
        );
        assert_eq!(
            value.validate().is_ok(),
            case.valid,
            "action case: {case:?}"
        );
    }
}

#[test]
fn normative_current_selection() {
    let suite = suite();
    let resolver = Resolver { suite: &suite };
    for case in &suite.selection_cases {
        let events = case
            .events
            .iter()
            .map(|event| {
                let mut value = judgment(
                    &resolver,
                    "source1",
                    Faculty::Block,
                    "global",
                    "p:bob",
                    &event.action,
                    event.time,
                    None,
                );
                value.event_id = Some(event_id(&event.id));
                value.evidence = JudgmentEvidence::FlockingEvent;
                value
            })
            .collect::<Vec<_>>();
        assert_eq!(
            select_current(&events).unwrap().event_id,
            Some(event_id(&case.expected)),
            "selection case: {}",
            case.name
        );
    }
}

fn event_id(alias: &str) -> EventId {
    let digit = alias.strip_prefix("event").unwrap();
    EventId::parse(digit.repeat(64)).unwrap()
}

#[test]
fn normative_precedence_and_completeness() {
    let suite = suite();
    let resolver = Resolver { suite: &suite };
    for case in &suite.evaluation_cases {
        let faculty: Faculty = case.faculty.parse().unwrap();
        let config = Config {
            version: CONFIG_VERSION.to_owned(),
            persona: resolver.person("alice"),
            sources: case
                .sources
                .iter()
                .map(|source| Source {
                    pubkey: resolver.person(&source.source),
                    grants: vec![FacultyGrant {
                        faculty,
                        global: source.global,
                        topics: source
                            .topics
                            .iter()
                            .map(|topic| Topic::parse(topic).unwrap())
                            .collect(),
                        rank: Some(source.rank),
                    }],
                    reverse_blocks: None,
                })
                .collect(),
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: Vec::new(),
        };
        let judgments = case
            .judgments
            .iter()
            .map(|value| {
                judgment(
                    &resolver,
                    &value.author,
                    faculty,
                    &value.scope,
                    &case.target,
                    &value.action,
                    value.time,
                    None,
                )
            })
            .collect::<Vec<_>>();
        let states = case
            .states
            .iter()
            .map(|state| SourceState {
                source: resolver.person(&state.source),
                faculty,
                scope: Resolver::scope(&state.scope),
                completeness: completeness(&state.completeness),
            })
            .collect::<Vec<_>>();
        let context = Context {
            topic: case
                .context
                .as_ref()
                .map(|value| Topic::parse(value).unwrap()),
        };
        let result = evaluate(
            EvaluationInput {
                config: &config,
                judgments: &judgments,
                source_states: &states,
                context: &context,
            },
            faculty,
            &resolver.target(&case.target),
        )
        .unwrap();
        assert_evaluation(&result, &case.expected, &case.name);
    }
}

fn completeness(value: &str) -> Completeness {
    match value {
        "complete" => Completeness::Complete,
        "stale" => Completeness::Stale,
        "unknown" => Completeness::Unknown,
        _ => panic!("unknown completeness"),
    }
}

fn assert_evaluation(result: &Evaluation, expected: &ExpectedEvaluation, name: &str) {
    match (expected.status.as_str(), result) {
        ("indeterminate", Evaluation::Indeterminate { .. }) => {}
        (
            "determinate",
            Evaluation::Determinate {
                effective: Some(effective),
                certainty,
            },
        ) => {
            assert_eq!(Some(effective.value), expected.value, "{name}");
            assert_eq!(
                Some(effective.action),
                expected.action.as_deref().map(parse_action),
                "{name}"
            );
            assert_eq!(
                *certainty,
                parse_certainty(expected.certainty.as_deref().unwrap()),
                "{name}"
            );
        }
        _ => panic!("unexpected evaluation for {name}: {result:?}"),
    }
}

fn parse_action(value: &str) -> Action {
    value.parse().unwrap()
}

fn parse_certainty(value: &str) -> Certainty {
    match value {
        "current" => Certainty::Current,
        "stale" => Certainty::Stale,
        _ => panic!("unknown certainty"),
    }
}

#[test]
fn normative_visibility() {
    let suite = suite();
    let resolver = Resolver { suite: &suite };
    for case in &suite.visibility_cases {
        let config = empty_config(&resolver);
        let judgments = case
            .judgments
            .iter()
            .map(|value| {
                let faculty: Faculty = value.faculty.parse().unwrap();
                let target = if matches!(faculty, Faculty::Block | Faculty::Silence) {
                    "p:bob"
                } else {
                    "e:post1"
                };
                judgment(
                    &resolver,
                    "alice",
                    faculty,
                    "global",
                    target,
                    &value.action,
                    10,
                    value.since,
                )
            })
            .collect::<Vec<_>>();
        let context = Context::default();
        let contribution = flocking_core::Contribution {
            author: resolver.person(&case.author),
            target: resolver.content("post1"),
            created_at: case.created_at,
            first_seen: case.first_seen,
        };
        let result = evaluate_visibility(VisibilityInput {
            evaluation: EvaluationInput {
                config: &config,
                judgments: &judgments,
                source_states: &[],
                context: &context,
            },
            contribution: &contribution,
        })
        .unwrap();
        if let Some(eligible) = case.eligible {
            assert_eq!(result.eligible, Some(eligible), "{}", case.name);
        }
        if let Some(exclusion) = &case.exclusion {
            let actual = match result.exclusion {
                Some(Exclusion::Block) => "block",
                Some(Exclusion::Silence {
                    local_timing_evidence: true,
                    ..
                }) => "silence_local",
                Some(Exclusion::Silence { .. }) => "silence",
                Some(Exclusion::Hide) => "hide",
                Some(Exclusion::CommunityRemoval) => "community_removal",
                None => "none",
            };
            assert_eq!(actual, exclusion, "{}", case.name);
        }
        if case.silence_effective {
            assert!(matches!(
                result.silence,
                Evaluation::Determinate {
                    effective: Some(ref effective),
                    ..
                } if effective.value
            ));
        }
    }
}

fn empty_config(resolver: &Resolver<'_>) -> Config {
    Config {
        version: CONFIG_VERSION.to_owned(),
        persona: resolver.person("alice"),
        sources: Vec::new(),
        appearance_sources: BTreeSet::new(),
        local_pin_dismissals: Vec::new(),
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn normative_pins() {
    let suite = suite();
    let resolver = Resolver { suite: &suite };
    for case in &suite.pin_cases {
        let all_sources = ["source1", "source2", "source3"];
        let topic = Topic::parse("science").unwrap();
        let config = Config {
            version: CONFIG_VERSION.to_owned(),
            persona: resolver.person("alice"),
            sources: all_sources
                .iter()
                .map(|source| Source {
                    pubkey: resolver.person(source),
                    grants: vec![FacultyGrant {
                        faculty: Faculty::Pin,
                        global: false,
                        topics: BTreeSet::from([topic.clone()]),
                        rank: None,
                    }],
                    reverse_blocks: None,
                })
                .collect(),
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: case
                .dismissed
                .iter()
                .map(|target| LocalPinDismissal {
                    topic: topic.clone(),
                    target_type: PinTargetType::Event,
                    target: match resolver.content(target) {
                        ContentTarget::Event(id) => id.to_string(),
                        ContentTarget::Address(_) => unreachable!(),
                    },
                })
                .collect(),
        };
        let judgments = case
            .pins
            .iter()
            .map(|pin| Judgment {
                author: resolver.person(&pin.source),
                faculty: Faculty::Pin,
                scope: Scope::Topic(topic.clone()),
                target: Target::Content(resolver.content(&pin.target)),
                action: Action::Pin,
                created_at: pin.time,
                event_id: None,
                since: None,
                reason: None,
                evidence: JudgmentEvidence::Local,
            })
            .collect::<Vec<_>>();
        let states = all_sources
            .iter()
            .map(|source| SourceState {
                source: resolver.person(source),
                faculty: Faculty::Pin,
                scope: Scope::Topic(topic.clone()),
                completeness: if case.unknown.iter().any(|unknown| unknown == source) {
                    Completeness::Unknown
                } else {
                    Completeness::Complete
                },
            })
            .collect::<Vec<_>>();
        let ineligible = case
            .ineligible
            .iter()
            .map(|target| resolver.content(target))
            .collect();
        let result = evaluate_pins(PinInput {
            config: &config,
            judgments: &judgments,
            source_states: &states,
            topic: &topic,
            ineligible: &ineligible,
        })
        .unwrap();
        let expected = case
            .expected
            .iter()
            .map(|target| resolver.content(target))
            .collect::<Vec<_>>();
        assert_eq!(
            result
                .pins
                .iter()
                .map(|pin| pin.target.clone())
                .collect::<Vec<_>>(),
            expected,
            "{}",
            case.name
        );
        assert_eq!(
            result
                .pins
                .iter()
                .map(|pin| pin.source_count)
                .collect::<Vec<_>>(),
            case.counts,
            "{}",
            case.name
        );
        assert_eq!(
            result.incomplete_sources,
            case.incomplete
                .iter()
                .map(|source| resolver.person(source))
                .collect::<Vec<_>>(),
            "{}",
            case.name
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn normative_reverse_and_rescue() {
    let suite = suite();
    let resolver = Resolver { suite: &suite };
    for case in &suite.reverse_cases {
        let all_sources = ["source1", "source2", "source3"];
        let config = Config {
            version: CONFIG_VERSION.to_owned(),
            persona: resolver.person("alice"),
            sources: all_sources
                .iter()
                .map(|source| Source {
                    pubkey: resolver.person(source),
                    grants: Vec::new(),
                    reverse_blocks: Some(ReverseBlockGrant {
                        global: true,
                        topics: BTreeSet::new(),
                    }),
                })
                .collect(),
            appearance_sources: BTreeSet::new(),
            local_pin_dismissals: Vec::new(),
        };
        let judgments = case
            .blocks
            .iter()
            .map(|block| Judgment {
                author: resolver.person(&block.source),
                faculty: Faculty::Block,
                scope: Scope::Global,
                target: Target::Person(resolver.person(&block.target)),
                action: Action::Block,
                created_at: block.time,
                event_id: None,
                since: None,
                reason: None,
                evidence: JudgmentEvidence::Local,
            })
            .collect::<Vec<_>>();
        let states = all_sources
            .iter()
            .map(|source| SourceState {
                source: resolver.person(source),
                faculty: Faculty::Block,
                scope: Scope::Global,
                completeness: if case.unknown.iter().any(|unknown| unknown == source) {
                    Completeness::Unknown
                } else {
                    Completeness::Complete
                },
            })
            .collect::<Vec<_>>();
        let context = Context::default();
        let result = evaluate_reverse(ReverseInput {
            config: &config,
            judgments: &judgments,
            source_states: &states,
            context: &context,
        })
        .unwrap();
        assert_eq!(
            result
                .targets
                .iter()
                .map(|target| target.target.clone())
                .collect::<Vec<_>>(),
            case.expected
                .iter()
                .map(|target| resolver.person(target))
                .collect::<Vec<_>>(),
            "{}",
            case.name
        );
        assert_eq!(
            result
                .targets
                .iter()
                .map(|target| target.source_count)
                .collect::<Vec<_>>(),
            case.counts,
            "{}",
            case.name
        );
        assert_eq!(
            result.incomplete_sources,
            case.incomplete
                .iter()
                .map(|source| resolver.person(source))
                .collect::<Vec<_>>(),
            "{}",
            case.name
        );
    }

    let rescue_case = &suite.rescue;
    let result: Rescue = rescue(
        &resolver.person(&rescue_case.persona),
        &resolver.person(&rescue_case.target),
        Resolver::scope(&rescue_case.scope),
        10,
    );
    assert_eq!(
        result.follow.action,
        parse_action(&rescue_case.follow_action)
    );
    assert_eq!(
        result.follow.scope,
        Resolver::scope(&rescue_case.follow_scope)
    );
    assert_eq!(
        result.unblock.action,
        parse_action(&rescue_case.unblock_action)
    );
    assert_eq!(
        result.unblock.scope,
        Resolver::scope(&rescue_case.unblock_scope)
    );
    assert_eq!(suite.fallback_cases.len(), 5);
}
