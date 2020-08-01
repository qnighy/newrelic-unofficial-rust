use std::time::{Duration, Instant};

use crate::analytics_events::AnalyticsEventWithAttrs;
use crate::app_run::AppRun;
use crate::collector::collector_request;
use crate::metrics::MetricTable;

#[derive(Debug)]
pub(crate) struct Harvest {
    metrics_traces_timer: HarvestTimer,
    span_events_timer: HarvestTimer,
    custom_events_timer: HarvestTimer,
    txn_events_timer: HarvestTimer,
    error_events_timer: HarvestTimer,
    pub(crate) txn_events: Vec<AnalyticsEventWithAttrs>,
    pub(crate) metric_table: MetricTable,
}

impl Harvest {
    pub(crate) fn new(run: &AppRun) -> Self {
        let last_harvest = Instant::now();
        let new_timer = |duration: Duration| HarvestTimer {
            duration,
            last_harvest,
        };
        Self {
            metrics_traces_timer: new_timer(run.metrics_traces_period),
            span_events_timer: new_timer(run.span_events_period),
            custom_events_timer: new_timer(run.custom_events_period),
            txn_events_timer: new_timer(run.txn_events_period),
            error_events_timer: new_timer(run.error_events_period),
            txn_events: vec![],
            metric_table: MetricTable::new(),
        }
    }

    pub(crate) fn harvest(&mut self, run: &AppRun, force: bool) {
        let now = Instant::now();
        if self.metrics_traces_timer.ready(now, force) {
            eprintln!("Processing metrics traces...");
            let metric_table = std::mem::replace(&mut self.metric_table, MetricTable::new());
            let payload = metric_table.payload(&run.agent_run_id);
            // TODO: handle errors
            collector_request(run, "metric_data", &payload).unwrap();
        }
        if self.span_events_timer.ready(now, force) {
            eprintln!("Processing span events...");
        }
        if self.custom_events_timer.ready(now, force) {
            eprintln!("Processing custom events...");
        }
        if self.txn_events_timer.ready(now, force) {
            use crate::analytics_events::{CollectorPayload, Properties};

            eprintln!("Processing txn events...");
            let txn_events = std::mem::replace(&mut self.txn_events, vec![]);
            // TODO: handle errors
            collector_request(
                run,
                "analytic_event_data",
                &CollectorPayload(
                    run.agent_run_id.clone(),
                    Properties {
                        reservoir_size: txn_events.capacity() as i32,
                        events_seen: txn_events.len() as i32,
                    },
                    txn_events.clone(),
                ),
            )
            .unwrap();
        }
        if self.error_events_timer.ready(now, force) {
            eprintln!("Processing error events...");
        }
    }
}

#[derive(Debug)]
struct HarvestTimer {
    duration: Duration,
    last_harvest: Instant,
}

impl HarvestTimer {
    fn ready(&mut self, now: Instant, force: bool) -> bool {
        if force || now >= self.last_harvest + self.duration {
            self.last_harvest = now;
            true
        } else {
            false
        }
    }
}