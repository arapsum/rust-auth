mod mail;

use std::{io, sync::Arc, time::Duration};

use apalis::prelude::Monitor;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::AppContext;

pub use self::mail::{MailJob, MailQueue, handle_forgot_password, handle_welcome};

/// Implemented by any struct that owns one or more apalis backends and knows
/// how to wire its handlers onto a `Monitor`.
///
/// Each background-job module (mail, notifications, exports, ...) implements
/// this once. `App` never needs to know about individual job types, backend
/// types, or handler functions — just that the module can register itself.
pub trait WorkerModule: Send + Sync + 'static {
    /// Register every worker owned by this module onto `monitor`, returning
    /// the updated monitor so calls can be folded/chained.
    fn register(&self, monitor: Monitor, ctx: Arc<AppContext>) -> Monitor;
}

/// Collects worker modules and turns them into a single running `Monitor`.
#[derive(Default)]
pub struct WorkerRegistry {
    modules: Vec<Box<dyn WorkerModule>>,
}

pub struct WorkerHandle {
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<io::Result<()>>,
}

impl WorkerHandle {
    /// Signal all workers to stop and wait for their monitor task to finish.
    ///
    /// # Errors
    ///
    /// Returns an error when the monitor fails to shut down or its Tokio task
    /// cannot be joined.
    pub async fn shutdown(self) -> io::Result<()> {
        tracing::info!("Stopping background workers");

        if self.shutdown.send(()).is_err() {
            tracing::debug!("Background workers had already stopped");
        }

        self.task.await.map_err(io::Error::other)??;
        tracing::info!("Background workers stopped");

        Ok(())
    }
}

impl WorkerRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn append(mut self, module: impl WorkerModule) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    /// Spawn a task that runs every registered module under one apalis `Monitor`.
    #[must_use]
    pub fn spawn(self, ctx: Arc<AppContext>) -> WorkerHandle {
        const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

        let (shutdown, shutdown_signal) = oneshot::channel();
        let task = tokio::spawn(async move {
            tracing::info!("Initialising workers");

            let monitor = self
                .modules
                .into_iter()
                .fold(Monitor::new(), |monitor, module| {
                    module.register(monitor, ctx.clone())
                })
                .with_terminator(tokio::time::sleep(SHUTDOWN_TIMEOUT));

            monitor
                .run_with_signal(async move {
                    let _ = shutdown_signal.await;
                    Ok(())
                })
                .await
        });

        WorkerHandle { shutdown, task }
    }
}
