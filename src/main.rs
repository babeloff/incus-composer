use std::fs;

mod schema;

use schema::IncusCompose;

fn main() {
    println!("Incus Composer - Schema Validation Tool");
    println!("========================================\n");

    // Try to load and parse the example file
    let example_path = "examples/incus-compose.yaml";
    
    match fs::read_to_string(example_path) {
        Ok(content) => {
            match serde_yaml::from_str::<IncusCompose>(&content) {
                Ok(compose) => {
                    println!("✓ Successfully parsed {}", example_path);
                    println!("\nSummary:");
                    println!("  Version: {}", compose.version);
                    println!("  Containers: {}", compose.containers.len());
                    println!("  Networks: {}", compose.networks.len());
                    println!("  Storage Pools: {}", compose.storage.len());
                    println!("  Profiles: {}", compose.profiles.len());
                    
                    if !compose.containers.is_empty() {
                        println!("\nContainer instances:");
                        for (name, container) in compose.containers.iter() {
                            println!("  - {} ({})", name, container.image);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to parse {}: {}", example_path, e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to read {}: {}", example_path, e);
            eprintln!("  (This is expected if you haven't created the example file yet)");
        }
    }
}
