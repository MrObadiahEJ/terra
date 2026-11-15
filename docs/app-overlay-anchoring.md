# App overlay anchoring

**Status:** policy, enforced at `POST /api/v1/components` (Sprint 054)
**Scope:** simulated, designed, proposed and speculative spatial content
**Related:** [Terra Protocol Directive §1 and §14](book-of-intents/TERRA_PROTOCOL_DIRECTIVE.md),
[RFC-015](rfc-015-spatial-component-package.md), [ADR-024](book-of-intents/ARCHITECTURAL_DECISIONS.md#adr-024)

---

## 1. Why this boundary exists

Terra's component registry is an evidence-bearing record of physical space.
A registered component answers: *what is there, what was observed or inferred,
by which engine, and how confident is the result?* Its provenance values are
defined by RFC-015 §3 — `imported`, `observed`, `surveyed`, `reconstructed`,
`inferred`, `manually_corrected`.

Creative construction is a first-class use of Terra, but it answers a different
question: *what could be there?* Proposed buildings, design options, game
worlds, scenario models and speculative reconstructions must not be presented as
evidence about the physical world, and they must not inherit the trust attached
to a verified component.

The protocol therefore keeps the two apart:

| Layer | Contents | Trust claim |
| --- | --- | --- |
| Protocol component registry | Evidence-backed spatial components (RFC-015) | Provenance + validation state are explicit and auditable |
| App overlay | Proposed / designed / simulated / speculative content | Authored visualization, clearly labeled, anchored to protocol objects |

The directive's requirement to "clearly distinguish observed, measured,
inferred, simulated, designed and proposed content" is satisfied by *where the
content lives* and by the overlay's explicit `mode`.

## 2. Enforcement

`POST /api/v1/components` returns `400 Bad Request` when
`provenance_classification` is one of:

`simulated`, `designed`, `proposed`, `speculative`, `ai-generated-world`,
`ai_generated`, `ai_generated_world`.

These are not RFC-015 provenance values. An application that wants to publish
such content uses an overlay (below), not the component registry.

This is deliberately a protocol-boundary check, not a content-moderation
judgment: `inferred`, `reconstructed` and AI-assisted reconstructions remain
admissible when they declare their engine, confidence and evidence references.

## 3. Overlay contract (application layer)

An overlay is ordinary application data; Terra does not need a new on-chain
instruction for it. A conforming overlay SHOULD declare:

```json
{
  "schema": "terra.app-overlay/v1",
  "app_id": "com.example.design-studio",
  "mode": "proposed",
  "anchor": {
    "parcel_id": "parcel-123",
    "component_id": "comp-roof-456",
    "manifest_content_address": "sha256:…"
  },
  "geometry_ref": "ipfs://…",
  "author": "did:key:…",
  "created_at": "2026-11-15T00:00:00Z",
  "disclosure": "Design proposal — not an observed or verified representation"
}
```

Rules:

1. **Explicit mode.** `mode` is one of `proposed`, `designed`, `simulated`,
   `speculative`. Never infer mode from appearance.
2. **Anchor, don't merge.** An overlay may reference a parcel, component,
   geometry version or signed manifest content address. It must not mutate or
   masquerade as the anchored protocol object.
3. **Label in every client.** Any renderer that displays an overlay must show
   its authored/simulated status. A screenshot or export carries the same
   disclosure.
4. **No trust inheritance.** An overlay must not reuse a component's validation
   state, evidence references or on-chain rights as if they applied to the
   proposed content.
5. **Revocable and app-scoped.** Overlay storage, retention and access are the
   application's responsibility; the protocol does not guarantee availability.

## 4. Examples

- A design studio proposes three roof options for an existing parcel: three
  overlays anchored to the parcel, `mode: "designed"`; the existing roof remains
  the protocol component.
- A game imports a city block and adds fictional towers: overlays anchored to
  the block's manifest address, `mode: "simulated"`.
- A historical-memory app reconstructs a demolished building from photographs:
  this *may* be a protocol component (`provenance_classification:
  "reconstructed"`) because it is evidence-backed; speculative interior details
  invented by the app stay in overlays (`mode: "speculative"`).

## 5. Non-goals

- This policy does not forbid creative or AI-assisted work.
- It does not decide ownership, licensing or revenue rights for overlays.
- It does not replace content moderation, privacy controls or legal review.
