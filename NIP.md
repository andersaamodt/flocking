# NIP-XX

## Flocking: Voluntary Community-Shaping Judgments

`draft` `optional`

This NIP defines addressable events through which a user publishes current,
scoped judgments about people and content. Other users may independently choose
which authors' judgments to incorporate into their own views and may revoke
that choice at any time.

Flocking does not define moderators, authoritative community state, or relay
enforcement. A judgment changes no underlying event and has no effect for a
viewer who has not chosen to use it.

## Kinds

This NIP defines two addressable event kinds:

- `kind:30820` publishes one current judgment about one faculty, scope, and
  target.
- `kind:30821` publishes one user's current image choice for one ownerless
  topic.

Relays require no behavior beyond ordinary NIP-01 handling of addressable
events.

## Motivation

Existing follow and mute lists provide useful compatibility state but cannot
express one current per-target contrary judgment, explicit withdrawal, topic
scope, or prospective silence. Reports request action from another party, and
labels do not by themselves define addressable current judgment state.

This NIP supplies one optional event shape for those meanings while leaving
every viewer free to choose which authors and faculties affect their view.

## Terms

- A **faculty** is one class of judgment, such as `block` or `pin`.
- A **scope** is either `global` or one normalized bare topic.
- A **target** is a public key, event ID, or addressable-event coordinate.
- A **source** is a pubkey whose authored judgments a viewer has chosen to use.
- A **direct judgment** is authored by the viewer.
- A **flocked judgment** is authored by a selected source.
- A **withdrawal** means that the author no longer takes a position on one
  faculty-scope-target tuple. It is not the contrary judgment.

Source selection and source ranking are local client configuration. They MUST
NOT be inferred from follower counts, relay policy, or a moderator registry.
Clients MUST use only a source's authored judgments, not the effective state
that source derives from other people. Flocking is therefore non-recursive.

## Topics and scopes

A topic is normalized by trimming surrounding whitespace and lowercasing ASCII
letters. The result MUST contain between 1 and 64 characters, each of which is
`a-z`, `0-9`, or `_`.

Paths, relay groups, and event coordinates are not topics. For example,
`science` is valid, while `/h/science`, `/r/science`, and `science-news` are
not. Applications MAY present multiple views of the same bare topic.

The canonical scope key is:

- `global` for global scope; or
- `topic:<topic>` for topic scope.

A topic-scoped judgment is considered only while rendering that topic. A
global judgment may be considered in topic, profile, search, and context-free
views. Neither scope changes the universal visibility or contents of the
target event.

## Judgment event

A judgment is a `kind:30820` event with the following form:

```jsonc
{
  "kind": 30820,
  "created_at": 1730000000,
  "tags": [
    ["d", "flocking:v1:<address-digest>"],
    ["v", "flocking/1"],
    ["f", "block"],
    ["j", "block"],
    ["c", "topic"],
    ["t", "science"],
    ["p", "<target-pubkey>"]
  ],
  "content": "Off-topic promotion across several threads"
}
```

The event MUST contain exactly one each of the following tags:

| Tag | Meaning |
| --- | --- |
| `d` | Stable address derived from the semantic tuple. |
| `v` | Protocol version; this NIP defines `flocking/1`. |
| `f` | Faculty. |
| `j` | Current action. |
| `c` | Scope form: `global` or `topic`. |

A topic-scoped event MUST contain exactly one `t` tag containing the normalized
topic. A global event MUST NOT contain a `t` tag.

The event MUST contain exactly one semantic target tag:

- `p` for a 32-byte lowercase hexadecimal public key;
- `e` for a 32-byte lowercase hexadecimal immutable event ID; or
- `a` for a canonical NIP-01 addressable-event coordinate.

A content target MUST identify the logical object being judged. An immutable
object may use its event ID. An editable addressable object MUST use its stable
`a` coordinate rather than the event ID of one revision. A coordinate and its
canonical target key MUST NOT contain a NUL byte.

Relay hints in target tags MAY be included as specified by NIP-01 and are not
part of target identity. A client MUST reject duplicate required tags, more
than one semantic target tag, an unknown version, or an unsupported combination
of faculty, scope, target, and action. Unknown nonsemantic tags MAY be ignored.

The content is an optional human-readable reason. It MUST be no more than 500
UTF-8 bytes. An empty content field means that no reason was supplied. A reason
is the author's public statement and MUST NOT be treated as proof.

### Faculties and actions

