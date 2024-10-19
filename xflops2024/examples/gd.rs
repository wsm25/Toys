use std::{io::{BufReader, Read, BufRead}, path::Path, time::{Duration, Instant}};

fn main(){
    let tests = [
        ("gddata/conf0.data", "gddata/ref0.data"),
        ("gddata/conf1.data", "gddata/ref1.data"),
        ("gddata/conf2.data", "gddata/ref2.data"),
        ("gddata/conf3.data", "gddata/ref3.data"),
        ("gddata/conf4.data", "gddata/ref4.data"),
    ];
    for i in 0..tests.len() {
        println!("Case {i}: {:?}", gd(tests[i].0, tests[i].1));
    }
}

extern crate xflops2024;

fn gd<P: AsRef<Path>>(input: P, refer: P)->Duration {
    let mut s = String::new();
    let mut infile = std::fs::File::open(input).unwrap();
    infile.read_to_string(&mut s).unwrap();
    drop(infile);
    let mut s = s.split_ascii_whitespace();

    let a:f32 = s.next().unwrap().parse().unwrap();
    let b:f32 = s.next().unwrap().parse().unwrap();
    let n:u32 = s.next().unwrap().parse().unwrap();
    let m:u32 = s.next().unwrap().parse().unwrap();
    let eta:f32 = s.next().unwrap().parse().unwrap();
    let params: [f32;4] = core::array::from_fn(|_| s.next().unwrap().parse().unwrap());
    
    println!("Search params: {a} {b} {n} {m} {eta}");
    println!("Function params: {params:?}");

    let mut points = Vec::with_capacity(n as usize);
    unsafe{points.set_len(n as usize);}
    let interval = (b - a) / (n-1) as f32;
    for i in 0..n as usize {
        points[i] = a + i as f32 * interval;
    }
    let start = Instant::now();
    core::hint::black_box(
        xflops2024::gd::gradient_descent(points.as_mut_ptr(), n, m, eta, &params)
    );
    let dur = start.elapsed();

    let reffile = std::fs::File::open(refer).unwrap();
    let mut reffile = BufReader::new(reffile);

    let mut line = String::new();
    for v in &points {
        line.clear();
        reffile.read_line(&mut line).unwrap();
        let l = line.trim();
        let value:f32 = l.parse().unwrap_or_else(|_|{
            panic!("Parse reference error: {l:?}")
        });
        assert!((*v-value).abs()<1e-6);
    }
    dur
}