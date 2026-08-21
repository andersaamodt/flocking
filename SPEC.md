# Flocking v1 Specification

## Status and authority

- This document is the semantic authority for Flocking v1 implementations.
- Flocking v1 is experimental until its wire format has independent client implementations.
- The provisional Flocking judgment event kind is `30820`.
- A future NIP may assign a different event kind without changing Flocking's semantic model.
- Normative terms `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` use their ordinary standards meanings.

## Purpose

- Flocking voluntarily incorporates another user's published community-shaping judgments into the follower's effective view.
- Flocking allows familiar forum boundaries to emerge without an intrinsically privileged moderator class.
- Flocking makes exclusion perspectival, voluntary, inspectable, reversible, and attributable.
- Flocking is a library and protocol capability rather than a standalone social application.
- Flocking does not make any judgment universally authoritative.
- Flocking does not produce objective group membership.
- Flocking may partially reproduce the practical experience of an objective forum.

## Institutional model

- A source acquires moderator-like effect only through followers' voluntary choices.
- Following a source MUST NOT confer official moderator status on that source.
- A large follower count MUST NOT create an official duty to govern followers.
- Flocking MUST NOT define an objective moderator registry.
- Flocking MUST NOT define one authoritative community boundary.
- A familiar objective-forum experience MAY emerge from convergent voluntary choices.
- Every user retains authority to revoke a source relationship.
- Every user retains authority to make a higher-precedence direct judgment.

## Design constraints

- Flocking v1 MUST expose a small orthogonal set of concrete faculties.
- Flocking v1 MUST NOT expose a generalized judgment calculus.
- Provenance MUST remain inspectable rather than hidden in inference.
- Missing information MUST fail legibly rather than silently inventing state.
- Meaningful implementation variance requires a versioned extension.
- Host institutions MUST NOT leak into the core semantic model.
- Independent applications MUST be able to implement the specification without Hydra.

## Core terms

- A `user` is the person whose effective state is being evaluated.
- A `source` is a Nostr public key whose judgments the user has chosen to follow.
- A `faculty` is one supported class of community-shaping judgment.
- A `scope` is either global or one canonical bare topic.
- A `target` is the person or logical content object judged by a faculty.
- An `authored judgment` is a local or published judgment made directly by its author.
- A `flocked judgment` is a source's authored judgment considered in the user's evaluation.
- `Effective state` is the result of applying direct judgments, flocked judgments, scope, and precedence.
- `Withdrawal` means the author no longer judges one faculty-scope-target tuple.
- `Provenance` is the evidence explaining which authored judgments produced effective state.
- A `logical object` is stable content identity across revisions.

## Faculties

- Flocking v1 supports `follow` for person targets in global scope.
- Flocking v1 supports `block` for person targets in global or topic scope.
- Flocking v1 supports `silence` for person targets in global or topic scope.
- Flocking v1 supports `hide` for content targets in global or topic scope.
- Flocking v1 supports `community_membership` for content targets in topic scope.
- Flocking v1 supports `pin` for content targets in topic scope.
- Reverse Flocking is a derived discovery operation over block judgments.
- Unsupported faculty-scope-target combinations MUST be rejected.
- New faculties require a versioned extension or a later protocol version.

## Judgment actions

- A follow judgment has action `follow`, `unfollow`, or `withdraw`.
- A block judgment has action `block`, `unblock`, or `withdraw`.
- A silence judgment has action `silence`, `unsilence`, or `withdraw`.
- A hide judgment has action `hide`, `unhide`, or `withdraw`.
- A community-membership judgment has action `remove`, `restore`, or `withdraw`.
- A pin judgment has action `pin` or `withdraw`.
- A published `unpin` action uses `withdraw` for the author's own pin judgment.
- Flocking v1 has no negative pin judgment.
- Withdrawal allows the next applicable judgment in precedence order to determine effective state.
- An affirmative contrary action remains a judgment and participates in precedence.

## Topic identity

- A topic identifier MUST be a normalized bare topic such as `science`.
- Topic normalization MUST trim surrounding whitespace.
- Topic normalization MUST convert ASCII letters to lowercase.
- A normalized topic MUST contain only `a-z`, `0-9`, and `_`.
- A normalized topic MUST contain between 1 and 64 characters.
- A path such as `/h/science` MUST NOT be accepted as a topic identifier.
- `/h/science` and `/r/science` MAY be application views of the topic `science`.
- Display names MAY differ from canonical topic identifiers.
- Ordinary hashtags outside the topic grammar MUST NOT automatically become Flocking communities.
- Alias resolution is outside Flocking v1.

