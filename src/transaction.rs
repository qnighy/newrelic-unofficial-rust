use std::sync::Weak;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::analytics_events::{
    AgentAttrs, AnalyticsEvent, AnalyticsEventWithAttrs, TransactionEvent, TransactionShared,
    UserAttrs,
};
use crate::{AppState, ApplicationInner};

#[derive(Debug)]
pub struct Transaction {
    app: Weak<ApplicationInner>,
    start: Instant,
    name: String,
    is_web: bool,
}

impl Transaction {
    pub(crate) fn new(app: &Weak<ApplicationInner>, name: &str) -> Self {
        Transaction {
            app: app.clone(),
            start: Instant::now(),
            name: name.to_owned(),
            is_web: false,
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(app) = self.app.upgrade() {
            let mut state = app.state.lock();
            if let AppState::Running { run: _, harvest } = &mut *state {
                let duration = Instant::now()
                    .checked_duration_since(self.start)
                    .unwrap_or(Duration::from_secs(0));
                let end = SystemTime::now();
                let start = end - duration;
                let attrs = AnalyticsEventWithAttrs(
                    AnalyticsEvent::Transaction(TransactionEvent {
                        name: format!("OtherTransaction/Go/{}", self.name),
                        timestamp: start.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
                        apdex_perf_zone: None,
                        error: false,
                        shared: TransactionShared {
                            duration: duration.as_secs_f64(),
                            queue_duration: None,
                            external_call_count: None,
                            external_duration: None,
                            database_call_count: None,
                            database_duration: None,
                            synthetics_resource_id: None,
                            synthetics_job_id: None,
                            synthetics_monitor_id: None,
                        },
                        total_time: duration.as_secs_f64(),
                    }),
                    UserAttrs {},
                    AgentAttrs {},
                );
                harvest.txn_events.push(attrs);
            }
        }
    }
}
