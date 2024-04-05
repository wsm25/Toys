use criterion::{black_box, criterion_group, criterion_main, Criterion};

use toys_rs::localpool::Pool;
use rand::random;

fn criterion_benchmark(c: &mut Criterion) {
    let mut p:Pool<[u8;65536]>=unsafe{Pool::new()};
    c.bench_function("pool one-shot", |b| b.iter(|| {
        black_box(p.get());
    }));
    let mut v=Vec::new();
    c.bench_function("pool random", |b| b.iter(|| {
        match random::<bool>(){ // 2ns
        true=>{v.push(p.get());}, // 2ns+4ns
        false=>{black_box(v.pop());}
        }
    }));
    println!("{} in use, {} idling", v.len(), p.idle());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);