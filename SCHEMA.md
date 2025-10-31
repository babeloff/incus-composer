# Incus Compose Schema Documentation

Version: 1.0

## Overview

The `incus-compose.yaml` schema is designed to define and manage Incus system containers and virtual machines, similar to how `docker-compose.yml` manages application containers. However, while Docker Compose is focused on application deployment, Incus Compose is focused on system-level infrastructure, including full system containers and virtual machines.

## Key Differences from Docker Compose

- **System vs Application Focus**: Incus manages complete system containers and VMs, not just application containers
- **Native VM Support**: First-class support for virtual machines alongside containers
- **Hardware Device Management**: Direct support for GPU, USB, and other hardware passthrough
- **Network Flexibility**: Support for various network types (bridge, macvlan, OVN, etc.)
- **Storage Pools**: Explicit storage pool management with multiple backend drivers
- **Profile System**: Reusable profiles for common configurations
- **Cloud-init Integration**: Built-in cloud-init support for initialization

## Schema Structure

```yaml
version: "1.0"
containers: {...}
networks: {...}
storage: {...}
profiles: {...}
```

## Top-Level Fields

### version (required)
The version of the incus-compose schema being used.

```yaml
version: "1.0"
```

### containers (required)
A map of container/VM definitions. Each key is the name of the instance.

### networks (optional)
A map of network definitions. Each key is the network name.

### storage (optional)
A map of storage pool definitions. Each key is the pool name.

### profiles (optional)
A map of profile definitions. Each key is the profile name.

## Container Definition

A container definition describes a single Incus instance (container or VM).

### Basic Fields

#### instance_type
Type of instance to create.

**Values**: `container` (default), `virtual-machine`

```yaml
containers:
  myvm:
    instance_type: virtual-machine
```

#### image (required)
The base image to use for the instance.

```yaml
containers:
  web:
    image: "ubuntu/22.04"
```

#### image_server
The image server to pull from.

**Default**: `images:`

**Common values**: `images:`, `ubuntu:`, `ubuntu-daily:`

```yaml
containers:
  web:
    image: "ubuntu/22.04"
    image_server: "ubuntu:"
```

#### description
Human-readable description of the instance.

```yaml
containers:
  web:
    description: "Production web server"
```

### Resource Limits

#### cpu
CPU resource limits and allocation.

```yaml
containers:
  web:
    cpu:
      limit: "2"           # Number of CPUs (or range like "1-3")
      allowance: "50%"     # CPU time percentage
      priority: 10         # Scheduling priority
```

#### memory
Memory resource limits.

```yaml
containers:
  web:
    memory:
      limit: "2GB"         # Memory limit
      swap: "1GB"          # Swap limit
      swap_priority: 5     # Swap priority
```

### Networking

#### networks
List of networks to attach to the instance.

```yaml
containers:
  web:
    networks:
      - frontend
      - backend
```

### Storage

#### volumes
List of storage volumes to attach.

```yaml
containers:
  web:
    volumes:
      - source: "web-data"
        target: "/var/www/html"
        pool: "default"
        readonly: false
```

**Fields**:
- `source`: Volume name or path
- `target`: Mount point in the container
- `pool`: Storage pool to use (optional)
- `readonly`: Mount as read-only (default: false)

### Devices

#### devices
Map of device configurations.

**Device Types**:
- `disk`: Block device or mount
- `nic`: Network interface
- `proxy`: Port proxy
- `gpu`: GPU passthrough
- `usb`: USB device passthrough

```yaml
containers:
  gaming:
    devices:
      gpu0:
        type: gpu
        id: "0"
      
      disk0:
        type: disk
        source: "/dev/sdb"
        path: "/mnt/data"
      
      eth1:
        type: nic
        network: "public"
        name: "eth1"
        hwaddr: "00:16:3e:xx:xx:xx"
      
      proxy-web:
        type: proxy
        listen: "tcp:0.0.0.0:80"
        connect: "tcp:127.0.0.1:8080"
```

### Configuration

#### config
Map of Incus configuration keys and values. These are passed directly to Incus.

```yaml
containers:
  web:
    config:
      security.nesting: "true"
      security.privileged: "false"
      boot.autostart: "true"
```

#### environment
Environment variables to set in the instance.

```yaml
containers:
  app:
    environment:
      APP_ENV: "production"
      DATABASE_URL: "postgresql://localhost/mydb"
```

