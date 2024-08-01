#![allow(non_camel_case_types)]
use core::arch::x86_64::*;
use std::mem::*;

pub const ZERO:      __m256i = unsafe { transmute([0b00000000_u8; 32]) };
pub const TOP_BIT:   __m256i = unsafe { transmute([0b10000000_u8; 32]) };
pub const BITS_23:   __m256i = unsafe { transmute([0b01100000_u8; 32]) };
pub const ASCII_END: __m256i = unsafe { transmute([0b01111111_u8; 32]) };

const VECTOR_SIZE: usize = std::mem::size_of::<__m256i>();

#[no_mangle]
#[inline(never)]
pub fn find_ascii_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p.add(VECTOR_SIZE) < range.end {
            let v = _mm256_loadu_si256(p as _);

            let step1 = _mm256_andnot_si256(v, TOP_BIT);
            let step1 = _mm256_cmpgt_epi8(ZERO, step1);

            let step2 = _mm256_and_si256(v, BITS_23);
            let step2 = _mm256_cmpgt_epi8(step2, ZERO);

            let step3 = _mm256_cmpeq_epi8(v, ASCII_END);
            let step3 = _mm256_cmpeq_epi8(step3, ZERO);

            let step1_2 = _mm256_and_si256(step1, step2);

            let mask = _mm256_and_si256(step1_2, step3);
            // inspect!(mask);
            let mask = _mm256_movemask_epi8(mask);
            // inspect!(mask);
            let offset = mask.trailing_zeros();
            // println!("leading zeros: {offset}");
            // println!("trailing zeros: {}", mask.trailing_zeros());

            if mask != 0 {
                // dbg!((p.offset_from(range.start), offset));
                return p.add(offset as _).offset_from(range.start) as _;
            }

            p = p.add(VECTOR_SIZE);
        }

        while p < range.end {
            if *p >= b' ' && *p <= b'~' {
                return p.offset_from(range.start) as _;
            }

            p = p.add(1);
        }
    }

    data.len()
}

#[no_mangle]
#[inline(never)]
pub fn find_ascii_non_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p < range.end {
            if *p >= b' ' && *p <= b'~' {
                return p.offset_from(range.start) as _;
            }

            p = p.add(1);
        }
    }

    data.len()
}

#[no_mangle]
#[inline(never)]
pub fn find_ascii_non_simd_unrolled(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p.add(VECTOR_SIZE) < range.end {
            if *p         >= b' ' && *p         <= b'~' { return p        .offset_from(range.start) as _; }
            if *p.add( 1) >= b' ' && *p.add( 1) <= b'~' { return p.add( 1).offset_from(range.start) as _; }
            if *p.add( 2) >= b' ' && *p.add( 2) <= b'~' { return p.add( 2).offset_from(range.start) as _; }
            if *p.add( 3) >= b' ' && *p.add( 3) <= b'~' { return p.add( 3).offset_from(range.start) as _; }
            if *p.add( 4) >= b' ' && *p.add( 4) <= b'~' { return p.add( 4).offset_from(range.start) as _; }
            if *p.add( 5) >= b' ' && *p.add( 5) <= b'~' { return p.add( 5).offset_from(range.start) as _; }
            if *p.add( 6) >= b' ' && *p.add( 6) <= b'~' { return p.add( 6).offset_from(range.start) as _; }
            if *p.add( 7) >= b' ' && *p.add( 7) <= b'~' { return p.add( 7).offset_from(range.start) as _; }
            if *p.add( 8) >= b' ' && *p.add( 8) <= b'~' { return p.add( 8).offset_from(range.start) as _; }
            if *p.add( 9) >= b' ' && *p.add( 9) <= b'~' { return p.add( 9).offset_from(range.start) as _; }
            if *p.add(10) >= b' ' && *p.add(10) <= b'~' { return p.add(10).offset_from(range.start) as _; }
            if *p.add(11) >= b' ' && *p.add(11) <= b'~' { return p.add(11).offset_from(range.start) as _; }
            if *p.add(12) >= b' ' && *p.add(12) <= b'~' { return p.add(12).offset_from(range.start) as _; }
            if *p.add(13) >= b' ' && *p.add(13) <= b'~' { return p.add(13).offset_from(range.start) as _; }
            if *p.add(14) >= b' ' && *p.add(14) <= b'~' { return p.add(14).offset_from(range.start) as _; }
            if *p.add(15) >= b' ' && *p.add(15) <= b'~' { return p.add(15).offset_from(range.start) as _; }
            if *p.add(16) >= b' ' && *p.add(16) <= b'~' { return p.add(16).offset_from(range.start) as _; }
            if *p.add(17) >= b' ' && *p.add(17) <= b'~' { return p.add(17).offset_from(range.start) as _; }
            if *p.add(18) >= b' ' && *p.add(18) <= b'~' { return p.add(18).offset_from(range.start) as _; }
            if *p.add(19) >= b' ' && *p.add(19) <= b'~' { return p.add(19).offset_from(range.start) as _; }
            if *p.add(20) >= b' ' && *p.add(20) <= b'~' { return p.add(20).offset_from(range.start) as _; }
            if *p.add(21) >= b' ' && *p.add(21) <= b'~' { return p.add(21).offset_from(range.start) as _; }
            if *p.add(22) >= b' ' && *p.add(22) <= b'~' { return p.add(22).offset_from(range.start) as _; }
            if *p.add(23) >= b' ' && *p.add(23) <= b'~' { return p.add(23).offset_from(range.start) as _; }
            if *p.add(24) >= b' ' && *p.add(24) <= b'~' { return p.add(24).offset_from(range.start) as _; }
            if *p.add(25) >= b' ' && *p.add(25) <= b'~' { return p.add(25).offset_from(range.start) as _; }
            if *p.add(26) >= b' ' && *p.add(26) <= b'~' { return p.add(26).offset_from(range.start) as _; }
            if *p.add(27) >= b' ' && *p.add(27) <= b'~' { return p.add(27).offset_from(range.start) as _; }
            if *p.add(28) >= b' ' && *p.add(28) <= b'~' { return p.add(28).offset_from(range.start) as _; }
            if *p.add(29) >= b' ' && *p.add(29) <= b'~' { return p.add(29).offset_from(range.start) as _; }
            if *p.add(30) >= b' ' && *p.add(30) <= b'~' { return p.add(30).offset_from(range.start) as _; }
            if *p.add(31) >= b' ' && *p.add(31) <= b'~' { return p.add(31).offset_from(range.start) as _; }
            p = p.add(VECTOR_SIZE);
        }

        while p < range.end {
            if *p >= b' ' && *p <= b'~' {
                return p.offset_from(range.start) as _;
            }

            p = p.add(1);
        }
    }

    data.len()
}
