# ADR 0001: Monorepo And Polyglot Service Stack

## Status
Accepted

## Context
PixelForge requires a browser editor, a high-performance API, AI inference service, and WASM pixel core. The system needs explicit boundaries while preserving shared workflows and CI quality gates.

## Decision
Adopt a monorepo with polyglot service ownership:
- `frontend`: SvelteKit + TypeScript
- `backend`: Rust + Axum
- `ai-service`: Python + FastAPI
- `wasm`: Rust + wasm-bindgen
- `common`: shared TypeScript contracts/schemas

## Consequences
### Positive
- Clear bounded contexts and language fit per domain.
- Unified CI and developer onboarding in one repository.
- Lower integration friction for contract changes.

### Negative
- Toolchain complexity across Node, Rust, and Python.
- Higher CI runtime and dependency management overhead.

## Follow-up
- Standardize local bootstrapping with lock files and package caching.
- Add automated contract tests between backend and AI service.
