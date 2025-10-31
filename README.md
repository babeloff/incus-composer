# incus-composer

Inspired by docker-compose and distinct as linux-containers are for applications and incus-containers are for machines.

## Overview

`incus-composer` is a tool for managing [Incus](https://linuxcontainers.org/incus/) system containers and virtual machines using a declarative YAML configuration file, similar to how docker-compose manages application containers.

While docker-compose is focused on application deployment, incus-composer is designed for system-level infrastructure management, including:

- Full system containers
- Virtual machines
- Network infrastructure
- Storage pools
- Hardware device passthrough
- Cloud-init integration

## Project Status

This project is in its initial phase. Currently implemented:

- [x] Pixi-based Rust project structure
- [x] Comprehensive `incus-compose.yaml` schema design
- [x] Rust schema definitions with serde support
- [ ] Incus REST API client (using reqwest)
- [ ] CLI interface
- [ ] Container/VM lifecycle management
- [ ] Network management
- [ ] Storage pool management

## Schema

The `incus-compose.yaml` schema is inspired by docker-compose but adapted for system containers and VMs. See [SCHEMA.md](SCHEMA.md) for complete documentation.

### Example Configuration

```yaml
version: "1.0"

containers:
  web:
    instance_type: container
    image: "ubuntu/22.04"
    memory:
      limit: "2GB"
    cpu:
      limit: "2"
    networks:
      - frontend
    autostart: true

networks:
  frontend:
    type: bridge
    config:
      ipv4.address: "10.0.1.1/24"
      ipv4.nat: "true"
```

See [examples/incus-compose.yaml](examples/incus-compose.yaml) for a comprehensive example.

## Development

This project uses [pixi](https://pixi.sh/) for dependency management instead of rustup.

### Prerequisites

- [pixi](https://pixi.sh/) - Package management tool

### Building

```bash
# Install dependencies and build
pixi run cargo build

# Run tests
pixi run cargo test

# Run the application
pixi run cargo run
```

### Project Structure

```
incus-composer/
├── src/
│   ├── main.rs          # Application entry point
│   └── schema.rs        # Schema definitions
├── examples/
│   └── incus-compose.yaml  # Example configuration
├── Cargo.toml           # Rust dependencies
├── pixi.toml            # Pixi configuration
├── SCHEMA.md            # Schema documentation
└── README.md            # This file
```

## Dependencies

- **reqwest**: HTTP client for Incus REST API communication
- **serde**: Serialization/deserialization framework
- **serde_yaml**: YAML support for configuration files
- **tokio**: Async runtime

## Incus API

This tool communicates with Incus via its REST API. See the [Incus REST API documentation](https://linuxcontainers.org/incus/docs/main/rest-api/) for details.

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! This project is in early development and there are many opportunities to contribute.

