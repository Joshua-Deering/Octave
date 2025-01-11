use std::{f32::consts::{PI, TAU}, ops::{AddAssign, Div, DivAssign, Mul}, thread};
use fastapprox::fast;
use cpal::Data;

pub fn generate_multichannel_signal(freq_data: &Vec<Vec<FreqData>>, sample_rate: usize, duration: f32) -> Vec<Vec<f32>> {
    let mut output = vec![vec![]; freq_data.len()];
    for i in 0..freq_data.len() {
        output[i] = generate_signal(&freq_data[i], sample_rate, duration);
    }
    output
}

pub fn generate_signal(freq_data: &Vec<FreqData>, sample_rate: usize, duration: f32) -> Vec<f32> {
    let mut samples = vec![0.; (sample_rate as f32 * duration) as usize];
    for i in 0..samples.len() {
        let time = i as f32 / sample_rate as f32;
        for f in freq_data {
            let mut angle = (TAU * time * f.frequency as f32) + f.phase;
            angle = ((angle + PI) % TAU) - PI;
            samples[i] += fast::cos(angle) * f.amplitude;
        }
    }
    samples
}


pub struct SignalPlayer {
    pub samples: Vec<Vec<f32>>,
    pub sample_rate: u32,
    channels: usize,
    duration: f32,
    pos: usize,
}

impl SignalPlayer {
    pub fn new(samples: Vec<Vec<f32>>, sample_rate: u32, channels: usize) -> Self {
        let duration = samples.len() as f32 / sample_rate as f32;
        Self {
            samples,
            sample_rate,
            channels,
            duration,
            pos: 0,
        }
    }
    pub fn next_chunk(&mut self, data: &mut Data) {
        let dat_slice = data.as_slice_mut().unwrap();
        let end = self.pos + (dat_slice.len() / self.channels);
        if end >= self.samples[0].len() {
            return;
        }
        for i in self.pos..end {
            for c in 0..self.channels {
                dat_slice[(i - self.pos) * self.channels + c] = self.samples[c][i];
            }
        }
        self.pos += data.len() / self.channels;
    }

    pub fn do_short_time_fourier_transform(&self, window_size: f32, overlap: f32) -> Vec<Vec<Vec<FreqData>>> {
        let mut out = vec![vec![]; self.channels];
        for i in 0..self.channels {
            out[i] = do_short_time_fourier_transform(&self.samples[i], self.sample_rate, self.duration, window_size, overlap);
        }
        out
    }
    
    pub fn do_fourier_transform(&self) -> Vec<Vec<FreqData>> {
        let mut out = vec![vec![]; self.channels];
        for i in 0..self.channels {
            out[i] = do_fourier_transform(&self.samples[i], self.sample_rate, self.duration);
        }
        out
    }

    pub fn do_fourier_transform_on_channel(&self, channel: usize) -> Vec<FreqData> {
        if channel > self.channels {
            return vec![];
        }
        do_fourier_transform(&self.samples[channel-1], self.sample_rate, self.duration)
    }
}

fn do_short_time_fourier_transform(samples: &Vec<f32>, sample_rate: u32, duration: f32, window_size: f32, overlap: f32) -> Vec<Vec<FreqData>> {
    let samples_per_window = (window_size * sample_rate as f32) as usize;
    let overlap_size = (samples_per_window as f32 * overlap) as usize;
    let step_size = samples_per_window - overlap_size;
    let num_windows = samples.len() / (samples_per_window - overlap_size);
    let duration_per_window = duration / num_windows as f32;
    
    let mut out: Vec<Vec<FreqData>> = vec![vec![]; num_windows - 1];
    let mut window_idx = 0;
    let mut i: usize = 0;
    while i + samples_per_window < samples.len() {
        out[window_idx] = do_fourier_transform_slice(&samples[i..i+samples_per_window], sample_rate, duration_per_window);
        window_idx += 1;
        i += step_size;
    }

    out
}


