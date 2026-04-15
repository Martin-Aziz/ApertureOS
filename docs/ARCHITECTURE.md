# Architecture Overview

## Bounded Contexts
1. Editor UI Context (`frontend`)
   - Owns user interactions, session state, and project workspace flows.
2. Core API Context (`backend`)
   - Owns authentication, project lifecycle, authorization, and audit trails.
3. AI Processing Context (`ai-service`)
   - Owns image inference endpoints and model orchestration.
4. Pixel Operations Context (`wasm`)
   - Owns deterministic client-side pixel transformations.
5. Shared Contract Context (`common`)
   - Owns validation schemas, limits, and shared domain terms.

## Data Flow
1. Frontend authenticates against backend and stores JWTs.
2. Frontend invokes backend project endpoints with bearer access token.
3. Backend validates token, applies authorization, and persists project mutations.
4. Backend invokes AI service using service secret for inference workloads.
5. Frontend consumes WASM pixel functions for local non-destructive operations.

## Security Boundaries
- External boundary: browser to backend over bearer-authenticated REST.
- Service boundary: backend to AI over shared-secret header.
- Domain boundary: service layer can mutate state; query code paths remain side-effect free.
