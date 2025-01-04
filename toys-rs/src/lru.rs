//! Least Recently Used cache. 
use core::{borrow::Borrow, hash::Hash, ptr::NonNull};
use std::hash::BuildHasher;
use hashbrown::HashTable;

type LRUPtr<K, V> = NonNull<LRUEntry<K,V>>;

struct LRUEntry<K, V> {
    inner: (K, V),
    prev: LRUPtr<K,V>,
    next: LRUPtr<K,V>,
}

/// Least Recently Used hashmap. 
pub struct LRUMap<K, V, S = hashbrown::DefaultHashBuilder> {
    memory: HashTable<LRUPtr<K,V>>, // share `len` with hashtable
    cap: usize,
    store: LRUPtr<K,V>, // pinned, pre-allocated memory
    // head and tail are store-2 and store-1, and are uninit
    hasher: S,
}

// no unique to use!
unsafe impl<K: Send, V: Send, S: Send> Send for LRUMap<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for LRUMap<K, V, S> {}

// impl LRUEntry<K, V>

fn pointer_inner<'a,K,V>(mut p: LRUPtr<K,V>)->&'a mut (K,V) {
    unsafe{&mut p.as_mut().inner}
}

impl<K, V> LRUMap<K,V> {
    /// Create a new LRU cache with the given capacity.
    pub fn with_capacity(cap: usize)-> Self {
        Self::with_capacity_and_hasher(cap, hashbrown::DefaultHashBuilder::default())
    }
}

impl<K, V, S> LRUMap<K, V, S> {
    /// Create a new LRU cache with the given capacity and hasher.
    pub fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        // we assume cap>0, or when len==cap, tail.prev is not a valid storage
        assert!(cap>0);
        let store: *mut LRUEntry<K,V> = Vec::leak(Vec::with_capacity(cap+2)).as_mut_ptr();
        let store = unsafe {
            let s = NonNull::new_unchecked(store);
            let (mut head, mut tail) = (s, s.add(1));
            head.as_mut().next = tail;
            tail.as_mut().prev = head;
            s.add(2)
        };
        Self { store, cap, memory: HashTable::with_capacity(cap), hasher}
    }
    const fn head(&self)->LRUPtr<K,V> {
        unsafe{self.store.sub(2)}
    }
    const fn tail(&self)->LRUPtr<K,V> {
        unsafe{self.store.sub(1)}
    }
    /// clear all items
    pub fn clear(&mut self) {
        // drop LRUEntry inner
        self.for_each_entry(|p| {
            unsafe{core::ptr::drop_in_place(p.as_ptr())};
        });
        unsafe{
            self.head().as_mut().next = self.tail();
            self.tail().as_mut().prev = self.head();
        };
        self.memory.clear();
    }
    /// number of items
    pub fn len(&self)->usize {
        self.memory.len()
    }
    /// is empty
    pub fn is_empty(&self) -> bool {
        unsafe{self.head().as_ref().next == self.tail()}
    }
    
    fn for_each_entry(&self, mut f: impl FnMut(LRUPtr<K,V>)) {
        let mut p = unsafe{self.head().as_ref().next};
        while p!=self.tail() {
            f(p);
            p = unsafe{p.as_ref().next};
        }
    }

    /// insert ptr at head
    fn insert_ptr(&self, mut b: LRUPtr<K, V>) {
        unsafe {
            let mut a = self.head();
            let mut c = a.as_ref().next;
            a.as_mut().next = b;
            b.as_mut().next = c;
            c.as_mut().prev = b;
            b.as_mut().prev = a;
        }
    }

    fn update_ptr(&self, mut b: LRUPtr<K, V>) {
        unsafe {
            // remove self
            let cur = b.as_mut();
            cur.next.as_mut().prev = cur.prev;
            cur.prev.as_mut().next = cur.next;
        }
        self.insert_ptr(b);
    }
}

fn make_eq<Q: Eq+?Sized, K: Borrow<Q>, V>(k: &Q) -> impl '_ + FnMut(&LRUPtr<K,V>)->bool {
    move |p| pointer_inner(*p).0.borrow() == k
}

fn make_hasher<S: BuildHasher, K: Hash, V>(s: &S) -> impl '_ + Fn(&LRUPtr<K,V>)->u64 {
    move |p| s.hash_one(&pointer_inner(*p).0)
}