## Scope

- Global scope is represented by the canonical scope key `global`.
- Topic scope is represented by the canonical scope key `topic:<topic>`.
- Flocking v1 MUST NOT encode arbitrary application scope strings.
- Flocking v1 MUST NOT treat NIP-29 groups as native scope forms.
- Flocking v1 MUST NOT treat NIP-72 coordinates as native scope forms.
- A host MAY explicitly map an external community to a canonical bare topic.
- A topic-scoped judgment applies only while evaluating that topic context.
- A global judgment applies across topic, profile, search, and context-free views.
- One content object MAY have different effective visibility in different topic contexts.
- A scoped judgment MUST NOT mutate the target's universal visibility.

## Person and content identity

- A person target is a lowercase 32-byte Nostr public key encoded as 64 hexadecimal characters.
- An immutable content target is a lowercase 32-byte Nostr event ID encoded as 64 hexadecimal characters.
- An addressable content target is a canonical Nostr coordinate of `kind:pubkey:d`.
- A Hydra post or comment target is its immutable anchor event ID.
- A content judgment MUST target the logical object rather than one editable head revision.
- A judgment about one revision without judging its logical object is outside Flocking v1.
- Relay hints MAY accompany target tags without becoming part of target identity.
- A malformed target MUST be rejected before evaluation.

## Local and published judgments

- A local-only judgment changes authored local state without creating a Nostr event.
- A published judgment enters the public Nostr information field.
- Flocking MUST NOT invent a private form of public judgment.
- A host MUST obtain explicit user intent before publishing a judgment.
- A host MUST distinguish local-only and published actions in its interface.
- A published judgment MAY remain retrievable after it is superseded or withdrawn.
- A NIP-09 deletion request MUST NOT substitute for an explicit Flocking withdrawal.
- A reason attached to a published judgment is public speech.
- A reason MUST NOT be interpreted as proof that the judgment is correct.

## Authored and effective state

- Authored state MUST remain distinct from effective state.
- A flocked judgment MUST NOT be copied into the user's authored state.
- A flocked follow MUST NOT be copied into the user's authored follow list.
- A flocked block MUST NOT be copied into the user's authored block list.
- Removing a source MUST remove effective state inherited solely from that source.
- Superseding a source judgment MUST update dependent effective state.
- A source's effective Flocking state MUST NOT be used as source input.
- Recursive Flocking is outside Flocking v1.
- Historical judgments MAY remain available as inactive provenance.

## Judgment event

- A published Flocking judgment MUST use addressable event kind `30820`.
- A judgment event MUST be signed according to NIP-01.
- A judgment event MUST contain exactly one `d` tag.
- A judgment event MUST contain exactly one `v` tag with value `flocking/1`.
- A judgment event MUST contain exactly one `f` tag naming the faculty.
- A judgment event MUST contain exactly one `j` tag naming the action.
- A judgment event MUST contain exactly one `c` tag naming the scope form.
- A global judgment MUST use the tag `["c","global"]`.
- A topic judgment MUST use the tag `["c","topic"]`.
- A topic judgment MUST contain exactly one `t` tag with the normalized topic.
- A global judgment MUST NOT contain a `t` tag.
- A judgment event MUST contain exactly one semantic target tag.
- A person target MUST use a `p` tag.
- An immutable content target MUST use an `e` tag.
- An addressable content target MUST use an `a` tag.
- A person-target judgment MUST NOT contain an `e` or `a` tag.
- A content-target judgment MUST NOT contain a `p` tag.
- A judgment event's content MAY contain a human-readable reason.
- A reason MUST contain no more than 500 UTF-8 bytes.
- An empty content field means no reason was supplied.
- Unknown nonsemantic tags MAY be ignored.
- Duplicate required tags MUST invalidate the event.
- An unknown version MUST prevent v1 evaluation.
- An unknown faculty MUST prevent v1 evaluation.
- An unknown action MUST prevent v1 evaluation.
- An unknown scope form MUST prevent v1 evaluation.

## Stable judgment address

