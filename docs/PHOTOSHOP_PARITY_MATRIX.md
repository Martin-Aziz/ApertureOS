# PixelForge Photoshop Parity Gap Matrix

## Scope
This matrix compares core Photoshop-class capabilities against the current PixelForge implementation.
Status values:

- `Implemented`: usable in product today.
- `Partial`: some foundation exists, but end-user capability is incomplete.
- `Missing`: no meaningful implementation yet.

## Capability Matrix

| Capability | Current Status | Evidence in Codebase | Gap | Suggested Delivery Slice |
| --- | --- | --- | --- | --- |
| Layers (stack, ordering, visibility, blend per-layer) | Missing | Frontend only exposes auth + project CRUD UI in `frontend/src/components/editor-shell.svelte`; no layer model in `common/schemas` | No non-destructive composition model | Add `Layer` domain schema in `common`, persistence model in `backend`, in-memory render graph in `wasm` |
| Layer masks (pixel/vector, invert, density/feather) | Missing | No mask entities or mask operations in `wasm/src/lib.rs` or frontend stores | Cannot perform localized edits | Add mask bitmap channel per layer with compositing ops in `wasm`; expose mask toggles in UI |
| Blend modes (multiply/screen/overlay/etc.) | Missing | WASM module currently exposes primitives (levels/hue-saturation/blur) but no blend engine | No equivalent to Photoshop layer compositing | Implement blend-mode shader/kernel set in `wasm`, with golden-image tests |
| Selections (marquee/lasso/magic-wand, refine edge) | Missing | No selection state or tools in `frontend/src/components` and no geometry contracts in `common/types/editor.ts` | No region-targeted editing | Add selection model + marching ants renderer + modifier-tool commands |
| Adjustment stack (non-destructive, reorderable) | Partial | Deterministic pixel transforms exist in `wasm`; no user-visible adjustment timeline in UI | Foundation exists without UX/state orchestration | Build adjustment graph abstraction and history stack in frontend + wasm interop |
| Smart objects (embedded/linked, transform-preserving) | Missing | No asset embedding/linking model in backend or frontend | Cannot preserve source fidelity across transforms | Add asset object model and transform metadata; defer rasterization until export |
| Text engine (editable typography layers, kerning, shaping) | Missing | No text layer model, no typography toolchain in frontend | No text editing workflow | Introduce text layer schema + canvas text editing + font metrics pipeline |
| RAW workflow (camera profiles, exposure pipeline) | Missing | No RAW decoder pipeline in backend/wasm and no import flow in frontend | Cannot ingest professional camera formats | Add backend RAW ingest service (libraw-based) and parametric development controls |
| Color management (ICC, wide-gamut, soft-proof) | Missing | No ICC profile handling in services or UI | Output not color-consistent across devices | Add color profile metadata on project/image and conversion pipeline |
| Plugin API / extensions | Missing | No extension contracts or sandbox runtime present | No ecosystem extensibility | Define plugin manifest + command/event API + sandboxed execution host |

## Existing Foundations That Reduce Risk

- Auth/session and protected APIs are production-oriented (`backend/src/api/auth.rs`, `backend/src/api/projects.rs`).
- AI service already exposes health/metrics and protected inference endpoint (`ai-service/src/api/routes.py`).
- WASM context exists for deterministic image operations (`wasm/src/lib.rs`).
- Shared contracts package is in place for cross-surface typing (`common/schemas`, `common/types`).

## Recommended Milestone Order

1. `M1 Editor Core`: layers + blend modes + selection primitives + undo/redo.
2. `M2 Pro Non-Destructive`: masks + adjustment stack + smart-object base.
3. `M3 Imaging Fidelity`: color management + RAW ingest/develop pipeline.
4. `M4 Creative Surface`: text engine + plugin API + advanced tool ecosystem.

## Acceptance Criteria Toward “Photoshop-class”

- Multi-layer composites render deterministically with reference-image parity tests.
- Non-destructive edit history survives project reload and cross-device sessions.
- Color-managed export matches ICC expectations under regression snapshots.
- Large documents remain interactive under defined performance budgets.
