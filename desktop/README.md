# PixelForge Desktop (Tauri)

This package wraps the existing PixelForge frontend in a native desktop shell using Rust + Tauri.

## Commands

From repository root:

- `pnpm run dev:desktop` starts the desktop app in development mode.
- `pnpm run build:desktop` builds native desktop bundles.

## How It Works

- During development, Tauri loads the frontend from `http://127.0.0.1:5173`.
- During desktop build, frontend assets are generated with `DESKTOP_BUILD=1` and bundled from `frontend/build`.
- The SvelteKit config switches to `@sveltejs/adapter-static` only for desktop builds.

## Managed Local Services

By default, the desktop app attempts to ensure local backend and AI services are available on startup:

- Backend health check: `http://127.0.0.1:8080/health/ready`
- AI health check: `http://127.0.0.1:8001/health/ready`

If a service is not healthy, the desktop app starts it and waits for readiness.
In packaged release builds, the app will skip managed startup if no explicit command is configured, rather than aborting launch.

Default startup commands (run from repo root):

- Backend: `cargo run --manifest-path backend/Cargo.toml`
- AI: `.venv/bin/uvicorn src.main:app --app-dir ai-service/src --host 127.0.0.1 --port 8001`

Environment variables:

- `PIXELFORGE_MANAGED_SERVICES` (`1` by default, set `0` to disable)
- `PIXELFORGE_BACKEND_START_COMMAND`
- `PIXELFORGE_AI_START_COMMAND`
- `PIXELFORGE_BACKEND_HEALTH_URL`
- `PIXELFORGE_AI_HEALTH_URL`
- `PIXELFORGE_HEALTH_TIMEOUT_SECONDS`

For fully self-contained offline installers, provide packaged backend/AI runtime commands via the two `*_START_COMMAND` variables (or bundle sidecars and point these variables to them).

## Current Feature Parity

The desktop app currently has parity with the browser MVP shell:

- Sign in and token-based session management
- Project list, create, and delete operations
- Protected API calls through the existing backend endpoints

Photoshop-class editing features are not implemented yet in either browser or desktop targets.
