// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::domain_defs::AgentRunId;
use crate::limits::MAX_METRICS;
use crate::payloads::metrics::{CollectorPayload, MetricId, MetricValue};

impl From<Metric> for MetricValue {
    fn from(m: Metric) -> Self {
        Self {
            count_satisfied: m.count_satisfied,
            total_tolerated: m.total_tolerated,
            exclusive_failed: m.exclusive_failed,
            min: m.min,
            max: m.max,
            sum_squares: m.sum_squares,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MetricTable {
    start: Instant,
    failed_harvests: usize,
    max_table_size: usize,
    metrics: HashMap<MetricId, Metric>,
}

impl MetricTable {
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
            failed_harvests: 0,
            max_table_size: MAX_METRICS,
            metrics: HashMap::new(),
        }
    }

    pub(crate) fn add_count(&mut self, name: &str, scope: Option<&str>, count: f64, forced: bool) {
        let metric = Metric::from_count(count);
        self.add(name, scope, metric, forced);
    }

    pub(crate) fn add_duration(
        &mut self,
        name: &str,
        scope: Option<&str>,
        duration: Duration,
        exclusive: Duration,
        forced: bool,
    ) {
        let metric = Metric::from_duration(duration, exclusive);
        self.add(name, scope, metric, forced);
    }

    fn add(&mut self, name: &str, scope: Option<&str>, metric: Metric, forced: bool) {
        use std::collections::hash_map::Entry;

        let id = MetricId {
            name: name.to_owned(),
            scope: scope.map(|s| s.to_owned()),
        };
        let len = self.metrics.len();
        match self.metrics.entry(id) {
            Entry::Occupied(mut entry) => {
                entry.insert(Metric::merge(*entry.get(), metric));
                return;
            }
            Entry::Vacant(entry) => {
                if len <= self.max_table_size || forced {
                    entry.insert(metric);
                    return;
                }
            }
        }
        self.add_count(crate::metric_names::SUPPORTABILITY_DROPPED, None, 1.0, true);
    }

    pub(crate) fn payload(&self, run_id: &AgentRunId) -> CollectorPayload {
        let duration = Instant::now()
            .checked_duration_since(self.start)
            .unwrap_or_else(|| Duration::from_secs(0));
        let end = SystemTime::now();
        let start = end - duration;
        CollectorPayload {
            agent_run_id: run_id.clone(),
            start: start.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            end: end.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            metrics: self
                .metrics
                .iter()
                .map(|(id, &metric)| (id.clone(), metric.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Metric {
    count_satisfied: f64,
    total_tolerated: f64,
    exclusive_failed: f64,
    min: f64,
    max: f64,
    sum_squares: f64,
}

impl Metric {
    fn merge(self, other: Metric) -> Metric {
        Self {
            count_satisfied: self.count_satisfied + other.count_satisfied,
            total_tolerated: self.total_tolerated + other.total_tolerated,
            exclusive_failed: self.exclusive_failed + other.exclusive_failed,
            min: f64::min(self.min, other.min),
            max: f64::max(self.max, other.max),
            sum_squares: self.sum_squares + other.sum_squares,
        }
    }
    fn from_duration(duration: Duration, exclusive: Duration) -> Self {
        let ds = duration.as_secs_f64();
        Self {
            count_satisfied: 1.0,
            total_tolerated: ds,
            exclusive_failed: exclusive.as_secs_f64(),
            min: ds,
            max: ds,
            sum_squares: ds * ds,
        }
    }
    fn from_count(count: f64) -> Self {
        Self {
            count_satisfied: count,
            total_tolerated: 0.0,
            exclusive_failed: 0.0,
            min: 0.0,
            max: 0.0,
            sum_squares: 0.0,
        }
    }
}

impl Default for Metric {
    fn default() -> Self {
        Self {
            count_satisfied: 0.0,
            total_tolerated: 0.0,
            exclusive_failed: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum_squares: 0.0,
        }
    }
}
