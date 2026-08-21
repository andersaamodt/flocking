use flocking_core::{Action, Faculty, JudgmentEvidence, canonical_current};
use flocking_nostr::{nip02_fallback, nip51_block_fallback};
use nostr::{EventBuilder, Keys, Kind, Tag, Timestamp};
use serde::Deserialize;

const VECTORS: &str = include_str!("../../../vectors/flocking-v1.json");

#[derive(Debug, Deserialize)]
struct Suite {
    fallback_cases: Vec<FallbackCase>,
}

#[derive(Debug, Deserialize)]
struct FallbackCase {
    name: String,
    kind: Option<u16>,
    faculty: Option<String>,
    action: Option<String>,
    #[serde(default)]
    absence: bool,
    canonical_action: Option<String>,
    fallback_action: Option<String>,
    expected: Option<String>,
}

#[test]
fn normative_standard_event_fallbacks() {
    let suite: Suite = serde_json::from_str(VECTORS).unwrap();
    let keys = Keys::generate();
    let person = "1".repeat(64);
    for case in suite
        .fallback_cases
        .iter()
        .filter(|case| case.kind.is_some())
    {
        let kind = case.kind.unwrap();
        let mut builder =
            EventBuilder::new(Kind::Custom(kind), "").custom_created_at(Timestamp::from_secs(10));
        if !case.absence {
            builder = builder.tag(Tag::parse(["p", person.as_str()]).unwrap());
        }
        let event = builder.sign_with_keys(&keys).unwrap();
        let judgments = match kind {
            3 => nip02_fallback(&event).unwrap(),
            10_000 => nip51_block_fallback(&event).unwrap(),
            _ => panic!("unexpected fallback kind"),
        };
        if case.absence {
            assert!(judgments.is_empty(), "{}", case.name);
        } else {
            assert_eq!(judgments.len(), 1, "{}", case.name);
            assert_eq!(
                judgments[0].faculty,
                case.faculty.as_deref().unwrap().parse::<Faculty>().unwrap(),
                "{}",
                case.name
            );
            assert_eq!(
                judgments[0].action,
                case.action.as_deref().unwrap().parse::<Action>().unwrap(),
                "{}",
                case.name
            );
        }
    }
}

#[test]
fn normative_canonical_withdrawal_suppresses_fallback() {
    let suite: Suite = serde_json::from_str(VECTORS).unwrap();
    let case = suite
        .fallback_cases
        .iter()
        .find(|case| case.canonical_action.is_some())
        .unwrap();
    let keys = Keys::generate();
    let person = "1".repeat(64);
    let event = EventBuilder::new(Kind::Custom(10_000), "")
        .tag(Tag::parse(["p", person.as_str()]).unwrap())
        .custom_created_at(Timestamp::from_secs(10))
        .sign_with_keys(&keys)
        .unwrap();
    let fallback = nip51_block_fallback(&event).unwrap().remove(0);
    assert_eq!(
        fallback.action.to_string(),
        case.fallback_action.as_deref().unwrap()
    );
    let mut canonical = fallback.clone();
    canonical.action = case.canonical_action.as_deref().unwrap().parse().unwrap();
    canonical.created_at = 11;
    canonical.evidence = JudgmentEvidence::FlockingEvent;
    let current = canonical_current(&[fallback, canonical]);
    assert_eq!(current.len(), 1);
    assert_eq!(
        current[0].action.to_string(),
        case.expected.as_deref().unwrap()
    );
}
