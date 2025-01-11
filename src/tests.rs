extern crate test;

use test::{Bencher, black_box};
use rand::Rng;
use std::f32::consts::{TAU, PI};
use fastapprox::{fast, faster};

use crate::FreqData;


#[bench]
fn bench_none(b: &mut Bencher) {
    let mut samples = vec![0.; 48000];
    let sample_rate = 48000;
    let mut freqs = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let f = rng.gen_range(0.0..24000.0);
        let amp = rng.gen_range(0.0..1.0);
        let ph = rng.gen_range(-TAU..TAU);
        freqs.push(FreqData::new(f, amp, ph));
    }

    b.iter(|| {
        for i in 0..samples.len() {
            let time = i as f32 / sample_rate as f32;
            for f in &freqs {
                let angle = (TAU * time * f.frequency as f32) + f.phase;
                //angle = ((angle + PI) % TAU) + PI;
                black_box(samples[i] += angle * f.amplitude);
            }
        }
    });
}


#[bench]
fn bench_cos(b: &mut Bencher) {
    let mut samples = vec![0.; 48000];
    let sample_rate = 48000;
    let mut freqs = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let f = rng.gen_range(0.0..24000.0);
        let amp = rng.gen_range(0.0..1.0);
        let ph = rng.gen_range(-TAU..TAU);
        freqs.push(FreqData::new(f, amp, ph));
    }

    b.iter(|| {
        for i in 0..samples.len() {
            let time = i as f32 / sample_rate as f32;
            for f in &freqs {
                let mut angle = (TAU * time * f.frequency as f32) + f.phase;
                angle = ((angle + PI) % TAU) + PI;
                black_box(samples[i] += f32::cos(angle) * f.amplitude);
            }
        }
    });
}

#[bench]
fn bench_fast_cos(b: &mut Bencher) {
    let mut samples = vec![0.; 48000];
    let sample_rate = 48000;
    let mut freqs = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let f = rng.gen_range(0.0..24000.0);
        let amp = rng.gen_range(0.0..1.0);
        let ph = rng.gen_range(-TAU..TAU);
        freqs.push(FreqData::new(f, amp, ph));
    }

    b.iter(|| {
        for i in 0..samples.len() {
            let time = i as f32 / sample_rate as f32;
            for f in &freqs {
                let mut angle = (TAU * time * f.frequency as f32) + f.phase;
                angle = ((angle + PI) % TAU) + PI;
                black_box(samples[i] += fast::cos(angle) * f.amplitude);
            }
        }
    });
}

#[bench]
fn bench_faster_cos(b: &mut Bencher) {
    let mut samples = vec![0.; 48000];
    let sample_rate = 48000;
    let mut freqs = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let f = rng.gen_range(0.0..24000.0);
        let amp = rng.gen_range(0.0..1.0);
        let ph = rng.gen_range(-TAU..TAU);
        freqs.push(FreqData::new(f, amp, ph));
    }

    b.iter(|| {
        for i in 0..samples.len() {
            let time = i as f32 / sample_rate as f32;
            for f in &freqs {
                let mut angle = (TAU * time * f.frequency as f32) + f.phase;
                angle = ((angle + PI) % TAU) + PI;
                black_box(samples[i] += faster::cos(angle) * f.amplitude);
            }
        }
    });
}
