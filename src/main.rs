// RUSTFLAGS='-C target-cpu=native' cargo run --release
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod rng;

use core::arch::x86_64::*;
use simd_rs::*;

unsafe fn perf_test(N: usize, L: usize) {
    println!("============================================================");
    println!("perf_test({N}, {L})");
    let n_bytes = N * L;
    let mut data: Vec<u8> = Vec::with_capacity(n_bytes);
    for i in 0..n_bytes / 8 {
        let n = rng::random_u64();
        data.extend_from_slice(as_slice(&n));
    }
    let get_slice = |i| &data[i * L..(i + 1) * L];

    let mut results_ascii_or_hex_simd_v1 = Vec::with_capacity(N);
    let mut results_ascii_or_hex_simd_v2 = Vec::with_capacity(N);
    let mut results_ascii_or_hex_non_simd = Vec::with_capacity(N);
    for i in 0..N {
        let slice = get_slice(i);
        results_ascii_or_hex_simd_v1.push(String::with_capacity(4 * slice.len()));
        results_ascii_or_hex_simd_v2.push(String::with_capacity(4 * slice.len()));
        results_ascii_or_hex_non_simd.push(String::with_capacity(4 * slice.len()));
    }

    for _ in 0..4 {

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            to_ascii_or_hex(slice, &mut results_ascii_or_hex_non_simd[i]);
        }
        println!("non simd: {:?}", t0.elapsed());

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            to_ascii_or_hex_simd_v1(slice, &mut results_ascii_or_hex_simd_v1[i]);
        }
        println!("simd_v1: {:?}", t0.elapsed());

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            to_ascii_or_hex_simd_v2(slice, &mut results_ascii_or_hex_simd_v2[i]);
        }
        println!("simd_v2: {:?}", t0.elapsed());

        for i in 0..N {
            assert_eq!(results_ascii_or_hex_simd_v1[i], results_ascii_or_hex_non_simd[i], "{:?}", get_slice(i));
            assert_eq!(results_ascii_or_hex_simd_v2[i], results_ascii_or_hex_non_simd[i], "{:?}", get_slice(i));

            reset_string(&mut results_ascii_or_hex_simd_v1[i]);
            reset_string(&mut results_ascii_or_hex_simd_v2[i]);
            reset_string(&mut results_ascii_or_hex_non_simd[i]);
        }
    }
}

fn reset_string(s: &mut String) {
    unsafe {
        let mut v = std::mem::take(s).into_bytes();
        v.set_len(0);
        *s = String::from_utf8_unchecked(v);
    }
}

unsafe fn perf_test_find(N: usize, L: usize) {
    println!("============================================================");
    println!("perf_test({N}, {L})");
    let n_bytes = N * L;
    let mut data: Vec<u8> = Vec::with_capacity(n_bytes);
    for i in 0..n_bytes / 8 {
        let n = rng::random_u64();
        data.extend_from_slice(as_slice(&n));
    }
    let get_slice = |i| &data[i * L..(i + 1) * L];
    let mut simd_results = vec![0_usize; N];
    let mut non_simd_results = vec![0_usize; N];
    let mut non_simd_unrolled_results = vec![0_usize; N];

    for _ in 0..4 {

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            let index = find_ascii_simd(slice);
            simd_results[i] = index;
        }
        println!("simd: {:?}", t0.elapsed());

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            let index = find_ascii_non_simd(slice);
            non_simd_results[i] = index;
        }
        println!("non simd: {:?}", t0.elapsed());

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            let index = find_ascii_non_simd_unrolled(slice);
            non_simd_unrolled_results[i] = index;
        }
        println!("non simd unrolled: {:?}", t0.elapsed());

        for i in 0..N {
            assert_eq!(simd_results[i], non_simd_results[i], "{:?}", get_slice(i));
            assert_eq!(non_simd_results[i], non_simd_unrolled_results[i], "{:?}", get_slice(i));
        }

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            let index = find_non_ascii_simd(slice);
            simd_results[i] = index;
        }
        println!("non-ascii simd: {:?}", t0.elapsed());

        let t0 = std::time::Instant::now();
        for i in 0..N {
            let slice = get_slice(i);
            let index = find_non_ascii_non_simd(slice);
            non_simd_results[i] = index;
        }
        println!("non-ascii non simd: {:?}", t0.elapsed());

        for i in 0..N {
            assert_eq!(simd_results[i], non_simd_results[i], "{:?}", get_slice(i));
        }
    }
}

#[inline(always)]
fn as_slice<T>(v: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}

fn check_find_ascii(data: &[u8]) {
    println!("------------------------------------------------------------------------------------------------");
    let i = find_ascii_simd(data);
    println!("{}", inspect_find_ascii_result(data, i));
    // for _ in 0..i { print!("    "); } println!("~~~~");
}

fn check_find_non_ascii(data: &[u8]) {
    println!("------------------------------------------------------------------------------------------------");
    let i = find_non_ascii_simd(data);
    println!("{}", inspect_find_ascii_result(data, i));
    // for _ in 0..i { print!("    "); } println!("~~~~");
}

fn main() {
    check();
    unsafe { perf_test(4096, 4096); }
    unsafe { perf_test(16 * 4096, 4096); }
    unsafe { perf_test(16 * 4096, 4 * 4096); }
    unsafe { perf_test(16 * 4096, 1024); }
}

fn check() {
    let mut all_bytes = Vec::with_capacity(256);
    for c in 0..256 {
        all_bytes.push(c as u8);
    }

    let mut s = String::with_capacity(256 * 4);
    to_ascii_or_hex_simd_v1(&all_bytes, &mut s);
    println!("{s}");

    check_find_non_ascii(b"foo");
    check_find_non_ascii(b"\x01\x02a");
    check_find_non_ascii(b"\x0f\x10\x12a");
    check_find_non_ascii(b"\x1e\x1f a");
    check_find_non_ascii(&all_bytes);
    check_find_non_ascii(&all_bytes[0x1e..]);
    check_find_non_ascii(&all_bytes[0x1f..]);
    check_find_non_ascii(&all_bytes[0x20..]);
    check_find_non_ascii(&all_bytes[0x7d..]);
}

struct inspect_find_ascii_result<'a>(&'a [u8], usize);
impl std::fmt::Display for inspect_find_ascii_result<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (byte, i) in self.0.iter().copied().zip(0..) {
            if i == self.1 { f.write_str("\x1b[7m")?; }
            if matches!(byte, b' '..=b'~') {
                write!(f, "{}", byte as char)?;
            } else {
                f.write_str("\\x")?;
                write!(f, "{byte:02x}")?;
            }
            if i == self.1 { f.write_str("\x1b[27m")?; }
        }
        Ok(())
    }
}

struct inspect<'a, T>(&'a T);
impl<T> std::fmt::Display for inspect<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.0 as *const T as *const u8;
        let count = std::mem::size_of::<T>();
        let slice = unsafe { std::slice::from_raw_parts(start, count) };
        for byte in slice {
            let low_nibble = byte >> 4;
            if low_nibble == 0 {
                f.write_str("\x1b[30m0\x1b[0m")?;
            } else {
                write!(f, "{low_nibble:x}")?;
            }

            let high_nibble = byte & 0xf;
            if high_nibble == 0 {
                f.write_str("\x1b[30m0\x1b[0m")?;
            } else {
                write!(f, "{high_nibble:x}")?;
            }
            f.write_str(" ")?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! inspect {
    ($v:expr) => {
        println!("{}:", stringify!($v));
        println!("{}", inspect(&$v));
    };
}
