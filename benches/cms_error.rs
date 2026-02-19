use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use count_min_sketch_rs::CountMinSketch;
use std::collections::HashMap;

fn bench_accuracy_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("CMS_Error_Measurement");

    // Configurazioni da testare
    let configurations = [
        (1024, 4),    // Cache L1/L2
        (65536, 8),   // Cache L3
        (1048576, 16) // RAM
    ];

    let n_elements = 1_000_000;

    for (width, depth) in configurations {
        let mut cms = CountMinSketch::new(width, depth);
        let mut ground_truth = HashMap::new();

        // 1. Popolamento (Setup)
        for i in 0..n_elements {
            let key = if i % 1000 == 0 { "hitter".into() } else { format!("i_{}", i) };
            cms.increment(&key);
            *ground_truth.entry(key).or_insert(0u64) += 1;
        }

        let mut total_relative_error = 0.0;

        for (_, (key, &actual)) in ground_truth.iter().enumerate() {
            let est = cms.estimate(key);
            let error = (est - actual) as f64;
            total_relative_error += error / actual as f64;
        }
        let avg_relative_error = total_relative_error / n_elements as f64;

        // 3. Reporting
        // Usiamo l'errore nel nome del benchmark così Criterion lo mostrerà nei grafici
        let bench_label = format!("W{}D{}_ARE_{:.4}", width, depth, avg_relative_error);

        group.bench_function(BenchmarkId::new("RelativeError", bench_label), |b| {
            b.iter(|| {
                cms.estimate(black_box("hitter"))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_accuracy_metrics);
criterion_main!(benches);