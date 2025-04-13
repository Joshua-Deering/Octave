use std::{hint::black_box, time::Duration};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

extern crate octave;
use octave::{file_analyzer::{calculate_true_peak, upsample}, file_io::WavInfo};

fn generate_sine_wave(frequency: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let sample = (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin();
        samples.push(sample);
    }
    samples
}

fn bench_upsample_sine(c: &mut Criterion) {
    let samples_src = vec![generate_sine_wave(16000., 48000, 48000*5); 2];
    c.bench_function("upsample 48kHz sine wave", |b| {
        b.iter_batched(
            || samples_src.clone(),
            |samples| black_box(upsample(samples, 48000)),
            BatchSize::LargeInput
        )
    });
}

fn bench_upsample_sine_long(c: &mut Criterion) {
    let long_samples = vec![generate_sine_wave(16000., 48000, 48000*150)];
    c.bench_function("upsample 48kHz sine wave long", |b| {
        b.iter_batched(
            || long_samples.clone(),
            |samples| black_box(upsample(samples, 48000)),
            BatchSize::LargeInput
        )
    });
}

fn bench_calc_true_peak(c: &mut Criterion) {
    let samples_src = vec![generate_sine_wave(16000., 48000, 48000*5); 2];
    let meta = WavInfo {
        sample_rate: 48000,
        ..Default::default()
    };

    c.bench_function("true peak 48kHz sine wave", |b| {
        b.iter(
            || black_box(calculate_true_peak(&samples_src, &meta)),
        )
    });
}

fn bench_calc_true_peak_long(c: &mut Criterion) {
    let samples_src = vec![generate_sine_wave(16000., 48000, 48000*150); 2];
    let meta = WavInfo {
        sample_rate: 48000,
        ..Default::default()
    };

    c.bench_function("true peak 48kHz sine wave long", |b| {
        b.iter(
            || black_box(calculate_true_peak(&samples_src, &meta)),
        )
    });
}

criterion_group!(benches, bench_upsample_sine, bench_calc_true_peak);
criterion_group!{
    name = long_benches;
    config = Criterion::default().measurement_time(Duration::from_secs(20));
    targets = bench_upsample_sine_long, bench_calc_true_peak_long
}
criterion_main!(benches, long_benches);
