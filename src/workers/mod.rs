mod mail;

use std::sync::Arc;

use apalis::prelude::Monitor;

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
    pub fn spawn(self, ctx: Arc<AppContext>) {
        tokio::spawn(async move {
            tracing::info!("Initialising workers");

            let monitor = self
                .modules
                .into_iter()
                .fold(Monitor::new(), |monitor, module| {
                    module.register(monitor, ctx.clone())
                });
            monitor
                .run()
                .await
                .unwrap_or_else(|e| tracing::error!(error = ?e, "Queue monitor crashed"));
        });
    }
}
