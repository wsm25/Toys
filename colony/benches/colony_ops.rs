use colony::{Colony, Handle};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

const SIZES: &[usize] = &[64, 1_024, 16_384];
const BATCH_OPS: usize = 4_096;

fn fill_colony(size: usize) -> (Colony<usize>, Vec<Handle>) {
    let mut colony = Colony::with_capacity(size);
    let mut handles = Vec::with_capacity(size);
    for value in 0..size {
        handles.push(colony.insert(value));
    }
    (colony, handles)
}

fn fill_hash_map(size: usize) -> (HashMap<usize, usize>, Vec<usize>) {
    let mut map = HashMap::with_capacity(size);
    let mut keys = Vec::with_capacity(size);
    for value in 0..size {
        map.insert(value, value);
        keys.push(value);
    }
    (map, keys)
}

fn make_sparse_colony(size: usize) -> (Colony<usize>, Vec<Handle>) {
    let (mut colony, handles) = fill_colony(size);
    let survivors: Vec<Handle> = handles
        .into_iter()
        .enumerate()
        .filter_map(|(index, handle)| {
            if index % 64 == 0 {
                Some(handle)
            } else {
                colony.remove(handle);
                None
            }
        })
        .collect();
    (colony, survivors)
}

fn make_sparse_hash_map(size: usize) -> (HashMap<usize, usize>, Vec<usize>) {
    let (mut map, keys) = fill_hash_map(size);
    let survivors: Vec<usize> = keys
        .into_iter()
        .enumerate()
        .filter_map(|(index, key)| {
            if index % 64 == 0 {
                Some(key)
            } else {
                map.remove(&key);
                None
            }
        })
        .collect();
    (map, survivors)
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &size| {
            b.iter(|| {
                let mut colony = Colony::with_capacity(size);
                for value in 0..size {
                    black_box(colony.insert(black_box(value)));
                }
                black_box(colony);
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = HashMap::with_capacity(size);
                for value in 0..size {
                    black_box(map.insert(black_box(value), black_box(value)));
                }
                black_box(map);
            });
        });
    }
    group.finish();
}

fn bench_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter");
    for &size in SIZES {
        let (colony, _) = fill_colony(size);
        let (map, _) = fill_hash_map(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for value in &colony {
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for value in map.values() {
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_get_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_random");
    for &size in SIZES {
        let (colony, handles) = fill_colony(size);
        let (map, keys) = fill_hash_map(size);
        let mut rng = fastrand::Rng::with_seed(size as u64 + 1);
        let indices: Vec<usize> = (0..BATCH_OPS)
            .map(|_| rng.usize(0..handles.len()))
            .collect();
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    let value = colony
                        .get(handles[index])
                        .expect("random benchmark handle should remain valid");
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    let value = map
                        .get(&keys[index])
                        .expect("random benchmark key should remain valid");
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_random_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_churn");
    for &size in SIZES {
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &size| {
            b.iter(|| {
                let (mut colony, mut handles) = fill_colony(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 9_001);
                let mut next_value = size;

                for _ in 0..BATCH_OPS {
                    let action = rng.usize(0..100);
                    if action < 45 {
                        let index = rng.usize(0..handles.len());
                        let value = colony
                            .get(handles[index])
                            .expect("random benchmark handle should remain valid");
                        black_box(*value);
                    } else {
                        let index = rng.usize(0..handles.len());
                        let handle = handles.swap_remove(index);
                        let removed = colony
                            .remove(handle)
                            .expect("random benchmark handle should remain valid");
                        black_box(removed);

                        let inserted = colony.insert(next_value);
                        next_value += 1;
                        handles.push(inserted);
                    }
                }

                black_box((colony, handles));
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let (mut map, mut keys) = fill_hash_map(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 9_001);
                let mut next_key = size;

                for _ in 0..BATCH_OPS {
                    let action = rng.usize(0..100);
                    if action < 45 {
                        let index = rng.usize(0..keys.len());
                        let value = map
                            .get(&keys[index])
                            .expect("random benchmark key should remain valid");
                        black_box(*value);
                    } else {
                        let index = rng.usize(0..keys.len());
                        let key = keys.swap_remove(index);
                        let removed = map
                            .remove(&key)
                            .expect("random benchmark key should remain valid");
                        black_box(removed);

                        map.insert(next_key, next_key);
                        keys.push(next_key);
                        next_key += 1;
                    }
                }

                black_box((map, keys));
            });
        });
    }
    group.finish();
}

fn bench_iter_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_sparse");
    for &size in SIZES {
        let (colony, colony_handles) = make_sparse_colony(size);
        let (map, _keys) = make_sparse_hash_map(size);
        let live = colony_handles.len();
        group.throughput(Throughput::Elements(live as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for value in &colony {
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for value in map.values() {
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_get_random_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_random_sparse");
    for &size in SIZES {
        let (colony, handles) = make_sparse_colony(size);
        let (map, keys) = make_sparse_hash_map(size);
        let mut rng = fastrand::Rng::with_seed(size as u64 + 77);
        let indices: Vec<usize> = (0..BATCH_OPS)
            .map(|_| rng.usize(0..handles.len()))
            .collect();
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    let value = colony
                        .get(handles[index])
                        .expect("sparse benchmark handle should remain valid");
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    let value = map
                        .get(&keys[index])
                        .expect("sparse benchmark key should remain valid");
                    sum = sum.wrapping_add(*value);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_churn_on_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("churn_on_sparse");
    for &size in SIZES {
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &size| {
            b.iter(|| {
                let (mut colony, mut handles) = make_sparse_colony(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 123_456);
                let mut next_value = size;

                for _ in 0..BATCH_OPS {
                    let index = rng.usize(0..handles.len());
                    let handle = handles.swap_remove(index);
                    let removed = colony
                        .remove(handle)
                        .expect("sparse churn handle should remain valid");
                    black_box(removed);

                    let inserted = colony.insert(next_value);
                    next_value += 1;
                    handles.push(inserted);
                }

                black_box((colony, handles));
            });
        });
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let (mut map, mut keys) = make_sparse_hash_map(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 123_456);
                let mut next_key = size;

                for _ in 0..BATCH_OPS {
                    let index = rng.usize(0..keys.len());
                    let key = keys.swap_remove(index);
                    let removed = map
                        .remove(&key)
                        .expect("sparse churn key should remain valid");
                    black_box(removed);

                    map.insert(next_key, next_key);
                    keys.push(next_key);
                    next_key += 1;
                }

                black_box((map, keys));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_iter,
    bench_iter_sparse,
    bench_get_random,
    bench_get_random_sparse,
    bench_random_churn,
    bench_churn_on_sparse
);
criterion_main!(benches);
