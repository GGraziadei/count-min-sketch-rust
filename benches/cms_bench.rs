use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use count_min_sketch_rs::CountMinSketch;
use std::time::Duration;
use std::alloc::System;
use std::num::NonZeroUsize;
use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
static ALLOC: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn generate_random_strings(count: usize, length: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("{:0>width$}", i, width = length))
        .collect()
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
        let total_allocs = end_stats.allocations - start_stats.allocations;
        let total_bytes = end_stats.bytes_allocated - start_stats.bytes_allocated;

        println!(
            "\n[Memory Report - {}] Allocs: {}, Total Bytes: {}",
            parameter_string, total_allocs, total_bytes
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
    targets = bench_cms_full_load
}
criterion_main!(benches);