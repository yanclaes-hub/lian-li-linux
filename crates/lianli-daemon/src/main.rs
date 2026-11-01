mod catalog_references;
mod controllers;
mod desktop_display;
mod ipc;
mod media_access;
mod media_decode;
mod openrgb_server;
mod persistence;
mod pidlock;
mod pixel_cleaner;
mod service;
mod state_backups;
mod state_health;
mod template_store;
mod thermal_alert;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

fn default_config_path(system: bool) -> PathBuf {
    if system {
        return PathBuf::from("/var/lib/lianli/config.json");
    }
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            PathBuf::from(home).join(".config")
        });
    config_dir.join("lianli").join("config.json")
}

fn default_socket_path(system: bool) -> PathBuf {
    if system {
        lianli_shared::installation::InstallationContext::detect().system_socket_path()
    } else {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(runtime_dir).join("lianli-daemon.sock")
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Wireless-only daemon for Lian Li fan control and RGB management"
)]
struct Cli {
    /// Path to the configuration file
    #[arg(long)]
    config: Option<PathBuf>,

    /// IPC socket path
    #[arg(long)]
    socket: Option<PathBuf>,

    /// Run as a system service
    #[arg(long)]
    system: bool,

    /// Runtime invocation ID supplied by the host service wrapper
    #[arg(long, hide = true, value_parser = lianli_shared::daemon::parse_service_invocation)]
    service_invocation: Option<String>,

    /// Logging verbosity (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print build and IPC capabilities without loading configuration or accessing hardware
    Capabilities,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let system = cli.system;
    let config = cli.config.unwrap_or_else(|| default_config_path(system));
    let socket = cli.socket.unwrap_or_else(|| default_socket_path(system));

    match cli.command {
        Some(Commands::Capabilities) => {
            serde_json::to_writer(std::io::stdout().lock(), &ipc::build_info())?;
            return Ok(());
        }
        None => {}
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();

    let signals = service::SignalMonitor::new()?;
    let context = lianli_shared::installation::InstallationContext::detect();
    let scope = if system {
        lianli_shared::services::ServiceScope::System
    } else {
        lianli_shared::services::ServiceScope::User
    };
    if let Some(reason) = lianli_control::service_selection::launch_block_reason(&context, scope)? {
        tracing::info!("{reason}");
        return Ok(());
    }
    let pidlock = pidlock::PidLock::acquire()?;
    if let Some(reason) = lianli_control::service_selection::launch_block_reason(&context, scope)? {
        tracing::info!("Host selection changed before startup. {reason}");
        return Ok(());
    }
    if let Some(backup) = lianli_control::state_transaction::recover(&config)? {
        tracing::warn!(backup = %backup, "Recovered interrupted state publication before starting hardware control");
    }

    if service::WIRELESS_ONLY_MODE {
        tracing::warn!(
            "wireless-only build active: LCD, media, desktop-display and startup-image code paths are disabled."
        );
        drop(pidlock);
        return Ok(());
    }

    let mode = if system {
        lianli_shared::daemon::DaemonMode::System
    } else {
        lianli_shared::daemon::DaemonMode::User
    };
    let mut manager = service::ServiceManager::new(config, socket, mode)?;
    manager.set_ownership_lock(pidlock.identity()?);
    manager.set_service_invocation(cli.service_invocation);
    let restart = manager.run(&signals)?;

    if restart && !signals.requested() {
        use std::os::unix::process::CommandExt;
        let exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();
        tracing::info!("Re-executing daemon: {} {}", exe.display(), args.join(" "));
        let err = std::process::Command::new(exe).args(args).exec();
        anyhow::bail!("Failed to re-exec daemon: {err}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_invocation_is_optional_but_must_be_valid_before_startup() {
        assert!(Cli::try_parse_from(["lianli-daemon"])
            .unwrap()
            .service_invocation
            .is_none());
        assert!(Cli::try_parse_from(["lianli-daemon", "--service-invocation", "invalid"]).is_err());
        let parsed = Cli::try_parse_from([
            "lianli-daemon",
            "--service-invocation",
            "ABCDEF0123456789ABCDEF0123456789",
        ])
        .unwrap();
        assert_eq!(
            parsed.service_invocation.as_deref(),
            Some("abcdef0123456789abcdef0123456789")
        );
    }
}
