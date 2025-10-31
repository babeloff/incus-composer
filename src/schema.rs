use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for incus-compose.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncusCompose {
    /// Version of the incus-compose schema
    #[serde(default = "default_version")]
    pub version: String,

    /// Collection of containers/VMs to manage
    #[serde(default)]
    pub containers: HashMap<String, Container>,

    /// Global networks configuration
    #[serde(default)]
    pub networks: HashMap<String, Network>,

    /// Global storage pools configuration
    #[serde(default)]
    pub storage: HashMap<String, StoragePool>,

    /// Global profiles to apply
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Container or VM definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Type of instance: container or virtual-machine
    #[serde(default = "default_container_type")]
    pub instance_type: InstanceType,

    /// Base image to use
    pub image: String,

    /// Image server (e.g., "images:", "ubuntu:", "ubuntu-daily:")
    #[serde(default = "default_image_server")]
    pub image_server: String,

    /// Container/VM description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Container/VM configuration
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Device configuration
    #[serde(default)]
    pub devices: HashMap<String, Device>,

    /// Network interfaces
    #[serde(default)]
    pub networks: Vec<String>,

    /// Storage volumes to attach
    #[serde(default)]
    pub volumes: Vec<Volume>,

    /// Profiles to apply
    #[serde(default)]
    pub profiles: Vec<String>,

    /// CPU limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<CpuLimits>,

    /// Memory limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryLimits>,

    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,

    /// Whether the instance should start automatically
    #[serde(default = "default_true")]
    pub autostart: bool,

    /// Boot order priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_priority: Option<u32>,

    /// Dependencies - containers that should start before this one
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Cloud-init configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_init: Option<CloudInit>,
}

fn default_container_type() -> InstanceType {
    InstanceType::Container
}

fn default_image_server() -> String {
    "images:".to_string()
}

fn default_true() -> bool {
    true
}

/// Instance type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceType {
    Container,
    VirtualMachine,
}

/// CPU resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLimits {
    /// Number of CPU cores (e.g., "2" or "1-3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,

    /// CPU allowance (percentage, e.g., "50%")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowance: Option<String>,

    /// CPU priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
}

/// Memory resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Memory limit (e.g., "2GB", "512MB")
    pub limit: String,

    /// Memory swap limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,

    /// Memory swap priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_priority: Option<u32>,
}

/// Device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Device {
    Disk {
        source: String,
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        readonly: Option<bool>,
    },
    Nic {
        network: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hwaddr: Option<String>,
    },
    Proxy {
        listen: String,
        connect: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        bind: Option<String>,
    },
    Gpu {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        vendorid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        productid: Option<String>,
    },
    Usb {
        #[serde(skip_serializing_if = "Option::is_none")]
        vendorid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        productid: Option<String>,
    },
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    /// Source path or volume name
    pub source: String,

    /// Target path in container
    pub target: String,

    /// Storage pool to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,

    /// Whether the volume is readonly
    #[serde(default)]
    pub readonly: bool,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Network type (bridge, macvlan, sriov, ovn, physical)
    #[serde(rename = "type")]
    pub network_type: NetworkType,

    /// Network configuration options
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Network description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Network types supported by Incus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    Bridge,
    Macvlan,
    Sriov,
    Ovn,
    Physical,
}

/// Storage pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePool {
    /// Pool driver (dir, btrfs, lvm, zfs, ceph)
    pub driver: StorageDriver,

    /// Pool configuration
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Pool description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Storage drivers supported by Incus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageDriver {
    Dir,
    Btrfs,
    Lvm,
    Zfs,
    Ceph,
}

/// Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Configuration options
    #[serde(default)]
    pub config: HashMap<String, String>,

    /// Devices in the profile
    #[serde(default)]
    pub devices: HashMap<String, Device>,
}

/// Cloud-init configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInit {
    /// User data (cloud-config)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,

    /// Network config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_config: Option<String>,

    /// Vendor data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic_compose() {
        let mut containers = HashMap::new();
        containers.insert(
            "web".to_string(),
            Container {
                instance_type: InstanceType::Container,
                image: "ubuntu/22.04".to_string(),
                image_server: "images:".to_string(),
                description: Some("Web server container".to_string()),
                config: HashMap::new(),
                devices: HashMap::new(),
                networks: vec!["default".to_string()],
                volumes: vec![],
                profiles: vec!["default".to_string()],
                cpu: None,
                memory: Some(MemoryLimits {
                    limit: "1GB".to_string(),
                    swap: None,
                    swap_priority: None,
                }),
                environment: HashMap::new(),
                autostart: true,
                boot_priority: None,
                depends_on: vec![],
                cloud_init: None,
            },
        );

        let compose = IncusCompose {
            version: "1.0".to_string(),
            containers,
            networks: HashMap::new(),
            storage: HashMap::new(),
            profiles: HashMap::new(),
        };

        let yaml = serde_yaml::to_string(&compose).unwrap();
        assert!(yaml.contains("version"));
        assert!(yaml.contains("web"));
    }

    #[test]
    fn test_deserialize_basic_compose() {
        let yaml = r#"
version: "1.0"
containers:
  web:
    instance_type: container
    image: "ubuntu/22.04"
    image_server: "images:"
    memory:
      limit: "1GB"
"#;

        let compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(compose.version, "1.0");
        assert!(compose.containers.contains_key("web"));
    }
}
