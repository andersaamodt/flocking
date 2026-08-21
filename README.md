# flocking

**Flocking is the voluntary following of another user's published
community-shaping judgments, allowing shared spaces to acquire increasingly
consensual boundaries without an objective moderator class.**

Traditional centralized moderation is the degenerate case in which everyone
is compelled to accept the same privileged set of judgments.

Flocking does not abolish exclusion. It makes exclusion perspectival,
voluntary, inspectable, and revocable.

Local actions alter your view. Published actions make claims about shared
space. Flocking turns those published claims into voluntarily shared practice.

Publication is the moment an individual judgment enters the social field.

## Actions

Applications can present flocking as ordinary actions. A local action changes
the actor's view; publishing makes an inspectable judgment that others may
voluntarily adopt. No action deletes a Nostr event or prevents its author from
publishing.

| Flocking action | Traditional moderator action |
| --- | --- |
| **Follow** people and let others adopt your follows. | **Join** a community whose membership is the same for everyone. |
| **Remove** items for yourself and those who follow your removals. | **Remove** items for everyone. |
| **Block** users for yourself and those who follow your blocks. | **Ban** users for everyone. |
| **Silence** users for yourself and those who follow your silences. | **Mute** users for everyone. |
| **Hide** items for yourself and those who follow your hides. | **Delete** items for everyone. |
| **Pin** items for yourself and those who follow your pins. | **Pin** items for everyone. |

### Block and silence

Block and silence exclude activity by the same person, but they differ in time:

- A block excludes the target's past and future activity in the applicable
  scope while the block is effective.
- A silence excludes contributions and revisions at or after its cutoff while
  the silence is effective, leaving earlier activity eligible.

Nostr events contain an author-controlled signed `created_at` time rather than
an objectively proven publication time. An author can therefore try to evade a
silence by backdating a new event. That false timestamp is visible to everyone,
not only the person applying the silence, and clients may use locally recorded
first-seen time as contrary evidence. Backdating cannot evade a block because
a block does not consult event time.

## Status

Flocking v1 now has an independent Rust reference implementation and an
authoritative experimental [specification](SPEC.md). Hydra integration remains
the next adopter step rather than part of this library's authority.

The workspace exposes two deliberately narrow crates:

- `flocking-core` validates configuration and judgments, resolves current
  state, evaluates precedence and visibility, aggregates pins, performs Reverse
  Flocking, and constructs Rescue transactions without I/O.
- `flocking-nostr` verifies and parses kind `30820` events, builds unsigned
  events for host-controlled signing, adapts NIP-02 and NIP-51 fallback inputs,
  and computes compatibility mirrors without relay access.

The [schemas](schemas) describe portable JSON boundaries, and the
[normative vectors](vectors/flocking-v1.json) are executed by the test suite.
The living [roadmap](ROADMAP.md) records the route from this implementation to
Hydra adoption, an independent second client, and a possible narrow NIP.

## Using the library

During the experimental phase, applications should use Git or path
dependencies so the chosen specification revision remains explicit:

```toml
[dependencies]
flocking-core = { git = "https://github.com/andersaamodt/flocking" }
flocking-nostr = { git = "https://github.com/andersaamodt/flocking" }
```

Call `flocking_nostr::parse_judgment` at the protocol boundary, retain relay
completeness as `SourceState`, and pass validated judgments plus local `Config`
to `flocking_core::evaluate`. Use `evaluate_visibility`, `evaluate_pins`, and
`evaluate_reverse` only for their named compositions.

The library does not fetch relays, store configuration, read the wall clock,
hold signing keys, publish events, choose UI behavior, or integrate with Hydra.

## Verification

Run the complete formatting, lint, unit, adapter, and normative-vector suite:

```sh
./.tests/run
```

The runner keeps compiled output in a temporary directory and removes it when
the run ends.

## License

Flocking is available under the Open Wizardry License 3.1. See
[LICENSE](LICENSE).
