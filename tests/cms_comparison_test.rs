#[cfg(test)]
mod tests {
    use count_min_sketch_rs::CountMinSketch;
    use std::num::NonZeroUsize;

    fn setup_sketch() -> CountMinSketch {
        // 1024 width, 8 depth
        CountMinSketch::new(
            NonZeroUsize::new(1000).unwrap(),
            NonZeroUsize::new(8).unwrap()
        )
    }

    #[test]
    fn test_l1_distance_identical() {
        let mut cms1 = setup_sketch();
        let mut cms2 = setup_sketch();

        let items = vec!["apple", "banana", "cherry", "date"];
        for item in items {
            cms1.increment(item);
            cms2.increment(item);
        }

        // Identical sketches should have an L1 distance of 0
        let dist = cms1.l1_distance(&cms2).unwrap();
        assert_eq!(dist, 0, "Distance between identical sketches should be 0");
    }

    #[test]
    fn test_l1_distance_different() {
        let mut cms1 = setup_sketch();
        let mut cms2 = setup_sketch();

        // Add 10 "apple" to cms1, 5 "banana" to cms2
        for _ in 0..10 { cms1.increment("apple"); }
        for _ in 0..5 { cms2.increment("banana"); }

        let dist = cms1.l1_distance(&cms2).unwrap();

        // Theoretically L1 is 10 + 5 = 15.
        // CMS provides an upper bound, but for sparse data, it should be exact.
        assert!(dist >= 15);
        // We allow a small margin for potential collisions if epsilon was very high,
        // but with 1024 width, it should be very close to 15.
        assert!(dist < 20);
    }

    #[test]
    fn test_cosine_similarity_identity() {
        let mut cms1 = setup_sketch();
        let mut cms2 = setup_sketch();

        let data = ["A", "B", "C", "D", "E"];
        for item in &data {
            cms1.increment(item);
            cms2.increment(item);
        }

        let sim = cms1.cosine_similarity(&cms2).unwrap();
        // Similarity of identical distributions must be 1.0 - the error 1/delta
        assert!(sim > (1. - 0.125))
    }

    #[test]
    fn test_cosine_similarity_orthogonality() {
        let mut cms1 = setup_sketch();
        let mut cms2 = setup_sketch();

        // Increment completely different items
        cms1.increment("unique_to_1");
        cms2.increment("unique_to_2");

        let sim = cms1.cosine_similarity(&cms2).unwrap();

        // With high width, probability of collision is low, so similarity should be near 0
        assert!(sim < 0.1, "Similarity of orthogonal items should be near 0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_partial_overlap() {
        let mut cms1 = setup_sketch();
        let mut cms2 = setup_sketch();

        // Vector A: [10, 10] (items "X", "Y")
        // Vector B: [10, 0]  (item "X")
        for _ in 0..10 {
            cms1.increment("X");
            cms1.increment("Y");
            cms2.increment("X");
        }

        let sim = cms1.cosine_similarity(&cms2).unwrap();

        // Analytical cosine similarity: (10*10 + 10*0) / (sqrt(10^2 + 10^2) * sqrt(10^2))
        // 100 / (14.14 * 10) = 100 / 141.4 ≈ 0.707
        let expected = 0.707;
        assert!((sim - expected).abs() < 0.05, "Expected approx {}, got {}", expected, sim);
    }

    #[test]
    fn test_incompatible_dimensions() {
        let cms1 = CountMinSketch::new(NonZeroUsize::new(1024).unwrap(), NonZeroUsize::new(4).unwrap());
        let cms2 = CountMinSketch::new(NonZeroUsize::new(512).unwrap(), NonZeroUsize::new(4).unwrap());

        assert!(cms1.l1_distance(&cms2).is_err());
        assert!(cms1.cosine_similarity(&cms2).is_err());
    }
}