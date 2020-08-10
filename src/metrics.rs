// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::domain_defs::AgentRunId;
use crate::limits::MAX_METRICS;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct CollectorPayload(
    pub(crate) AgentRunId,
    /// period start (unix time)
    pub(crate) i64,
    /// period end (unix time)
    pub(crate) i64,
    pub(crate) Vec<(MetricId, MetricValue)>,
);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct MetricId {
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct MetricValue(
    /// count_satisfied
    pub(crate) f64,
    /// total_tolerated
    pub(crate) f64,
    /// exclusive_failed
    pub(crate) f64,
    /// min
    pub(crate) f64,
    /// max
    pub(crate) f64,
    /// sum_squares
    pub(crate) f64,
);

impl From<Metric> for MetricValue {
    fn from(m: Metric) -> Self {
        Self(
            m.count_satisfied,
            m.total_tolerated,
            m.exclusive_failed,
            m.min,
            m.max,
            m.sum_squares,
        )
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

    pub(crate) fn add_duration(
        &mut self,
        name: &str,
        scope: Option<&str>,
        duration: Duration,
        exclusive: Duration,
        _forced: bool,
    ) {
        let id = MetricId {
            name: name.to_owned(),
            scope: scope.map(|s| s.to_owned()),
        };
        let entry = self.metrics.entry(id).or_default();
        *entry = Metric::merge(*entry, Metric::from_duration(duration, exclusive))
    }

    pub(crate) fn payload(&self, run_id: &AgentRunId) -> CollectorPayload {
        let duration = Instant::now()
            .checked_duration_since(self.start)
            .unwrap_or(Duration::from_secs(0));
        let end = SystemTime::now();
        let start = end - duration;
        CollectorPayload(
            run_id.clone(),
            start.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            end.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            self.metrics
                .iter()
                .map(|(id, &metric)| (id.clone(), metric.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct Metric {
    forced: bool,
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
            forced: self.forced || other.forced,
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
            forced: false,
            count_satisfied: 1.0,
            total_tolerated: ds,
            exclusive_failed: exclusive.as_secs_f64(),
            min: ds,
            max: ds,
            sum_squares: ds * ds,
        }
    }
    // fn from_count(count: f64) -> Self {
    //     Self {
    //         forced: false,
    //         count_satisfied: count,
    //         total_tolerated: 0.0,
    //         exclusive_failed: 0.0,
    //         min: 0.0,
    //         max: 0.0,
    //         sum_squares: 0.0,
    //     }
    // }
}

impl Default for Metric {
    fn default() -> Self {
        Self {
            forced: false,
            count_satisfied: 0.0,
            total_tolerated: 0.0,
            exclusive_failed: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum_squares: 0.0,
        }
    }
}
