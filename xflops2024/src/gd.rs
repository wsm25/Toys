use core::slice;
use crate::parallel_for;

type Float4 = wide::f32x4;

#[no_mangle]
pub extern "C" fn gradient_descent(points: *mut f32, n: u32, m: u32, eta: f32, params: &[f32;4]) {
    if m==0 {return;}
    let p = [4.*params[0]*eta, 3.*params[1]*eta, 2.*params[2]*eta, params[3]*eta];
    let n = n as usize;
    let points = if points as usize % 8 == 0 { // ensure alignment
        points
    } else {
        unsafe{
            run_gd(&mut *points, m, &p);
            points.add(1)
        }
    };
    let threads = std::thread::available_parallelism().unwrap().get();
    // just assume 4 threads
    let slice_16 = unsafe{ slice::from_raw_parts_mut(points as *mut [Float4;4], n/16) };
    parallel_for(slice_16, threads, &|v| run_gd_16(v, m, &p));

    let slice_4 = unsafe{ slice::from_raw_parts_mut((points.add(n/16*16)) as *mut Float4, (n%16)/4) };
    parallel_for(slice_4, threads, &|v| run_gd_4(v, m, &p));
    
    let slice_1 = unsafe{ slice::from_raw_parts_mut((points.add(n/4*4)) as *mut f32, n%4) };
    parallel_for(slice_1, threads, &|v| run_gd(v, m, &p));
}

pub fn run_gd(px: &mut f32, rounds: u32, params: &[f32;4]) {
    let [a,b,c,d] = *params;
    let mut x = *px;
    for _ in 0..rounds {
        let (x2, ax) = (x*x, a*x);
        x -= (ax*x2+b*x2)+(c*x+d);
    }
    *px = x;
}

pub fn run_gd_4(px: &mut Float4, rounds: u32, params: &[f32;4]) {
    let [a, b, c, d] = params.map(|v| Float4::splat(v));
    let mut x = *px;
    for _ in 0..rounds {
        let (x2, ax) = (x*x, a*x);
        x -= (ax*x2+b*x2)+(c*x+d);
    }
    *px = x;
}

pub fn run_gd_16(px: &mut [Float4;4], rounds: u32, params: &[f32;4]) {
    let [a, b, c, d] = params.map(|v| Float4::splat(v));
    let mut x = *px;
    for _ in 0..rounds {
        for i in 0..4 {
            let (x2, ax) = (x[i]*x[i], a*x[i]);
            x[i] -= (ax*x2+b*x2)+(c*x[i]+d);
        }
    }
    *px = x;
}
