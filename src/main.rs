use clap::Parser;
use std::fs;
use std::path::Path;
use std::process;

mod schema;

use schema::{IncusCompose, IncusLockfile};

/// A tool for managing Incus system containers and VMs using declarative YAML configuration
#[derive(Parser)]
#[command(name = "incus-composer")]
#[command(author = "Incus Composer Contributors")]
#[command(version = "0.1.0")]
#[command(about = "A tool for managing Incus system containers and VMs using declarative YAML configuration", long_about = None)]
struct Cli {
    /// Path to the incus-compose.yaml configuration file
    #[arg(
        short = 'c',
        long = "config",
        value_name = "FILE",
        default_value = "incus-compose.yaml"
    )]
    config: String,

    /// Path to the lockfile (defaults to config file with .lock extension)
    #[arg(short = 'l', long = "lockfile", value_name = "FILE")]
    lockfile: Option<String>,

    /// Generate incus commands to FILE instead of executing them
    #[arg(short = 'd', long = "dry-run", value_name = "FILE")]
    dry_run: Option<String>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let config_path = &cli.config;
    let verbose = cli.verbose;

    // Determine lockfile path
    let lockfile_path = if let Some(path) = &cli.lockfile {
        path.clone()
    } else {
        format!("{}.lock", config_path)
    };

    if verbose {
        println!("Incus Composer v0.1.0");
        println!("====================\n");
        println!("Configuration file: {}", config_path);
        println!("Lockfile: {}", lockfile_path);
        if let Some(dry_run_file) = &cli.dry_run {
            println!("Dry-run output: {}", dry_run_file);
        }
        println!();
    }

    // Load the configuration file
    let compose = match load_compose_file(config_path, verbose) {
        Ok(compose) => compose,
        Err(e) => {
            eprintln!(
                "✗ Error loading configuration file '{}': {}",
                config_path, e
            );
            process::exit(1);
        }
    };

    if verbose {
        print_compose_summary(&compose);
    }

    // Load existing lockfile if it exists
    let existing_lockfile = if Path::new(&lockfile_path).exists() {
        match IncusLockfile::load_from_file(&lockfile_path) {
            Ok(lockfile) => {
                if verbose {
                    println!("✓ Loaded existing lockfile: {}", lockfile_path);
                }
                Some(lockfile)
            }
            Err(e) => {
                if verbose {
                    println!(
                        "⚠ Could not load existing lockfile (will create new): {}",
                        e
                    );
                }
                None
            }
        }
    } else {
        if verbose {
            println!("ℹ No existing lockfile found, will create new one");
        }
        None
    };

    // Generate new lockfile from compose configuration
    let mut lockfile = compose.generate_lockfile();

    // If we had an existing lockfile, preserve stable values where possible
    if let Some(existing) = existing_lockfile {
        lockfile = merge_lockfiles(lockfile, existing, verbose);
    }

    // Save the updated lockfile
    if let Err(e) = lockfile.save_to_file(&lockfile_path) {
        eprintln!("✗ Error saving lockfile '{}': {}", lockfile_path, e);
        process::exit(1);
    }

    if verbose {
        println!("✓ Updated lockfile: {}", lockfile_path);
        print_lockfile_summary(&lockfile);
    }

    // Handle dry-run mode
    if let Some(dry_run_file) = &cli.dry_run {
        match generate_dry_run(dry_run_file, &lockfile, verbose) {
            Ok(()) => {
                if verbose {
                    println!("✓ Dry-run commands written to: {}", dry_run_file);
                }
            }
            Err(e) => {
                eprintln!("✗ Error writing dry-run file '{}': {}", dry_run_file, e);
                process::exit(1);
            }
        }
    } else {
        if verbose {
            println!("ℹ Use --dry-run to generate incus commands without executing");
        }
    }

    if verbose {
        println!("\n✓ Operation completed successfully");
    }
}