- One author-faculty-scope-target tuple MUST have one stable `d` value.
- The canonical target key is `p:<pubkey>`, `e:<event-id>`, or `a:<coordinate>`.
- A canonical target key MUST NOT contain a NUL byte.
- The canonical address material is `flocking/1`, faculty, scope key, and target key joined by NUL bytes in that order.
- The address digest is the SHA-256 digest of the canonical address material.
- The `d` value is `flocking:v1:<lowercase-hex-address-digest>`.
- A receiver MUST recompute the `d` value from semantic tags.
- A receiver MUST reject an event whose recomputed `d` value differs from its `d` tag.
- Changing an action MUST NOT change the tuple's `d` value.
- Changing a reason MUST NOT change the tuple's `d` value.

## Current judgment selection

- Current state is selected independently for each event author and `d` value.
- The valid event with the greatest `created_at` is current.
- Equal timestamps MUST select the event with the lexicographically lowest event ID.
- A current `withdraw` event represents known withdrawal rather than missing data.
- Older valid events MAY be retained as provenance history.
- An invalid newer event MUST NOT displace the newest valid event.
- A deleted event MUST NOT be inferred to mean withdrawal.
- An unavailable event MUST NOT be inferred to mean withdrawal.

## Silence cutoff

- A `silence` event MUST contain exactly one `since` tag with an integer Unix timestamp.
- The initial `since` value SHOULD equal the judgment event's `created_at` value.
- Reissuing one continuous silence MAY preserve its original `since` value.
- A new silence after `unsilence` or `withdraw` MUST use a new cutoff.
- A `since` value MUST NOT exceed the judgment event's `created_at` value.
- An action other than `silence` MUST NOT contain a `since` tag.
- Beginning to follow a source later MUST NOT move that source's silence cutoff.
- The signed content-event `created_at` value is the normative portable contribution time.
- A newly signed revision has its own contribution time.
- A revision at or after the silence cutoff is a future contribution.
- A pre-cutoff revision MAY remain visible when a later revision is silenced.
- A client without a pre-cutoff revision MAY omit the object or show a placeholder.
- A client MAY retain first-seen time as local evidence of backdating.
- A client MAY conservatively silence an apparently backdated event it observed arriving after the cutoff.
- A client without local timing evidence MUST use the signed event time.
- Backdating cannot evade a block because block evaluation ignores contribution time.

## Source configuration

- Source choices MUST remain local by default.
- Source ranks MUST remain local by default.
- Flocking v1 MUST NOT require publication of source relationships.
- A portable configuration MUST be encoded as a UTF-8 JSON object.
- A portable configuration MUST declare version `flocking-config/1`.
- A portable configuration MUST contain a `persona` field with the user's public key.
- A portable configuration MUST contain a `sources` array.
- A portable configuration MAY contain a `local_pin_dismissals` array.
- One source record MUST identify exactly one source public key.
- A source record MUST use fields `pubkey` and `grants`.
- One source record MAY enable several faculties.
- A faculty grant MUST use fields `faculty`, `global`, and `topics`.
- A non-pin faculty grant MUST contain one positive integer rank.
- A non-pin faculty grant MUST store its rank in the `rank` field.
- A pin faculty grant MUST omit the `rank` field.
- Ranks MUST be unique within each non-pin faculty.
- A lower numeric rank has higher precedence.
- A faculty grant MUST declare whether global judgments are enabled.
- A faculty grant MAY declare zero or more enabled canonical topics.
- Pin grants MUST NOT use rank for effective pin ordering.
- A source record MAY contain a `reverse_blocks` scope object.
- A `reverse_blocks` object MUST use fields `global` and `topics`.
- A `reverse_blocks` object MUST NOT contain a rank.
- Duplicate source records MUST invalidate a portable configuration.
- Duplicate faculty grants within one source record MUST invalidate a portable configuration.
- Imported topic values MUST pass canonical topic validation.
- Unknown configuration versions MUST be rejected.
- Unknown fields in a known configuration version MAY be ignored.
- One unified application view SHOULD present people before their enabled faculties.
- A source MAY be enabled for normal block Flocking and Reverse Flocking simultaneously.
- Portable configuration MAY be exported.
- Portable configuration MAY be synchronized with user-controlled encryption.
- Portable configuration MUST NOT silently become public metadata.

## Precedence

