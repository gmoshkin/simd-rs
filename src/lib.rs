#![allow(non_camel_case_types)]
use core::arch::x86_64::*;
use std::mem::*;

pub const ASCII_END:    __m256i = unsafe { transmute([b'~' + 1; 32]) };
pub const ASCII_START:  __m256i = unsafe { transmute([b' ' - 1; 32]) };
pub const BACKSLASH:    __m256i = unsafe { transmute([b'\\'; 32]) };
pub const DOUBLE_QUOTE: __m256i = unsafe { transmute([b'"'; 32]) };

const VECTOR_SIZE: usize = std::mem::size_of::<__m256i>();

#[no_mangle]
// #[inline(never)]
pub fn find_ascii_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p.add(VECTOR_SIZE) < range.end {
            let v = _mm256_loadu_si256(p as _);

            let lower_bound = _mm256_cmpgt_epi8(v, ASCII_START);
            let upper_bound = _mm256_cmpgt_epi8(ASCII_END, v);
            let in_bounds = _mm256_and_si256(lower_bound, upper_bound);
            let mask = _mm256_movemask_epi8(in_bounds);
            let offset = mask.trailing_zeros();

            if mask != 0 {
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
pub fn find_backslash_or_double_quote_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p.add(VECTOR_SIZE) < range.end {
            let v = _mm256_loadu_si256(p as _);

            let backslash = _mm256_cmpeq_epi8(v, BACKSLASH);
            let double_quote = _mm256_cmpeq_epi8(v, DOUBLE_QUOTE);
            let either = _mm256_or_si256(backslash, double_quote);
            let mask = _mm256_movemask_epi8(either);
            let offset = mask.trailing_zeros();

            if mask != 0 {
                return p.add(offset as _).offset_from(range.start) as _;
            }

            p = p.add(VECTOR_SIZE);
        }

        while p < range.end {
            if *p == b'\\' || *p == b'"' {
                return p.offset_from(range.start) as _;
            }

            p = p.add(1);
        }
    }

    data.len()
}

#[no_mangle]
// #[inline(never)]
pub fn find_non_ascii_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut p = range.start;

    unsafe {
        while p.add(VECTOR_SIZE) < range.end {
            let v = _mm256_loadu_si256(p as _);

            let lower_bound = _mm256_cmpgt_epi8(v, ASCII_START);
            let upper_bound = _mm256_cmpgt_epi8(ASCII_END, v);
            let in_bounds = _mm256_and_si256(lower_bound, upper_bound);
            let mask = _mm256_movemask_epi8(in_bounds);
            let offset = mask.trailing_ones();

            if mask != 0xffff_ffff_u32 as i32 {
                return p.add(offset as _).offset_from(range.start) as _;
            }

            p = p.add(VECTOR_SIZE);
        }

        while p < range.end {
            if *p < b' ' || *p > b'~' {
                return p.offset_from(range.start) as _;
            }

            p = p.add(1);
        }
    }

    data.len()
}

#[no_mangle]
#[inline(never)]
pub fn to_ascii_or_hex_simd_v1(data: &[u8], out: &mut String) {
    static mut BUFFER: [u8; 4] = *b"\\x00";
    let mut tail = data;
    while !tail.is_empty() {
        let (ascii_piece, non_ascii_piece);

        let i = find_ascii_simd(tail);
        (non_ascii_piece, tail) = tail.split_at(i);
        for c in non_ascii_piece {
            out.push_str("\\x");
            out.push(HEX_DIGIT[(c >> 4) as usize] as _);
            out.push(HEX_DIGIT[(c & 0xf) as usize] as _);
        }

        if tail.is_empty() { return; }

        let i = find_non_ascii_simd(tail);
        (ascii_piece, tail) = tail.split_at(i);

        for &c in ascii_piece {
            if matches!(c, b'\\' | b'"') { out.push('\\'); }
            out.push(c as _);
        }
        // let mut ascii_tail = ascii_piece;
        // while !ascii_tail.is_empty() {
        //     let i = find_backslash_or_double_quote_simd(ascii_tail);
        //     let verbatim_piece;
        //     (verbatim_piece, ascii_tail) = ascii_tail.split_at(i);

        //     unsafe { out.push_str(std::str::from_utf8_unchecked(verbatim_piece)); }

        //     if ascii_tail.is_empty() {
        //         break;
        //     }
        //     out.push('\\');
        //     let special_char;
        //     (special_char, ascii_tail) = ascii_tail.split_at(1);
        //     out.push(special_char[0] as _);
        // }

    }
}

#[no_mangle]
#[inline(never)]
pub fn to_ascii_or_hex_simd_v2(data: &[u8], out: &mut String) {
    let dummy = String::new();
    let mut buffer = std::mem::replace(out, dummy).into_bytes();
    buffer.reserve_exact(buffer.capacity() - data.len() * 4);

    let mut tail = data;
    while !tail.is_empty() {
        let (ascii_piece, non_ascii_piece);

        let i = find_ascii_simd(tail);
        (non_ascii_piece, tail) = tail.split_at(i);
        for c in non_ascii_piece {
            buffer.extend_from_slice(b"\\x");
            buffer.push(HEX_DIGIT[(c >> 4) as usize]);
            buffer.push(HEX_DIGIT[(c & 0xf) as usize]);
            // out.push_str("\\x");
            // out.push((c >> 4) as _);
            // out.push((c & 0xf) as _);
        }

        if tail.is_empty() { break; }

        let i = find_non_ascii_simd(tail);
        (ascii_piece, tail) = tail.split_at(i);
        for &c in ascii_piece {
            if matches!(c, b'\\' | b'"') { buffer.push(b'\\'); }
            buffer.push(c);
        }
    }

    let s = unsafe { String::from_utf8_unchecked(buffer) };
    std::mem::replace(out, s);
}

const HEX_DIGIT: [u8; 16] = *b"0123456789abcdef";

#[no_mangle]
#[inline(never)]
pub fn to_ascii_or_hex(data: &[u8], out: &mut String) {
    for &c in data {
        if matches!(c, b' '..=b'~') {
            if matches!(c, b'\\' | b'"') { out.push('\\'); }
            out.push(c as _);
        } else {
            out.push_str("\\x");
            out.push(HEX_DIGIT[(c >> 4) as usize] as _);
            out.push(HEX_DIGIT[(c & 0xf) as usize] as _);
        }
    }
}

#[no_mangle]
#[inline(never)]
pub fn find_ascii_non_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut i = 0;

    unsafe {
        while i < data.len() {
            let c = data[i];
            if c >= b' ' && c <= b'~' {
                return i;
            }

            i += 1;
        }
    }

    return i;
}

#[no_mangle]
#[inline(never)]
pub fn find_non_ascii_non_simd(data: &[u8]) -> usize {
    let range = data.as_ptr_range();
    let mut i = 0;

    unsafe {
        while i < data.len() {
            let c = data[i];
            if c < b' ' || c > b'~' {
                return i;
            }

            i += 1;
        }
    }

    return i;
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
