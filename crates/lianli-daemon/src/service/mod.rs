use anyhow::Result;

pub const WIRELESS_ONLY_MODE: bool = true;

pub struct SignalMonitor;

impl SignalMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn requested(&self) -> bool {
        false
    }
}

pub struct ServiceManager {
    _config_path: std::path::PathBuf,
    _socket_path: std::path::PathBuf,
}

impl ServiceManager {
    pub fn new(
        config_path: std::path::PathBuf,
        socket_path: std::path::PathBuf,
        _mode: lianli_shared::daemon::DaemonMode,
    ) -> Result<Self> {
        Ok(Self {
            _config_path: config_path,
            _socket_path: socket_path,
        })
    }

    pub fn set_service_invocation(&mut self, _invocation: Option<String>) {}

    pub fn set_ownership_lock(&mut self, _identity: lianli_shared::daemon::FileIdentity) {}

    pub fn run(&mut self, _signals: &SignalMonitor) -> Result<bool> {
        Ok(false)
    }
}