| Faculty | Target | Scope | Valid actions |
| --- | --- | --- | --- |
| `follow` | `p` | global | `follow`, `unfollow`, `withdraw` |
| `block` | `p` | global or topic | `block`, `unblock`, `withdraw` |
| `silence` | `p` | global or topic | `silence`, `unsilence`, `withdraw` |
| `hide` | `e` or `a` | global or topic | `hide`, `unhide`, `withdraw` |
| `community_membership` | `e` or `a` | topic | `remove`, `restore`, `withdraw` |
| `pin` | `e` or `a` | topic | `pin`, `withdraw` |

There is no negative pin action. Withdrawing a pin removes only that author's
support for the target; it does not oppose another author's pin.

### Stable address

Each author has one addressable event per faculty-scope-target tuple. The
canonical target key is `p:<pubkey>`, `e:<event-id>`, or `a:<coordinate>`.

The address material is the UTF-8 encoding of these four values, joined by a
single NUL byte in this order:

```text
flocking/1
<faculty>
<scope-key>
<target-key>
```

Equivalently:

```text
flocking/1\0<faculty>\0<scope-key>\0<target-key>
```

The `d` value is `flocking:v1:` followed by the lowercase hexadecimal SHA-256
digest of that material. A client MUST recompute the address and reject an
event whose `d` tag does not match. Changing the action, reason, or silence
cutoff does not change the address.

### Current state

For one `kind`, author, and `d` value, the valid event with the greatest
`created_at` is current. When timestamps are equal, the event with the
lexicographically lowest event ID is current, following NIP-01.

An invalid newer event MUST NOT displace the newest valid event. An explicit
`withdraw` is known current state. Event absence, relay unavailability, and a
NIP-09 deletion request MUST NOT be interpreted as withdrawal.

## Silence cutoff

A `silence` event MUST contain exactly one `since` tag holding a non-negative
integer Unix timestamp. The value MUST NOT exceed the judgment event's
`created_at` and SHOULD initially equal it. No other action may contain a
`since` tag.

Reissuing one uninterrupted silence MAY retain the original cutoff. A new
silence after `unsilence` or `withdraw` MUST use a new cutoff.

Silence excludes contributions by the target whose signed `created_at` is at
or after the effective cutoff. A newly signed revision is a new contribution
for this comparison. Clients MAY retain local first-seen times and
conservatively exclude an apparently backdated event observed after the
cutoff, but this local evidence is not portable.

Block excludes the target's past and future contributions without consulting
their timestamps. Block therefore controls visibility while both block and
silence are effective, without erasing the silence judgment.

## Determining effective judgments

For faculties other than `pin`, a client evaluates each faculty and target
independently in this order:

1. the viewer's topic-scoped judgment, when in a topic context;
2. the viewer's global judgment;
3. selected sources' topic-scoped judgments, in local source-rank order;
4. selected sources' global judgments, in local source-rank order;
5. no judgment.

A lower numeric source rank has higher precedence. Ranks MUST be unique within
one faculty. A source may be enabled independently for each faculty and for
global and topic scopes.

At each step, absence and `withdraw` are skipped. An affirmative contrary
action such as `unblock`, `unhide`, or `restore` is not skipped: it determines
that faculty when it is the highest applicable judgment.

A client MUST NOT treat a failed or incomplete relay query as proof that a
source has no judgment. If missing higher-precedence source state could change
the result, the result is indeterminate. Clients SHOULD preserve the winning
author, event ID, faculty, scope, action, cutoff when applicable, and local
source rank so the result remains inspectable.

### Faculty composition

- `follow` adds the target to the viewer's effective follows; `unfollow`
  excludes it from effective follows. A flocked follow MUST NOT be copied into
  the viewer's authored NIP-02 list.
- `block` excludes past and future contributions authored by the target.
- `silence` excludes contributions authored by the target at or after its
  cutoff.
- `hide` excludes one logical content object from ordinary rendering in its
  effective scope.
- `remove` rejects one content object's claimed membership in one topic;
  `restore` accepts it with respect to that faculty.
- Ending an exclusion makes the stored content eligible again under that
  faculty, in its original chronology.

Block and silence apply only to contributions authored by their person target.
Hide and removal apply only to their logical content target. None of these
judgments automatically excludes descendants authored by other people.

An effective block is considered before silence. An effective author
exclusion is considered before hide and community membership. Restoration does
not override a hide, block, or silence, and unhide does not override an author
exclusion.

Ending an exclusion MUST NOT generate retroactive notifications or present
newly eligible content as newly published.

## Pin aggregation

Pin is an aggregation exception to ordinary source precedence.

Current direct pins are listed before flocked pins and are ordered by newest
support, then canonical target key. A flocked target receives at most one unit
of support from each selected source with a current `pin` action in that topic.
Flocked targets are ordered by:

