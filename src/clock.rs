use derive_more::Display;
use num_traits::{PrimInt, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

pub fn local_timestamp() -> Timestamp {
    Timestamp::from_mills(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .expect("Unix timestamp to be NON-ZERO")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, Ord, PartialOrd, Eq, PartialEq)]
pub struct Timestamp(NonZeroU64);

impl Timestamp {
    pub fn from_mills(num: impl ToPrimitive) -> Option<Self> {
        let num: u64 = num.to_u64()?;
        NonZeroU64::new(num).map(Self)
    }
}

impl Timestamp {
    pub fn as_millis(&self) -> u64 {
        self.0.get()
    }
}

pub struct ClockSource {
    baseline_time_and_instant: watch::Sender<Option<(Timestamp, Instant)>>,
}

impl Default for ClockSource {
    fn default() -> Self {
        let (tx, _) = watch::channel(None);
        ClockSource {
            baseline_time_and_instant: tx,
        }
    }
}

impl ClockSource {
    pub(crate) fn submit_calibration(&self, instant: Instant, timestamp: Timestamp) {
        let error_mills = (local_timestamp().as_millis() as i64) - (timestamp.as_millis() as i64);
        log::info!(
            "Calibrating clock with timestamp: {:?}, current error = {error_mills}ms",
            timestamp
        );

        self.baseline_time_and_instant
            .send_replace(Some((timestamp, instant)));
    }

    pub fn calibrated_now(&self) -> Option<Timestamp> {
        let adjusted = self.baseline_time_and_instant.borrow();
        if let Some(baseline) = *adjusted {
            Some(Self::calibrated(&baseline))
        } else {
            None
        }
    }

    pub fn now_or_uncalibrated(&self) -> Timestamp {
        self.calibrated_now().unwrap_or_else(|| local_timestamp())
    }

    pub async fn await_calibrated(&self) -> Timestamp {
        let mut rx = self.baseline_time_and_instant.subscribe();
        let b = rx.wait_for(|v| v.is_some()).await.unwrap();
        Self::calibrated(b.as_ref().unwrap())
    }

    fn calibrated((baseline_mills, instant): &(Timestamp, Instant)) -> Timestamp {
        let elapsed_mills = instant.elapsed().as_millis() as u64;
        Timestamp::from_mills(baseline_mills.as_millis() + elapsed_mills)
            .expect("Calibrated time to be NON-ZERO")
    }
}
