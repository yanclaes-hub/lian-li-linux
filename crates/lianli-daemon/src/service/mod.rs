    pub fn run(&mut self, signals: &SignalMonitor) -> Result<bool> {
        if WIRELESS_ONLY_MODE {
            tracing::warn!(
                "wireless-only daemon: keeping startup limited to wireless IPC and controller setup; LCD/media/display loops are disabled."
            );
            return Ok(false);
        }

        let mut monitor = OperationMonitor::new()?;
