use std::{f32::consts::{PI, TAU}, num::NonZero, ops::{Add, AddAssign, Div, Mul, MulAssign, Sub}, thread};
use fastapprox::fast;
use crate::audio::{FreqData, WindowFunction};

pub fn create_dft(dft_type: Option<DftType>, sample_rate: u32, buffer_size: usize, window_function: WindowFunction) -> Box<dyn DFT> {
    // if dft_type is specified, just return that
    if let Some(tp) = dft_type {
        match tp {
            DftType::NAIVE => return Box::new(NaiveDft::new(sample_rate, buffer_size, window_function)),
            DftType::FFT => return Box::new(Fft::new(sample_rate, buffer_size, window_function)),
        }
    }
    //otherwise, do some stuff to figure out which dft to use
    Box::new(NaiveDft::new(sample_rate, buffer_size, window_function))
}

pub trait DFT {
    // should panic if given the wrong size buffer (buffer.len != self.buffer_size)
    fn process(&self, buffer: &[f32]) -> Vec<FreqData>;
}

#[allow(unused)]
pub enum DftType {
    NAIVE,
    FFT
}

pub struct Fft {
    pub frequency_step: f32,
    pub buffer_size: usize,
    pub twiddle_factors: Vec<Complex>,
    non_padded_buf_size: usize,
    window_function: Vec<f32>,
}

impl Fft {
    pub fn new(sample_rate: u32, buffer_size: usize, window_function: WindowFunction) -> Self {

        let mut padded_buf_size = buffer_size;
        if buffer_size & (buffer_size - 1) != 0 {
            padded_buf_size = usize::pow(2, f32::log2(buffer_size as f32) as u32 + 1);
        }

        let frequency_step = sample_rate as f32 / padded_buf_size as f32;

        let twiddle_factors = Self::compute_twiddles(padded_buf_size);
        let window_function = Self::compute_window_func(padded_buf_size, window_function);

        Self{
            frequency_step,
            buffer_size: padded_buf_size,
            twiddle_factors,
            window_function,
            non_padded_buf_size: buffer_size
        }
    }

    fn compute_twiddles(n: usize) -> Vec<Complex> {
        use std::f64::consts::TAU;
        let mut twiddles = Vec::with_capacity(n / 2);

        let cos_base = f64::cos(TAU / n as f64);
        let sin_base = f64::sin(TAU / n as f64);

        twiddles.push(Complex::new(1., 0.));

        // use f64 to avoid rounding errors because of repeated operations
        let mut real = 1f64;
        let mut imag = 0f64;

        for _ in 1..(n/2) {
            let next_r = real * cos_base - imag * sin_base;
            let next_i = imag * cos_base + real * sin_base;

            real = next_r;
            imag = next_i;

            twiddles.push(Complex::new(real as f32, -imag as f32));
        }

        twiddles
    }

    fn compute_window_func(n: usize, window_function: WindowFunction) -> Vec<f32> {
        let mut out;
        match window_function {
            WindowFunction::Hann => {
                out = Vec::with_capacity(n);
                for i in 0..n {
                    let cur_hann = 0.5 * (1. - f32::cos((TAU * i as f32) / n as f32));
                    out.push(cur_hann);
                }
            },
            WindowFunction::Square => {
                out = vec![1f32; n];
            }
        }
        out
    }

    fn compute_fft_in_place(&self, buffer: &mut [Complex]) {
        let n = buffer.len();
        if n == 1 {
            return;
        }

        //bit reversal to allow for iterative fft implementation
        let mut i = 0;
        for j in 1..n {
            let mut bit = n >> 1;
            while i & bit != 0 {
                i &= !bit;
                bit >>= 1;
            }
            i |= bit;

            if j < i {
                buffer.swap(j, i);
            }
        }

        let mut len = 2;
        while len <= n {
            for start in (0..n).step_by(len) {
                for k in 0..(len/2) {
                    let twiddle = self.twiddle_factors[k * (n / len)];
                    let even = buffer[start + k];
                    let odd = buffer[start + k + len / 2] * twiddle;

                    buffer[start + k] = even + odd;
                    buffer[start + k + len / 2] = even - odd;

                }
            }
            len *= 2;
        }
    }
}

