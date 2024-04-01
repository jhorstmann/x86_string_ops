use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::ops::Range;
use x86_strings_ops::SliceExt;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const BATCH_SIZE: usize = 4096;

#[inline(never)]
fn bench_inline_copy(dst: &mut [u8], src: &[u8], ranges: &[Range<usize>]) {
    assert_eq!(dst.len(), src.len());
    for range in ranges {
        dst[range.clone()].inline_copy_from(&src[range.clone()]);
    }
}

#[inline(never)]
fn bench_memcpy_default(dst: &mut [u8], src: &[u8], ranges: &[Range<usize>]) {
    assert_eq!(dst.len(), src.len());
    for range in ranges {
        dst[range.clone()].copy_from_slice(&src[range.clone()]);
    }
}

#[inline(never)]
fn bench_memcpy_simple(dst: &mut [u8], src: &[u8], ranges: &[Range<usize>]) {
    assert_eq!(dst.len(), src.len());
    for range in ranges {
        memcpy_simple(&mut dst[range.clone()], &src[range.clone()]);
    }
}

fn memcpy_simple(dst: &mut [u8], src: &[u8]) {
    let mut len = dst.len();
    assert_eq!(len, src.len());
    let mut dst = dst.as_mut_ptr();
    let mut src = src.as_ptr();
    unsafe {
        while len >= 8 {
            dst.cast::<[u8; 8]>()
                .write_volatile(src.cast::<[u8; 8]>().read());
            dst = dst.add(8);
            src = src.add(8);
            len -= 8;
        }
        if len >= 4 {
            dst.cast::<[u8; 4]>()
                .write_volatile(src.cast::<[u8; 4]>().read());
            dst = dst.add(4);
            src = src.add(4);
            len -= 4;
        }
        if len >= 2 {
            dst.cast::<[u8; 2]>()
                .write_volatile(src.cast::<[u8; 2]>().read());
            dst = dst.add(2);
            src = src.add(2);
            len -= 2;
        }
        if len >= 1 {
            dst.write_volatile(src.read());
        }
    }
}

fn bench_slice(c: &mut Criterion, rng: &mut StdRng, len_range: Range<usize>, name: &str) {
    let mut dst = vec![0_u8; (16 * 1024).max(len_range.end)];
    let src = vec![0_u8; dst.len()];
    let ranges = (0..BATCH_SIZE)
        .map(|_| {
            let len = rng.gen_range(len_range.clone());
            assert!(len <= src.len());
            assert!(len <= dst.len());
            let start = rng.gen_range(0..src.len() - len);
            let end = start + len;
            Range { start, end }
        })
        .collect::<Vec<Range<usize>>>();
    let bytes = ranges.iter().map(|r| r.len()).sum::<usize>() as u64;
    c.benchmark_group(name)
        .throughput(Throughput::Bytes(bytes))
        .bench_function("inline", |b| {
            b.iter(|| bench_inline_copy(&mut dst, &src, &ranges))
        })
        .bench_function("default", |b| {
            b.iter(|| bench_memcpy_default(&mut dst, &src, &ranges))
        })
        .bench_function("simple", |b| {
            b.iter(|| bench_memcpy_simple(&mut dst, &src, &ranges))
        });
}

fn fixed(len: usize) -> Range<usize> {
    len..len.checked_add(1).unwrap()
}

pub fn bench_copy(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);

    bench_slice(c, &mut rng, fixed(16), "len_16");
    bench_slice(c, &mut rng, fixed(64), "len_64");
    bench_slice(c, &mut rng, fixed(512), "len_512");
    bench_slice(c, &mut rng, fixed(2048), "len_2k");
    bench_slice(c, &mut rng, fixed(8192), "len_8k");

    bench_slice(c, &mut rng, 1..16, "len_1_to_16");
    bench_slice(c, &mut rng, 4..64, "len_4_to_64");
    bench_slice(c, &mut rng, 16..512, "len_16_to_512");
}

criterion_group!(benches, bench_copy);
criterion_main!(benches);