### Lifecycle

#### autostart
Whether the instance should start automatically.

**Default**: `true`

```yaml
containers:
  web:
    autostart: true
```

#### boot_priority
Boot order priority (higher numbers boot first).

```yaml
containers:
  database:
    boot_priority: 10
  
  web:
    boot_priority: 5
```

#### depends_on
List of instances that should start before this one.

```yaml
containers:
  web:
    depends_on:
      - database
      - cache
```

### Profiles

#### profiles
List of profiles to apply to the instance.

```yaml
containers:
  web:
    profiles:
      - default
      - web-server
```

### Cloud-init

#### cloud_init
Cloud-init configuration for instance initialization.

```yaml
containers:
  web:
    cloud_init:
      user_data: |
        #cloud-config
        packages:
          - nginx
          - certbot
        runcmd:
          - systemctl enable nginx
          - systemctl start nginx
      
      network_config: |
        version: 2
        ethernets:
          eth0:
            dhcp4: true
```

**Fields**:
- `user_data`: Cloud-config user data
- `network_config`: Network configuration
- `vendor_data`: Vendor-specific data

## Network Definition

Networks define the network infrastructure for instances.

```yaml
networks:
  frontend:
    type: bridge
    description: "Public-facing network"
    config:
      ipv4.address: "10.0.1.1/24"
      ipv4.nat: "true"
      ipv6.address: "none"
```

### type
The type of network.

**Values**: `bridge`, `macvlan`, `sriov`, `ovn`, `physical`

### config
Network-specific configuration options.

**Common options**:
- `ipv4.address`: IPv4 address and subnet
- `ipv4.nat`: Enable NAT (true/false)
- `ipv6.address`: IPv6 address or "none"
- `bridge.driver`: Bridge driver (native/openvswitch)
- `dns.domain`: DNS domain

## Storage Pool Definition

Storage pools define where and how storage is allocated.

```yaml
storage:
  default:
    driver: dir
    description: "Default storage pool"
    config:
      source: "/var/lib/incus/storage-pools/default"
  
  fast:
    driver: zfs
    description: "ZFS pool for databases"
    config:
      source: "tank/incus"
      volume.size: "50GB"
```

### driver
The storage driver to use.

**Values**: `dir`, `btrfs`, `lvm`, `zfs`, `ceph`

### config
Driver-specific configuration.

**Common options**:
- `source`: Source path or device
- `size`: Pool size
- `volume.size`: Default volume size

## Profile Definition

Profiles are reusable configuration templates.

```yaml
profiles:
  web-server:
    description: "Common web server configuration"
    config:
      security.nesting: "true"
      linux.kernel_modules: "ip_tables,ip6_tables"
    devices:
      eth0:
        type: nic
        network: "frontend"
        name: "eth0"
```

## Complete Example

See `examples/incus-compose.yaml` for a complete example demonstrating all features.

## Implementation Notes

### Incus API Integration

The schema is designed to map closely to the [Incus REST API](https://linuxcontainers.org/incus/docs/main/rest-api/). Each field in the schema corresponds to API endpoints and parameters:

- Container creation: `POST /1.0/instances`
- Network creation: `POST /1.0/networks`
- Storage pool creation: `POST /1.0/storage-pools`
- Profile creation: `POST /1.0/profiles`

### Future Enhancements

Potential additions to future schema versions:

1. **Secrets Management**: Integration with secrets providers
2. **Health Checks**: Built-in health checking and restart policies
3. **Scaling**: Support for instance scaling and clustering
4. **Backup Configuration**: Automated backup policies
5. **Migration Rules**: Cross-host migration policies
6. **Resource Quotas**: Project-level resource quotas
7. **Snapshots**: Snapshot scheduling and management
8. **Monitoring**: Built-in monitoring and alerting

## Compatibility

This schema is designed for Incus 6.0 and later. Some features may require specific Incus versions or configurations.

## Validation

The Rust implementation includes full schema validation using serde for deserialization. Invalid configurations will produce clear error messages indicating the problem.

## See Also

- [Incus Documentation](https://linuxcontainers.org/incus/docs/main/)
- [Incus REST API](https://linuxcontainers.org/incus/docs/main/rest-api/)
- [Docker Compose Specification](https://docs.docker.com/compose/compose-file/)