impl<K, V, S> LRUMap<K, V, S> 
where 
    K: Eq + Hash,
    S: BuildHasher
{
    /// get pointer and update lru linklist
    /// 
    /// assume Q and S have same hash method
    fn get_ptr<Q>(&self, k: &Q) -> Option<LRUPtr<K,V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let hash = self.hasher.hash_one(k);
        let b = *self.memory.find(hash, make_eq(k))?;
        self.update_ptr(b);
        Some(b)
    }
    /// get value in lru cache
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.get_ptr(k).map(|p| &pointer_inner(p).1 )
    }
    /// get mutable value in lru cache
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.get_ptr(k).map(|p| &mut pointer_inner(p).1 )
    }

    /// Insert a key-value pair into the cache.
    /// 
    /// It silently covers old value and kick out non-recently used items.
    pub fn insert(&mut self, k: K, v: V) {
        let hash = self.hasher.hash_one(&k);
        if let Some(vv) = self.get_mut(&k) {
            *vv = v;
            return;
        }
        // newbie
        if self.len()<self.cap {
            // new entry
            let p = unsafe{self.store.add(self.len())};
            self.insert_ptr(p);
            // write to uninit
            unsafe{core::ptr::write(pointer_inner(p), (k, v))};
            self.memory.insert_unique(hash, p, make_hasher(&self.hasher));
        } else {
            // reuse entry
            let p = unsafe{self.tail().as_ref().prev};
            self.update_ptr(p);
            let k_old = &pointer_inner(p).0;
            let entry_old = self.memory.find_entry(self.hasher.hash_one(k_old), make_eq(k_old));
            unsafe{entry_old.unwrap_unchecked().remove()};
            // write to init
            *pointer_inner(p) = (k, v);
            self.memory.insert_unique(hash, p, make_hasher(&self.hasher));
        }
    }
}

impl<K, V, S> Drop for LRUMap<K, V, S> {
    /// drop all items
    fn drop(&mut self) {
        self.for_each_entry(|p| {
            unsafe{core::ptr::drop_in_place(p.as_ptr())};
        });
        // deallocate memory
        drop(unsafe{Vec::from_raw_parts(
            self.store.sub(2).as_ptr(), 0, self.cap+2
        )});
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CAP: usize = 4;
    struct TestCache {
        // use Box<i32> to check lifetime compatibility
        inner: LRUMap<Box<i32>, ()>,
    }

    impl core::fmt::Debug for TestCache {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.inner.for_each_entry(|p| {
                let x = unsafe{*p.as_ref().inner.0};
                let cur = p.as_ptr() as usize;
                let prev = unsafe{p.as_ref().prev.as_ptr() as usize};
                let next = unsafe{p.as_ref().next.as_ptr() as usize};
                write!(f, "({x},{prev:#x}, {next:#x})@{cur:#x}, ").unwrap();
            });
            Ok(())
        }
    }
    
    fn items(lru: &TestCache)->Vec<i32> {
        let lru = &lru.inner;
        let mut v = Vec::with_capacity(lru.len());
        lru.for_each_entry(|p| {
            let x = unsafe{*p.as_ref().inner.0};
            v.push(x);
        });
        v
    }

    impl Default for TestCache {
        fn default() -> Self {
            let inner = LRUMap::with_capacity(CAP);
            Self { inner }
        }
    }

    impl TestCache {
        pub fn insert(&mut self, x: i32) {
            self.inner.insert(Box::new(x), ());
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn clear(&mut self) {
            self.inner.clear();
        }
    }
    
    #[test]
    fn insert() {
        let mut cache = TestCache::default();
        cache.insert(1);
        assert_eq!(cache.len(), 1);
        cache.insert(2);
        assert_eq!(cache.len(), 2);
        cache.insert(3);
        assert_eq!(cache.len(), 3);
        cache.insert(4);
        assert_eq!(cache.len(), 4);
        assert_eq!(
            items(&cache),
            [4, 3, 2, 1],
            "Ordered from most- to least-recent."
        );
    
        cache.insert(5);
        assert_eq!(cache.len(), 4);
        assert_eq!(
            items(&cache),
            [5, 4, 3, 2],
            "Least-recently-used item evicted."
        );
    
        cache.insert(6);
        cache.insert(7);
        cache.insert(8);
        cache.insert(9);
        assert_eq!(
            items(&cache),
            [9, 8, 7, 6],
            "Least-recently-used item evicted."
        );
    }
    
    #[test]
    fn clear() {
        let mut cache = TestCache::default();
        for i in 1..5 {
            cache.insert(i);
        }
        cache.clear();
        assert_eq!(items(&cache), [], "all items evicted");
    
        cache.insert(1);
        cache.insert(2);
        cache.insert(3);
        cache.insert(4);
        assert_eq!(items(&cache), [4, 3, 2, 1]);
        cache.clear();
        assert_eq!(items(&cache), [], "all items evicted again");
    }
}