use std::time::Duration;

pub(crate) const FIXED_HARVEST_PERIOD: Duration = Duration::from_secs(60);
// pub(crate) const COLLECTOR_TIMEOUT: Duration = Duration::from_secs(20);
pub(crate) const MAX_METRICS: usize = 2 * 1000;
pub(crate) const DEFAULT_REPORT_PERIOD_MS: u32 = 60 * 1000;
pub(crate) const MAX_PAYLOAD_SIZE: usize = 1000 * 1000;
pub(crate) const MAX_CUSTOM_EVENTS: u32 = 10 * 1000;
pub(crate) const MAX_TXN_EVENTS: u32 = 10 * 1000;
pub(crate) const MAX_ERROR_EVENTS: u32 = 100;

pub(crate) const DEFAULT_CONFIGURABLE_EVENT_HARVEST: Duration = Duration::from_secs(60);
