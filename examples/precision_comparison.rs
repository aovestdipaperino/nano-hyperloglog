//! Compares different precision values and their accuracy/memory tradeoffs.

use hyperloglog::HyperLogLog;

fn test_precision(precision: u8, n_items: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut hll = HyperLogLog::new(precision)?;

    for i in 0..n_items {
        hll.add(&i);
    }

    let estimated = hll.count();
    let error = ((estimated as f64 - n_items as f64) / n_items as f64).abs() * 100.0;
    let memory = 1 << precision; // 2^precision bytes

    println!(
        "Precision {:2} | Memory: {:6} bytes | Estimated: {:8} | Error: {:5.2}%",
        precision, memory, estimated, error
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n_items = 100_000;

    println!("Testing with {} unique items\n", n_items);
    println!("Theoretical error rates:");
    println!("  Precision 10: ~1.625%");
    println!("  Precision 12: ~0.813%");
    println!("  Precision 14: ~0.406%");
    println!("  Precision 16: ~0.203%");
    println!();

    for precision in [10, 12, 14, 16] {
        test_precision(precision, n_items)?;
    }

    println!("\nConclusion:");
    println!("  Higher precision = better accuracy but more memory");
    println!("  Precision 14 is a good default (16KB, ~0.8% error)");

    Ok(())
}
