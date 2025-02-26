use std::{fmt, mem::size_of_val};

use crate::fft::Fft;

pub struct ShortTimeDftData {
    pub dft_data: Vec<Vec<FreqData>>,
    pub num_dfts: u32,
    pub num_freq: u32,
    pub sample_rate: u32,
    pub data_size: usize,
}

impl ShortTimeDftData {
    pub fn new(dft_data: Vec<Vec<FreqData>>, num_dfts: u32, num_freq: u32, sample_rate: u32) -> Self {
        let data_size = (size_of_val(&dft_data[0][0]) as u32 * num_dfts * num_freq) as usize + (size_of::<u32>() * 4);
        Self { dft_data, num_dfts, num_freq, sample_rate, data_size }
    }
}

impl fmt::Display for ShortTimeDftData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("Short Time DFT Data:\nSample Rate: {} Hz\n# DFTs: {}\n# Frequencies: {}\nTotal Data Size: {} bytes", self.sample_rate, self.num_dfts, self.num_freq, self.data_size).as_str())
    }
}

pub fn do_short_time_fourier_transform(samples: &Vec<f32>, sample_rate: u32, window_size: f32, overlap: f32, window_func: WindowFunction) -> Vec<Vec<FreqData>> {
    let samples_per_window = (window_size * sample_rate as f32).round() as usize;
    let overlap_size = (samples_per_window as f32 * overlap).floor() as usize;
    let step_size = samples_per_window - overlap_size;
    let num_windows = samples.len() / (samples_per_window - overlap_size);

    let dft = Fft::new(sample_rate, samples_per_window, window_func);
    
    let mut out: Vec<Vec<FreqData>> = vec![vec![]; num_windows];
    let mut window_idx = 0;
    let mut i: usize = 0;
    while i + samples_per_window < samples.len() {
        out[window_idx] = dft.process(&samples[i..i+samples_per_window]);
        window_idx += 1;
        i += step_size;
    }

    // remove the empty dft(s) at the end that are
    // caused by the rounding of window size
    while out[out.len()-1].len() == 0 {
        out.pop();
    }

    out
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum WindowFunction {
    Square,
    Hann,
}

impl WindowFunction {
    pub fn from_str(str: &str) -> Option<Self> {
        match str.to_lowercase().as_str() {
            "square" => Some(WindowFunction::Square),
            "hann" | "bellcurve" => Some(WindowFunction::Hann),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Square => "Square".into(),
            Self::Hann => "Hann".into()
        }
    }
}

impl From<usize> for WindowFunction {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Square,
            1 => Self::Hann,
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
    pub fn new(frequency: f32, amplitude: f32, phase: f32) -> Self {
        Self {
            frequency,
            amplitude,
            phase,
        }
    }
}
