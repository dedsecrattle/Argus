use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const SIMHASH_BITS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Simhash(u64);

impl Simhash {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn from_text(text: &str) -> Self {
        let tokens = tokenize(text);
        Self::from_tokens(&tokens)
    }

    pub fn from_tokens(tokens: &[String]) -> Self {
        let mut v = vec![0i32; SIMHASH_BITS];

        for token in tokens {
            let hash = hash_token(token);
            for (i, count) in v.iter_mut().enumerate().take(SIMHASH_BITS) {
                if (hash >> i) & 1 == 1 {
                    *count += 1;
                } else {
                    *count -= 1;
                }
            }
        }

        let mut fingerprint: u64 = 0;
        for (i, &count) in v.iter().enumerate().take(SIMHASH_BITS) {
            if count > 0 {
                fingerprint |= 1 << i;
            }
        }

        Self(fingerprint)
    }

    pub fn hamming_distance(&self, other: &Simhash) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    pub fn similarity(&self, other: &Simhash) -> f64 {
        let distance = self.hamming_distance(other);
        1.0 - (distance as f64 / SIMHASH_BITS as f64)
    }

    pub fn is_near_duplicate(&self, other: &Simhash, threshold: u32) -> bool {
        self.hamming_distance(other) <= threshold
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_lowercase())
        .collect()
}

fn hash_token(token: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_texts_have_zero_distance() {
        let text = "The quick brown fox jumps over the lazy dog";
        let hash1 = Simhash::from_text(text);
        let hash2 = Simhash::from_text(text);
        assert_eq!(hash1.hamming_distance(&hash2), 0);
        assert_eq!(hash1.similarity(&hash2), 1.0);
    }

    #[test]
    fn similar_texts_have_small_distance() {
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox jumps over a lazy dog";
        let hash1 = Simhash::from_text(text1);
        let hash2 = Simhash::from_text(text2);
        let distance = hash1.hamming_distance(&hash2);
        assert!(distance < 10, "Distance should be small: {}", distance);
        assert!(hash1.similarity(&hash2) > 0.8);
    }

    #[test]
    fn different_texts_have_large_distance() {
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "Python is a programming language";
        let hash1 = Simhash::from_text(text1);
        let hash2 = Simhash::from_text(text2);
        let distance = hash1.hamming_distance(&hash2);
        assert!(distance > 20, "Distance should be large: {}", distance);
        assert!(hash1.similarity(&hash2) < 0.7);
    }

    #[test]
    fn near_duplicate_detection() {
        let text1 = "This is a test document with some content";
        let text2 = "This is a test document with similar content";
        let text3 = "Completely different text about something else";

        let hash1 = Simhash::from_text(text1);
        let hash2 = Simhash::from_text(text2);
        let hash3 = Simhash::from_text(text3);

        let distance_similar = hash1.hamming_distance(&hash2);
        let distance_different = hash1.hamming_distance(&hash3);

        assert!(
            distance_similar < distance_different,
            "Similar texts should have smaller distance: {} vs {}",
            distance_similar,
            distance_different
        );
        assert!(hash1.is_near_duplicate(&hash2, 15));
        assert!(!hash1.is_near_duplicate(&hash3, 15));
    }

    #[test]
    fn empty_text_handling() {
        let hash1 = Simhash::from_text("");
        let hash2 = Simhash::from_text("");
        assert_eq!(hash1.hamming_distance(&hash2), 0);
    }

    #[test]
    fn case_insensitive() {
        let text1 = "The Quick Brown Fox";
        let text2 = "the quick brown fox";
        let hash1 = Simhash::from_text(text1);
        let hash2 = Simhash::from_text(text2);
        assert_eq!(hash1.hamming_distance(&hash2), 0);
    }
}
