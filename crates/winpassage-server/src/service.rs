#[cfg(windows)]
use anyhow::Result;

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use tokio::sync::oneshot;
#[cfg(windows)]
use windows_service::define_windows_service;
#[cfg(windows)]
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
#[cfg(windows)]
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
#[cfg(windows)]
use windows_service::service_dispatcher;

#[cfg(windows)]
const SERVICE_NAME: &str = "WinPassage";

#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

#[cfg(windows)]
pub fn run_service_dispatcher() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

#[cfg(windows)]
fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service() {
        eprintln!("WinPassage service failed: {error:#}");
    }
}

#[cfg(windows)]
fn run_service() -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_tx = std::sync::Mutex::new(Some(shutdown_tx));

    let status_handle = service_control_handler::register(SERVICE_NAME, move |control_event| {
        match control_event {
            ServiceControl::Stop => {
                if let Some(sender) = shutdown_tx.lock().ok().and_then(|mut guard| guard.take()) {
                    let _ = sender.send(());
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 1,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(async {
        let config = crate::config::ServerConfig::from_env()?;
        crate::http::serve_with_shutdown(config, shutdown_rx).await
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    result
}