impl DFT for Fft {
    fn process(&self, buffer: &[f32]) -> Vec<FreqData> {
        let mut complex_buffer = Vec::with_capacity(self.buffer_size);
        // convert real-valued inputs to complex inputs, and premultiply by the window function
        for i in 0..self.non_padded_buf_size {
            complex_buffer.push(Complex::new(buffer[i] * self.window_function[i], 0.));
        }
        for _ in self.non_padded_buf_size..self.buffer_size {
            complex_buffer.push(Complex::new(0., 0.));
        }
        //modifies the complex_buffer in-place to be the output of the fft
        self.compute_fft_in_place(complex_buffer.as_mut_slice());

        let mut out: Vec<FreqData> = Vec::with_capacity(self.buffer_size / 2);
        for i in 0..self.buffer_size/2 {
            let center = complex_buffer[i] / self.buffer_size as f32;

            let freq = i as f32 * self.frequency_step;
            let amp = center.magnitude() * 2.;
            let phase = f32::atan2(complex_buffer[i].i, complex_buffer[i].r);
            let cur_data = FreqData::new(freq, amp, phase);
            out.push(cur_data);
        }
        out
    }
}

pub struct NaiveDft {
    pub frequency_step: f32,
    pub buffer_size: usize,
    num_frequencies: usize,
    window_function: Vec<f32>,
    available_threads: NonZero<usize>,
}

impl NaiveDft {
    pub fn new(sample_rate: u32, buffer_size: usize, window_function: WindowFunction) -> Self {
        let frequency_step = sample_rate as f32 / buffer_size as f32;
        let num_frequencies = buffer_size / 2;
        
        let available_threads = thread::available_parallelism().unwrap();
        
        // pre-compute the values for the window function
        let mut window_function_vec: Vec<f32>;
        match window_function {
            WindowFunction::Hann => {
                window_function_vec = Vec::with_capacity(buffer_size);
                for i in 0..buffer_size {
                    let cur_hann = 0.5 * (1. - f32::cos((TAU * i as f32) / buffer_size as f32));
                    window_function_vec.push(cur_hann);
                }
            },
            WindowFunction::Square => {
                window_function_vec = vec![1f32; buffer_size];
            }
        }

        Self {
            frequency_step,
            buffer_size,
            num_frequencies,
            window_function: window_function_vec,
            available_threads,
        }
    }
}

impl DFT for NaiveDft {
    fn process(&self, buffer: &[f32]) -> Vec<FreqData> {
        assert!(buffer.len() == self.buffer_size);
        
        let mut frequencies = vec![FreqData::ZERO; self.num_frequencies];
        let chunk_size = self.num_frequencies / self.available_threads;
        
        thread::scope(|s| {
            for (idx, frequency_block) in frequencies.chunks_mut(chunk_size).enumerate() {
                s.spawn(move || {
                    let chk_pos = idx * chunk_size;
                    for f in chk_pos..chk_pos + frequency_block.len() {
                        let mut sum = Complex::zero();
                        for i in 0..self.buffer_size {
                            let mut angle = (i as f32/self.buffer_size as f32) * TAU * f as f32;
                            angle = ((angle + PI) % TAU) - PI;
                            let test_pt = Complex::new(fast::cos(angle), fast::sin(angle));
                            sum += &test_pt * (buffer[i] * self.window_function[i]);
                        }

                        let sample_center = sum / self.buffer_size as f32;
                        frequency_block[f - chk_pos].frequency = (f as f32 * self.frequency_step) as f32;
                        frequency_block[f - chk_pos].amplitude = sample_center.magnitude() * 2.;
                        frequency_block[f - chk_pos].phase = -f32::atan2(sample_center.i, sample_center.r);
                    }
                });
            }
        });

        frequencies[0].amplitude /= 2.;
        frequencies[self.num_frequencies-1].amplitude /= 2.;
        
        frequencies
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Complex {
    pub r: f32,
    pub i: f32,
}

impl Complex {
    fn zero() -> Self {
        Self { r: 0., i: 0. }
    }
    
    fn new(r: f32, i: f32) -> Self {
        Self { r, i }
    }

    fn magnitude(&self) -> f32 {
        f32::sqrt(self.r * self.r + self.i * self.i)
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.i += rhs.i;
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            r: self.r + rhs.r,
            i: self.i + rhs.i,
        }
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            r: self.r - rhs.r,
            i: self.i - rhs.i,
        }
    }
}

impl Div<f32> for Complex {
    type Output = Complex;
    fn div(self, rhs: f32) -> Self::Output {
        Complex {
            r: self.r / rhs,
            i: self.i / rhs
        }
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Self::Output {
        Complex {
            r: self.r * rhs.r - self.i * rhs.i,
            i: self.r * rhs.i + self.i * rhs.r,
        }
    }
}

impl MulAssign for Complex {
    fn mul_assign(&mut self, rhs: Self) {
        let new_r = self.r * rhs.r - self.i * rhs.i;
        self.i = self.r * rhs.i + self.i * rhs.r;
        self.r = new_r;
    }
}

impl<'a> Mul<f32> for &'a Complex {
    type Output = Complex;
    fn mul(self, rhs: f32) -> Self::Output {
        Complex {
            r: self.r * rhs,
            i: self.i * rhs,
        }
    }
}
