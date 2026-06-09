use anyhow::{Context, Result};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::mpsc;
use winpassage_core::AuditEvent;

#[derive(Clone)]
pub struct AuditWriter {
    sender: mpsc::UnboundedSender<AuditEvent>,
}

impl AuditWriter {
    pub fn spawn(path: PathBuf) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<AuditEvent>();

        std::thread::spawn(move || {
            while let Some(event) = receiver.blocking_recv() {
                if let Err(error) = append_event(&path, &event) {
                    eprintln!("failed to write WinPassage audit event: {error:#}");
                }
            }
        });

        Self { sender }
    }

    pub fn write(&self, event: AuditEvent) {
        let _ = self.sender.send(event);
    }
}

fn append_event(path: &PathBuf, event: &AuditEvent) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("failed to create audit log directory {parent:?}"))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open audit log {path:?}"))?;

    let line = serde_json::to_string(event)?;
    writeln!(file, "{line}")?;
    Ok(())
}