fn do_fourier_transform_slice(samples: &[f32], sample_rate: u32, duration: f32) -> Vec<FreqData> {
    let num_frequencies = samples.len() / 2 + 1;
    let mut frequency_data = vec![FreqData::ZERO; num_frequencies];
    let frequency_step = sample_rate as f32 / samples.len() as f32;
    let chunk_size = num_frequencies/6;
    
    thread::scope(|s| {
        for (idx, frequencies) in frequency_data.chunks_mut(chunk_size).enumerate() {
            s.spawn(move || {
                for freq_index in idx*chunk_size..idx*chunk_size+frequencies.len() {
                    let mut sum = Vec2::ZERO;
                    for i in 0..samples.len() {
                        let mut angle = (i as f32/samples.len() as f32) * TAU * freq_index as f32;
                        angle = ((angle + PI) % TAU) - PI;
                        let test_pt = Vec2::new(fast::cos(angle), fast::sin(angle));
                        sum += test_pt * samples[i];
                    }
                    let is_nyquist_or_zero = (freq_index == num_frequencies-1 && samples.len() % 2 == 0) || freq_index == 0;

                    let sample_center = sum / samples.len();
                    frequencies[freq_index - idx*chunk_size].frequency = (freq_index as f32 * frequency_step) as f32;
                    frequencies[freq_index - idx*chunk_size].amplitude = if is_nyquist_or_zero {sample_center.magnitude()} else {sample_center.magnitude() * 2.};
                    frequencies[freq_index - idx*chunk_size].phase = -f32::atan2(sample_center.y, sample_center.x);
                }
            });
        }
    });

    frequency_data
}

fn do_fourier_transform(samples: &Vec<f32>, sample_rate: u32, duration: f32) -> Vec<FreqData> {
    let num_frequencies = samples.len() / 2 + 1;
    let mut frequency_data = vec![FreqData::ZERO; num_frequencies];
    let frequency_step = sample_rate as f32 / samples.len() as f32;
    let chunk_size = num_frequencies/6;
    
    thread::scope(|s| {
        for (idx, frequencies) in frequency_data.chunks_mut(chunk_size).enumerate() {
            s.spawn(move || {
                for freq_index in idx*chunk_size..idx*chunk_size+frequencies.len() {
                    let mut sum = Vec2::ZERO;
                    for i in 0..samples.len() {
                        let mut angle = (i as f32/samples.len() as f32) * TAU * freq_index as f32;
                        angle = ((angle + PI) % TAU) - PI;
                        let test_pt = Vec2::new(fast::cos(angle), fast::sin(angle));
                        sum += test_pt * samples[i];
                    }
                    let is_nyquist_or_zero = (freq_index == num_frequencies-1 && samples.len() % 2 == 0) || freq_index == 0;

                    let sample_center = sum / samples.len();
                    frequencies[freq_index - idx*chunk_size].frequency = (freq_index as f32 * frequency_step) as f32;
                    frequencies[freq_index - idx*chunk_size].amplitude = if is_nyquist_or_zero {sample_center.magnitude()} else {sample_center.magnitude() * 2.};
                    frequencies[freq_index - idx*chunk_size].phase = -f32::atan2(sample_center.y, sample_center.x);
                }
            });
        }
    });

    frequency_data
}

#[derive(Clone, Debug)]
pub struct FreqData {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl FreqData {
    const ZERO: Self = FreqData { frequency: 0., amplitude: 0., phase: 0. };
    pub fn new(frequency: f32, amplitude: f32, phase: f32) -> Self {
        Self { frequency, amplitude, phase }
    }
}

struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Vec2 {
    const ZERO: Self = Vec2 { x: 0., y: 0. };
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn magnitude(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        return Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Div<usize> for Vec2 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        return Self { x: self.x / rhs as f32, y: self.y / rhs as f32}
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
