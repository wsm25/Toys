use colony::{Colony, Handle};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use slotmap::{DefaultKey, SlotMap};

const SIZES: &[usize] = &[64, 16_384, 1024 * 1024];
const BATCH_OPS: usize = 4_096;
const SPARSE_LOAD_DENOMINATOR: u64 = 100;

#[repr(C)]
struct PlfColonyOpaque {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn plf_colony_fill(size: usize) -> *mut PlfColonyOpaque;
    fn plf_colony_sparse(size: usize) -> *mut PlfColonyOpaque;
    fn plf_colony_free(container: *mut PlfColonyOpaque);
    fn plf_colony_len(container: *const PlfColonyOpaque) -> usize;
    fn plf_colony_insert_loop(size: usize) -> usize;
    fn plf_colony_iter_loop(container: *const PlfColonyOpaque) -> usize;
    fn plf_colony_get_random_loop(
        container: *const PlfColonyOpaque,
        indices: *const usize,
        len: usize,
    ) -> usize;
    fn plf_colony_random_churn_loop(size: usize, ops: usize, seed: u64) -> usize;
    fn plf_colony_churn_sparse_loop(size: usize, ops: usize, seed: u64) -> usize;
}

struct PlfColony {
    ptr: *mut PlfColonyOpaque,
}

impl PlfColony {
    fn filled(size: usize) -> Self {
        let ptr = unsafe { plf_colony_fill(size) };
        assert!(!ptr.is_null());
        Self { ptr }
    }

    fn sparse(size: usize) -> Self {
        let ptr = unsafe { plf_colony_sparse(size) };
        assert!(!ptr.is_null());
        Self { ptr }
    }

    fn len(&self) -> usize {
        unsafe { plf_colony_len(self.ptr) }
    }

    fn iter_sum(&self) -> usize {
        unsafe { plf_colony_iter_loop(self.ptr) }
    }

    fn get_random_sum(&self, indices: &[usize]) -> usize {
        unsafe { plf_colony_get_random_loop(self.ptr, indices.as_ptr(), indices.len()) }
    }
}

impl Drop for PlfColony {
    fn drop(&mut self) {
        unsafe { plf_colony_free(self.ptr) };
    }
}

fn fill_colony(size: usize) -> (Colony<usize>, Vec<Handle>) {
    let mut colony = Colony::with_capacity(size);
    let mut handles = Vec::with_capacity(size);
    for value in 0..size {
        handles.push(colony.insert(value));
    }
    (colony, handles)
}

fn fill_slotmap(size: usize) -> (SlotMap<DefaultKey, usize>, Vec<DefaultKey>) {
    let mut map = SlotMap::with_capacity_and_key(size);
    let mut keys = Vec::with_capacity(size);
    for value in 0..size {
        keys.push(map.insert(value));
    }
    (map, keys)
}

fn sparse_keep(index: usize, size: usize) -> bool {
    let mut value = (index as u64) ^ (size as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value = value ^ (value >> 31);
    value.is_multiple_of(SPARSE_LOAD_DENOMINATOR)
}

fn make_sparse_colony(size: usize) -> (Colony<usize>, Vec<Handle>) {
    let (mut colony, handles) = fill_colony(size);
    let survivors = handles
        .into_iter()
        .enumerate()
        .filter_map(|(index, handle)| {
            if sparse_keep(index, size) {
                Some(handle)
            } else {
                colony.remove(handle);
                None
            }
        })
        .collect();
    (colony, survivors)
}

fn make_sparse_slotmap(size: usize) -> (SlotMap<DefaultKey, usize>, Vec<DefaultKey>) {
    let (mut map, keys) = fill_slotmap(size);
    let survivors = keys
        .into_iter()
        .enumerate()
        .filter_map(|(index, key)| {
            if sparse_keep(index, size) {
                Some(key)
            } else {
                map.remove(key);
                None
            }
        })
        .collect();
    (map, survivors)
}

fn random_indices(len: usize, ops: usize, seed: u64) -> Vec<usize> {
    let mut rng = fastrand::Rng::with_seed(seed);
    (0..ops).map(|_| rng.usize(0..len)).collect()
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
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map: SlotMap<DefaultKey, usize> = SlotMap::with_capacity_and_key(size);
                for value in 0..size {
                    black_box(map.insert(black_box(value)));
                }
                black_box(map);
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &size| {
            b.iter(|| black_box(unsafe { plf_colony_insert_loop(black_box(size)) }));
        });
    }
    group.finish();
}

fn bench_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter");
    for &size in SIZES {
        let (colony, _) = fill_colony(size);
        let (slotmap, _) = fill_slotmap(size);
        let plf = PlfColony::filled(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                black_box(
                    colony
                        .iter()
                        .fold(0usize, |sum, value| sum.wrapping_add(*value)),
                )
            });
        });
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &_size| {
            b.iter(|| {
                black_box(
                    slotmap
                        .values()
                        .fold(0usize, |sum, value| sum.wrapping_add(*value)),
                )
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &_size| {
            b.iter(|| black_box(plf.iter_sum()));
        });
    }
    group.finish();
}

