// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

// const APDEX_ROLLUP: &str = "Apdex";
// const APDEX_PREFIX: &str = "Apdex/";

pub(crate) const WEB_METRIC_PREFIX: &str = "WebTransaction/Go";
pub(crate) const BACKGROUND_METRIC_PREFIX: &str = "OtherTransaction/Go";

const WEB_ROLLUP: &str = "WebTransaction";
const BACKGROUND_ROLLUP: &str = "OtherTransaction/all";

const TOTAL_TIME_WEB: &str = "WebTransactionTotalTime";
const TOTAL_TIME_BACKGROUND: &str = "OtherTransactionTotalTime";

pub(crate) fn rollup_name(is_web: bool) -> &'static str {
    if is_web {
        WEB_ROLLUP
    } else {
        BACKGROUND_ROLLUP
    }
}

pub(crate) fn total_time_name(name: &str, is_web: bool) -> String {
    let name_without_first_segment = if let Some(pos) = name.find('/') {
        &name[pos + 1..]
    } else {
        &name
    };
    let prefix = if is_web {
        TOTAL_TIME_WEB
    } else {
        TOTAL_TIME_BACKGROUND
    };
    format!("{}/{}", prefix, name_without_first_segment)
}

pub(crate) fn total_time_rollup_name(is_web: bool) -> &'static str {
    if is_web {
        TOTAL_TIME_WEB
    } else {
        TOTAL_TIME_BACKGROUND
    }
}

// const ERRORS_PREFIX: &str = "Errors/";

// "HttpDispatcher" metric is used for the overview graph, and
// therefore should only be made for web transactions.
pub(crate) const DISPATCHER_METRIC: &str = "HttpDispatcher";

pub(crate) const INSTANCE_REPORTING: &str = "Instance/Reporting";

pub(crate) const SUPPORTABILITY_DROPPED: &str = "Supportability/MetricsDropped";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_time_name() {
        assert_eq!(
            total_time_name("WebTransaction/Go/test", true),
            "WebTransactionTotalTime/Go/test"
        );
        assert_eq!(
            total_time_name("OtherTransaction/Go/test", false),
            "OtherTransactionTotalTime/Go/test"
        );
        assert_eq!(total_time_name("foo", true), "WebTransactionTotalTime/foo");
        assert_eq!(
            total_time_name("foo", false),
            "OtherTransactionTotalTime/foo"
        );
    }
}
