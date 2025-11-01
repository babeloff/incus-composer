use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Root structure for incus-compose.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncusCompose {
    /// Version of the incus-compose schema
    #[serde(default = "default_version")]
    pub version: String,

    /// Default configuration for optional elements
    #[serde(default)]
    pub defaults: Defaults,

    /// Collection of hosts to manage
    pub hosts: Vec<Host>,

    /// Network subnets configuration
    pub subnets: Vec<Subnet>,

    /// Global flavors configuration (optional, can be defined externally)
    #[serde(default)]
    pub flavors: HashMap<String, Flavor>,

    /// Global images configuration (optional, can be defined externally)
    #[serde(default)]
    pub images: HashMap<String, Image>,
}

/// Expanded lockfile structure with all optional fields made explicit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncusLockfile {
    /// Version of the incus-compose schema
    pub version: String,

    /// Default configuration used during generation
    pub defaults: Defaults,

    /// Collection of hosts with all optional fields populated
    pub hosts: Vec<ExpandedHost>,

    /// Network subnets with all optional fields populated
    pub subnets: Vec<ExpandedSubnet>,

    /// Resolved flavor definitions
    pub flavors: HashMap<String, Flavor>,

    /// Resolved image definitions
    pub images: HashMap<String, Image>,

    /// Generated metadata
    pub metadata: LockfileMetadata,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Default configuration for optional elements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Defaults {
    /// IP address ranges for regular hosts
    #[serde(default)]
    pub host_ip4_ranges: Vec<IpRange>,

    /// IP address ranges for router hosts
    #[serde(default)]
    pub router_ip4_ranges: Vec<IpRange>,

    /// CIDR ranges for automatic subnet assignment
    #[serde(default)]
    pub cidr4_ranges: Vec<CidrRange>,
}

/// IP address range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRange {
    /// Starting IP address
    pub start: String,

    /// Ending IP address
    pub end: String,
}

/// CIDR range specification for subnet assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidrRange {
    /// Starting CIDR block
    pub start: String,

    /// Ending CIDR block
    pub end: String,
}

/// Host definition in incus-compose file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    /// Name of the host
    pub name: String,

    /// Flavor reference (defines resource allocation)
    pub flavor: String,

    /// Image reference
    pub image: String,

    /// Whether this host should have a floating IP
    #[serde(default)]
    pub floating_ip: bool,

    /// Whether this host is the master node
    #[serde(default)]
    pub master: bool,

    /// Whether this host acts as a router
    #[serde(default)]
    pub is_router: bool,

    /// Roles assigned to this host
    #[serde(default)]
    pub roles: Vec<Role>,

    /// Subnet assignments (can be single or multiple)
    #[serde(default)]
    pub subnets: Vec<String>,

    /// Backward compatibility: single subnet assignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,

    /// Backward compatibility: multiple subnet assignments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_list: Option<Vec<String>>,
}

impl Host {
    /// Normalize subnet fields into the subnets field
    pub fn normalize(&mut self) {
        // If subnets is empty, populate it from subnet or subnet_list
        if self.subnets.is_empty() {
            if let Some(ref subnet) = self.subnet {
                self.subnets.push(subnet.clone());
            }
            if let Some(ref subnet_list) = self.subnet_list {
                self.subnets.extend(subnet_list.iter().cloned());
            }
        }

        // Clear the backward compatibility fields
        self.subnet = None;
        self.subnet_list = None;
    }
}

/// Expanded host definition in lockfile with all fields explicit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedHost {
    /// Name of the host
    pub name: String,

    /// Flavor reference
    pub flavor: String,

    /// Image reference
    pub image: String,

    /// Whether this host has a floating IP (always explicit)
    pub floating_ip: bool,

    /// Whether this host is the master node (always explicit)
    pub master: bool,

    /// Whether this host acts as a router (always explicit)
    pub is_router: bool,

    /// Roles assigned to this host (always present, may be empty)
    pub roles: Vec<RoleConfig>,

    /// Subnet assignments (always present, may be empty)
    pub subnets: Vec<String>,

    /// Generated unique identifier
    pub id: String,

    /// Generated MAC address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,

    /// Assigned IP addresses per subnet
    pub ip_addresses: HashMap<String, String>,

    /// Instance type (derived from flavor and configuration)
    pub instance_type: InstanceType,

    /// Resolved resource limits (from flavor)
    pub resources: Resources,
}

