# Implement notes

今天看到 C++26 引入了 std::hive / plf::colony，想效仿实现一个，遂 vibe coding 了一番。

colony 提供的是以下接口：
- insert: (T)->Iter<T>, 均摊 O(1)
- remove: (Iter<T>)->void, O(1)
- iterate: ()->Iter<T>, 遍历速度接近 vector

其实这些接口 vector 也能提供且有时性能更高（考虑 swap remove），唯一的缺点就是导致顺序不稳定，地址不稳定导致 pointer / 引用可能悬空（Rust mutable 语义最严厉的父亲）

它的特性包括
- 是一个 iteration-first 的对象表容器
- 任何元素直到 remove 前都是固定地址，不似 vector 扩缩容 / remove 会移动（类似 allocator）

也带上了 flat container 的缺点
- 不能使用 intrusive pattern 组合很多容器的功能；如 linux rbnode / list_head
- 在浪费空间和移动中选择了浪费空间

它最大的特点就是 iterate 顺序和内存顺序一致，而不似用外部 container 遍历会在访存特征上乱序，尤其是高频随机增删场景；因此它比从外部表乱序访问快，遍历性能接近 vector。

它适用于：

1. 高频线性扫描
2. 要求内存地址稳定
3. 高频创建删除

这要求还是蛮苛刻的，考虑

只有 1：vector 更好
只有 2：slab / pool 更好
1 + 3：sparse set 更好（用唯一 id 表示，如游戏引擎的 Archetype）

事实上要求 1 + 2 + 3 的少之又少，毕竟内存地址本身就是一个 id 嘛，通常多一次访存能在各种操作的算法上、数据结构上带来更大的优化。不过若不追求极致性能，作为一个性能适中的可遍历 flat object pool，colony 还是很实用的。

## Design

我们的实现没有采用原版 colony 的 skip field 或是 slotmap 的 split metadata，而是用了 occupy bitflag（由于 version mismatch 是小概率事件，而不似 swisstable 强依赖 hash match）；我们只需要 1bit 存加上用 trailing zero 即可。

同时，我们加入了 handle 和 version，handle 采用了 slotmap 同款设计，32bit index, 43bit slot version；支持最多 4G 元素（有点少？）

chunk size 设计原本参考了一些 allocator tiering, 采用了下限 64，上限 4096，其他按 2 幂做。好处就是 index=i 则它在第 i<4096 ? (log2 i)-3 : i/4096 + 6 块；可以超快计算。最大块占 64byte + 4096*(32+x) 内存。

为了处理块回收，我们需要能 O(1) 查到 index 最小的非满块，从后往前回收块。Heap 最为适合了。

bitvec 的设计已经能充分加速遍历了，除非 load rate <= 1/64，当然这种情况下应当考虑做一次 gc 了。

---

结果发现性能还远不如朴素的 fix size chunk...

## Interface

- insert(&mut self, T) -> Handle
- get(&self, Handle)->Option<&T>
- get_mut(&mut self, Handle)->Option<&mut T>
- pop(&mut self, Handle)->T

我们没提供 gc, 不如直接 IntoIter + Collect.

我们的 bitvec 接口要求不高
- iterate next non-empty / empty
- set / unset bit

## Benchmark

**Size 64**

| Benchmark            |     colony |    slotmap | plf_colony |
| -------------------- | ---------: | ---------: | ---------: |
| insert/64            | ~184.45 ns | ~127.61 ns | ~106.55 ns |
| iter/64              |  ~30.70 ns |  ~29.45 ns |  ~69.74 ns |
| iter_sparse/64       |  ~0.935 ns |  ~17.00 ns |   ~2.09 ns |
| get_random/64        |   ~4.18 µs |   ~3.57 µs |          - |
| get_random_sparse/64 |   ~4.84 µs |   ~3.50 µs |          - |
| random_churn/64      |  ~28.66 µs |  ~15.10 µs |  ~35.15 µs |
| churn_on_sparse/64   |  ~27.56 µs |  ~23.70 µs |  ~88.54 µs |

**Size 16K**

| Benchmark               |     colony |   slotmap | plf_colony |
| ----------------------- | ---------: | --------: | ---------: |
| insert/16384            |  ~42.96 µs | ~27.42 µs |  ~28.12 µs |
| iter/16384              |   ~8.54 µs |  ~7.78 µs |  ~20.59 µs |
| iter_sparse/16384       |    ~128 ns |  ~5.46 µs |    ~196 ns |
| get_random/16384        |   ~7.36 µs |  ~6.98 µs |          - |
| get_random_sparse/16384 |   ~4.94 µs |  ~3.59 µs |          - |
| random_churn/16384      |  ~95.53 µs | ~44.65 µs | ~110.11 µs |
| churn_on_sparse/16384   | ~137.39 µs | ~81.53 µs | ~194.19 µs |

**Size 1M**

| Benchmark                 |     colony |    slotmap | plf_colony |
| ------------------------- | ---------: | ---------: | ---------: |
| insert/1048576            |   ~3.59 ms |   ~2.00 ms |   ~1.86 ms |
| iter/1048576              | ~796.76 µs |   ~1.12 ms |   ~1.63 ms |
| iter_sparse/1048576       |  ~26.81 µs | ~945.16 µs |  ~25.54 µs |
| get_random/1048576        |  ~27.34 µs |  ~23.10 µs |          - |
| get_random_sparse/1048576 |  ~15.78 µs |  ~12.74 µs |          - |
| random_churn/1048576      |   ~6.36 ms |   ~2.74 ms |   ~6.62 ms |
| churn_on_sparse/1048576   |   ~8.83 ms |   ~5.47 ms |  ~11.67 ms |

被稳压一头...但差距不算太大。