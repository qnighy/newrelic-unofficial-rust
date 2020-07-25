use serde::{Deserialize, Serialize};
use sysinfo::SystemExt;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    config: Option<ConfigOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vendors: Option<Vendors>,
}

impl UtilizationData {
    pub(crate) fn gather() -> Self {
        let mut system = sysinfo::System::new_all();
        system.refresh_memory();
        let logical_processors = system.get_processors().len();
        let total_ram_mib = system.get_total_memory() / 1024; // KiB -> MiB
        let hostname = if let Ok(hostname) = hostname::get() {
            hostname.to_string_lossy().into_owned()
        } else {
            "unknown".to_owned()
        };
        let ip_address = ip_addresses().unwrap_or_else(|e| {
            log::debug!("error gathering ip addresses: {}", e);
            vec![]
        });
        let boot_id = boot_id().unwrap_or_else(|e| {
            log::debug!("error gathering boot id: {}", e);
            None
        });
        UtilizationData {
            metadata_version: 5,
            logical_processors: Some(logical_processors as i32),
            total_ram_mib: Some(total_ram_mib),
            hostname,
            // TODO
            full_hostname: "".to_owned(),
            ip_address,
            boot_id,
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

fn ip_addresses() -> std::io::Result<Vec<String>> {
    use std::net::{SocketAddr, UdpSocket};

    let zero = &[
        "0.0.0.0:0".parse::<SocketAddr>().unwrap(),
        "[::]:0".parse::<SocketAddr>().unwrap(),
    ][..];
    let socket = UdpSocket::bind(zero)?;
    socket.set_broadcast(true)?;
    socket.connect("newrelic.com:10002")?;
    let addr = socket.local_addr()?;
    if addr.ip().is_loopback() || addr.ip().is_unspecified() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unexpected connection address: {:?}", addr),
        ));
    }
    let ip = addr.ip();
    let ifaces = get_if_addrs::get_if_addrs()?;
    if let Some(name) = ifaces
        .iter()
        .find(|iface| iface.ip() == ip)
        .map(|iface| &iface.name)
    {
        let addrs = ifaces
            .iter()
            .filter(|iface| &iface.name == name)
            .map(|iface| iface.ip().to_string())
            .collect::<Vec<_>>();
        Ok(addrs)
    } else {
        Ok(vec![])
    }
}

fn boot_id() -> std::io::Result<Option<String>> {
    use std::fs::read_to_string;
    use std::io;

    if std::env::consts::OS != "linux" {
        return Ok(None);
    }

    let content = read_to_string("/proc/sys/kernel/random/boot_id")?;
    let content = content.trim();
    if !content.is_ascii() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "boot_id contains non-ascii letters",
        ));
    }
    let content = if content.len() > 128 {
        &content[..128] // it must succeed because of the check above
    } else {
        content
    };
    Ok(Some(content.to_owned()))
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
