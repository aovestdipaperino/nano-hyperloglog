//! Demonstrates merging HyperLogLogs from multiple sources.

use hyperloglog::HyperLogLog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate three different servers tracking visitors
    println!("Simulating three web servers tracking unique visitors...\n");

    let mut server1 = HyperLogLog::new(14)?;
    let mut server2 = HyperLogLog::new(14)?;
    let mut server3 = HyperLogLog::new(14)?;

    // Server 1: Users 0-4999
    for i in 0..5000 {
        server1.add_str(&format!("user_{}", i));
    }
    println!("Server 1 unique visitors: {}", server1.count());

    // Server 2: Users 2500-7499 (overlaps with server 1)
    for i in 2500..7500 {
        server2.add_str(&format!("user_{}", i));
    }
    println!("Server 2 unique visitors: {}", server2.count());

    // Server 3: Users 5000-9999 (overlaps with server 2)
    for i in 5000..10000 {
        server3.add_str(&format!("user_{}", i));
    }
    println!("Server 3 unique visitors: {}", server3.count());

    // Merge all servers to get total unique visitors
    let mut total = server1.clone();
    total.merge(&server2)?;
    total.merge(&server3)?;

    let estimated_total = total.count();
    let actual_total = 10000; // Users 0-9999

    println!("\n--- Merged Results ---");
    println!("Actual total unique visitors: {}", actual_total);
    println!("Estimated total: {}", estimated_total);
    println!(
        "Error: {:.2}%",
        ((estimated_total as f64 - actual_total as f64) / actual_total as f64).abs() * 100.0
    );

    Ok(())
}
