//! Application startup and top-level composition.

/// Starts the command-line shell and displays a read-only local network summary.
pub fn run() {
    println!("NetShift {}", crate::VERSION);

    #[cfg(windows)]
    {
        use crate::network::NetworkInspector;
        use crate::platform::windows::WindowsNetworkInspector;

        match WindowsNetworkInspector.inspect() {
            Ok(snapshot) => {
                println!("Interfaces: {}", snapshot.interfaces.len());
                for interface in &snapshot.interfaces {
                    println!(
                        "- {} [{}] ({})",
                        interface.name,
                        interface.id,
                        if interface.is_up { "up" } else { "down" }
                    );
                    for address in &interface.addresses {
                        println!("  address: {address}");
                    }
                    for dns_server in &interface.dns_servers {
                        println!("  DNS: {dns_server}");
                    }
                }

                println!(
                    "IPv4 configured: {}, usable on an up interface: {}",
                    snapshot.ipv4.configured, snapshot.ipv4.usable_on_up_interface
                );
                println!(
                    "IPv6 configured: {}, usable on an up interface: {}",
                    snapshot.ipv6.configured, snapshot.ipv6.usable_on_up_interface
                );

                println!("Routes: {}", snapshot.routes.len());
                for route in &snapshot.routes {
                    let kind = if route.is_default() {
                        "default"
                    } else {
                        "route"
                    };
                    println!(
                        "- {kind}: {}/{} via {} interface {} (index {}) metric {}",
                        route.destination.address,
                        route.destination.prefix_length,
                        route
                            .next_hop
                            .map_or_else(|| "on-link".to_owned(), |ip| ip.to_string()),
                        route.interface_id.as_deref().unwrap_or("unresolved"),
                        route.interface_index,
                        route.metric
                    );
                }
            }
            Err(error) => {
                eprintln!("Network inspection failed: {error}");
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(windows))]
    println!("Live network discovery is currently supported on Windows only.");
}