/// Role definition
/// Can be either a string (shorthand) or full object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Role {
    /// Shorthand string format (just the role name)
    Name(String),
    /// Full role configuration
    Full(RoleConfig),
}

/// Full role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// Name of the role
    pub name: String,

    /// Optional values/parameters for the role
    #[serde(default)]
    pub values: Vec<String>,
}

impl Role {
    /// Get the role name regardless of format
    pub fn name(&self) -> &str {
        match self {
            Role::Name(name) => name,
            Role::Full(config) => &config.name,
        }
    }

    /// Get the role values
    pub fn values(&self) -> &[String] {
        match self {
            Role::Name(_) => &[],
            Role::Full(config) => &config.values,
        }
    }

    /// Convert to full configuration format
    pub fn to_full_config(self) -> RoleConfig {
        match self {
            Role::Name(name) => RoleConfig {
                name,
                values: vec![],
            },
            Role::Full(config) => config,
        }
    }
}

/// Subnet definition in incus-compose file
/// Can be either a string (shorthand) or full object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Subnet {
    /// Shorthand string format (just the subnet name)
    Name(String),
    /// Full subnet configuration
    Full(SubnetConfig),
}

/// Full subnet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetConfig {
    /// Name of the subnet
    pub name: String,

    /// CIDR notation for the subnet (optional, may be auto-assigned)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr: Option<String>,
}

impl Subnet {
    /// Get the subnet name regardless of format
    pub fn name(&self) -> &str {
        match self {
            Subnet::Name(name) => name,
            Subnet::Full(config) => &config.name,
        }
    }

    /// Get the CIDR if explicitly specified
    pub fn cidr(&self) -> Option<&str> {
        match self {
            Subnet::Name(_) => None,
            Subnet::Full(config) => config.cidr.as_deref(),
        }
    }

    /// Convert to full configuration format
    pub fn to_full_config(self) -> SubnetConfig {
        match self {
            Subnet::Name(name) => SubnetConfig { name, cidr: None },
            Subnet::Full(config) => config,
        }
    }
}

/// Expanded subnet definition in lockfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedSubnet {
    /// Name of the subnet
    pub name: String,

    /// CIDR notation (always explicit in lockfile)
    pub cidr: String,

    /// Generated unique identifier
    pub id: String,

    /// Gateway IP address
    pub gateway: String,

    /// Network type
    #[serde(default = "default_network_type")]
    pub network_type: NetworkType,

    /// Network configuration
    #[serde(default)]
    pub config: HashMap<String, String>,
}

fn default_network_type() -> NetworkType {
    NetworkType::Bridge
}

/// Flavor definition (resource allocation template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flavor {
    /// Flavor name
    pub name: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// CPU allocation
    pub cpu: CpuSpec,

    /// Memory allocation
    pub memory: MemorySpec,

    /// Storage allocation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageSpec>,

    /// Instance type for this flavor
    #[serde(default = "default_instance_type")]
    pub instance_type: InstanceType,
}

fn default_instance_type() -> InstanceType {
    InstanceType::Container
}

/// Image definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Image name/identifier
    pub name: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source (e.g., "images:", "ubuntu:", local path)
    #[serde(default = "default_image_source")]
    pub source: String,

    /// Image fingerprint or tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// Architecture
    #[serde(default = "default_architecture")]
    pub architecture: String,

    /// Operating system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
}

fn default_image_source() -> String {
    "images:".to_string()
}

fn default_architecture() -> String {
    "x86_64".to_string()
}

/// Instance type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceType {
    Container,
    VirtualMachine,
}

/// Network type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    Bridge,
    Macvlan,
    Sriov,
    Ovn,
    Physical,
}

/// CPU specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSpec {
    /// Number of CPU cores
    pub cores: u32,

    /// CPU limit (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,

    /// CPU allowance (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowance: Option<String>,

    /// CPU priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
}

/// Memory specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpec {
    /// Memory limit (e.g., "2GB", "512MB")
    pub limit: String,

    /// Memory swap limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,

    /// Memory swap priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_priority: Option<u32>,
}

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    /// Storage size
    pub size: String,

    /// Storage pool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,

    /// Storage type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_type: Option<String>,
}

