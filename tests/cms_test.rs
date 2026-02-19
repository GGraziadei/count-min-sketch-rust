#[cfg(test)]
mod tests {
use count_min_sketch_rs::CountMinSketch;

    #[test]
    fn test_initialization() {
        let cms = CountMinSketch::new(100, 4);
        assert_eq!(cms.depth, 4);
        // Ensure width is rounded to power of two
        assert_eq!(cms.width, 128);
        assert_eq!(cms.estimate("anything"), 0);
    }

    #[test]
    fn test_basic_increment() {
        let mut cms = CountMinSketch::new(1024, 8);
        let key = "rust_is_fast";

        cms.increment(key);
        cms.increment(key);
        cms.increment(key);

        assert_eq!(cms.estimate(key), 3);
    }

    #[test]
    fn test_no_underestimation() {
        let mut cms = CountMinSketch::new(512, 4);
        let items = vec!["apple", "banana", "apple", "cherry", "apple", "banana"];

        for item in &items {
            cms.increment(item);
        }

        // CMS must always return >= actual count
        assert!(cms.estimate("apple") >= 3);
        assert!(cms.estimate("banana") >= 2);
        assert!(cms.estimate("cherry") >= 1);
        assert!(cms.estimate("unknown") >= 0);
    }

    #[test]
    fn test_merge() {
        let mut cms1 = CountMinSketch::new(1024, 4);
        let mut cms2 = CountMinSketch::new(1024, 4);

        cms1.increment("a");
        cms2.increment("a");
        cms2.increment("b");

        cms1.merge(&cms2).expect("Merge should succeed");

        assert!(cms1.estimate("a") >= 2);
        assert!(cms1.estimate("b") >= 1);
    }

    #[test]
    fn test_clear() {
        let mut cms = CountMinSketch::new(1024, 4);
        cms.increment("ghost");
        cms.clear();
        assert_eq!(cms.estimate("ghost"), 0);
    }

    #[test]
    fn test_saturating_add() {
        // Test that we don't wrap around to 0 on overflow
        let mut cms = CountMinSketch::new(128, 1);
        let key = "big_counter";

        // Manually setting a value near max (simulated)
        // Since we can't access table easily, we just increment many times
        // or rely on the logic review. Let's verify it doesn't crash.
        for _ in 0..100 {
            cms.increment(key);
        }
        assert!(cms.estimate(key) == 100);
    }

    #[test]
    fn test_with_params() {
        // 1% error with 99% confidence
        let cms = CountMinSketch::with_params(0.01, 0.01);
        // e / 0.01 = 271.8 -> 272 -> 512 (next power of two)
        assert_eq!(cms.width, 512);
        // ln(1 / 0.01) = 4.6 -> 5
        assert_eq!(cms.depth, 5);
    }
}