- Precedence is evaluated independently for each faculty-scope-target question.
- A direct topic-scoped judgment has first precedence in that topic.
- A direct global judgment has second precedence in that topic.
- A flocked topic-scoped judgment has third precedence in that topic.
- A flocked global judgment has fourth precedence in that topic.
- Flocked conflicts at one scope are resolved by source rank.
- A withdrawn judgment is skipped during precedence evaluation.
- Absence of a judgment is skipped during precedence evaluation.
- An affirmative contrary judgment wins when it is the highest applicable judgment.
- A context-free evaluation MUST omit topic-scoped judgments.
- Pin aggregation is the only v1 exception to ordinary source precedence.

## Cross-faculty composition

- Faculties retain independent authored and effective state.
- An effective block controls author visibility before silence is considered.
- An unblock changes only block state.
- An unsilence changes only silence state.
- A follow does not imply an unblock.
- An unblock does not imply a follow.
- A hide may exclude content whose author remains otherwise visible.
- An unhide does not override an effective author block.
- A community restoration does not override a global hide.
- A removed object is ineligible for an effective pin in that topic.
- A temporarily ineligible pin remains authored and may reactivate later.

## Follow behavior

- `follow` positively includes the target in effective follows.
- `unfollow` affirmatively excludes the target from effective follows.
- `withdraw` removes the author's current follow position.
- Effective follow state MUST be evaluated per target.
- A lower-ranked source MAY determine follow state after a higher-ranked source withdraws.
- Flocked follows MUST NOT mutate the user's NIP-02 list.
- Following a source's follows MUST use that source's authored state only.

## Block behavior

- `block` excludes all past and future activity by the target in the applicable scope.
- `unblock` affirmatively permits the target with respect to the block faculty.
- `withdraw` removes the author's current block position.
- Block evaluation MUST NOT depend on content timestamps.
- A block MUST NOT be represented as deletion of target content.
- A block MUST NOT claim to prevent the target from publishing.
- A topic block MUST NOT affect the same target outside that topic context.
- A global block applies across all contexts unless a higher-precedence topic unblock applies.
- Ending an effective block makes previously blocked content eligible again.
- Ending a block MUST NOT generate retroactive notifications.
- Ending a block MUST NOT present blocked-interval content as newly published.

## Silence behavior

- `silence` excludes target contributions at or after its cutoff in the applicable scope.
- `unsilence` affirmatively permits the target with respect to the silence faculty.
- `withdraw` removes the author's current silence position.
- Pre-cutoff activity remains eligible under silence alone.
- Ending effective silence makes interval contributions eligible again.
- Restored interval contributions MUST retain their original chronology.
- Ending silence MUST NOT generate retroactive notifications.
- Ending silence MUST NOT present interval contributions as newly published.
- A block and a silence MAY remain effective simultaneously.
- A latent silence becomes relevant again when a controlling block ends.

## Descendant behavior

- Block and silence apply only to contributions authored by their person target.
- Block and silence MUST NOT automatically exclude descendants authored by other people.
- Hide and removal apply only to their content target.
- Hide and removal MUST NOT automatically exclude descendants authored by other people.
- A host SHOULD preserve enough thread topology to make visible descendants intelligible.
- A host MAY represent an excluded ancestor with a provenance-bearing placeholder.
- Reveal interactions are host policy rather than Flocking protocol state.

## Hide behavior

- `hide` excludes the target logical object from ordinary rendering in the applicable scope.
- `unhide` affirmatively permits the object with respect to the hide faculty.
- `withdraw` removes the author's current hide position.
- A global hide applies across Flocking-aware views.
- A topic hide applies only in that topic context.
- Hide MUST NOT be represented as deletion of the underlying Nostr event.
- A hidden object MAY remain inspectable through an explicit reveal action.
- Unhide does not restore an object removed from a topic.

## Community-membership behavior

- `remove` rejects the target object's claimed membership in one topic.
- `restore` affirmatively accepts the target object's membership in one topic.
- `withdraw` removes the author's current membership position.
- Removal MUST NOT hide the object globally.
- Removal MUST NOT remove the object from other topics.
- Removal MUST NOT remove the object from profiles or context-free search.
- One multi-topic object MAY be removed from one topic and retained in another.
- Restoration MUST NOT override an independent hide, block, or silence.
- Removal and restoration create inspectable folksonomic membership information.

## Pin behavior

