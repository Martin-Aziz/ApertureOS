# DECISIONS

## Implementation Decisions And Assumptions

### Assumptions
1. Single-tenant MVP auth flow is acceptable for the first implementation slice.
   - Chosen approach: seed admin user from environment variables.
   - If wrong: replace with Supabase JWT verification middleware without changing API contracts.
2. Local-first project persistence is acceptable while PostgreSQL migrations are staged.
   - Chosen approach: in-memory repository behind `ProjectRepository` trait.
   - If wrong: implement `PostgresProjectRepository` and swap dependency injection only.
3. AI background removal can start as a service-gated endpoint before frontend integration.
   - Chosen approach: FastAPI endpoint protected by shared secret and rate limiter.
   - If wrong: enforce user-level JWT delegation from backend.
4. Browser shell should focus on auth + project lifecycle before full canvas toolchain.
   - Chosen approach: Svelte shell with typed API client and project management UI.
   - If wrong: maintain API layer and replace component shell incrementally.

## Technology Choices

| Decision | Alternatives Considered | Rationale | Migration Path |
|---|---|---|---|
| Rust Axum backend | Node/Express, Go/Fiber | Strong type safety and performance for editor APIs | Keep REST contract stable and replace internals via service layer |
| FastAPI AI service | Rust ML service, Node service | Best ecosystem support for REMBG/ONNX | Replace service implementation behind `/v1/remove-background` |
| SvelteKit frontend | React/Next.js | Lean runtime and strong DX for UI-heavy editor shells | Keep API-client contracts isolated for frontend swaps |
| In-memory repository first | Direct SQLX from day one | Faster vertical slice while preserving repository boundary | Implement SQL-backed repository adapter |

## Deviations From Target End-State
1. PostgreSQL and Redis are configured but not yet wired into runtime repositories.
2. JWT auth currently uses seeded credentials, not Supabase verification.
3. Canvas editing tools are scaffolded at shell level; full WebGL integration is pending.

## Known Technical Debt
1. High: Replace in-memory project repository with PostgreSQL-backed repository and migrations.
2. High: Add refresh-token revocation persistence in Redis instead of process memory.
3. Medium: Add distributed tracing correlation across backend and AI service.
4. Medium: Add frontend route guards and automatic token refresh flow.
