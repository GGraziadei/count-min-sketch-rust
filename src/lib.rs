use std::hash::Hash;
use ahash::RandomState;

pub struct CountMinSketch {
    pub width: usize,
    width_mask: usize,
    pub depth: usize,
    table: Box<[u64]>,
    hasher: RandomState,
}

impl CountMinSketch {
    pub fn with_params(epsilon: f64, delta: f64) -> Self {
        let width = (std::f64::consts::E / epsilon).ceil() as usize;
        let depth = (1.0 / delta).ln().ceil() as usize;
        Self::new(width, depth)
    }

    pub fn new(width: usize, depth: usize) -> Self {
        let w = width.next_power_of_two();
        Self {
            width: w,
            width_mask: w - 1,
            depth,
            table: vec![0u64; w * depth].into_boxed_slice(),
            hasher: RandomState::with_seeds(2025, 2, 18, 2118),
        }
    }

    pub fn with_seeds(width: usize, depth: usize, seeds: [u64; 4]) -> Self {
        if seeds.len() != 4 {
            panic!("seeds must have 4 elements");
        }
        let w = width.next_power_of_two();
        Self {
            width: w,
            width_mask: w - 1,
            depth,
            table: vec![0u64; w * depth].into_boxed_slice(),
            hasher: RandomState::with_seeds(seeds[0], seeds[1], seeds[2], seeds[3]),
        }
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

    pub fn merge(&mut self, other: &Self) -> Result<(), &'static str> {
        if self.width != other.width || self.depth != other.depth {
            return Err("Incompatible dimensions");
        }
        for (a, b) in self.table.iter_mut().zip(other.table.iter()) {
            *a = a.saturating_add(*b);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.table.fill(0);
    }
}