/// Resolved resource limits (combination of CPU, memory, storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    /// CPU specification
    pub cpu: CpuSpec,

    /// Memory specification
    pub memory: MemorySpec,

    /// Storage specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageSpec>,
}

/// Lockfile metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileMetadata {
    /// Generation timestamp
    pub generated_at: String,

    /// Generator version
    pub generator_version: String,

    /// Source compose file hash
    pub source_hash: String,

    /// Used value tracker for uniqueness
    #[serde(default)]
    pub used_values: UsedValues,
}

/// Tracker for used values to ensure uniqueness
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsedValues {
    /// Used IP addresses
    #[serde(default)]
    pub ip_addresses: HashMap<String, Vec<String>>,

    /// Used MAC addresses
    #[serde(default)]
    pub mac_addresses: Vec<String>,

    /// Used host IDs
    #[serde(default)]
    pub host_ids: Vec<String>,

    /// Used subnet IDs
    #[serde(default)]
    pub subnet_ids: Vec<String>,
}

impl IncusCompose {
    /// Load an incus-compose.yaml file from disk
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut compose: IncusCompose = serde_yaml::from_str(&content)?;

        // Normalize legacy subnet fields
        for host in &mut compose.hosts {
            host.normalize();
        }

        Ok(compose)
    }

    /// Generate a lockfile from this compose configuration
    pub fn generate_lockfile(&self) -> IncusLockfile {
        let mut used_values = UsedValues::default();
        let mut expanded_hosts = Vec::new();
        let mut expanded_subnets = Vec::new();

        // Generate expanded subnets first (needed for IP allocation)
        for (idx, subnet) in self.subnets.iter().enumerate() {
            let subnet_id = format!("subnet_{:03}", idx + 1);
            let subnet_name = subnet.name();

            // Use explicit CIDR or auto-assign
            let cidr = subnet
                .cidr()
                .map(|c| c.to_string())
                .unwrap_or_else(|| self.auto_assign_cidr(&mut used_values));

            // Calculate gateway (typically .1)
            let gateway = self.calculate_gateway(&cidr);

            expanded_subnets.push(ExpandedSubnet {
                name: subnet_name.to_string(),
                cidr: cidr.clone(),
                id: subnet_id.clone(),
                gateway,
                network_type: default_network_type(),
                config: HashMap::new(),
            });

            used_values.subnet_ids.push(subnet_id);
        }

        // Generate expanded hosts
        for (idx, host) in self.hosts.iter().enumerate() {
            let host_id = format!("host_{:03}", idx + 1);
            let mac_address = self.generate_mac_address(&mut used_values);

            // Assign IP addresses for each subnet
            let mut ip_addresses = HashMap::new();
            for subnet_name in &host.subnets {
                if let Some(expanded_subnet) =
                    expanded_subnets.iter().find(|s| &s.name == subnet_name)
                {
                    let ip = self.assign_ip_address(
                        &expanded_subnet.cidr,
                        host.is_router,
                        &mut used_values,
                    );
                    ip_addresses.insert(subnet_name.clone(), ip);
                }
            }

            // Resolve instance type and resources from flavor (simplified for now)
            let instance_type = default_instance_type();
            let resources = Resources {
                cpu: CpuSpec {
                    cores: 2, // Default values - should be looked up from flavor
                    limit: Some("100%".to_string()),
                    allowance: None,
                    priority: None,
                },
                memory: MemorySpec {
                    limit: "2GB".to_string(), // Default - should be from flavor
                    swap: None,
                    swap_priority: None,
                },
                storage: None,
            };

            expanded_hosts.push(ExpandedHost {
                name: host.name.clone(),
                flavor: host.flavor.clone(),
                image: host.image.clone(),
                floating_ip: host.floating_ip,
                master: host.master,
                is_router: host.is_router,
                roles: host
                    .roles
                    .iter()
                    .map(|r| r.clone().to_full_config())
                    .collect(),
                subnets: host.subnets.clone(),
                id: host_id.clone(),
                mac_address: Some(mac_address),
                ip_addresses,
                instance_type,
                resources,
            });

            used_values.host_ids.push(host_id);
        }

        IncusLockfile {
            version: self.version.clone(),
            hosts: expanded_hosts,
            subnets: expanded_subnets,
            flavors: self.flavors.clone(),
            images: self.images.clone(),
            defaults: self.defaults.clone(),
            metadata: LockfileMetadata {
                generated_at: simple_timestamp(),
                generator_version: "0.1.0".to_string(),
                source_hash: self.calculate_hash(),
                used_values,
            },
        }
    }

    /// Auto-assign a CIDR block from configured ranges
    fn auto_assign_cidr(&self, used_values: &mut UsedValues) -> String {
        // Simplified implementation - should use actual CIDR range logic
        let base_cidr = "192.168.{}.0/24";
        let subnet_num = used_values.subnet_ids.len() + 10; // Start from 192.168.10.0/24
        base_cidr.replace("{}", &subnet_num.to_string())
    }

    /// Calculate gateway IP for a CIDR block
    fn calculate_gateway(&self, cidr: &str) -> String {
        // Simplified - typically .1 of the network
        if let Some(network_part) = cidr.split('/').next() {
            let parts: Vec<&str> = network_part.split('.').collect();
            if parts.len() == 4 {
                return format!("{}.{}.{}.1", parts[0], parts[1], parts[2]);
            }
        }
        "192.168.1.1".to_string() // Fallback
    }

    /// Generate a unique MAC address
    fn generate_mac_address(&self, used_values: &mut UsedValues) -> String {
        let mut counter = used_values.mac_addresses.len() + 1;
        loop {
            let mac = format!(
                "02:00:00:00:{:02x}:{:02x}",
                (counter >> 8) & 0xff,
                counter & 0xff
            );
            if !used_values.mac_addresses.contains(&mac) {
                used_values.mac_addresses.push(mac.clone());
                return mac;
            }
            counter += 1;
        }
    }

    /// Assign IP address within a subnet
    fn assign_ip_address(
        &self,
        cidr: &str,
        is_router: bool,
        used_values: &mut UsedValues,
    ) -> String {
        // Simplified implementation
        let network_base = if let Some(network_part) = cidr.split('/').next() {
            let parts: Vec<&str> = network_part.split('.').collect();
            if parts.len() == 4 {
                format!("{}.{}.{}", parts[0], parts[1], parts[2])
            } else {
                "192.168.1".to_string()
            }
        } else {
            "192.168.1".to_string()
        };

        let subnet_name = format!("subnet_{}", network_base.replace(".", "_"));
        let used_ips = used_values
            .ip_addresses
            .entry(subnet_name.clone())
            .or_insert_with(Vec::new);

        // Start from .10 for regular hosts, .2 for routers (after gateway .1)
        let start_ip = if is_router { 2 } else { 10 };

        for i in start_ip..255 {
            let ip = format!("{}.{}", network_base, i);
            if !used_ips.contains(&ip) {
                used_ips.push(ip.clone());
                return ip;
            }
        }

        format!("{}.100", network_base) // Fallback
    }

    /// Calculate hash of the compose file for change detection
    fn calculate_hash(&self) -> String {
        // Simplified implementation - should use proper hashing
        format!(
            "sha256:abc123def456_{}",
            self.hosts.len() + self.subnets.len()
        )
    }
}

