use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

/// HyperLogLog implementation for cardinality estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperLogLog {
    /// Precision parameter (typically 4-16)
    precision: u8,
    /// Number of registers (2^precision)
    m: usize,
    /// Registers storing max leading zeros
    registers: Vec<u8>,
}

impl HyperLogLog {
    /// Create a new HyperLogLog with given precision
    /// Precision must be between 4 and 16
    pub fn new(precision: u8) -> Result<Self, crate::error::HllError> {
        if !(4..=16).contains(&precision) {
            return Err(crate::error::HllError::InvalidPrecision(precision));
        }

        let m = 1 << precision;
        Ok(HyperLogLog {
            precision,
            m,
            registers: vec![0; m],
        })
    }

    /// Add an element to the HyperLogLog
    pub fn add<T: Hash>(&mut self, element: &T) {
        let hash = self.hash_element(element);

        // Use first 'precision' bits for register index
        let idx = (hash >> (64 - self.precision)) as usize;

        // Count leading zeros in remaining bits + 1
        let remaining = hash << self.precision;
        let leading_zeros = if remaining == 0 {
            64 - self.precision + 1
        } else {
            remaining.leading_zeros() as u8 + 1
        };

        // Store max leading zeros for this register
        if leading_zeros > self.registers[idx] {
            self.registers[idx] = leading_zeros;
        }
    }

    /// Add a raw string element (for Redis compatibility)
    pub fn add_str(&mut self, element: &str) {
        self.add(&element);
    }

    /// Estimate cardinality
    pub fn count(&self) -> u64 {
        let m = self.m as f64;

        // Calculate raw estimate
        let sum: f64 = self.registers.iter()
            .map(|&val| 2.0_f64.powi(-(val as i32)))
            .sum();

        let alpha = self.alpha_m();
        let raw_estimate = alpha * m * m / sum;

        // Apply bias correction for different ranges
        if raw_estimate <= 2.5 * m {
            // Small range correction
            let zeros = self.registers.iter().filter(|&&x| x == 0).count();
            if zeros != 0 {
                return (m * (m / zeros as f64).ln()) as u64;
            }
        }

        if raw_estimate <= (1.0 / 30.0) * (1u64 << 32) as f64 {
            return raw_estimate as u64;
        }

        // Large range correction
        (-((1u64 << 32) as f64) * (1.0 - raw_estimate / ((1u64 << 32) as f64)).ln()) as u64
    }

    /// Merge another HyperLogLog into this one
    pub fn merge(&mut self, other: &HyperLogLog) -> Result<(), crate::error::HllError> {
        if self.precision != other.precision {
            return Err(crate::error::HllError::Storage(
                "Cannot merge HyperLogLogs with different precision".to_string()
            ));
        }

        for (i, &val) in other.registers.iter().enumerate() {
            if val > self.registers[i] {
                self.registers[i] = val;
            }
        }

        Ok(())
    }

    /// Get precision
    pub fn precision(&self) -> u8 {
        self.precision
    }

