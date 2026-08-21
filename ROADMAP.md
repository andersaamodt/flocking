# Flocking roadmap

Flocking should begin as an independent semantic specification and reference
library. Hydra will be its first adopter, not the authority for its meaning.

## Destination

Flocking should eventually propose a narrow NIP for the interoperable wire
representation of voluntary community-shaping judgments. The NIP should not
contain Flocking's political rationale, application UX, relay access, storage,
or Hydra-specific behavior.

The NIP should be proposed after the protocol and deterministic evaluator have
been exercised by at least two independent clients. The NIPs repository asks
for two client implementations and one relay implementation where applicable,
as well as optionality and backwards compatibility:

<https://github.com/nostr-protocol/nips/blob/master/README.md#criteria-for-acceptance-in-this-repository>

## Road to a NIP

1. Publish a versioned Flocking semantic specification.
2. Define a small, experimental Nostr wire format for current judgments.
3. Publish normative schemas and deterministic test vectors.
4. Implement a pure reference evaluator with no storage, signing, relay, or UI
   dependencies.
5. Implement protocol adapters for standard Nostr events where their meaning
   fits without invention.
6. Integrate the library into Hydra as the first adopter.
7. Validate the protocol through a second independent client implementation.
8. Propose only the proven interoperable wire surface as a NIP.

## Required semantic decisions

- A current judgment has three states: positive, negative, or withdrawn.
  Withdrawal means that the author no longer judges the question and allows a
  lower-precedence judgment to become effective. It is distinct from making the
  contrary judgment.
- The normative precedence order is:

  1. direct community-scoped judgment;
  2. direct global judgment;
  3. flocked community-scoped judgment by source rank;
  4. flocked global judgment by source rank;
  5. no judgment.

- Pins are an explicit aggregation exception rather than an accidental second
  conflict algorithm.
- Silence is prospective from the publication time of the effective silence
  judgment. Beginning to flock after that source later does not move the
  cutoff: contributions before the source's judgment remain unsilenced, while
  contributions after it are silenced.
- A newly signed edit or revision after the silence cutoff is a future
  contribution and is silenced. A pre-cutoff revision of the same object may
  remain visible when available. If no pre-cutoff revision is available, the
  client may omit the object or show a provenance-bearing placeholder rather
  than expose the silenced revision. This prevents evasion by replacing the
  contents of an old object.
- When the effective silence ends, contributions and revisions from the silent
  interval become eligible again unless another judgment excludes them. They
  retain their original chronology: clients must not generate retroactive
  notifications or present them as newly published merely because the silence
  ended.
- Block and silence are independent author-exclusion faculties with one simple
  difference: silence applies prospectively from its cutoff, while block also
  excludes all past activity by the target in the applicable scope. Block
  therefore controls visibility when both apply, without erasing the stored
  silence judgment. Unblock and unsilence reverse only their corresponding
  faculties; withdrawal defers that faculty to the next applicable judgment.
- Block and silence apply only to contributions authored by their target. They
  do not automatically exclude replies or other descendants authored by other
  people. Implementations should preserve enough thread topology to make those
  descendants intelligible; the exact placeholder and reveal interaction are
  host-application policy.
- The signed event `created_at` is the normative portable time used to compare
  a contribution or revision with a silence cutoff. Because Nostr does not
  prove an objective publication time, a client may retain first-seen time as
  inspectable local evidence and conservatively silence an apparently
  backdated event it observed arriving after the cutoff. Without such local
  evidence, it must fall back to the signed time. Backdating cannot evade a
  block, which excludes activity without consulting time.
- Source input must distinguish complete, stale, and unknown state. Missing
  relay data must not silently become an empty judgment set.
- Counts mean distinct applicable source keys, not verified independent people
  or general social proof.
- Inherited state always retains its author and source-event provenance and is
  never copied into the follower's authored state.
