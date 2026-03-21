use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::signal;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct ShutdownSignal {
    sender: broadcast::Sender<()>,
    is_shutdown: Arc<AtomicBool>,
}

impl ShutdownSignal {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            sender,
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    pub fn trigger(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        let _ = self.sender.send(());
        tracing::info!("shutdown signal triggered");
    }

    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }

    pub async fn wait_for_signal(&self) {
        let mut receiver = self.subscribe();
        let _ = receiver.recv().await;
    }
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn listen_for_shutdown(signal: ShutdownSignal) {
    tokio::select! {
        _ = signal::ctrl_c() => {
            tracing::info!("received SIGINT (Ctrl+C)");
            signal.trigger();
        }
        _ = wait_for_sigterm() => {
            tracing::info!("received SIGTERM");
            signal.trigger();
        }
    }
}

#[cfg(unix)]
async fn wait_for_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to setup SIGTERM handler");
    sigterm.recv().await;
}

#[cfg(not(unix))]
async fn wait_for_sigterm() {
    std::future::pending::<()>().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn shutdown_signal_triggers() {
        let signal = ShutdownSignal::new();
        assert!(!signal.is_shutdown());

        signal.trigger();
        assert!(signal.is_shutdown());
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_signal() {
        let signal = ShutdownSignal::new();
        let mut rx1 = signal.subscribe();
        let mut rx2 = signal.subscribe();

        signal.trigger();

        tokio::time::timeout(Duration::from_millis(100), rx1.recv())
            .await
            .expect("timeout")
            .expect("receive");
        tokio::time::timeout(Duration::from_millis(100), rx2.recv())
            .await
            .expect("timeout")
            .expect("receive");
    }

    #[tokio::test]
    async fn wait_for_signal_completes_on_trigger() {
        let signal = ShutdownSignal::new();
        let signal_clone = signal.clone();

        let handle = tokio::spawn(async move {
            signal_clone.wait_for_signal().await;
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        signal.trigger();

        tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("timeout")
            .expect("join");
    }
}
