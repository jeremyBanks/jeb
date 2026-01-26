use std::future::Future;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::interval;
use tokio::time::Interval;
use tokio::time::MissedTickBehavior;
use tracing::debug;
use tracing::instrument;

use super::*;

#[derive(Debug, Clone)]
pub struct Throttle(Arc<Impl>);

#[derive(Debug)]
struct Impl {
    name: &'static str,
    interval: Mutex<Interval>,
}

pub fn throttle(name: &'static str, ms: u64) -> Throttle {
    Throttle::new(name, ms)
}

impl Throttle {
    pub fn new(name: &'static str, ms: u64) -> Self {
        info!("Initializing throttle {name:?} with {ms} ms minimum interval");
        let mut interval = interval(Duration::from_millis(ms));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let interval = Mutex::new(interval);
        Self(Arc::new(Impl { name, interval }))
    }

    #[instrument(skip_all)]
    pub async fn tick(&self) {
        debug!("Waiting for throttle {:?}...", self.0.name);
        self.0.interval.lock().await.tick().await;
        debug!("{:?} ticked!", self.0.name);
    }

    #[instrument(skip_all)]
    pub async fn with<T, E, F: Future<Output = Result<T, E>>>(
        &self,
        f: impl FnOnce() -> F,
    ) -> Result<T, E> {
        self.tick().await;
        f().await
    }
}
