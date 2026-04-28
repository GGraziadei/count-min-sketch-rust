#[cfg(test)]
mod tests {
use std::num::NonZeroUsize;
use count_min_sketch_rs::CountMinSketch;

    #[test]
    fn test_initialization() {
        let cms = CountMinSketch::new(NonZeroUsize::try_from(512usize).unwrap(),NonZeroUsize::try_from(4usize).unwrap());
        assert_eq!(cms.get_depth(), 4);
        assert_eq!(cms.get_width(), 512);
        assert_eq!(cms.estimate("anything"), 0);
    }

    #[test]
    fn test_basic_increment() {
        let mut cms = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());
        let key = "rust_is_fast";

        cms.increment(key);
        cms.increment(key);
        cms.increment(key);

        assert_eq!(cms.estimate(key), 3);
    }

    #[test]
    fn test_no_underestimation() {
        let mut cms = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());
        let items = vec!["apple", "banana", "apple", "cherry", "apple", "banana"];

        for item in &items {
            cms.increment(item);
        }

        // CMS must always return >= actual count
        assert!(cms.estimate("apple") >= 3);
        assert!(cms.estimate("banana") >= 2);
        assert!(cms.estimate("cherry") >= 1);
    }

    #[test]
    fn test_merge() {
        let mut cms1 = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());
        let mut cms2 = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());

        cms1.increment("a");
        cms2.increment("a");
        cms2.increment("b");

        cms1.merge(&cms2).expect("Merge should succeed");

        assert!(cms1.estimate("a") >= 2);
        assert!(cms1.estimate("b") >= 1);
    }

    #[test]
    fn test_clear() {
        let mut cms = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());
        cms.increment("ghost");
        cms.clear();
        assert_eq!(cms.estimate("ghost"), 0);
    }

    #[test]
    fn test_saturating_add() {
        let mut cms = CountMinSketch::new(NonZeroUsize::try_from(1024usize).unwrap(),NonZeroUsize::try_from(8usize).unwrap());
        let key = "big_counter";
        for _ in 0..100 {
            cms.increment(key);
        }
        assert_eq!(cms.estimate(key), 100);
    }

    #[test]
    fn test_with_params() {
        // 1% error with 99% confidence
        let cms = CountMinSketch::with_params(0.01, 0.02);
        // e / 0.01 = 271.8 -> 272 -> 512 (next power of two)
        assert_eq!(cms.get_width(), 512);
        // ln(1 / 0.02) = 3.9 -> 4
        assert_eq!(cms.get_depth(), 4);
    }
}