fn bench_get_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_random");
    for &size in SIZES {
        let (colony, handles) = fill_colony(size);
        let (slotmap, keys) = fill_slotmap(size);
        let plf = PlfColony::filled(size);
        let indices = random_indices(size, BATCH_OPS, size as u64 + 1);
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    sum = sum.wrapping_add(*colony.get(handles[index]).unwrap());
                }
                black_box(sum);
            });
        });
        group.bench_with_input(
            BenchmarkId::new("colony_unchecked", size),
            &size,
            |b, &_size| {
                b.iter(|| {
                    let mut sum = 0usize;
                    for &index in &indices {
                        sum = sum.wrapping_add(unsafe { *colony.get_unchecked(handles[index]) });
                    }
                    black_box(sum);
                });
            },
        );
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    sum = sum.wrapping_add(*slotmap.get(keys[index]).unwrap());
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &_size| {
            b.iter(|| black_box(plf.get_random_sum(&indices)));
        });
    }
    group.finish();
}

fn bench_iter_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_sparse");
    for &size in SIZES {
        let (colony, handles) = make_sparse_colony(size);
        let (slotmap, keys) = make_sparse_slotmap(size);
        let plf = PlfColony::sparse(size);
        let live = handles.len();
        assert_eq!(live, keys.len());
        assert_eq!(live, plf.len());
        group.throughput(Throughput::Elements(live as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                black_box(
                    colony
                        .iter()
                        .fold(0usize, |sum, value| sum.wrapping_add(*value)),
                )
            });
        });
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &_size| {
            b.iter(|| {
                black_box(
                    slotmap
                        .values()
                        .fold(0usize, |sum, value| sum.wrapping_add(*value)),
                )
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &_size| {
            b.iter(|| black_box(plf.iter_sum()));
        });
    }
    group.finish();
}

fn bench_get_random_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_random_sparse");
    for &size in SIZES {
        let (colony, handles) = make_sparse_colony(size);
        let (slotmap, keys) = make_sparse_slotmap(size);
        let plf = PlfColony::sparse(size);
        let indices = random_indices(handles.len(), BATCH_OPS, size as u64 + 77);
        group.throughput(Throughput::Elements(BATCH_OPS as u64));
        group.bench_with_input(BenchmarkId::new("colony", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    sum = sum.wrapping_add(*colony.get(handles[index]).unwrap());
                }
                black_box(sum);
            });
        });
        group.bench_with_input(
            BenchmarkId::new("colony_unchecked", size),
            &size,
            |b, &_size| {
                b.iter(|| {
                    let mut sum = 0usize;
                    for &index in &indices {
                        sum = sum.wrapping_add(unsafe { *colony.get_unchecked(handles[index]) });
                    }
                    black_box(sum);
                });
            },
        );
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &_size| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indices {
                    sum = sum.wrapping_add(*slotmap.get(keys[index]).unwrap());
                }
                black_box(sum);
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &_size| {
            b.iter(|| black_box(plf.get_random_sum(&indices)));
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
                        black_box(*colony.get(handles[index]).unwrap());
                    } else {
                        let index = rng.usize(0..handles.len());
                        let handle = handles.swap_remove(index);
                        black_box(colony.remove(handle).unwrap());
                        handles.push(colony.insert(next_value));
                        next_value += 1;
                    }
                }
                black_box((colony, handles));
            });
        });
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &size| {
            b.iter(|| {
                let (mut map, mut keys) = fill_slotmap(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 9_001);
                let mut next_value = size;
                for _ in 0..BATCH_OPS {
                    let action = rng.usize(0..100);
                    if action < 45 {
                        let index = rng.usize(0..keys.len());
                        black_box(*map.get(keys[index]).unwrap());
                    } else {
                        let index = rng.usize(0..keys.len());
                        let key = keys.swap_remove(index);
                        black_box(map.remove(key).unwrap());
                        keys.push(map.insert(next_value));
                        next_value += 1;
                    }
                }
                black_box((map, keys));
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &size| {
            b.iter(|| {
                black_box(unsafe {
                    plf_colony_random_churn_loop(size, BATCH_OPS, size as u64 + 9_001)
                })
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
                    black_box(colony.remove(handle).unwrap());
                    handles.push(colony.insert(next_value));
                    next_value += 1;
                }
                black_box((colony, handles));
            });
        });
        group.bench_with_input(BenchmarkId::new("slotmap", size), &size, |b, &size| {
            b.iter(|| {
                let (mut map, mut keys) = make_sparse_slotmap(size);
                let mut rng = fastrand::Rng::with_seed(size as u64 + 123_456);
                let mut next_value = size;
                for _ in 0..BATCH_OPS {
                    let index = rng.usize(0..keys.len());
                    let key = keys.swap_remove(index);
                    black_box(map.remove(key).unwrap());
                    keys.push(map.insert(next_value));
                    next_value += 1;
                }
                black_box((map, keys));
            });
        });
        group.bench_with_input(BenchmarkId::new("plf_colony", size), &size, |b, &size| {
            b.iter(|| {
                black_box(unsafe {
                    plf_colony_churn_sparse_loop(size, BATCH_OPS, size as u64 + 123_456)
                })
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
