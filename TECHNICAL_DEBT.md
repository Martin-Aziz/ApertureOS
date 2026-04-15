# TECHNICAL_DEBT

## High Priority
1. Persist backend projects and audit logs in PostgreSQL via SQLx migrations.
2. Persist refresh token records in Redis for multi-instance token revocation.
3. Replace seeded credentials with Supabase JWT verification for production auth.

## Medium Priority
1. Integrate WebGL2 canvas with WASM primitives in frontend editor runtime.
2. Add security headers and stricter CORS policies per environment.
3. Add mutation testing step for backend and frontend CI jobs.

## Low Priority
1. Replace in-memory rate limiter in AI service with Redis-backed limiter.
2. Add cloud object storage adapters (S3/R2) and signed URL export pipeline.
