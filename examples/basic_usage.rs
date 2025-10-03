//! Basic usage of HyperLogLog for cardinality estimation.

use hyperloglog::HyperLogLog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new HyperLogLog with precision 14 (default)
    // This gives ~0.81% standard error with 16KB memory
    let mut hll = HyperLogLog::new(14)?;

    // Add some unique visitors
    println!("Adding unique visitors...");
    for i in 0..10000 {
        hll.add_str(&format!("user_{}", i));
    }

    // Get the estimated count
    let count = hll.count();
    let actual = 10000;
    let error = ((count as f64 - actual as f64) / actual as f64).abs() * 100.0;

    println!("Actual unique visitors: {}", actual);
    println!("Estimated count: {}", count);
    println!("Error: {:.2}%", error);

    // Add duplicates - count shouldn't change much
    println!("\nAdding duplicates...");
    for i in 0..5000 {
        hll.add_str(&format!("user_{}", i));
    }

    let new_count = hll.count();
    println!("Count after duplicates: {}", new_count);
    println!("Change: {} visitors", new_count as i64 - count as i64);

    Ok(())
}