- `pin` contributes one source's current support for prominence in one topic.
- `withdraw` removes only that author's pin support.
- An unpin action MUST NOT subtract another source's support.
- Flocking v1 MUST NOT publish an unpin-with-prejudice judgment.
- A user MAY locally dismiss a flock-derived pin from their own pinned area.
- A local pin dismissal MUST NOT be published or flocked.
- Removing a local dismissal restores ordinary pin aggregation.
- Direct authored pins rank above all flock-derived pins.
- Flock-derived pins rank first by distinct applicable sources with current pin support.
- Equal source support is resolved by the newest active supporting pin judgment.
- A canonical target identifier resolves any remaining pin tie.
- Pin source rank MUST NOT alter source-support counts.
- A hidden object is ineligible for display as a pin in that context.
- A removed object is ineligible for display as a pin in that context.
- Content excluded through its author is ineligible for display as a pin.
- An ineligible current pin MAY become displayable when the exclusion ends.
- Visible pin-slot count is host policy.
- Two visible pin slots are the recommended default.
- Additional eligible pins SHOULD remain inspectable through an expansion.

## Community appearance behavior

- A community appearance is one person's current image choice for one bare topic.
- A direct appearance choice overrides all followed appearance sources for that viewer.
- Without a direct choice, identical image hashes aggregate support from distinct explicitly selected sources.
- Appearance support MUST NOT be counted from people the viewer did not select for this purpose.
- Equal support is resolved by newest current choice and then by canonical image metadata.
- A withdrawal is a newer replaceable appearance event with no image metadata.
- A direct withdrawal restores resolution from followed appearance sources.
- Sources declaring the same hash contribute support even when their delivery metadata differs.
- A topic name remains its normalized bare identifier and MUST NOT be replaced by appearance metadata.
- An image reference MUST include an HTTPS URL, SHA-256 hash, MIME type, dimensions, and alt text.
- Clients MUST verify downloaded image bytes against the declared SHA-256 hash before display.
- Clients SHOULD generate a deterministic local identicon when no effective appearance is available.
- Community appearance event kind `30821` is addressable by the bare topic in its `d` tag.
- Community appearance events MUST carry `v`, `t`, and `j` tags using the canonical Flocking version and topic.
- A set event MUST carry exactly one `url`, `x`, `m`, `dim`, and `alt` tag.
- A withdrawal event MUST NOT carry image metadata tags.

## Reverse Flocking

- Reverse Flocking treats selected sources' current blocks as positive discovery inputs.
- Reverse-Flocking results MUST remain separate from ordinary follow state.
- Reverse Flocking MUST NOT silently follow or unblock a target.
- Reverse-Flocking targets MUST be deduplicated by public key.
- Reverse-Flocking targets rank first by distinct applicable blocking sources.
- Equal blocker support is resolved by the newest active supporting block judgment.
- The target public key resolves any remaining Reverse-Flocking tie.
- A Reverse-Flocking result MUST expose its blocking sources.
- Normal block Flocking and Reverse Flocking MAY consume the same source simultaneously.
- A normally blocked target MAY remain discoverable only in the Reverse-Flocking view.
- `Rescue` is an explicit local transaction containing direct follow and direct unblock actions.
- Rescue SHOULD create a global follow and an unblock matching the discovery scope.
- Rescue MUST NOT erase the inherited blocks it overrides.

## Standard Nostr inputs

- A current NIP-02 kind-3 list MAY provide fallback positive global follow judgments.
- Membership in a fallback kind-3 list means `follow`.
- Absence from a fallback kind-3 list means no judgment rather than `unfollow`.
- A current NIP-51 kind-10000 public-key mute list MAY provide fallback positive global block judgments.
- Membership in a fallback public-key mute list means `block`.
- Absence from a fallback public-key mute list means no judgment rather than `unblock`.
- A canonical Flocking event for one tuple MUST supersede standard-event fallback for that tuple.
- A canonical Flocking withdrawal MUST prevent fallback from resurrecting the same source judgment.
- NIP-32 labels MAY be consumed only through an explicitly configured ontology adapter.
- NIP-56 reports MUST NOT be interpreted as enacted Flocking judgments.
- NIP-72 approvals MUST NOT be interpreted as Flocking restoration without an explicit adapter.
- NIP-29 moderation events MUST NOT be interpreted as voluntary Flocking judgments.

## Standard Nostr mirrors

