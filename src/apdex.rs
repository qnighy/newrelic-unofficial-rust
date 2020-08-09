use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

fn apdex_failing_threshold(threshold: Duration) -> Duration {
    threshold * 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ApdexZoneParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ApdexZone {
    Satisfying,
    Tolerating,
    Failing,
}

impl ApdexZone {
    pub(crate) fn calculate(threshold: Duration, duration: Duration) -> Self {
        if duration < threshold {
            ApdexZone::Satisfying
        } else if duration < apdex_failing_threshold(threshold) {
            ApdexZone::Tolerating
        } else {
            ApdexZone::Failing
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ApdexZone::Satisfying => "S",
            ApdexZone::Tolerating => "T",
            ApdexZone::Failing => "F",
        }
    }
}

impl FromStr for ApdexZone {
    type Err = ApdexZoneParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "S" => ApdexZone::Satisfying,
            "T" => ApdexZone::Tolerating,
            "F" => ApdexZone::Failing,
            _ => return Err(ApdexZoneParseError),
        })
    }
}

impl fmt::Display for ApdexZone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ApdexZone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ApdexZone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected};

        let s = Cow::<str>::deserialize(deserializer)?;
        FromStr::from_str(&s)
            .map_err(|_| Error::invalid_value(Unexpected::Str(&s), &"a string S, T, or F"))
    }
}
