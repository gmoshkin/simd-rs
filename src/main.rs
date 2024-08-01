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
    }
}

#[inline(always)]
fn as_slice<T>(v: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}

unsafe fn run() {
    // dbg!(std::mem::size_of::<__m256i>());
    // dbg!(std::mem::size_of::<__m512i>());
    // dbg!(core::arch::x86_64::_mm256_cmpeq_epi8(1.into(), 1.into()));
    // let all_bytes_zero = core::arch::x86_64::_mm256_setzero_si256();
    // println!("{all_bytes_zero:?}");

    println!("first ascii: ' ' {:02x}", b' ');
    println!(" last ascii: '~' {:02x}", b'~');

    let mut all_bytes = Vec::with_capacity(256);
    for c in 0..256 {
        all_bytes.push(c as u8);
    }

    let bytes_0_31 = _mm256_loadu_si256(all_bytes[0..32].as_ptr().cast());
    println!("{}", inspect(&bytes_0_31));
    let bytes_32_63 = _mm256_loadu_si256(all_bytes[32..64].as_ptr().cast());
    println!("{}", inspect(&bytes_32_63));

    let bytes_64_95 = _mm256_loadu_si256(all_bytes[64..96].as_ptr().cast());
    println!("{}", inspect(&bytes_64_95));
    let bytes_96_127 = _mm256_loadu_si256(all_bytes[96..128].as_ptr().cast());
    println!("{}", inspect(&bytes_96_127));

    let bytes_128_159 = _mm256_loadu_si256(all_bytes[128..160].as_ptr().cast());
    println!("{}", inspect(&bytes_128_159));
    let bytes_160_191 = _mm256_loadu_si256(all_bytes[160..192].as_ptr().cast());
    println!("{}", inspect(&bytes_160_191));

    let bytes_192_223 = _mm256_loadu_si256(all_bytes[192..224].as_ptr().cast());
    println!("{}", inspect(&bytes_192_223));
    let bytes_224_255 = _mm256_loadu_si256(all_bytes[224..256].as_ptr().cast());
    println!("{}", inspect(&bytes_224_255));

    let byte_ranges = [ bytes_0_31, bytes_32_63, bytes_64_95, bytes_96_127, bytes_128_159, bytes_160_191, bytes_192_223, bytes_224_255, ];

    inspect!(ZERO);
    inspect!(TOP_BIT);
    inspect!(BITS_23);
    inspect!(ASCII_END);

    println!("================================================================================================");


    // println!("------------------------------------------------------------------------------------------------");

    // for bytes in byte_ranges {
    //     let step1 = _mm256_andnot_si256(bytes, top_bit);
    //     // No unsigned greater than???
    //     let step1 = _mm256_cmpgt_epi8(ZERO, step1);
    //     println!("{}", inspect(&step1));
    // }

    // println!("------------------------------------------------------------------------------------------------");

    // for bytes in byte_ranges {
    //     let step2 = _mm256_and_si256(bytes, BITS_23);
    //     let step2 = _mm256_cmpgt_epi8(step2, ZERO);
    //     println!("{}", inspect(&step2));
    // }

    // println!("------------------------------------------------------------------------------------------------");

    // for bytes in byte_ranges {
    //     let step3 = _mm256_cmpeq_epi8(bytes, ASCII_END);
    //     let step3 = _mm256_cmpeq_epi8(step3, ZERO);
    //     println!("{}", inspect(&step3));
    // }

    println!("================================================================================================");
    println!("step 1 & 2 & 3");

    println!("------------------------------------------------------------------------------------------------");

    let mut masks: [__m256i; 8] = unsafe { std::mem::zeroed() };
    for (&bytes, i) in byte_ranges.iter().zip(0..) {
        let step1 = _mm256_andnot_si256(bytes, TOP_BIT);
        let step1 = _mm256_cmpgt_epi8(ZERO, step1);

        let step2 = _mm256_and_si256(bytes, BITS_23);
        let step2 = _mm256_cmpgt_epi8(step2, ZERO);

        let step3 = _mm256_cmpeq_epi8(bytes, ASCII_END);
        let step3 = _mm256_cmpeq_epi8(step3, ZERO);

        let step1_2 = _mm256_and_si256(step1, step2);
        let step1_2_3 = _mm256_and_si256(step1_2, step3);

        masks[i] = step1_2_3;

        println!("{}", inspect(&step1_2_3));
    }

    for mask in masks {
        let mask = _mm256_movemask_epi8(mask);
        println!("{}", mask.leading_zeros());
        // println!("{}", inspect(&mask));
    }
}

// let zero = _mm256_set1_epi8(0b00000000 as _);
// let top_bit = _mm256_set1_epi8(0b10000000 as _);
// let bits_23 = _mm256_set1_epi8(0b01100000 as _);
// let ascii_end = _mm256_set1_epi8(0x7f as _);

fn check_find_ascii(data: &[u8]) {
    println!("------------------------------------------------------------------------------------------------");
    let i = find_ascii_simd(data);
    println!("{}", inspect_find_ascii_result(data, i));
    // for _ in 0..i { print!("    "); } println!("~~~~");
}

fn main() {
    // unsafe { run(); }
    // check();
    unsafe { perf_test(4096, 4096); }
    unsafe { perf_test(16 * 4096, 4096); }
    unsafe { perf_test(16 * 4096, 8 * 4096); }
    unsafe { perf_test(16 * 4096, 1024); }
}

fn check() {
    let mut all_bytes = Vec::with_capacity(256);
    for c in 0..256 {
        all_bytes.push(c as u8);
    }

    check_find_ascii(b"foo");
    check_find_ascii(b"\x01\x02a");
    check_find_ascii(b"\x0f\x10\x12a");
    check_find_ascii(b"\x1e\x1f a");
    check_find_ascii(&all_bytes);
    check_find_ascii(&all_bytes[0x1e..]);
    check_find_ascii(&all_bytes[0x1f..]);
    check_find_ascii(&all_bytes[0x20..]);
    check_find_ascii(&all_bytes[0x7d..]);
}

struct inspect_find_ascii_result<'a>(&'a [u8], usize);
impl std::fmt::Display for inspect_find_ascii_result<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (byte, i) in self.0.iter().copied().zip(0..) {
            if i == self.1 { f.write_str("\x1b[7m")?; }
            if matches!(byte, b' '..=b'~') {
                f.write_str("   ")?;
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