- A publisher SHOULD mirror current positive global follows into its NIP-02 list.
- An `unfollow` mirror is represented by absence from the NIP-02 list.
- A publisher MAY mirror current positive global blocks into its NIP-51 public-key mute list.
- An `unblock` mirror is represented by absence from the NIP-51 mute list.
- Silence MUST NOT be mirrored as a NIP-51 mute because the time semantics differ.
- Contextual pins MUST NOT be mirrored as NIP-51 profile pins.
- Community removal MUST NOT be mirrored as NIP-72 moderation.
- A publisher MAY emit auxiliary NIP-32 reasons without making them semantic authority.
- A Flocking event remains authoritative when a compatibility mirror disagrees.
- Publishing a judgment and updating a mirror need not be relay-atomic.
- A host SHOULD queue related publications as one retry-safe local operation.

## Input completeness

- Every source-faculty-scope input MUST be marked `complete`, `stale`, or `unknown`.
- `Complete` means the adapter asserts that it has the current required source state.
- `Stale` means usable state exists but may no longer be current.
- `Unknown` means the adapter cannot safely assert current source state.
- Missing relay responses MUST NOT be converted into empty source state.
- Stale inputs MAY produce effective state marked stale.
- Unknown higher-precedence input MUST produce an indeterminate result when it could change the outcome.
- Unknown lower-precedence input MAY be ignored when a known higher-precedence judgment already determines the outcome.
- Pin aggregation MUST report incomplete support when an enabled pin source is unknown.
- Reverse-Flocking counts MUST report incomplete support when an enabled block source is unknown.

## Provenance

- Every effective flock-derived result MUST retain its source public key.
- Every effective flock-derived result MUST retain its source event ID when one exists.
- Every effective result MUST identify its faculty and scope.
- Every effective ordinary-precedence result MUST identify the winning rank when flock-derived.
- Every effective silence MUST identify its cutoff.
- Every pin result MUST identify all current supporting sources.
- Every result SHOULD retain overridden applicable judgments for explanation.
- Provenance MUST identify direct evidence.
- Provenance MUST identify flocked evidence.
- Provenance MUST identify standard-event fallback evidence.
- Provenance MUST identify stale evidence.
- Provenance MUST identify locally observed timing evidence.
- A `Why?` explanation MUST be derivable without reconstructing the raw event graph.
- Inherited judgments MUST NOT be presented as judgments authored by the follower.

## Determinism

- Equivalent validated inputs and configuration MUST produce equivalent effective state.
- Every unordered source collection MUST use deterministic canonical ordering.
- Every unordered target collection MUST use deterministic canonical ordering.
- Public keys MUST use lowercase hexadecimal canonical form.
- Event IDs MUST use lowercase hexadecimal canonical form.
- Digests MUST use lowercase hexadecimal canonical form.
- Timestamps MUST be integer Unix seconds.
- Integer overflow MUST fail legibly.
- A malformed timestamp MUST fail legibly.
- An invalid encoding MUST fail legibly.
- The evaluator MUST NOT access the network.
- The evaluator MUST NOT access durable storage.
- The evaluator MUST NOT read the wall clock.
- The evaluator MUST NOT invoke a signer.
- The evaluator MUST NOT read user-interface state.
- Host-specific ranking MUST occur after Flocking eligibility and pin ordering are computed.

## Library boundary

- The core library MUST accept validated authored judgments, source configuration, context, and completeness metadata.
- The core library MUST return effective state, provenance, and uncertainty.
- Nostr parsing SHOULD live in a protocol adapter.
- Nostr event construction SHOULD live in a protocol adapter.
- Relay fetching SHOULD live outside the core evaluator.
- Durable storage SHOULD live outside the core evaluator.
- Signing SHOULD live outside the core evaluator.
- User-interface behavior SHOULD live outside the core evaluator.
- Hydra-specific identifiers MUST remain outside the core semantic model.
- Hydra-specific view paths MUST remain outside the core semantic model.
- A host adapter MAY translate canonical bare topics into host view paths.

## Local pin dismissal

- A local pin dismissal is keyed by user, topic, and logical content target.
- A portable dismissal record MUST use fields `topic`, `target_type`, and `target`.
- A dismissal `target_type` MUST be `e` or `a`.
- A local pin dismissal affects only the user's rendered pinned area.
- A local pin dismissal MUST NOT alter authored pin state.
- A local pin dismissal MUST NOT appear in published judgment events.
- A local pin dismissal MAY be included in encrypted portable configuration.

## No reports

- Flocking v1 has no reporting faculty.
- A user who wants a judgment propagated SHOULD make that judgment directly.
- Flocking propagates judgments rather than soliciting action from authorities.
- Public argument for a judgment remains ordinary speech.
- Reports from external protocols MUST remain distinguishable from enacted judgments.

