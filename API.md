# API

## Authentication
All protected endpoints require `Authorization: Bearer <access_token>`.

### POST /api/v1/auth/login
Authenticate a user and issue access/refresh tokens.

Request:
```json
{
  "email": "admin@pixelforge.local",
  "password": "ChangeMe123!"
}
```

Response `200`:
```json
{
  "tokens": {
    "access_token": "...",
    "refresh_token": "...",
    "expires_in_seconds": 900
  },
  "user": {
    "user_id": "00000000-0000-0000-0000-000000000001",
    "email": "admin@pixelforge.local",
    "role": "admin"
  }
}
```

### POST /api/v1/auth/refresh
Rotate refresh token and return a new token pair.

Request:
```json
{
  "refresh_token": "..."
}
```

Response `200`: same token payload shape as login.

## Projects

### GET /api/v1/projects
List active projects for the authenticated user.

### POST /api/v1/projects
Create a project.

Request:
```json
{
  "name": "Landing Page Concepts",
  "description": "Initial campaign iterations"
}
```

### DELETE /api/v1/projects/{projectId}
Soft-delete a project owned by the authenticated user.

Response `204` on success.

## Admin

### GET /api/v1/admin/audit-logs
Return audit entries for privileged users (`role=admin`).

## Health

### GET /health/live
Liveness probe.

### GET /health/ready
Readiness probe with dependency configuration checks.

### GET /metrics
Prometheus text exposition for backend service and dependency configuration flags.

## AI Service

### GET /health/live
AI service liveness probe.

### GET /health/ready
AI model readiness probe.

### GET /metrics
Prometheus text exposition for AI service metadata and model readiness.

### POST /v1/remove-background
Removes background from base64 encoded image bytes.

Validation:
- `output_format` must be `png`.
- Decoded payload size must be less than or equal to `AI_MAX_IMAGE_BYTES` (default `8_000_000`).

Headers:
- `x-ai-service-secret`: shared secret

Request:
```json
{
  "image_base64": "<base64-image-bytes>",
  "output_format": "png"
}
```

Response `200`:
```json
{
  "image_base64": "<base64-processed-image>",
  "provider": "rembg",
  "processing_ms": 42
}
```

Response `413`:
```json
{
  "error": {
    "code": "payload_too_large",
    "message": "decoded image exceeds the configured limit of 8000000 bytes"
  }
}
```
