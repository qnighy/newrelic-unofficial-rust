use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UtilizationData {
    metadata_version: i32,
    logical_processors: Option<i32>,
    total_ram_mib: Option<u64>,
    hostname: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    full_hostname: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ip_address: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    boot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    config: Option<ConfigOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vendors: Option<Vendors>,
}

impl UtilizationData {
    pub(crate) fn gather() -> Self {
        let hostname = if let Ok(hostname) = hostname::get() {
            hostname.to_string_lossy().into_owned()
        } else {
            "unknown".to_owned()
        };
        UtilizationData {
            metadata_version: 5,
            // TODO
            logical_processors: Some(4),
            // TODO
            total_ram_mib: Some(16305),
            hostname,
            // TODO
            full_hostname: "".to_owned(),
            // TODO
            ip_address: vec![
                "192.168.1.3".to_owned(),
                "2409:10:87e0:3802:4ef:176c:9999:c5".to_owned(),
                "2409:10:87e0:3802:5af:37c9:9af:785a".to_owned(),
                "fe80::84ea:76c:499:1".to_owned(),
            ],
            // TODO
            boot_id: "34caa33e-b1dd-4511-a27e-952e12f1ee3b".to_owned(),
            // TODO
            config: None,
            // TODO
            vendors: None,
        }
    }

    pub(crate) fn hostname(&self) -> &str {
        &self.hostname
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    logical_processors: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    total_ram_mib: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vendors {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aws: Option<Aws>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    azure: Option<Azure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gcp: Option<Gcp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pcf: Option<Pcf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    docker: Option<Docker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kubernetes: Option<Kubernetes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Aws {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instance_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    availability_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Azure {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vm_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vm_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Gcp {
    #[serde(with = "numeric_string")]
    id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    machine_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pcf {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cf_instance_guid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cf_instance_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Docker {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Kubernetes {
    kubernetes_service_host: String,
}

mod numeric_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::borrow::Cow;
    use std::fmt::Display;
    use std::str::FromStr;

    pub(super) fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }

    pub(super) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        let s = Cow::<str>::deserialize(deserializer)?;
        s.parse::<T>().map_err(serde::de::Error::custom)
    }
}
