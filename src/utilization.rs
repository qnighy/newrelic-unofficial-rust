// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

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
    #[serde(default, skip_serializing_if = "Vendors::is_empty")]
    vendors: Vendors,
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
        let docker = Docker::gather().unwrap_or_else(|e| {
            log::debug!("error gathering docker: {}", e);
            None
        });
        let vendors = Vendors {
            aws: None,
            azure: None,
            gcp: None,
            pcf: None,
            docker,
            kubernetes: Kubernetes::gather(),
        };
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
            vendors,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Vendors {
    fn is_empty(&self) -> bool {
        self.aws.is_none()
            && self.azure.is_none()
            && self.gcp.is_none()
            && self.pcf.is_none()
            && self.docker.is_none()
            && self.kubernetes.is_none()
    }
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

impl Docker {
    fn gather() -> std::io::Result<Option<Self>> {
        use std::fs::read_to_string;
        use std::io;

        if std::env::consts::OS != "linux" {
            return Ok(None);
        }
        let content = read_to_string("/proc/self/cgroup")?;
        for line in content.lines() {
            let parts = line.split(':').collect::<Vec<_>>();
            if parts.len() < 3 {
                continue;
            }
            if !parts[1].split(',').any(|s| s == "cpu") {
                continue;
            }
            if let Some(docker_id) = find_docker_id(parts[2]) {
                if docker_id.len() != DOCKER_ID_LENGTH {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{} is not {} characters long", docker_id, DOCKER_ID_LENGTH),
                    ));
                }
                return Ok(Some(Self {
                    id: Some(docker_id.to_owned()),
                }));
            }
        }

        Ok(Some(Self { id: None }))
    }
}

const DOCKER_ID_LENGTH: usize = 64;

fn find_docker_id(s: &str) -> Option<&str> {
    let mut start = 0;
    for i in 0..s.len() {
        let byte = s.as_bytes()[i];
        if b'0' <= byte && byte <= b'9' || b'a' <= byte && byte <= b'f' {
            // continue
        } else if i - start >= DOCKER_ID_LENGTH {
            return Some(&s[start..i]);
        } else {
            start = i + 1;
        }
    }
    if s.len() - start >= DOCKER_ID_LENGTH {
        return Some(&s[start..]);
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Kubernetes {
    kubernetes_service_host: String,
}

impl Kubernetes {
    fn gather() -> Option<Self> {
        let value = std::env::var_os("KUBERNETES_SERVICE_HOST")?;
        Some(Self {
            kubernetes_service_host: value.to_string_lossy().into_owned(),
        })
    }
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
