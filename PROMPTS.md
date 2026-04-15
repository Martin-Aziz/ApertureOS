# PROMPTS

## System Prompt Strategy Used In This Repository
- Role framing: autonomous MVP implementor with architecture-first execution.
- Context split: requirements were mapped into domain, service, and infrastructure workstreams.
- Delivery style: production-lean vertical slice first, then hardening and documentation.

## Context Optimization Notes
- PixelForge specification in `pixelforge_mvp_overview.html` is treated as primary source of scope.
- V1/V2 roadmap items are deferred unless required for technical scaffolding.
- All assumptions are surfaced in `DECISIONS.md` to avoid hidden architectural drift.

## Iterative Refinement Log
1. Scaffolded monorepo foundations and environment contracts.
2. Implemented backend auth/projects/audit vertical slice and passing tests.
3. Implemented AI service endpoint with service-token and rate limiting.
4. Implemented frontend shell with typed API client and project lifecycle UI.
5. Added WASM image primitives and repository-wide CI workflow.
