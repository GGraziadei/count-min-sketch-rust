use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use count_min_sketch_rs::CountMinSketch; // Ensure this matches your crate name
use std::time::Duration;
use std::alloc::System;
use std::num::NonZeroUsize;
use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};
use rand::prelude::*;
use rand_distr::{Distribution, Normal, Uniform};

#[global_allocator]
static ALLOC: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

// --- Helper Functions ---

fn generate_random_strings(count: usize, length: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("{:0>width$}", i, width = length))
        .collect()
}

/// Generates two sketches representing different statistical distributions
fn setup_distribution_sketches(w: usize, d: usize) -> (CountMinSketch, CountMinSketch) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut cms_uniform = CountMinSketch::new(NonZeroUsize::new(w).unwrap(), NonZeroUsize::new(d).unwrap());
    let mut cms_normal = CountMinSketch::new(NonZeroUsize::new(w).unwrap(), NonZeroUsize::new(d).unwrap());

    // Fill Uniform: Values spread evenly across 0..10000
    let dist_u = Uniform::new(0u64, 10000u64).expect("Failed to create Uniform distribution");
    for _ in 0..20_000 {
        let val = dist_u.sample(&mut rng);
        cms_uniform.increment(&val);
    }

    // Fill Normal: Values clustered around 5000 (standard deviation 1000)
    let dist_n = Normal::new(5000.0, 1000.0).unwrap();
    for _ in 0..20_000 {
        let val = (dist_n.sample(&mut rng) as i64).to_string();
        cms_normal.increment(&val);
    }

    (cms_uniform, cms_normal)
}

// --- Benchmarks ---

fn bench_cms_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("CMS_Distribution_Comparison");

    // Large enough to simulate real-world data, small enough for fast benches
    let (w, d) = (65536, 8);
    let (cms_u, cms_n) = setup_distribution_sketches(w, d);
    let parameter_string = format!("W{}xD{}", w, d);

    // Benchmark Cosine Similarity
    group.bench_function(BenchmarkId::new("Cosine_Similarity", &parameter_string), |b| {
        b.iter(|| {
            black_box(cms_u.cosine_similarity(black_box(&cms_n)).unwrap())
        })
    });

    // Benchmark Cosine ok
    group.bench_function(BenchmarkId::new("Cosine_Similarity_true", &parameter_string), |b| {
        b.iter(|| {
            black_box(cms_u.cosine_similarity(black_box(&cms_u)).unwrap())
        })
    });

    // Benchmark L1 Distance
    group.bench_function(BenchmarkId::new("L1_Distance", &parameter_string), |b| {
        b.iter(|| {
            black_box(cms_u.l1_distance(black_box(&cms_n)).unwrap())
        })
    });

    // Benchmark L1 Distance 0
    group.bench_function(BenchmarkId::new("L1_Distance_0", &parameter_string), |b| {
        b.iter(|| {
            black_box(cms_u.l1_distance(black_box(&cms_u)).unwrap())
        })
    });

    // Print a one-time qualitative report of the similarity
    let sim = cms_u.cosine_similarity(&cms_n).unwrap();
    println!("\n[Statistical Insight] Similarity Uniform vs Normal: {:.4}", sim);

    group.finish();
}

fn bench_cms_full_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("CountMinSketch_Performance");

    let configurations : [(usize, usize); 3] = [
        (1024, 4),    // Cache L1/L2 friendly
        (65536, 8),   // Cache L3 boundary
        (1048576, 16) // RAM heavy
    ];

    let dataset_size = 100_000;
    let keys = generate_random_strings(dataset_size, 16);

    for (w, d) in configurations {
        let mut cms = CountMinSketch::new(NonZeroUsize::try_from(w).unwrap(), NonZeroUsize::try_from(d).unwrap());
        let parameter_string = format!("W{}xD{}", w, d);
        group.throughput(Throughput::Elements(1));

        let start_stats = ALLOC.stats();

        group.bench_with_input(
            BenchmarkId::new("Incremental_Update", &parameter_string),
            &keys,
            |b, keys| {
                let mut i = 0;
                b.iter(|| {
                    let key = &keys[i % dataset_size];
                    cms.increment(black_box(key));
                    i += 1;
                });
            },
        );

        let end_stats = ALLOC.stats();
        println!(
            "\n[Memory Report - {}] Allocs: {}, Total Bytes: {}",
            parameter_string,
            end_stats.allocations - start_stats.allocations,
            end_stats.bytes_allocated - start_stats.bytes_allocated
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .significance_level(0.01)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets = bench_cms_full_load, bench_cms_comparison
}
criterion_main!(benches);