- Flocking configuration is local by default. Portable configuration should
  have a versioned data schema, but public disclosure is not required for the
  social mechanism to work.

## Community identity

For ownerless topic communities, the canonical identifier is a normalized bare
topic such as `science`. Paths such as `/h/science` and `/r/science` are views or
projections of that topic, not different Flocking communities.

The initial protocol should specify topic normalization exactly. It should not
make Hydra paths, Reddit paths, a NIP-72 owner coordinate, or a NIP-29 relay
group the canonical identity of an ownerless topic. Adapters may map those
external structures to a bare topic only when the host application explicitly
knows that the mapping is valid.

A community-scoped judgment affects only a rendering in that bare-topic
context. The same multi-topic object may therefore be excluded in `science`
while remaining eligible in `biology`, a profile, search, or a context-free
direct view. A global judgment applies across those contexts. The judgment
never mutates the underlying object's universal visibility.

## Why existing NIPs are insufficient

- [NIP-02](https://github.com/nostr-protocol/nips/blob/master/02.md) follow lists
  provide positive authored follow state, but not an affirmative contrary
  judgment or withdrawal with per-target provenance.
- [NIP-51](https://github.com/nostr-protocol/nips/blob/master/51.md) mute lists
  are useful compatibility inputs for things a user does not want to see, but
  do not distinguish block from silence, express community scope, or preserve
  affirmative inverses. Its pin list means profile showcasing rather than
  contextual community pinning.
- [NIP-32](https://github.com/nostr-protocol/nips/blob/master/32.md) labels can
  supply auxiliary public claims, but ordinary label events are not
  addressable current-state records and do not cleanly express the relation
  among a judgment, a target, and a community scope.
- [NIP-56](https://github.com/nostr-protocol/nips/blob/master/56.md) reports
  announce objectionable material to other actors. Flocking instead propagates
  a judgment the author has enacted for their own view; it has no request to an
  authority.
- [NIP-72](https://github.com/nostr-protocol/nips/blob/master/72.md) allows
  clients to choose whose approvals they honor, but begins with an owned
  community definition and moderator registry, lacks symmetric current-state
  judgments, and is marked unrecommended.
- [NIP-29](https://github.com/nostr-protocol/nips/blob/master/29.md) defines
  relay-enforced groups, privileged roles, and authoritative group state. That
  is a different constitutional model.
- [NIP-78](https://github.com/nostr-protocol/nips/blob/master/78.md) can store
  application-specific settings, but explicitly does not establish an
  interoperable social meaning and should not be Flocking's public foundation.

Existing NIPs should be reused or mirrored where they faithfully preserve
meaning. They should not be stretched until unlike judgments appear
equivalent.

## Proposed wire shape

The leading candidate is one addressable current-state event per:

`author x faculty x scope x target`

The stable address does not change when the judgment becomes positive,
negative, or withdrawn. This permits atomic updates, explicit inverses,
per-target recency, stable provenance, and deterministic current-state
selection. Exact event kind and tags remain undecided until the semantic model
is complete.

## Decision queue

Resolve these questions one at a time before freezing the v1 wire format:

1. Silence and unsilence across time, edits, and withdrawal.
2. Canonical topic normalization and whether non-topic community identifiers
   belong in v1.
3. Pin, unpin, withdrawal, direct vetoes, and aggregation.
4. Exact target identity for immutable and addressable content.
5. Standard-event compatibility and mirroring rules.
6. Event addressing, tags, tie-breaking, and reasons.
7. Completeness and freshness contracts at the library boundary.
8. Portable local Flocking configuration.

## Language about familiar forums

Flocking does not produce objective group membership. It partially reconstructs
the practical function and embodied experience of objective membership:
people can inhabit something recognizably like the same forum--the same named
place, with a largely shared sense of who and what belongs there--without
making any person's judgment authoritative for everyone.

The familiar objective forum is therefore a possible convergent experience,
not Flocking's constitutional starting point.
