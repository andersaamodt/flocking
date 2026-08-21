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

Flocking does not produce objective group membership. It reconstructs much of
the embodied experience of an objective forum: people can enter the same named
place, encounter largely shared boundaries, and understand what belongs there,
while every boundary remains the result of inspectable and revocable choices
about whose judgments to follow.

The familiar objective forum is therefore a possible convergent experience,
not Flocking's constitutional starting point.
