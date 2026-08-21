# Flocking

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

## Block and silence

Block and silence exclude activity by the same target, but they differ in
time:

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

Neither action deletes Nostr events or prevents the target from publishing.
They determine the effective view of the person applying or flocking after the
judgment.

## Status

Flocking v1 has an authoritative experimental [specification](SPEC.md). The
living [roadmap](ROADMAP.md) records the route from an independent reference
library and experimental wire format to a possible NIP after multiple client
implementations.
