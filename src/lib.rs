use std::hash::Hash;
use std::num::NonZeroUsize;
use ahash::RandomState;


/// A high-performance, memory-efficient probabilistic data structure for frequency estimation.
///
/// `CountMinSketch` uses a fixed-size table to estimate the frequency of items in a stream.
/// It provides an upper-bound estimate with a controlled error margin ($\epsilon$) and 
/// confidence level ($\delta$).
pub struct CountMinSketch {
    width: usize,
    width_mask: usize,
    depth: usize,
    table: Box<[u64]>,
    hasher: RandomState,
}

impl CountMinSketch {
    /// Creates a new sketch with dimensions derived from statistical parameters.
    ///
    /// # Arguments
    /// * `epsilon` - The error margin. The estimation will be within `actual + epsilon * total_increments`, it is a positive between 0 and 1 excluded.
    /// * `delta` - The error probability. The confidence of the estimate is `1 - delta`, it is a positive between 0 and 1 excluded.
    ///
    pub fn with_params(epsilon: f64, delta: f64) -> Self {
        assert!(epsilon > 0. && epsilon < 1., "epsilon must be a positive between 0 and 1 excluded.");
        assert!(delta > 0. && delta < 1., "delta must be a positive between 0 and 1 excluded.");
        let width = (std::f64::consts::E / epsilon).ceil() as usize;
        let depth = (1.0 / delta).ln().ceil() as usize;
        Self::new(NonZeroUsize::try_from(width).unwrap(), NonZeroUsize::try_from(depth).unwrap())
    }

    /// Creates a new sketch with explicit `width` and `depth`.
    ///
    /// `width` will be automatically rounded up to the nearest power of two to optimize 
    /// index calculations using bitwise masking.
    pub fn new(width: NonZeroUsize, depth: NonZeroUsize) -> Self {
        let w = width.get().next_power_of_two();
        let d = depth.get().next_power_of_two();
        Self {
            width: w,
            width_mask: w - 1,
            depth: d,
            table: vec![0u64; w * d].into_boxed_slice(),
            hasher: RandomState::with_seeds(2025, 2, 18, 2118),
        }
    }
    
    /// Creates a new sketch with explicit dimensions and custom hash seeds.
    ///
    /// Useful for deterministic testing or distributed sketches that must use the same hash network.
    ///
    /// Panics if the seeds array does not contain exactly 4 elements (standard for `RandomState`).
    pub fn with_seeds(width: NonZeroUsize, depth: NonZeroUsize, seeds: [u64; 4]) -> Self {
        let w = width.get().next_power_of_two();
        let d = depth.get().next_power_of_two();
        Self {
            width: w,
            width_mask: w - 1,
            depth: d,
            table: vec![0u64; w * d].into_boxed_slice(),
            hasher: RandomState::with_seeds(seeds[0], seeds[1], seeds[2], seeds[3]),
        }
    }
    /// Returns the table width
    pub fn get_width(&self) -> usize {
        self.width
    }
    
    /// Returns the table depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }
    
    #[inline(always)]
    fn calculate_indices<F>(h1: u64, depth: usize, width: usize, mask: usize, mut f: F)
    where
        F: FnMut(usize),
    {
        let mut h2 = h1.wrapping_add(0x9E3779B97F4A7C15);
        h2 = (h2 ^ (h2 >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        h2 = (h2 ^ (h2 >> 27)).wrapping_mul(0x94D049BB133111EB);
        h2 = h2 ^ (h2 >> 31);
        let h2 = h2 | 1;

        for i in 0..depth {
            let bucket = (h1.wrapping_add((i as u64).wrapping_mul(h2)) as usize) & mask;
            f(i * width + bucket);
        }
    }

    /// Increments the frequency count for the given item.
    ///
    /// This operation is $O(depth)$ and involves zero heap allocations. 
    /// It uses saturating addition to prevent counter overflow.
    #[inline]
    pub fn increment<T: Hash + ?Sized>(&mut self, item: &T) {
        let h1 = self.hasher.hash_one(item);
        let d = self.depth;
        let w = self.width;
        let m = self.width_mask;

        Self::calculate_indices(h1, d, w, m, |idx| unsafe {
            let ptr = self.table.as_mut_ptr().add(idx);
            *ptr = (*ptr).saturating_add(1);
        });
    }

    /// Estimates the frequency count of the given item.
    ///
    /// Returns the minimum value across all hash rows. 
    /// Guaranteed to be greater than or equal to the actual count.
    #[inline]
    pub fn estimate<T: Hash + ?Sized>(&self, item: &T) -> u64 {
        let h1 = self.hasher.hash_one(item);
        let mut min_val = u64::MAX;

        Self::calculate_indices(h1, self.depth, self.width, self.width_mask, |idx| {
            let val = unsafe { *self.table.get_unchecked(idx) };
            if val < min_val {
                min_val = val;
            }
        });

        if min_val == u64::MAX { 0 } else { min_val }
    }

    /// Merges another Count-Min Sketch into this one.
    ///
    /// # Errors
    /// Returns an error if the sketches have different `width` or `depth` dimensions.
    pub fn merge(&mut self, other: &Self) -> Result<(), &'static str> {
        if self.width != other.width || self.depth != other.depth {
            return Err("Incompatible dimensions");
        }
        for (a, b) in self.table.iter_mut().zip(other.table.iter()) {
            *a = a.saturating_add(*b);
        }
        Ok(())
    }

    /// Calculates the L1 distance (Manhattan Distance) between two sketches.
    /// Estimates the sum of absolute differences in frequencies.
    pub fn l1_distance(&self, other: &Self) -> Result<u64, &'static str> {
        if self.width != other.width || self.depth != other.depth {
            return Err("Incompatible dimensions.");
        }
        let mut min_l1 = u64::MAX;
        for d in 0..self.depth {
            let start = d * self.width;
            let end = start + self.width;
            let row_l1: u64 = self.table[start..end]
                .iter()
                .zip(&other.table[start..end])
                .map(|(&a, &b)| a.abs_diff(b))
                .sum();
            min_l1 = min_l1.min(row_l1);
        }
        Ok(min_l1)
    }

    /// Calculates the Cosine Similarity between two sketches [0.0 to 1.0].
    /// A value of 1.0 means the distributions are identical.
    pub fn cosine_similarity(&self, other: &Self) -> Result<f64, &'static str> {
        if self.width != other.width || self.depth != other.depth {
            return Err("Incompatible dimensions.");
        }
        let mut max_sim: f64 = 0.0;
        for d in 0..self.depth {
            let (mut dot, mut n_a, mut n_b) = (0.0, 0.0, 0.0);
            let start = d * self.width;
            for (&a, &b) in self.table[start..start+self.width].iter().zip(&other.table[start..start+self.width]) {
                let (fa, fb) = (a as f64, b as f64);
                dot += fa * fb;
                n_a += fa * fa;
                n_b += fb * fb;
            }
            if n_a > 0.0 && n_b > 0.0 {
                max_sim = max_sim.max(dot / (n_a.sqrt() * n_b.sqrt()));
            }
        }
        Ok(max_sim)
    }

    /// Resets all frequency counters to zero.
    ///
    /// This operation clears the internal table, effectively resetting the sketch
    /// to its initial state while preserving its dimensions and hash configuration.
    pub fn clear(&mut self) {
        self.table = vec![0u64; self.width * self.depth].into_boxed_slice();
    }
}