pub mod gd;
pub mod bithack;
pub mod par_sort;

pub fn parallel_for<T: Send>(slice: &mut [T], n_threads: usize, f: &(dyn Fn(&mut T)+Sync)) {
    let chunk_size = (slice.len()+n_threads-1)/n_threads;
    if chunk_size==0 {return;}
    std::thread::scope(|s| {
        for ch in slice.chunks_mut(chunk_size) {
            s.spawn(move || { for i in ch { f(i); } });
        }
    });
}