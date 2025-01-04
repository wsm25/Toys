use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hashbrown::HashMap;
use lru_foo::LruCache;
use toys_rs::lru::LRUMap;
use std::{collections::BTreeMap, num::NonZeroUsize};

// Assuming your custom LRU map is in a module called `my_lru`

const CAP: usize = 1024*32;

fn hashtable(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hashtable");
    group.bench_function("Cache hit", |b| {
        let mut map = HashMap::with_capacity(CAP);
        let mut i = 0;
        b.iter(|| {
            map.insert(i, i);
            i = (i+1)%(CAP);
            black_box(map.get(&i));
            black_box(map.remove(&(i+CAP)));
        });
    });
    
    group.bench_function("Cache miss", |b| {
        let mut map = HashMap::with_capacity(CAP);
        let mut i = 0;
        b.iter(|| {
            map.insert(i, i);
            i += 1;
            black_box(map.get(&i));
            black_box(map.remove(&(i-CAP)));
        });
    });
}


fn btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("BTreeMap");
    group.bench_function("Cache hit", |b| {
        let mut map = BTreeMap::new();
        let mut i = 0;
        b.iter(|| {
            map.insert(i, i);
            i = (i+1)%(CAP);
            black_box(map.get(&i));
            black_box(map.remove(&(i+CAP)));
        });
    });
    
    group.bench_function("Cache miss", |b| {
        let mut map = BTreeMap::new();
        let mut i = 0;
        b.iter(|| {
            map.insert(i, i);
            i += 1;
            black_box(map.get(&i));
            black_box(map.remove(&(i-CAP)));
        });
    });
}

fn lru_wsm(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU-wsm");

    group.bench_function("Cache hit", |b| {
        let mut map = LRUMap::with_capacity(CAP);
        let mut i=0;
        b.iter(|| {
            map.insert(i, i);
            i = (i+1) % CAP;
            black_box(map.get(&i));
        });
    });

    // Benchmark your custom LRU map
    group.bench_function("Cache miss", |b| {
        let mut map = LRUMap::with_capacity(CAP);
        let mut i=0;
        b.iter(|| {
            map.insert(i, i);
            i += 1;
            black_box(map.get(&i));
        });
    });
    

    group.finish();
}


// Define the benchmark function
fn lru_foo(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Foo");

    // Benchmark your custom LRU map
    
    // Benchmark the `lru` crate's LRU cache
    group.bench_function("Cache hit", |b| {
        let mut cache = LruCache::new(NonZeroUsize::new(CAP).unwrap());
        let mut i = 0;
        b.iter(|| {
            cache.put(i, i);
            i = (i+1)%CAP;
            black_box(cache.get(&i));
        });
    });

    // Benchmark the `lru` crate's LRU cache
    group.bench_function("Cache miss", |b| {
        let mut cache = LruCache::new(NonZeroUsize::new(CAP).unwrap());
        let mut i = 0;
        b.iter(|| {
            cache.put(i, i);
            i += 1;
            black_box(cache.get(&i));
        });
    });

    group.finish();
}


criterion_group!(benches, btreemap, hashtable, lru_wsm, lru_foo);
criterion_main!(benches);