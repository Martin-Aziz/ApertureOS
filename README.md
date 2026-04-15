# PixelForge (ApertureOS)

## The Idea
PixelForge is an open-source, browser-first image editor designed as a modern Photoshop-class alternative.
This repository implements the initial production-oriented MVP foundation across frontend, backend, AI service, and WASM pixel core.

## Technical Architecture

### Services
- `frontend`: SvelteKit + TypeScript UI shell for authenticated project workflows.
- `backend`: Rust + Axum API for auth, projects, audit logs, and health probes.
- `ai-service`: FastAPI inference service for background removal.
- `wasm`: Rust `wasm-bindgen` module for deterministic pixel operations.
- `common`: Shared TypeScript constants and validation schemas.

### Bounded Contexts
- Editor UI context
- Core API context
- AI processing context
- Pixel operations context
- Shared contract context

Detailed architecture notes: `docs/ARCHITECTURE.md`

## Quick Start

### Prerequisites
- Node.js 20+
- pnpm 9+
- Rust stable toolchain
- Python 3.11+
- Docker 24+

### Setup
```bash
cp .env.example .env
pnpm install --no-frozen-lockfile
python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip
.venv/bin/python -m pip install -e "./ai-service[dev]"
```

### Run Locally
```bash
# Terminal 1
cargo run --manifest-path backend/Cargo.toml

# Terminal 2
.venv/bin/uvicorn src.main:app --app-dir ai-service/src --host 0.0.0.0 --port 8001 --reload

# Terminal 3
pnpm --filter pixelforge-frontend dev
```

### Run With Docker Compose
```bash
./scripts/dev-up.sh
```

## Testing
```bash
# Backend
cargo test --manifest-path backend/Cargo.toml

# WASM core
cargo test --manifest-path wasm/Cargo.toml

# AI service
.venv/bin/pytest ai-service/tests

# Frontend
pnpm --filter pixelforge-frontend test
```

## Implemented Features (Current Slice)
- JWT login and token refresh endpoints.
- Protected project CRUD subset (create/list/soft-delete).
- Admin audit log endpoint.
- Prometheus-style `/metrics` endpoint on backend and AI service.
- AI background-removal endpoint with service-secret auth and rate limiting.
- Frontend authenticated shell for login and project lifecycle.
- WASM primitives for levels, hue/saturation, and blur transforms.
- Docker Compose stack with PostgreSQL, Redis, and reverse proxy.
- CI pipeline for Node, Rust, and Python validation.

## Documentation Index
- Decisions and assumptions: `DECISIONS.md`
- API contracts: `API.md`
- Prompt strategy log: `PROMPTS.md`
- Technical debt backlog: `TECHNICAL_DEBT.md`
- ADRs: `docs/adr/`

## License
MIT (see `LICENSE`)