fn load_compose_file(
    path: &str,
    verbose: bool,
) -> Result<IncusCompose, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err(format!("Configuration file '{}' does not exist", path).into());
    }

    if verbose {
        println!("📖 Loading configuration file: {}", path);
    }

    let compose = IncusCompose::load_from_file(path)?;

    if verbose {
        println!("✓ Successfully parsed configuration file");
    }

    Ok(compose)
}

fn print_compose_summary(compose: &IncusCompose) {
    println!("Configuration Summary:");
    println!("  Version: {}", compose.version);
    println!("  Hosts: {}", compose.hosts.len());
    println!("  Subnets: {}", compose.subnets.len());
    println!("  Flavors: {}", compose.flavors.len());
    println!("  Images: {}", compose.images.len());

    if !compose.defaults.host_ip4_ranges.is_empty()
        || !compose.defaults.router_ip4_ranges.is_empty()
        || !compose.defaults.cidr4_ranges.is_empty()
    {
        println!("  Default IP ranges configured:");
        println!(
            "    Host IP ranges: {}",
            compose.defaults.host_ip4_ranges.len()
        );
        println!(
            "    Router IP ranges: {}",
            compose.defaults.router_ip4_ranges.len()
        );
        println!("    CIDR ranges: {}", compose.defaults.cidr4_ranges.len());
    }

    if !compose.hosts.is_empty() {
        println!("\nHost Configuration:");
        for host in &compose.hosts {
            let mut flags = Vec::new();
            if host.floating_ip {
                flags.push("floating-ip");
            }
            if host.master {
                flags.push("master");
            }
            if host.is_router {
                flags.push("router");
            }

            let flag_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };

            let subnet_str = if host.subnets.is_empty() {
                String::new()
            } else if host.subnets.len() == 1 {
                format!(" → {}", host.subnets[0])
            } else {
                format!(" → {}", host.subnets.join(", "))
            };

            println!(
                "  • {} ({}{}){}",
                host.name, host.image, flag_str, subnet_str
            );
        }
    }

    if !compose.subnets.is_empty() {
        println!("\nSubnet Configuration:");
        for subnet in &compose.subnets {
            let cidr_str = if let Some(cidr) = subnet.cidr() {
                format!(" ({})", cidr)
            } else {
                " (auto-assigned)".to_string()
            };
            println!("  • {}{}", subnet.name(), cidr_str);
        }
    }
    println!();
}

fn print_lockfile_summary(lockfile: &IncusLockfile) {
    println!("\nLockfile Summary:");
    println!("  Generated: {}", lockfile.metadata.generated_at);
    println!(
        "  Generator version: {}",
        lockfile.metadata.generator_version
    );
    println!("  Source hash: {}", lockfile.metadata.source_hash);

    println!("  Resource allocation:");
    println!(
        "    Host IDs: {}",
        lockfile.metadata.used_values.host_ids.len()
    );
    println!(
        "    Subnet IDs: {}",
        lockfile.metadata.used_values.subnet_ids.len()
    );
    println!(
        "    MAC addresses: {}",
        lockfile.metadata.used_values.mac_addresses.len()
    );

    let total_ips: usize = lockfile
        .metadata
        .used_values
        .ip_addresses
        .values()
        .map(|ips| ips.len())
        .sum();
    println!("    IP addresses: {}", total_ips);

    if !lockfile.hosts.is_empty() {
        println!("\nExpanded Host Configuration:");
        for host in &lockfile.hosts {
            let ip_list: Vec<String> = host
                .ip_addresses
                .iter()
                .map(|(subnet, ip)| format!("{}:{}", subnet, ip))
                .collect();

            let ip_str = if ip_list.is_empty() {
                String::new()
            } else {
                format!(" [{}]", ip_list.join(", "))
            };

            println!(
                "  • {} (ID: {}) → MAC: {}{}",
                host.name,
                host.id,
                host.mac_address.as_ref().unwrap_or(&"none".to_string()),
                ip_str
            );
        }
    }

    if !lockfile.subnets.is_empty() {
        println!("\nExpanded Subnet Configuration:");
        for subnet in &lockfile.subnets {
            println!(
                "  • {} (ID: {}) → {} (GW: {})",
                subnet.name, subnet.id, subnet.cidr, subnet.gateway
            );
        }
    }
    println!();
}

