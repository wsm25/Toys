use std::{alloc::{alloc, dealloc, Layout}, marker::PhantomData, mem::transmute, thread::Scope};

pub fn par_sort<T:Ord>(buf: &mut [T]) {
    let num_threads = match std::thread::available_parallelism() {
        Ok(ns) => ns.get(),
        Err(_) => 1
    };
    if num_threads==1 {
        buf.sort();
        return;
    }
    // sort chunks
    let chunk_size = ((buf.len()+num_threads-1)/num_threads).max(1024);
    std::thread::scope(|s| {
        for ch in buf.chunks_mut(chunk_size) {
            let (ptr, len) = (ch.as_mut_ptr() as usize, ch.len());
            s.spawn(move || {
                let ptr = ptr as *mut T;
                let ch = unsafe{core::slice::from_raw_parts_mut(ptr, len)};
                ch.sort();
            });
        } 
    });
    // merge chunks
    let ptmp = unsafe{alloc(Layout::for_value(buf))} as usize; // usize to send
    let pbuf = buf.as_mut_ptr() as usize;
    let chunks: Vec<(usize, usize)> = buf.chunks_mut(chunk_size).map(|ch| (ch.as_mut_ptr() as usize, ch.len())).collect();
    // merge closure
    struct MergeInfo<T> {scope: usize /* &Scope */, ptmp:usize, pbuf:usize, phantom: PhantomData<T>}
    unsafe impl<T> Sync for MergeInfo<T> {}
    unsafe impl<T> Send for MergeInfo<T> {}
    impl<T:Ord> MergeInfo<T> {
        fn merge<'a>(&'a self, ch: &'a [(usize, usize)]) {
            if ch.len()==1 {} 
            else if ch.len()==2 {
                let (ch1, ch2) = (ch[0], ch[1]);
                unsafe{merge(
                    ch1.0 as *mut T,
                    (ch1.0-self.pbuf+self.ptmp) as *mut T,
                    ch1.1+ch2.1,
                    ch1.1
                )};
            } else {
                let (ch1, ch2) = ch.split_at((ch.len()+1)/2);
                // recursion
                let scope: &'a Scope<'a, '_> = unsafe{transmute(self.scope)};
                let j = scope.spawn(|| self.merge(ch2));
                self.merge(ch1);
                j.join().unwrap();
                // merge
                let first = ch1.first().unwrap().0 as *mut T;
                let mid = ch2.first().unwrap().0 as *mut T;
                let last = { let ch = ch2.last().unwrap(); unsafe{(ch.0 as *mut T).add(ch.1)}};
                unsafe{merge(
                    first,
                    (first as usize-self.pbuf+self.ptmp) as *mut T,
                    last.offset_from(first) as usize,
                    mid.offset_from(first) as usize,
                )};
            }
        }
    }
    std::thread::scope(|scope| {
        // root thread!
        let mi: MergeInfo<T> = MergeInfo{ scope: unsafe{transmute(scope)}, ptmp, pbuf, phantom:PhantomData };
        mi.merge(&chunks);
    });
    unsafe{dealloc(ptmp as *mut u8, Layout::for_value(buf));}
}

// merge src[0..chunk-1] and src[chunk..]
// unsafe: src and buf must be valid and does not overlap; buf must not owns `T`.
unsafe fn merge<T: Ord>(src: *mut T, buf: *mut T, len: usize, chunk: usize) {
    let mut b = buf;
    let (end1, end2) = (src.add(chunk), src.add(len));
    let (mut p1, mut p2) = (src, end1);
    while p1<end1 && p2<end2 {
        if *p1 < *p2 {
            b.write(p1.read());
            p1 = p1.add(1);
        } else {
            b.write(p2.read());
            p2 = p2.add(1);
        }
        b = b.add(1);
    }
    // flush remain
    while p1<end1 {
        b.write(p1.read());
        b=b.add(1); p1=p1.add(1);
    }
    while p2<end2 {
        b.write(p2.read());
        b=b.add(1); p2=p2.add(1);
    }
    // copy back
    core::ptr::copy_nonoverlapping(buf, src, len);
}

#[test]
fn test_par_sort() {
    const LEN:usize = 65536;
    // use std::mem::MaybeUninit;
    #[allow(invalid_value)]
    let mut buf = vec![0u64;LEN];
    let b = &mut buf[..];
    use rand::Rng;
    rand::rngs::OsRng.fill(b);
    par_sort(b);
    for i in 0..LEN-1 {
        if b[i]>b[i+1] {
            panic!("check failed on index {i}");
        }
    }
}

#[test]
fn bench_par_sort() {
    use rand::Rng;
    use rayon::slice::ParallelSliceMut;
    use std::time::Instant;
    const LEN:usize = 256*1024*1024;
    // use std::mem::MaybeUninit;
    #[allow(invalid_value)]
    let mut buf = vec![0u64;LEN];
    let b = &mut buf[..];
    rand::rngs::OsRng.fill(b);
    let backup = b.to_owned();
    let start = Instant::now();
    par_sort(b);
    println!("new par_sort: {:?}", start.elapsed());
    b.copy_from_slice(&backup[..]);
    let start = Instant::now();
    b.par_sort();
    println!("rayon par_sort: {:?}", start.elapsed());
    b.copy_from_slice(&backup[..]);
    let start = Instant::now();
    b.sort();
    println!("rust stdlib sort: {:?}", start.elapsed());
}