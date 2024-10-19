
use std::{alloc::Layout, fmt::{Debug, Write}};

pub struct BitSlice {
    buf: *mut u128,
    bits: usize,
}

impl BitSlice {
    pub fn new(bits: usize, mut init: impl std::io::Read) -> Self {
        let bs = unsafe{Self::uninit(bits)};
        let slice = unsafe{std::slice::from_raw_parts_mut(bs.buf as *mut u8, (bits+7)/8)};
        init.read_exact(slice).unwrap();
        bs
    }
    pub unsafe fn uninit(bits: usize)->Self {
        let buf = unsafe{std::alloc::alloc(Layout::array::<u128>((bits+127)/128).unwrap())};
        Self{ buf: buf as *mut u128, bits }
    }
    pub fn from_fn(bits: usize, init: impl Fn(usize)->bool) -> Self {
        let mut bs = unsafe{Self::uninit(bits)};
        for i in 0..bits {
            bs.set(i, init(i))
        }
        bs
    }
    pub fn get(&self, index:usize) -> bool {
        debug_assert!(index<self.bits);
        unsafe{
            *self.buf.add(index/128) & (1u128<<(index%128)) != 0
        }
    }
    pub fn set(&mut self, index:usize, value: bool) {
        debug_assert!(index<self.bits);
        if self.get(index)!=value {
            unsafe{
                *self.buf.add(index/128) ^= (value as u128)<<(index%128) ;
            }
        }
    }
    pub fn bits(&self)->usize {
        self.bits
    }

    pub fn rotate_left(&mut self, offset: usize, len: usize, n: usize) {
        assert!(offset+len<=self.bits);
        let n = n%len;
        if n==0 {return;}
        self.reverse_range(offset, offset+len-1);
        self.reverse_range(offset, offset+n-1);
        self.reverse_range(offset+n, offset+len-1);
    }

    pub fn reverse_range(&mut self, mut from: usize, mut to: usize) {
        // small range
        if to-from<512 {
            while from<to {
                self.switch_bits(from, to);
                from+=1; to-=1;
            }
            return;
        }
        // shrink to align
        while from%128 != 0 {
            self.switch_bits(from, to);
            from+=1; to-=1;
        }
        unsafe{
            // now aligned, prepare
            let (buf, bits) = (self.buf.add(from/128), to-from+1);
            let offset = (bits%128) as u32;
            let mask1 = (1u128<<offset)-1;
            let mask2 = 0^mask1;
            let (mut p1, mut p2) = (buf, buf.add(bits/128));
            let (mut t1, mut t2) = (*p1, *p2);

            macro_rules! magic {($t:ident) => { $t = $t.rotate_right(offset).reverse_bits() }; }
            // last chunk
            if offset!=0 {
                t2 = t2.rotate_right(offset).reverse_bits();
                *p1 = (*p1 & mask2) | (t2 & mask1);
            }
            loop {
                // map left to right
                magic!(t1);
                *p2 = (*p2&mask2) | (t1&mask1);
                p2 = p2.sub(1);
                t2=*p2;
                if p2==p1 {
                    *p2 = (*p2&mask1) | (t1&mask2);
                    break;
                }
                *p2 = t1;
                // map right to left
                magic!(t2);
                *p1 = (*p1&mask1) | (t2&mask2);
                p1=p1.add(1);
                t1=*p1;
                if p2==p1 {
                    *p1 = (*p1&mask2) | (t2&mask1);
                    break;
                }
                *p1 = t2; // higher half will be overwritten
            }
        }
    }

    fn switch_bits(&mut self, m:usize, n:usize) {
        let (mb, nb) = (self.get(m), self.get(n));
        self.set(m, nb); self.set(n, mb);
    }
}

impl Drop for BitSlice {
    fn drop(&mut self) {
        unsafe{std::alloc::dealloc(self.buf as _, Layout::array::<u128>((self.bits+127)/128).unwrap());}
    }
}

impl Debug for BitSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BitSlice(")?;
        for i in 0..self.bits {
            f.write_char(match self.get(i){
                true => '1',
                false => '0'
            })?;
        }
        f.write_char(')')
    }
}

impl PartialEq for BitSlice {
    fn eq(&self, other: &Self) -> bool {
        if self.bits != other.bits {return false;}
        for i in 0..self.bits/128 {
            unsafe {
                if *self.buf.add(i) != *other.buf.add(i) {
                    return false;
                }
            }
        }
        let mask = (1u128<<(self.bits%128))-1;
        unsafe{
            *self.buf.add(self.bits/128) & mask == *other.buf.add(self.bits/128) & mask
        }
    }
}