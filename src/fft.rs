use std::{thread, num::NonZero, f32::consts::{PI, TAU}, ops::{AddAssign, Div, Mul}};
use crate::audio::{FreqData, WindowFunction};

pub fn create_dft(dft_type: Option<DftType>, sample_rate: u32, buffer_size: usize, window_function: WindowFunction) -> Box<dyn DFT> {
    // if dft_type is specified, just return that
    if let Some(tp) = dft_type {
        match tp {
            DftType::NAIVE => return Box::new(NaiveDft::new(sample_rate, buffer_size, window_function)),
        }
    }
    //otherwise, do some stuff to figure out which dft to use
    Box::new(NaiveDft::new(sample_rate, buffer_size, window_function))
}

pub trait DFT {
    // should panic if given the wrong size buffer (buffer.len != self.buffer_size)
    fn process(&self, buffer: &[f32]) -> Vec<FreqData>;
}

pub enum DftType {
    NAIVE
}

//pub struct Fft {
//
//}

pub struct NaiveDft {
    pub frequency_step: f32,
    pub buffer_size: usize,
    num_frequencies: usize,
    twiddle_factors: Vec<Complex>,
    window_function: Vec<f32>,
    available_threads: NonZero<usize>
}

impl NaiveDft {
    pub fn new(sample_rate: u32, buffer_size: usize, window_function: WindowFunction) -> Self {
        let frequency_step = sample_rate as f32 / buffer_size as f32;
        let num_frequencies = buffer_size / 2 + 1;
        let mut twiddle_factors: Vec<Complex> = Vec::with_capacity(num_frequencies * buffer_size);

        for f in 0..num_frequencies {
            for i in 0..buffer_size {
                let mut angle = (i as f32/buffer_size as f32) * TAU * f as f32;
                angle = ((angle + PI) % TAU) - PI;
                twiddle_factors.push(Complex::new(f32::cos(angle), f32::sin(angle)));
            }
        }
        
        let available_threads = thread::available_parallelism().unwrap();
        
        // pre-compute the values for the window function
        let mut window_function_vec: Vec<f32> = Vec::with_capacity(buffer_size);
        match window_function {
            WindowFunction::Hann => {
                for i in 0..buffer_size {
                    let cur_hann = f32::powi(f32::sin((PI * i as f32) / buffer_size as f32), 2);
                    window_function_vec.push(cur_hann);
                }
            },
            WindowFunction::Square => {
                window_function_vec.fill(1f32);
            }
        }

        Self {
            frequency_step,
            buffer_size,
            num_frequencies,
            twiddle_factors,
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
                            sum += &self.twiddle_factors[f * self.buffer_size + i] * (buffer[i] * self.window_function[i]);
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

#[derive(Debug, Clone, PartialEq)]
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

impl Div<f32> for Complex {
    type Output = Complex;
    fn div(self, rhs: f32) -> Self::Output {
        Complex {
            r: self.r / rhs,
            i: self.i / rhs
        }
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