1. greatest number of distinct supporting sources;
2. newest current supporting pin event; and
3. lexicographically lowest canonical target key.

Source rank does not affect pin support. A target excluded by an effective
block, silence, hide, or removal is ineligible for display as a pin, but its
authored pin state remains current and may become eligible again.

Clients MAY let a viewer locally dismiss an inherited pin. Such a dismissal is
local configuration and MUST NOT be published as a judgment.

## Community appearance event

A `kind:30821` event publishes one user's current image choice for one topic:

```jsonc
{
  "kind": 30821,
  "tags": [
    ["d", "science"],
    ["v", "flocking/1"],
    ["t", "science"],
    ["j", "set"],
    ["url", "https://images.example/science.png"],
    ["x", "<lowercase-sha256-of-image-bytes>"],
    ["m", "image/png"],
    ["dim", "256x256"],
    ["alt", "A blue atom"]
  ],
  "content": ""
}
```

The `d` and `t` values MUST both be the same normalized bare topic. The event
MUST contain exactly one each of `d`, `v`, `t`, and `j`.

For `j=set`, it MUST also contain exactly one each of:

- `url`: an HTTPS URL no longer than 2048 bytes;
- `x`: the lowercase hexadecimal SHA-256 hash of the image bytes;
- `m`: `image/png`, `image/jpeg`, or `image/webp`;
- `dim`: non-zero `WIDTHxHEIGHT`, with neither dimension above 4096; and
- `alt`: alternative text containing a non-whitespace character and no more
  than 280 UTF-8 bytes.

Before displaying the image, a client MUST download it over HTTPS and verify
its bytes against `x`. The downloaded file's detected type MUST agree with `m`.

For `j=withdraw`, the event MUST NOT contain `url`, `x`, `m`, `dim`, or `alt`.
Withdrawal removes only that author's current image choice.

A direct current `set` by the viewer is used first. Otherwise, a client may use
current choices from sources explicitly selected for community appearance.
Choices with the same `x` hash aggregate one unit of support per distinct
source. Candidates are ordered by greatest source support, newest supporting
choice, then lexicographically lowest complete image metadata. A direct
withdrawal returns resolution to the selected sources.

The topic identifier is not replaced by image metadata. When no effective
choice exists, clients SHOULD use a deterministic local fallback rather than
implying an authoritative owner.

## Compatibility with existing NIPs

A client MAY use membership in the author's current NIP-02 kind `3` follow list
as a fallback positive global `follow`. Absence from that list means no
judgment, not `unfollow`.

A client MAY use a public-key entry in the author's current NIP-51 kind `10000`
mute list as a fallback positive global `block`. Absence means no judgment, not
`unblock`.

A canonical `kind:30820` event, including `withdraw`, supersedes these fallbacks
for the same author-faculty-scope-target tuple.

Authors SHOULD mirror current positive global follows into NIP-02 and MAY
mirror current positive global blocks into the public portion of NIP-51.
Silence MUST NOT be mirrored as mute because its time semantics differ.
Topic pins MUST NOT be mirrored as NIP-51 profile pins, and community removal
MUST NOT be represented as NIP-72 moderation.

NIP-56 reports are requests for action and MUST NOT be interpreted as enacted
Flocking judgments. NIP-09 deletion is likewise distinct from withdrawal.

## Privacy and security

Judgment and appearance events are public, signed statements. Their reasons,
targets, timestamps, and revision history may remain retrievable after
replacement or withdrawal. Clients MUST obtain explicit intent before
publishing them and SHOULD make the public nature of a reason clear.

Source choices and ranks may reveal sensitive social relationships. This NIP
defines no public event for them. Clients SHOULD keep them local or synchronize
them only with user-controlled encryption.

Flocking does not establish that a source is correct, that several pubkeys are
independent people, or that a topic has objective membership. Counts mean
distinct applicable pubkeys only. Clients SHOULD expose provenance and MUST
allow the viewer to stop using any source.

A malicious author can backdate a contribution to evade a portable silence
cutoff. Local first-seen evidence can mitigate this, and a block is unaffected
because it does not consult contribution time.

Clients MUST validate signatures, event IDs, semantic combinations, target
encodings, stable addresses, and bounded fields before evaluation. Unsupported
versions and malformed events MUST fail closed without displacing an older
valid current event.

## Backwards compatibility

Both kinds are optional addressable events. Relays and clients that do not
implement this NIP continue to handle them as unknown events under NIP-01.
Flocking-aware clients may use standard NIP-02 and NIP-51 inputs as the
lower-fidelity fallbacks described above.

This NIP defines no relay-enforced roles, deletion power, admission gate, or
authoritative community record.
