use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::ops::Range;
use x86_strings_ops::SliceExt;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const BATCH_SIZE: usize = 4096;

#[inline(never)]
fn bench_inline_fill(buffer: &mut [u8], ranges: &[Range<usize>], value: u8) {
    for range in ranges {
        buffer[range.clone()].inline_fill(value);
    }
}

#[inline(never)]
fn bench_memset(buffer: &mut [u8], ranges: &[Range<usize>], value: u8) {
    for range in ranges {
        buffer[range.clone()].fill(value);
    }
}

fn bench_fill_ranges(c: &mut Criterion, rng: &mut StdRng, len_range: Range<usize>, name: &str) {
    let mut buffer = vec![0_u8; (16 * 1024).max(len_range.end)];
    let ranges = (0..BATCH_SIZE)
        .map(|_| {
            let len = rng.gen_range(len_range.clone());
            assert!(len <= buffer.len());
            let start = rng.gen_range(0..buffer.len() - len);
            let end = start + len;
            Range { start, end }
        })
        .collect::<Vec<Range<usize>>>();
    let bytes = ranges.iter().map(|r| r.len()).sum::<usize>() as u64;
    let value = black_box(42_u8);

    c.benchmark_group(name)
        .throughput(Throughput::Bytes(bytes))
        .bench_function("inline_fill", |b| {
            b.iter(|| bench_inline_fill(&mut buffer, &ranges, value))
        })
        .bench_function("memset", |b| {
            b.iter(|| bench_memset(&mut buffer, &ranges, value))
        });
}

fn fixed(len: usize) -> Range<usize> {
    len..len.checked_add(1).unwrap()
}

pub fn bench_fill(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);

    bench_fill_ranges(c, &mut rng, fixed(16), "len_16");
    bench_fill_ranges(c, &mut rng, fixed(64), "len_64");
    bench_fill_ranges(c, &mut rng, fixed(512), "len_512");
    bench_fill_ranges(c, &mut rng, fixed(2048), "len_2k");
    bench_fill_ranges(c, &mut rng, fixed(8192), "len_8k");

    bench_fill_ranges(c, &mut rng, 4..64, "len_1_to_16");
    bench_fill_ranges(c, &mut rng, 4..64, "len_4_to_64");
    bench_fill_ranges(c, &mut rng, 16..512, "len_16_to_512");
}

criterion_group!(benches, bench_fill);
criterion_main!(benches);
