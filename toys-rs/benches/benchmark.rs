use criterion::{black_box, criterion_group, criterion_main, Criterion};

use toys_rs::localpool::Pool;
use rand::random;

fn bench_pool(c: &mut Criterion) {
    let mut p:Pool<[u64;16]>=Pool::new();
    p.reserve(8192);
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
    p.release(1024);
}


criterion_group!(benches, bench_pool);
criterion_main!(benches);