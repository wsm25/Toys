//! Fast Fourier Transform implementation
//!
//! Main formula: X(k)_N,X(k+N/2)_N = ((1,W^k_N),(1,-W^k_N)) (X(k)^{even}_{N/2},X(k)^{odd}_{N/2})
//! Storage: (X(0)^{even}_{N/2}, X(1)^{e}_{N/2}...X(N/2-1)^{e}_{N/2}, X(0)^{odd}_{N/2}...X(N/2-1)^{o}_{N/2})
//!
//! Note: both fft and ifft do not normalize, like [rustfft](https://docs.rs/rustfft/latest/rustfft/#normalization).
//!
//! This implement is about 4x slower than rustfft.

use num_complex::Complex;
use num_traits::{Float, NumCast, One};

pub fn fft_impl<T: Float>(b: &mut [Complex<T>], forward: bool) {
    let n = b.len();
    if n <= 1 {
        return;
    }
    assert!(n.is_power_of_two());

    // permutation i <-> rev(i) is reversible
    let shift_bits = n.leading_zeros() + 1;
    for i in 1..n {
        let j = i.reverse_bits() >> shift_bits;
        // SAFETY: math
        unsafe { core::hint::assert_unchecked(j < n) };
        if i < j {
            b.swap(i, j);
        }
    }

    // redouble
    let mut chunk = 2;
    while chunk <= n {
        let mut arg = <T as NumCast>::from(core::f64::consts::PI * 2.0 / (chunk as f64)).unwrap();
        if forward {
            arg = arg.neg();
        }
        let w_n = Complex::from_polar(T::one(), arg);
        for ch in b.chunks_exact_mut(chunk) {
            // merge
            let mut w_kn: Complex<T> = Complex::one();
            for i in 0..(chunk / 2) {
                let j = i + chunk / 2;
                (ch[i], ch[j]) = (ch[i] + w_kn * ch[j], ch[i] - w_kn * ch[j]);
                w_kn = w_kn * w_n;
            }
        }
        chunk *= 2;
    }
}

pub fn fft<T: Float>(b: &mut [Complex<T>]) {
    fft_impl(b, true);
}

pub fn ifft<T: Float>(b: &mut [Complex<T>]) {
    fft_impl(b, false);
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::{Rng, thread_rng};
    use rustfft::{Fft, FftPlanner, algorithm::Radix4};

    fn generate_random_signal(n: usize) -> Vec<Complex<f64>> {
        let mut rng = thread_rng();
        (0..n)
            .map(|_| Complex::new(rng.r#gen::<f64>(), rng.r#gen::<f64>()))
            .collect()
    }

    #[test]
    fn test_fft() {
        let sizes = [2, 8, 1024];

        for &size in &sizes {
            let signal = generate_random_signal(size);
            let mut custom_result = signal.to_vec();
            fft(&mut custom_result);

            let mut ref_result = signal.clone();
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(signal.len());
            fft.process(&mut ref_result);

            assert_complex_eq(&custom_result, &ref_result);
        }
    }

    #[test]
    fn test_ifft() {
        let sizes = [2, 8, 1024];

        for &size in &sizes {
            let mut signal = generate_random_signal(size);
            fft(&mut signal);
            let mut custom_result = signal.clone();
            ifft(&mut custom_result);

            let mut ref_result = signal.clone();
            let planner = Radix4::new(signal.len(), rustfft::FftDirection::Inverse);
            planner.process(&mut ref_result);

            assert_complex_eq(&custom_result, &ref_result);
        }
    }

    fn assert_complex_eq<T: Float + std::fmt::Display>(d1: &[Complex<T>], d2: &[Complex<T>]) {
        assert_eq!(d1.len(), d2.len());
        let delta = <T as NumCast>::from(1e-10).unwrap();
        for (i, (custom, reference)) in d1.iter().zip(d2.iter()).enumerate() {
            assert!(
                (custom.re - reference.re).abs() < delta,
                "Real part mismatch at {}:{}/{}",
                i,
                custom.re,
                reference.re
            );
            assert!(
                (custom.im - reference.im).abs() < delta,
                "Imaginary part mismatch at {}:{}/{}",
                i,
                custom.re,
                reference.re
            );
        }
    }
}
