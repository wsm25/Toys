use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fft::fft;
use num_complex::Complex;
use rand::{Rng, thread_rng};
use rustfft::FftPlanner;

fn generate_random_signal(n: usize) -> Vec<Complex<f64>> {
    let mut rng = thread_rng();
    (0..n)
        .map(|_| Complex::new(rng.r#gen::<f64>(), rng.r#gen::<f64>()))
        .collect()
}

pub fn bench_fft_implementations(c: &mut Criterion) {
    let sizes = [1024, 4096, 16384, 65536, 262144, 1048576];

    let mut group = c.benchmark_group("FFT");

    for &size in &sizes {
        group.throughput(Throughput::Elements(size as u64));

        // Benchmark custom FFT implementation
        group.bench_function(format!("custom_{}", size), |b| {
            let signal = generate_random_signal(size);
            b.iter(|| {
                let mut input = black_box(signal.clone());
                fft(black_box(&mut input));
            });
        });

        // Benchmark rustfft implementation
        group.bench_function(format!("rustfft_{}", size), |b| {
            let signal = generate_random_signal(size);
            let mut planner = FftPlanner::new();
            let fft_plan = planner.plan_fft_forward(size);

            b.iter(|| {
                let mut input = black_box(signal.clone());
                fft_plan.process(black_box(&mut input));
            });
        });
    }

    group.finish();
}

// Additional benchmark group for very large FFTs
pub fn bench_large_fft(c: &mut Criterion) {
    let large_sizes = [4 * 1024 * 1024, 16 * 1024 * 1024];

    let mut group = c.benchmark_group("Large FFT");
    group.sample_size(10); // Reduce sample size for large benchmarks

    for &size in &large_sizes {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(format!("custom_{}", size), |b| {
            let signal = generate_random_signal(size);
            b.iter(|| {
                let mut input = black_box(signal.clone());
                fft(black_box(&mut input));
            });
        });

        group.bench_function(format!("rustfft_{}", size), |b| {
            let signal = generate_random_signal(size);
            let mut planner = FftPlanner::new();
            let fft_plan = planner.plan_fft_forward(size);

            b.iter(|| {
                let mut input = black_box(signal.clone());
                fft_plan.process(black_box(&mut input));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fft_implementations, bench_large_fft);
criterion_main!(benches);