fn merge_lockfiles(
    new_lockfile: IncusLockfile,
    existing: IncusLockfile,
    verbose: bool,
) -> IncusLockfile {
    if verbose {
        println!("🔄 Merging with existing lockfile to preserve stable values");
    }

    let mut merged = new_lockfile;

    // Preserve MAC addresses and IDs for existing hosts
    for new_host in &mut merged.hosts {
        if let Some(existing_host) = existing.hosts.iter().find(|h| h.name == new_host.name) {
            // Preserve stable identifiers
            new_host.id = existing_host.id.clone();
            new_host.mac_address = existing_host.mac_address.clone();

            // Preserve IP addresses where subnets haven't changed
            for (subnet_name, existing_ip) in &existing_host.ip_addresses {
                if new_host.subnets.contains(subnet_name) {
                    new_host
                        .ip_addresses
                        .insert(subnet_name.clone(), existing_ip.clone());
                }
            }

            if verbose {
                println!("  ↻ Preserved identifiers for host: {}", new_host.name);
            }
        }
    }

    // Preserve subnet IDs and configurations where possible
    for new_subnet in &mut merged.subnets {
        if let Some(existing_subnet) = existing.subnets.iter().find(|s| s.name == new_subnet.name) {
            // Only preserve if CIDR hasn't changed
            if new_subnet.cidr == existing_subnet.cidr {
                new_subnet.id = existing_subnet.id.clone();
                new_subnet.gateway = existing_subnet.gateway.clone();

                if verbose {
                    println!(
                        "  ↻ Preserved configuration for subnet: {}",
                        new_subnet.name
                    );
                }
            }
        }
    }

    // Update metadata but preserve some used values tracking
    merged.metadata.used_values.mac_addresses = existing.metadata.used_values.mac_addresses;
    merged.metadata.used_values.host_ids = existing.metadata.used_values.host_ids;
    merged.metadata.used_values.subnet_ids = existing.metadata.used_values.subnet_ids;

    merged
}

fn generate_dry_run(
    output_file: &str,
    lockfile: &IncusLockfile,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("📝 Generating incus commands for dry-run");
    }

    let commands = lockfile.generate_incus_commands();

    let mut output = Vec::new();
    output.push("#!/bin/bash".to_string());
    output.push("# Generated by incus-composer".to_string());
    output.push(format!(
        "# Generated at: {}",
        lockfile.metadata.generated_at
    ));
    output.push(format!(
        "# Generator version: {}",
        lockfile.metadata.generator_version
    ));
    output.push(format!("# Source hash: {}", lockfile.metadata.source_hash));
    output.push("".to_string());
    output.push("set -e  # Exit on any error".to_string());
    output.push("".to_string());

    if verbose {
        output.push("echo 'Starting incus-composer deployment...'".to_string());
        output.push("".to_string());
    }

    // Add section comments
    output.push("# ============================================".to_string());
    output.push("# Network Creation".to_string());
    output.push("# ============================================".to_string());
    output.push("".to_string());

    let mut in_network_section = true;
    for command in &commands {
        if command.starts_with("incus create") && in_network_section {
            output.push("".to_string());
            output.push("# ============================================".to_string());
            output.push("# Instance Creation and Configuration".to_string());
            output.push("# ============================================".to_string());
            output.push("".to_string());
            in_network_section = false;
        }

        if command.starts_with('#') {
            output.push(command.clone());
        } else {
            if verbose {
                output.push(format!("echo 'Executing: {}'", command));
            }
            output.push(command.clone());
        }
    }

    output.push("".to_string());
    if verbose {
        output.push("echo 'Deployment completed successfully!'".to_string());
    }

    let script_content = output.join("\n");
    fs::write(output_file, script_content)?;

    // Make the script executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output_file, perms)?;
    }

    if verbose {
        println!("  📊 Generated {} commands", commands.len());
        println!("  📄 Script saved as executable: {}", output_file);
    }

    Ok(())
}
