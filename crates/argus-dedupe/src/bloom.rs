use bloomfilter::Bloom;
use std::sync::{Arc, RwLock};

/// Bloom filter-based probabilistic deduplication
/// Provides fast "probably not seen" checks with minimal memory
pub struct BloomDeduplicator {
    filter: Arc<RwLock<Bloom<String>>>,
    expected_items: usize,
    false_positive_rate: f64,
}

impl BloomDeduplicator {
    /// Create a new Bloom filter deduplicator
    ///
    /// # Arguments
    /// * `expected_items` - Expected number of URLs (e.g., 1_000_000_000 for 1B URLs)
    /// * `false_positive_rate` - Acceptable false positive rate (e.g., 0.01 for 1%)
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let filter = Bloom::new_for_fp_rate(expected_items, false_positive_rate);

        Self {
            filter: Arc::new(RwLock::new(filter)),
            expected_items,
            false_positive_rate,
        }
    }

    /// Check if URL might have been seen (probabilistic)
    /// Returns true if URL might be in the set (could be false positive)
    /// Returns false if URL is definitely not in the set
    pub fn might_contain(&self, url: &str) -> bool {
        let filter = self.filter.read().unwrap();
        filter.check(&url.to_string())
    }

    /// Insert URL into the bloom filter
    pub fn insert(&self, url: &str) {
        let mut filter = self.filter.write().unwrap();
        filter.set(&url.to_string());
    }

    /// Get the number of bits used by the filter
    pub fn bit_count(&self) -> u64 {
        let filter = self.filter.read().unwrap();
        filter.number_of_bits()
    }

    /// Get the number of hash functions used
    pub fn hash_count(&self) -> u32 {
        let filter = self.filter.read().unwrap();
        filter.number_of_hash_functions()
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        (self.bit_count() / 8) as usize
    }

    /// Clear the bloom filter (use with caution)
    pub fn clear(&self) {
        let mut filter = self.filter.write().unwrap();
        *filter = Bloom::new_for_fp_rate(self.expected_items, self.false_positive_rate);
    }
}

impl Default for BloomDeduplicator {
    fn default() -> Self {
        // Default: 10M URLs with 1% false positive rate
        Self::new(10_000_000, 0.01)
    }
}

impl Clone for BloomDeduplicator {
    fn clone(&self) -> Self {
        Self {
            filter: Arc::clone(&self.filter),
            expected_items: self.expected_items,
            false_positive_rate: self.false_positive_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_filter_basic() {
        let bloom = BloomDeduplicator::new(1000, 0.01);

        assert!(!bloom.might_contain("https://example.com"));

        bloom.insert("https://example.com");

        assert!(bloom.might_contain("https://example.com"));
        assert!(!bloom.might_contain("https://other.com"));
    }

    #[test]
    fn bloom_filter_memory_efficient() {
        // 1M URLs with 1% false positive rate
        let bloom = BloomDeduplicator::new(1_000_000, 0.01);

        // Should use ~1.2MB for 1M URLs
        let memory = bloom.memory_usage();
        assert!(memory < 2_000_000, "Memory usage: {} bytes", memory);
    }

    #[test]
    fn bloom_filter_false_positive_rate() {
        let bloom = BloomDeduplicator::new(1000, 0.01);

        // Insert 1000 URLs
        for i in 0..1000 {
            bloom.insert(&format!("https://example.com/{}", i));
        }

        // Check for false positives
        let mut false_positives = 0;
        for i in 1000..2000 {
            if bloom.might_contain(&format!("https://example.com/{}", i)) {
                false_positives += 1;
            }
        }

        let fp_rate = false_positives as f64 / 1000.0;
        // Should be close to 1% (allow some variance)
        assert!(
            fp_rate < 0.05,
            "False positive rate: {:.2}%",
            fp_rate * 100.0
        );
    }

    #[test]
    fn bloom_filter_clone() {
        let bloom1 = BloomDeduplicator::new(1000, 0.01);
        bloom1.insert("https://example.com");

        let bloom2 = bloom1.clone();
        assert!(bloom2.might_contain("https://example.com"));
    }

    #[test]
    fn bloom_filter_clear() {
        let bloom = BloomDeduplicator::new(1000, 0.01);
        bloom.insert("https://example.com");
        assert!(bloom.might_contain("https://example.com"));

        bloom.clear();
        assert!(!bloom.might_contain("https://example.com"));
    }

    #[test]
    fn bloom_filter_large_scale() {
        // Test with 1B URL configuration
        let bloom = BloomDeduplicator::new(1_000_000_000, 0.01);

        // Should use ~1.2GB for 1B URLs
        let memory_gb = bloom.memory_usage() as f64 / 1_073_741_824.0;
        assert!(memory_gb < 2.0, "Memory usage: {:.2} GB", memory_gb);

        // Verify it works
        bloom.insert("https://example.com");
        assert!(bloom.might_contain("https://example.com"));
    }
}
