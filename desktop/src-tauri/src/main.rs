#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const DEFAULT_BACKEND_HEALTH_URL: &str = "http://127.0.0.1:8080/health/ready";
const DEFAULT_AI_HEALTH_URL: &str = "http://127.0.0.1:8001/health/ready";

const DEFAULT_BACKEND_START_COMMAND: &str = "cargo run --manifest-path backend/Cargo.toml";
const DEFAULT_AI_START_COMMAND: &str =
    ".venv/bin/uvicorn src.main:app --app-dir ai-service/src --host 127.0.0.1 --port 8001";

static SERVICE_MANAGER: OnceLock<Mutex<ServiceManager>> = OnceLock::new();

#[derive(Default)]
struct ServiceManager {
    backend: Option<Child>,
    ai: Option<Child>,
}

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .to_path_buf()
}

fn managed_services_enabled() -> bool {
    std::env::var("PIXELFORGE_MANAGED_SERVICES")
        .map(|value| value != "0")
        .unwrap_or(true)
}

fn managed_service_command(env_key: &str, _default_command: &str) -> Option<String> {
    match std::env::var(env_key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            #[cfg(debug_assertions)]
            {
                Some(default_command.to_string())
            }

            #[cfg(not(debug_assertions))]
            {
                None
            }
        }
    }
}

fn health_check_timeout() -> u64 {
    std::env::var("PIXELFORGE_HEALTH_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(30)
}

fn run_health_check(url: &str) -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    match client.get(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

fn wait_until_healthy(url: &str, timeout_seconds: u64) -> bool {
    let attempts = timeout_seconds.max(1);

    for _ in 0..attempts {
        if run_health_check(url) {
            return true;
        }

        std::thread::sleep(Duration::from_secs(1));
    }

    false
}

fn spawn_service(command: &str, cwd: &std::path::Path) -> Result<Child, String> {
    Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn `{command}`: {error}"))
}

fn ensure_backend(manager: &mut ServiceManager, cwd: &std::path::Path) -> Result<(), String> {
    let health_url = std::env::var("PIXELFORGE_BACKEND_HEALTH_URL")
        .unwrap_or_else(|_| DEFAULT_BACKEND_HEALTH_URL.to_string());

    if run_health_check(&health_url) {
        eprintln!("[desktop] backend already healthy");
        return Ok(());
    }

    let Some(start_command) = managed_service_command(
        "PIXELFORGE_BACKEND_START_COMMAND",
        DEFAULT_BACKEND_START_COMMAND,
    ) else {
        eprintln!("[desktop] backend start command not configured; skipping managed startup");
        return Ok(());
    };

    eprintln!("[desktop] starting backend service");

    manager.backend = Some(match spawn_service(&start_command, cwd) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("[desktop] {error}");
            return Ok(());
        }
    });

    if wait_until_healthy(&health_url, health_check_timeout()) {
        eprintln!("[desktop] backend health check passed");
        Ok(())
    } else {
        if let Some(mut backend_child) = manager.backend.take() {
            let _ = backend_child.kill();
            let _ = backend_child.wait();
        }

        eprintln!(
            "[desktop] backend failed health check at {health_url} after startup"
        );
        Err(format!(
            "backend failed health check at {health_url} after startup"
        ))
    }
}

fn ensure_ai_service(manager: &mut ServiceManager, cwd: &std::path::Path) -> Result<(), String> {
    let health_url =
        std::env::var("PIXELFORGE_AI_HEALTH_URL").unwrap_or_else(|_| DEFAULT_AI_HEALTH_URL.to_string());

    if run_health_check(&health_url) {
        eprintln!("[desktop] ai service already healthy");
        return Ok(());
    }

    let Some(start_command) = managed_service_command(
        "PIXELFORGE_AI_START_COMMAND",
        DEFAULT_AI_START_COMMAND,
    ) else {
        eprintln!("[desktop] ai start command not configured; skipping managed startup");
        return Ok(());
    };

    eprintln!("[desktop] starting ai service");

    manager.ai = Some(match spawn_service(&start_command, cwd) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("[desktop] {error}");
            return Ok(());
        }
    });

    if wait_until_healthy(&health_url, health_check_timeout()) {
        eprintln!("[desktop] ai service health check passed");
        Ok(())
    } else {
        if let Some(mut ai_child) = manager.ai.take() {
            let _ = ai_child.kill();
            let _ = ai_child.wait();
        }

        eprintln!("[desktop] ai failed health check at {health_url} after startup");
        Err(format!(
            "ai service failed health check at {health_url} after startup"
        ))
    }
}

fn start_managed_services() -> Result<(), String> {
    if !managed_services_enabled() {
        eprintln!("[desktop] managed services disabled (PIXELFORGE_MANAGED_SERVICES=0)");
        return Ok(());
    }

    let service_mutex = SERVICE_MANAGER.get_or_init(|| Mutex::new(ServiceManager::default()));
    let mut manager = service_mutex
        .lock()
        .map_err(|_| "failed to lock service manager".to_string())?;
    let cwd = workspace_root();

    ensure_backend(&mut manager, &cwd)?;
    ensure_ai_service(&mut manager, &cwd)?;

    Ok(())
}

fn stop_managed_services() {
    let Some(service_mutex) = SERVICE_MANAGER.get() else {
        return;
    };

    let Ok(mut manager) = service_mutex.lock() else {
        return;
    };

    if let Some(mut ai_child) = manager.ai.take() {
        let _ = ai_child.kill();
        let _ = ai_child.wait();
    }

    if let Some(mut backend_child) = manager.backend.take() {
        let _ = backend_child.kill();
        let _ = backend_child.wait();
    }
}

fn main() {
    let app = tauri::Builder::default()
        .setup(|_| {
            if let Err(error) = start_managed_services() {
                eprintln!("[desktop] managed service startup finished with error: {error}");
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running PixelForge desktop");

    app.run(|_, event| {
        if matches!(event, tauri::RunEvent::Exit { .. }) {
            stop_managed_services();
        }
    });
}