## Explicit non-goals

- Flocking v1 does not support thread locking.
- Flocking v1 does not support admission gates.
- Flocking v1 does not support private-room membership.
- Flocking v1 does not support moderator-distinguished speech.
- Flocking v1 does not support official moderator registries.
- Flocking v1 does not support reports or modmail.
- Flocking v1 does not support community-rule enforcement.
- Flocking v1 does not support generalized tagging delegation.
- Flocking v1 does not support recursive delegation.
- Flocking v1 does not support liquid democracy.
- Flocking v1 does not support reputation scores.
- Flocking v1 does not support general recommendation ranking.
- Flocking v1 does not support suggested comment sorting.
- Flocking v1 does not support automated judgment authorship.
- Flocking v1 does not define relay enforcement behavior.

## Classification boundary

- Ordinary descriptive tags are not Flocking faculties.
- NSFW, spoiler, flair, and similar metadata SHOULD use ordinary classification protocols.
- Community removal is reserved for a strong contextual belonging judgment.
- Generalized following of another person's tagging activity is outside v1.

## Security and privacy

- Published judgments MUST be treated as permanently public information.
- Public reasons MUST be rendered as untrusted text.
- Target identifiers MUST be validated before use.
- Relay hints MUST be validated before use.
- Event content MUST be validated before use.
- A host MUST NOT execute links or markup found in reasons.
- Local source configuration SHOULD be encrypted at rest when host policy protects comparable social data.
- Portable configuration synchronization SHOULD use user-controlled encryption.
- Flocking MUST NOT claim Sybil resistance.
- Distinct-source counts mean distinct configured public keys rather than verified independent people.
- Source counts MUST be computed only from sources chosen by the user for that faculty.

## User experience requirements

- A host SHOULD describe each grant concretely, such as `Follow Alice's blocks`.
- A host SHOULD present one unified people-first source list.
- A host SHOULD expose per-faculty rank where rank affects precedence.
- A host SHOULD omit rank controls for additive pin aggregation.
- A host MUST distinguish authored and inherited state.
- A host MUST distinguish local-only and published actions.
- A host SHOULD offer a simple provenance explanation for every inherited effect.
- A host SHOULD fail visibly when required source state is unknown.
- A host MUST NOT imply that a hidden or removed Nostr event was deleted.
- A host MAY provide an explicit reveal interaction for excluded content.

## Conformance

- A conforming implementation MUST implement every normative v1 faculty it advertises.
- A conforming evaluator MUST pass the published normative test vectors.
- Test vectors MUST cover every judgment action.
- Test vectors MUST cover direct and flocked precedence at both scope forms.
- Test vectors MUST cover withdrawal exposing a lower-precedence judgment.
- Test vectors MUST cover same-timestamp event-ID tie-breaking.
- Test vectors MUST cover stale source state.
- Test vectors MUST cover unknown source state.
- Test vectors MUST cover silence cutoffs.
- Test vectors MUST cover revisions during silence.
- Test vectors MUST cover local backdating evidence.
- Test vectors MUST cover unsilence.
- Test vectors MUST cover block dominance without erasing silence state.
- Test vectors MUST cover descendants authored by other people.
- Test vectors MUST cover multi-topic contextual visibility.
- Test vectors MUST cover pin support counts.
- Test vectors MUST cover pin recency.
- Test vectors MUST cover local pin dismissal.
- Test vectors MUST cover latent pin reactivation.
- Test vectors MUST cover Reverse-Flocking deduplication.
- Test vectors MUST cover Rescue.
- Test vectors MUST cover standard-event fallback.
- Test vectors MUST cover canonical-event authority over fallback.
- Schemas MUST be versioned with the specification.
- Test vectors MUST be versioned with the specification.

## Route to a NIP

- Flocking MUST be implemented as an independent library before Hydra integration defines application behavior.
- Hydra SHOULD serve as the first adopter rather than the semantic authority.
- At least one additional independent client SHOULD implement the experimental wire format before a NIP proposal.
- The eventual NIP SHOULD specify only interoperable event representation and resolution behavior.
- The eventual NIP SHOULD exclude political rationale.
- The eventual NIP SHOULD exclude host UX.
- The eventual NIP SHOULD exclude storage behavior.
- The eventual NIP SHOULD exclude Hydra-specific behavior.
- Flocking v1 SHOULD remain backwards-compatible with clients that ignore its events.
