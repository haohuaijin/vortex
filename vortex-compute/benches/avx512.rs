// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

#![expect(clippy::cast_possible_truncation)]

use itertools::Itertools;
use rand::Rng;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use vortex_compute::filter::slice::in_place::avx512::filter_in_place_avx512;
use vortex_compute::filter::slice::in_place::filter_in_place_scalar;

fn main() {
    divan::main();
}

// Create a random mask where each bit has `probability` chance of being set.
fn create_random_mask(size: usize, probability: f64) -> Vec<u8> {
    let mut rng = rand::rng();
    let num_bytes = size.div_ceil(8);
    let mut mask = Vec::with_capacity(num_bytes);

    for _ in 0..num_bytes {
        let mut byte = 0u8;
        for bit in 0..8 {
            if rng.random::<f64>() < probability {
                byte |= 1 << bit;
            }
        }
        mask.push(byte);
    }

    mask
}

// Benchmark different data sizes.
const SIZES: &[usize] = &[1 << 10, 1 << 14, 1 << 17];

// Different probability values to benchmark.
const PROBABILITIES: &[f64] = &[0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];

#[divan::bench(sample_size = 64, args = SIZES.iter().copied().cartesian_product(PROBABILITIES.iter().copied()))]
fn random_probability_scalar(bencher: divan::Bencher, (size, probability): (usize, f64)) {
    let mask = create_random_mask(size, probability);
    bencher
        .with_inputs(|| (0..size as i32).collect::<Vec<_>>())
        .bench_values(|mut data| filter_in_place_scalar(&mut data, &mask))
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[divan::bench(sample_size = 64, args = SIZES.iter().copied().cartesian_product(PROBABILITIES.iter().copied()))]
fn random_probability_avx512(bencher: divan::Bencher, (size, probability): (usize, f64)) {
    let mask = create_random_mask(size, probability);
    bencher
        .with_inputs(|| (0..size as i32).collect::<Vec<_>>())
        .bench_values(|mut data| unsafe { filter_in_place_avx512(&mut data, &mask) })
}

const LARGE_SIZE: usize = 1024 * 1024; // 4 MB

#[divan::bench(sample_size = 16, args = PROBABILITIES)]
fn scalar_throughput(bencher: divan::Bencher, probability: f64) {
    let mask = create_random_mask(LARGE_SIZE, probability);
    bencher
        .counter(divan::counter::BytesCount::new(LARGE_SIZE * 4))
        .with_inputs(|| (0..LARGE_SIZE as i32).collect::<Vec<_>>())
        .bench_values(|mut data| filter_in_place_scalar(&mut data, &mask))
}

#[divan::bench(sample_size = 16, args = PROBABILITIES)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn avx512_throughput(bencher: divan::Bencher, probability: f64) {
    let mask = create_random_mask(LARGE_SIZE, probability);
    bencher
        .counter(divan::counter::BytesCount::new(LARGE_SIZE * 4))
        .with_inputs(|| (0..LARGE_SIZE as i32).collect::<Vec<_>>())
        .bench_values(|mut data| unsafe { filter_in_place_avx512(&mut data, &mask) })
}