    /// Hash an element using xxHash
    fn hash_element<T: Hash>(&self, element: &T) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        element.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate alpha constant based on m
    fn alpha_m(&self) -> f64 {
        match self.m {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / self.m as f64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_precision() {
        for precision in 4..=16 {
            let hll = HyperLogLog::new(precision);
            assert!(hll.is_ok(), "Precision {} should be valid", precision);

            let hll = hll.unwrap();
            assert_eq!(hll.precision(), precision);
            assert_eq!(hll.m, 1 << precision);
            assert_eq!(hll.registers.len(), 1 << precision);
        }
    }

    #[test]
    fn test_new_invalid_precision() {
        assert!(HyperLogLog::new(3).is_err());
        assert!(HyperLogLog::new(17).is_err());
        assert!(HyperLogLog::new(0).is_err());
        assert!(HyperLogLog::new(255).is_err());
    }

    #[test]
    fn test_add_deduplication() {
        let mut hll = HyperLogLog::new(10).unwrap();

        // Add same element multiple times
        for _ in 0..100 {
            hll.add_str("same_element");
        }

        let count = hll.count();
        // Should estimate close to 1
        assert!(count <= 5, "Count should be close to 1, got {}", count);
    }

    #[test]
    fn test_basic_counting_small() {
        let mut hll = HyperLogLog::new(14).unwrap();

        for i in 0..100 {
            hll.add(&i);
        }

        let count = hll.count();
        let error_rate = ((count as f64 - 100.0) / 100.0).abs();

        // Should be within ~10% for small counts
        assert!(error_rate < 0.15, "Error rate: {:.2}%", error_rate * 100.0);
    }

    #[test]
    fn test_basic_counting_medium() {
        let mut hll = HyperLogLog::new(14).unwrap();

        for i in 0..10000 {
            hll.add(&i);
        }

        let count = hll.count();
        let error_rate = ((count as f64 - 10000.0) / 10000.0).abs();

        // HyperLogLog should be within ~2% error for precision 14
        assert!(error_rate < 0.05, "Error rate: {:.2}%", error_rate * 100.0);
    }

    #[test]
    fn test_basic_counting_large() {
        let mut hll = HyperLogLog::new(14).unwrap();

        for i in 0..100000 {
            hll.add(&i);
        }

        let count = hll.count();
        let error_rate = ((count as f64 - 100000.0) / 100000.0).abs();

        // Should be within ~2% for large counts
        assert!(error_rate < 0.03, "Error rate: {:.2}%", error_rate * 100.0);
    }

    #[test]
    fn test_string_elements() {
        let mut hll = HyperLogLog::new(10).unwrap();

        hll.add_str("user:1");
        hll.add_str("user:2");
        hll.add_str("user:3");

        let count = hll.count();
        assert!(count >= 2 && count <= 5, "Count should be ~3, got {}", count);
    }

    #[test]
    fn test_merge_disjoint() {
        let mut hll1 = HyperLogLog::new(10).unwrap();
        let mut hll2 = HyperLogLog::new(10).unwrap();

        for i in 0..100 {
            hll1.add(&i);
        }

        for i in 100..200 {
            hll2.add(&i);
        }

        hll1.merge(&hll2).unwrap();
        let count = hll1.count();

        // Should estimate around 200
        assert!(count > 150 && count < 250, "Count should be ~200, got {}", count);
    }

    #[test]
    fn test_merge_overlapping() {
        let mut hll1 = HyperLogLog::new(12).unwrap();
        let mut hll2 = HyperLogLog::new(12).unwrap();

        // 50% overlap
        for i in 0..150 {
            hll1.add(&i);
        }

        for i in 100..250 {
            hll2.add(&i);
        }

        let count1 = hll1.count();
        let count2 = hll2.count();

        hll1.merge(&hll2).unwrap();
        let merged_count = hll1.count();

        // Merged should be around 250 (0..250)
        assert!(
            merged_count > 200 && merged_count < 300,
            "Merged count should be ~250, got {}. Individual counts: {}, {}",
            merged_count,
            count1,
            count2
        );
    }

    #[test]
    fn test_merge_precision_mismatch() {
        let mut hll1 = HyperLogLog::new(10).unwrap();
        let hll2 = HyperLogLog::new(12).unwrap();

        let result = hll1.merge(&hll2);
        assert!(result.is_err(), "Should fail to merge different precisions");
    }

    #[test]
    fn test_merge_same_data() {
        let mut hll1 = HyperLogLog::new(10).unwrap();
        let mut hll2 = HyperLogLog::new(10).unwrap();

        // Both see same data
        for i in 0..100 {
            hll1.add(&i);
            hll2.add(&i);
        }

        let count_before = hll1.count();
        hll1.merge(&hll2).unwrap();
        let count_after = hll1.count();

        // Count shouldn't change much (within 10%)
        let diff = ((count_after as f64 - count_before as f64) / count_before as f64).abs();
        assert!(diff < 0.1, "Counts should be similar: {} vs {}", count_before, count_after);
    }

    #[test]
    fn test_clone() {
        let mut hll = HyperLogLog::new(10).unwrap();

        for i in 0..1000 {
            hll.add(&i);
        }

        let hll_clone = hll.clone();

        assert_eq!(hll.precision(), hll_clone.precision());
        assert_eq!(hll.count(), hll_clone.count());
        assert_eq!(hll.registers, hll_clone.registers);
    }

    #[test]
    fn test_serialization() {
        let mut hll = HyperLogLog::new(12).unwrap();

        for i in 0..5000 {
            hll.add(&i);
        }

        // Serialize
        let serialized = serde_json::to_string(&hll).unwrap();

        // Deserialize
        let deserialized: HyperLogLog = serde_json::from_str(&serialized).unwrap();

        assert_eq!(hll.precision(), deserialized.precision());
        assert_eq!(hll.count(), deserialized.count());
        assert_eq!(hll.registers, deserialized.registers);
    }

    #[test]
    fn test_empty_count() {
        let hll = HyperLogLog::new(10).unwrap();
        let count = hll.count();

        // Empty HLL should return 0 or very close to 0
        assert!(count < 10, "Empty HLL count should be ~0, got {}", count);
    }

    #[test]
    fn test_different_types() {
        let mut hll = HyperLogLog::new(10).unwrap();

        // Add different types
        hll.add(&42u32);
        hll.add(&"string");
        hll.add(&true);
        hll.add(&3.14f64.to_bits()); // Hash the bits representation

        let count = hll.count();
        assert!(count >= 3 && count <= 6, "Should count ~4 items, got {}", count);
    }

    #[test]
    fn test_precision_memory_size() {
        for precision in 4..=16 {
            let hll = HyperLogLog::new(precision).unwrap();
            let expected_size = 1 << precision;
            assert_eq!(
                hll.registers.len(),
                expected_size,
                "Precision {} should have {} registers",
                precision,
                expected_size
            );
        }
    }
}
