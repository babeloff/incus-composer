use std::fs;

mod schema;

use schema::IncusCompose;

fn main() {
    println!("Incus Composer - Schema Validation Tool");
    println!("========================================\n");

    // Try to load and parse the example file
    let example_path = "examples/base-hospital-network.yaml";

    match fs::read_to_string(example_path) {
        Ok(content) => match serde_yaml::from_str::<IncusCompose>(&content) {
            Ok(mut compose) => {
                // Normalize subnet fields for backward compatibility
                for host in &mut compose.hosts {
                    host.normalize();
                }
                println!("✓ Successfully parsed {}", example_path);
                println!("\nSummary:");
                println!("  Version: {}", compose.version);
                println!("  Hosts: {}", compose.hosts.len());
                println!("  Subnets: {}", compose.subnets.len());
                println!("  Flavors: {}", compose.flavors.len());
                println!("  Images: {}", compose.images.len());

                if !compose.hosts.is_empty() {
                    println!("\nHost instances:");
                    for host in compose.hosts.iter() {
                        let subnet_info = if host.subnets.is_empty() {
                            String::new()
                        } else if host.subnets.len() == 1 {
                            format!(" (subnet: {})", host.subnets[0])
                        } else {
                            format!(" (subnets: {})", host.subnets.join(", "))
                        };
                        println!("  - {} ({}){}", host.name, host.image, subnet_info);
                    }
                }

                if !compose.subnets.is_empty() {
                    println!("\nSubnets:");
                    for subnet in compose.subnets.iter() {
                        let cidr_info = subnet
                            .cidr()
                            .map(|c| format!(" ({})", c))
                            .unwrap_or_else(|| " (auto-assigned)".to_string());
                        println!("  - {}{}", subnet.name(), cidr_info);
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to parse {}: {}", example_path, e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("✗ Failed to read {}: {}", example_path, e);
            eprintln!("  (This is expected if you haven't created the example file yet)");
        }
    }
}