impl IncusLockfile {
    /// Save lockfile to disk
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }

    /// Load lockfile from disk
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let lockfile: IncusLockfile = serde_yaml::from_str(&content)?;
        Ok(lockfile)
    }

    /// Generate incus commands for dry-run
    pub fn generate_incus_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        // Create networks first
        for subnet in &self.subnets {
            commands.push(format!(
                "incus network create {} --type=bridge",
                subnet.name
            ));
            commands.push(format!(
                "incus network set {} ipv4.address={}",
                subnet.name, subnet.gateway
            ));
            commands.push(format!("incus network set {} ipv4.dhcp=false", subnet.name));
        }

        // Create instances
        for host in &self.hosts {
            let instance_type = match host.instance_type {
                InstanceType::Container => "container",
                InstanceType::VirtualMachine => "virtual-machine",
            };

            commands.push(format!(
                "incus create {} {} --type={}",
                host.image, host.name, instance_type
            ));

            // Set resource limits
            commands.push(format!(
                "incus config set {} limits.cpu={}",
                host.name, host.resources.cpu.cores
            ));
            commands.push(format!(
                "incus config set {} limits.memory={}",
                host.name, host.resources.memory.limit
            ));

            // Set MAC address
            if let Some(ref mac) = host.mac_address {
                commands.push(format!(
                    "incus config device add {} eth0 nic network={} hwaddr={}",
                    host.name,
                    host.subnets.get(0).unwrap_or(&"bridge".to_string()),
                    mac
                ));
            }

            // Assign to networks and set IP addresses
            for (i, subnet_name) in host.subnets.iter().enumerate() {
                let device_name = if i == 0 {
                    "eth0".to_string()
                } else {
                    format!("eth{}", i)
                };

                if i > 0 {
                    // eth0 already added above
                    commands.push(format!(
                        "incus config device add {} {} nic network={}",
                        host.name, device_name, subnet_name
                    ));
                }

                if let Some(ip) = host.ip_addresses.get(subnet_name) {
                    commands.push(format!(
                        "incus config device set {} {} ipv4.address={}",
                        host.name, device_name, ip
                    ));
                }
            }

            // Configure roles (simplified - would need actual role implementation)
            for role in &host.roles {
                commands.push(format!(
                    "# Apply role '{}' to {} with values: {:?}",
                    role.name, host.name, role.values
                ));
            }

            // Start the instance
            commands.push(format!("incus start {}", host.name));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic_compose() {
        let hosts = vec![Host {
            name: "web-server".to_string(),
            flavor: "small_flavor".to_string(),
            image: "base_image".to_string(),
            floating_ip: false,
            master: false,
            is_router: false,
            roles: vec![Role::Full(RoleConfig {
                name: "web".to_string(),
                values: vec![],
            })],
            subnets: vec!["frontend".to_string()],
            subnet: None,
            subnet_list: None,
        }];

        let subnets = vec![Subnet::Full(SubnetConfig {
            name: "frontend".to_string(),
            cidr: Some("10.0.1.0/24".to_string()),
        })];

        let compose = IncusCompose {
            version: "1.0".to_string(),
            hosts,
            subnets,
            flavors: HashMap::new(),
            images: HashMap::new(),
            defaults: Defaults::default(),
        };

        let yaml = serde_yaml::to_string(&compose).unwrap();
        assert!(yaml.contains("version"));
        assert!(yaml.contains("web-server"));
        assert!(yaml.contains("frontend"));
    }

    #[test]
    fn test_deserialize_hospital_example() {
        let yaml = r#"
hosts:
  - name: web_server
    flavor: small_flavor
    image: base_image
    roles:
      - web
      - name: monitoring
        values: ["prometheus"]
    subnet: frontend

subnets:
  - name: frontend
    cidr: 10.0.1.0/24
  - backend
"#;

        let mut compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();

        // Normalize subnet fields
        for host in &mut compose.hosts {
            host.normalize();
        }

        assert_eq!(compose.version, "1.0"); // default value
        assert_eq!(compose.hosts.len(), 1);
        assert_eq!(compose.subnets.len(), 2);

        let host = &compose.hosts[0];
        assert_eq!(host.name, "web_server");
        assert_eq!(host.roles.len(), 2);
        assert_eq!(host.roles[0].name(), "web");
        assert_eq!(host.roles[0].values(), &[] as &[String]);
        assert_eq!(host.roles[1].name(), "monitoring");
        assert_eq!(host.roles[1].values(), &["prometheus".to_string()]);
        assert_eq!(host.subnets, vec!["frontend"]);

        // Test subnet formats
        assert_eq!(compose.subnets[0].name(), "frontend");
        assert_eq!(compose.subnets[0].cidr(), Some("10.0.1.0/24"));
        assert_eq!(compose.subnets[1].name(), "backend");
        assert_eq!(compose.subnets[1].cidr(), None);
    }

    #[test]
    fn test_router_with_multiple_subnets() {
        let yaml = r#"
hosts:
  - name: core_router
    flavor: medium_flavor
    image: router_image
    is_router: true
    roles:
      - router
    subnet_list:
      - frontend
      - backend
      - dmz

subnets:
  - frontend
  - backend
  - dmz
"#;

        let mut compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();

        // Normalize subnet fields
        for host in &mut compose.hosts {
            host.normalize();
        }

        let host = &compose.hosts[0];
        assert!(host.is_router);
        assert_eq!(host.subnets.len(), 3);
        assert_eq!(host.subnets, vec!["frontend", "backend", "dmz"]);

        // Test shorthand subnet format
        assert_eq!(compose.subnets[0].name(), "frontend");
        assert_eq!(compose.subnets[1].name(), "backend");
        assert_eq!(compose.subnets[2].name(), "dmz");
        for subnet in &compose.subnets {
            assert_eq!(subnet.cidr(), None);
        }
    }

    #[test]
    fn test_defaults_configuration() {
        let yaml = r#"
defaults:
  host_ip4_ranges:
    - start: 192.168.10.100
      end: 192.168.10.200
  router_ip4_ranges:
    - start: 192.168.1.100
      end: 192.168.1.200
  cidr4_ranges:
    - start: 192.168.20.0/16
      end: 192.168.80.0/16

hosts:
  - name: test_host
    flavor: small_flavor
    image: base_image

subnets:
  - test_subnet
"#;

        let compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(compose.defaults.host_ip4_ranges.len(), 1);
        assert_eq!(compose.defaults.host_ip4_ranges[0].start, "192.168.10.100");
        assert_eq!(compose.defaults.host_ip4_ranges[0].end, "192.168.10.200");

        assert_eq!(compose.defaults.router_ip4_ranges.len(), 1);
        assert_eq!(compose.defaults.router_ip4_ranges[0].start, "192.168.1.100");
        assert_eq!(compose.defaults.router_ip4_ranges[0].end, "192.168.1.200");

        assert_eq!(compose.defaults.cidr4_ranges.len(), 1);
        assert_eq!(compose.defaults.cidr4_ranges[0].start, "192.168.20.0/16");
        assert_eq!(compose.defaults.cidr4_ranges[0].end, "192.168.80.0/16");
    }

    #[test]
    fn test_lockfile_generation() {
        let yaml = r#"
hosts:
  - name: test_host
    flavor: small_flavor
    image: base_image
    subnets: [test_subnet]

subnets:
  - name: test_subnet
    cidr: 192.168.100.0/24
"#;

        let compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();
        let lockfile = compose.generate_lockfile();

        assert_eq!(lockfile.hosts.len(), 1);
        assert_eq!(lockfile.subnets.len(), 1);

        let host = &lockfile.hosts[0];
        assert_eq!(host.name, "test_host");
        assert_eq!(host.id, "host_001");
        assert!(host.mac_address.is_some());
        assert_eq!(host.ip_addresses.len(), 1);

        let subnet = &lockfile.subnets[0];
        assert_eq!(subnet.name, "test_subnet");
        assert_eq!(subnet.cidr, "192.168.100.0/24");
        assert_eq!(subnet.gateway, "192.168.100.1");
    }

    #[test]
    fn test_incus_commands_generation() {
        let yaml = r#"
hosts:
  - name: web_server
    flavor: medium_flavor
    image: base_image
    subnets: [frontend]

subnets:
  - name: frontend
    cidr: 10.0.1.0/24
"#;

        let compose: IncusCompose = serde_yaml::from_str(yaml).unwrap();
        let lockfile = compose.generate_lockfile();
        let commands = lockfile.generate_incus_commands();

        assert!(!commands.is_empty());

        // Check that network creation commands are present
        assert!(commands
            .iter()
            .any(|cmd| cmd.contains("incus network create frontend")));

        // Check that instance creation commands are present
        assert!(commands
            .iter()
            .any(|cmd| cmd.contains("incus create base_image web_server")));

        // Check that start command is present
        assert!(commands
            .iter()
            .any(|cmd| cmd.contains("incus start web_server")));
    }
}

// Add chrono dependency for timestamp generation
use std::time::{SystemTime, UNIX_EPOCH};

// Simple timestamp implementation since we don't want to add chrono dependency yet
fn simple_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            format!(
                "2024-01-01T{:02}:{:02}:{:02}Z",
                (secs / 3600) % 24,
                (secs / 60) % 60,
                secs % 60
            )
        }
        Err(_) => "2024-01-01T00:00:00Z".to_string(),
    }
}
