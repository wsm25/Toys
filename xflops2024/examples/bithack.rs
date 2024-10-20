
const FIB_SIZE:usize = 53;
const FIBS: [usize; FIB_SIZE] = [
    1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 
    17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040, 1346269, 2178309, 3524578, 
    5702887, 9227465, 14930352, 24157817, 39088169, 63245986, 102334155, 165580141, 267914296, 
    433494437, 701408733, 1134903170, 1836311903, 2971215073, 4807526976, 7778742049, 12586269025, 
    20365011074, 32951280099, 53316291173, 86267571272
];

const TIME_LIMIT:usize = 1000;

const ASNI_COLOR_GREEN: &str = "\x1B[0;32m";
const ASNI_COLOR_RED: &str = "\x1B[0;31m";
const ASNI_COLOR_RESET: &str = "\x1B[0m";

extern crate xflops2024;
use xflops2024::bithack::BitSlice;
fn main() {
    let mut tier = 0;
    while tier < FIB_SIZE-3 {
        let [offset, n, len, bits] = FIBS[tier..tier+4].try_into().unwrap();
        let mut bs = unsafe{BitSlice::uninit(bits)};
        let start = std::time::Instant::now();
        bs.rotate_left(offset, len, n);
        let dur = start.elapsed();
        if (dur.as_millis() as usize) < TIME_LIMIT {
            println!("Tier {tier} completed in {ASNI_COLOR_GREEN}{dur:?}{ASNI_COLOR_RESET}");
        } else {
            println!("Tier {tier} completed in {ASNI_COLOR_RED}{dur:?}{ASNI_COLOR_RESET}");
            println!("Succesfully completed tier: {ASNI_COLOR_GREEN}{}{ASNI_COLOR_RESET}", tier-1);
            break;
        }
        tier+=1;
    }
}
