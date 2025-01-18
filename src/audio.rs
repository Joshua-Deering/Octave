use std::{f32::consts::{PI, TAU}, fmt, mem::size_of_val, ops::{AddAssign, Div, DivAssign, Mul}, thread};
use fastapprox::fast;

#[allow(unused)]
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

pub struct ShortTimeDftData {
    pub dft_data: Vec<Vec<Vec<FreqData>>>,
    pub window_type: WindowFunction,
    pub overlap: f32,
    pub num_channels: u32,
    pub num_dfts: u32,
    pub num_freq: u32,
    pub sample_rate: u32,
    pub data_size: usize,
}

impl ShortTimeDftData {
    pub fn new(dft_data: Vec<Vec<Vec<FreqData>>>, window_type: WindowFunction, overlap: f32, num_channels: u32, num_dfts: u32, num_freq: u32, sample_rate: u32) -> Self {
        let data_size = (size_of_val(&dft_data[0][0][0]) as u32 * num_channels * num_dfts * num_freq) as usize + (size_of::<u32>() * 4);
        Self { dft_data, window_type, overlap, num_channels, num_dfts, num_freq, sample_rate, data_size }
    }
    pub fn new_with_size(dft_data: Vec<Vec<Vec<FreqData>>>, window_type: WindowFunction, overlap: f32, num_channels: u32, num_dfts: u32, num_freq: u32, sample_rate: u32, data_size: usize) -> Self {
        Self { dft_data, window_type, overlap, num_channels, num_dfts, num_freq, sample_rate, data_size }
    }
}

impl fmt::Display for ShortTimeDftData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("Short Time DFT Data:\nSample Rate: {} Hz\n# Channels: {}\n# DFTs: {}\n# Frequencies: {}\nTotal Data Size: {} bytes", self.sample_rate, self.num_channels, self.num_dfts, self.num_freq, self.data_size).as_str())
    }
}

pub fn do_short_time_fourier_transform(samples: &Vec<f32>, sample_rate: u32, window_size: f32, overlap: f32, window_func: WindowFunction) -> Vec<Vec<FreqData>> {
    let samples_per_window = (window_size * sample_rate as f32).round() as usize;
    let overlap_size = (samples_per_window as f32 * overlap).round() as usize;
    let step_size = samples_per_window - overlap_size;
    let num_windows = samples.len() / (samples_per_window - overlap_size);
    
    let mut out: Vec<Vec<FreqData>> = vec![vec![]; num_windows];
    let mut window_idx = 0;
    let mut i: usize = 0;
    while i + samples_per_window < samples.len() {
        out[window_idx] = do_fourier_transform_slice(&samples[i..i+samples_per_window], sample_rate, window_func);
        window_idx += 1;
        i += step_size;
    }
    //account for the fact that the audio may not have been perfectly divisible by the window size
    // todo: do something about this, since adding an extra, shorter dft to the end is problematic,
    // because it means the last dft in the stdft will always have less frequencies.
    //if i < samples.len() {
    //    out[window_idx] = do_fourier_transform_slice(&samples[i..], sample_rate, window_func);
    //}

    //remove the empty dft at the end
    if out[out.len()-1].len() == 0 {
        out.pop();
    }

    out
}


pub fn do_fourier_transform_slice(samples: &[f32], sample_rate: u32, window_func: WindowFunction) -> Vec<FreqData> {
    let num_frequencies = samples.len() / 2 + 1;
    let mut frequency_data = vec![FreqData::ZERO; num_frequencies];
    let frequency_step = sample_rate as f32 / samples.len() as f32;
    let chunk_size = num_frequencies/6;

    let window_bound_adj = f32::powi(4. / samples.len() as f32, 2);
    let window = | idx: usize | {
        if window_func == WindowFunction::Square {
            1.
        } else {
            1./((idx * idx) as f32 * window_bound_adj + 1.)
        }
    };
    
    thread::scope(|s| {
        for (idx, frequencies) in frequency_data.chunks_mut(chunk_size).enumerate() {
            s.spawn(move || {
                for freq_index in idx*chunk_size..idx*chunk_size+frequencies.len() {
                    let mut sum = Vec2::ZERO;
                    for i in 0..samples.len() {
                        let mut angle = (i as f32/samples.len() as f32) * TAU * freq_index as f32;
                        angle = ((angle + PI) % TAU) - PI;
                        let test_pt = Vec2::new(fast::cos(angle), fast::sin(angle));
                        sum += test_pt * samples[i] * window(i);
                    }
                    let is_nyquist_or_zero = (freq_index == num_frequencies-1 && samples.len() % 2 == 0) || freq_index == 0;

                    let sample_center = sum / samples.len();
                    let ampl = if is_nyquist_or_zero {sample_center.magnitude()} else {sample_center.magnitude() * 2.};
                    frequencies[freq_index - idx*chunk_size].frequency = (freq_index as f32 * frequency_step) as f32;
                    frequencies[freq_index - idx*chunk_size].amplitude = ampl;
                    frequencies[freq_index - idx*chunk_size].phase = -f32::atan2(sample_center.y, sample_center.x);
                }
            });
        }
    });

    frequency_data
}

pub fn do_fourier_transform(samples: &Vec<f32>, sample_rate: u32) -> Vec<FreqData> {
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
                    let ampl = if is_nyquist_or_zero {sample_center.magnitude()} else {sample_center.magnitude() * 2.};
                    frequencies[freq_index - idx*chunk_size].frequency = (freq_index as f32 * frequency_step) as f32;
                    frequencies[freq_index - idx*chunk_size].amplitude = ampl;
                    frequencies[freq_index - idx*chunk_size].phase = -f32::atan2(sample_center.y, sample_center.x);
                }
            });
        }
    });

    frequency_data
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum WindowFunction {
    Square,
    BellCurve,
}

impl WindowFunction {
    pub fn from_str(str: &str) -> Option<Self> {
        match str.to_lowercase().as_str() {
            "none" => Some(WindowFunction::Square),
            "bellcurve" => Some(WindowFunction::BellCurve),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Square => "Square".into(),
            Self::BellCurve => "BellCurve".into()
        }
    }
}

impl From<usize> for WindowFunction {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Square,
            1 => Self::BellCurve,
            _ => Self::Square
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FreqData {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl FreqData {
    pub const ZERO: Self = FreqData { frequency: 0., amplitude: 0., phase: 0. };
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
