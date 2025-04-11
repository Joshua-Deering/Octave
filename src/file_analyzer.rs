use std::{thread, sync::Arc, fs::File, io::BufReader};

use crate::circular_buffer::CircularBuffer;
use crate::file_io::{read_data, read_wav_meta, WavInfo};
use crate::fir_filter::FIRFilter;
use crate::parametric_eq::{EqNode, FilterType, ParametricEq, Biquad};
use crate::fir_filter_constants::*;

#[derive(Debug)]
pub struct FileResults {
    pub metadata: WavInfo,
    pub lkfs_i: f64, //LKFS
    pub lkfs_s: f64, //LKFS
    pub lkfs_m: f64, //LKFS
    pub true_peaks: Vec<f32>, // dBTP (dB True-Peak)
}

pub fn analyze_file(path: String) -> Option<FileResults> {
    let file = File::open(path);
    match file {
        Ok(_) => {}
        Err(_) => return None
    }
    let mut reader = BufReader::new(file.unwrap());
    let metadata = read_wav_meta(&mut reader);

    let samples = match read_data(&mut reader, &metadata, 0., metadata.audio_duration) {
        Some(data) => data,
        None => return None,
    };

    let true_peaks = match calculate_true_peak(&samples, &metadata) {
        Some(d) => d,
        None => vec![]
    };
    let (lkfs_i, lkfs_m, lkfs_s) = calculate_file_loudness(&samples, &metadata);

    Some(
        FileResults {
            metadata,
            lkfs_i,
            lkfs_s,
            lkfs_m,
            true_peaks,
        }
    )
}


pub fn calculate_true_peak(samples: &Vec<Vec<f32>>, metadata: &WavInfo) -> Option<Vec<f32>> {
    // first upsample to 192kHz
    let upsampled = upsample(samples.clone(), metadata.sample_rate);

    // now find highest absolute magnitude(s)
    let mut ch_maxes = vec![];
    for c in 0..upsampled.len() {
        let mut ch_max = 0.;
        for s in &upsampled[c] {
            if s.abs() > ch_max {
                ch_max = s.abs();
            }
        }
        ch_maxes.push(20. * ch_max.log10());
    }

    Some(ch_maxes)
}

// LKFS Measurements are based on: 
// Recommendation ITU-R BS.1770-5 (11/2023) Algorithms to measure audio programme loudness and true-peak audio level
fn calculate_file_loudness(samples: &Vec<Vec<f32>>, metadata: &WavInfo) -> (f64, f64, f64) {
    let mut samples = samples.clone();

    if metadata.sample_rate == 48000 {
        // with 48kHz audio, we can directly use the Biquad filters given in the ITU-R document

        let mut stage1_filter = Biquad::with_coefficients(1.53512485958697, -2.69169618940638, 1.19839281085285, -1.69065929318241, 0.73248077421585, 48000);
        let mut stage2_filter = Biquad::with_coefficients(1., -2., 1., -1.99004745483398, 0.99007225036621, 48000);

        for c in 0..samples.len() {
            for i in 0..samples[c].len() {
                samples[c][i] = stage1_filter.process(samples[c][i]);
                samples[c][i] = stage2_filter.process(samples[c][i]);
            }
            stage1_filter.reset_mem();
            stage2_filter.reset_mem();
        }

    } else {
        // these are frequency, q, and gain based filters derived from the coefficients in the ITU-R recommendation,
        // and are slightly less accurate, but work for any sample rate
        let stage1_filter = EqNode { filter_type: FilterType::HIGHSHELF, freq: 1500.2, gain: 4., q: 1. };
        let mut stage1 = ParametricEq::new(vec![stage1_filter], metadata.sample_rate);
        let stage2_filter = EqNode { filter_type: FilterType::HPF, freq: 50.42, gain: 0., q: 1. };
        let mut stage2 = ParametricEq::new(vec![stage2_filter], metadata.sample_rate);

        // process all channels through 2-stage K-weighting filters
        for c in 0..samples.len() {
            stage1.process(&mut samples[c]);
            stage2.process(&mut samples[c]);
            
            stage1.reset_filter_mem();
            stage2.reset_filter_mem();
        }
    }

    let samples_squared = samples.iter().map(|v| v.iter().map(|&x| (x as f64).powf(2.)).collect::<Vec<f64>>()).collect::<Vec<Vec<f64>>>();

    let samples_per_block = (0.4 * metadata.sample_rate as f32).round() as usize;
    let step_size = (0.25 * samples_per_block as f32).round() as usize;

    let mut mean_squares = vec![vec![]; metadata.channels as usize];
    for c in 0..samples_squared.len() {
        let mut i = 0;
        while i + samples_per_block < samples_squared[c].len() {
            let mean = samples_squared[c][i..i+samples_per_block].iter().sum::<f64>() / samples_per_block as f64;
            mean_squares[c].push(mean);
            i += step_size;
        }
    }

    // calculate LKFS-M
    let mut lkfs_m = f64::MIN;
    for i in 0..mean_squares[0].len() {
        let mut sum = 0.;
        for c in 0..metadata.channels as usize {
            sum += mean_squares[c][i];
        }
        let block_loudness = loudness(sum);
        if block_loudness > lkfs_m {
            lkfs_m = block_loudness;
        }
    }

    // calculate LKFS-S
    let mut lkfs_s = f64::MIN;
    if mean_squares[0].len() >= 30 {
        for i in 0..(mean_squares[0].len() - 30) {
            let mut sum = 0.;
            for c in 0..metadata.channels as usize {
                sum += mean_squares[c][i..i+30].iter().sum::<f64>() / 30.;
            }
            let block_loudness = loudness(sum);
            if block_loudness > lkfs_s {
                lkfs_s = block_loudness;
            }
        }
    }

    // first gating stage
    let gamma_a = -70.;
    mean_squares = filter_blocks(mean_squares, gamma_a);

    let mut gamma_r = 0.;
    for c in 0..metadata.channels as usize {
        let ch_sum = mean_squares[c].iter().sum::<f64>();
        gamma_r += ch_sum / mean_squares[c].len() as f64;
    }
    gamma_r = loudness(gamma_r) - 10.;

    // second gating stage
    mean_squares = filter_blocks(mean_squares, gamma_r);

    let mut final_loudness_sum = 0.;
    for c in 0..metadata.channels as usize {
        let ch_sum = mean_squares[c].iter().sum::<f64>();
        final_loudness_sum += ch_sum / mean_squares[c].len() as f64;
    }
    let lkfs_i = loudness(final_loudness_sum);
    
    //if there were not enough samples for a 3s measurement
    if lkfs_s == f64::MIN {
        lkfs_s = lkfs_i;
    }

    (lkfs_i, lkfs_m, lkfs_s)
}

// helpers for calculate_true_peak
fn upsample(samples: Vec<Vec<f32>>, sample_rate: u32) -> Vec<Vec<f32>> {
    //FIR Filters
    let mut upsampling_filters = vec![];

    match sample_rate {
        48000 => {
            for f in FIR_COEFF_48K {
                upsampling_filters.push(FIRFilter::new(FIR_UPSAMPLING_DEG, f.to_vec()));
            }
        },
        44100 => {
            for f in FIR_COEFF_44_1K {
                upsampling_filters.push(FIRFilter::new(FIR_UPSAMPLING_DEG, f.to_vec()));
            }
        },
        8000 => {
            for f in FIR_COEFF_8K {
                upsampling_filters.push(FIRFilter::new(FIR_UPSAMPLING_DEG, f.to_vec()));
            }
        }
        _ => {
            panic!();
        }
    }

    let channels = samples.len();

    let samples_arc = Arc::new(samples);
    let filters_arc = Arc::new(upsampling_filters);

    let mut handles = Vec::with_capacity(channels);
    for c in 0..channels {
        let samples_clone = Arc::clone(&samples_arc);
        let filters_clone = Arc::clone(&filters_arc);
        let handle = thread::spawn(move || {
            let mut circ_buffer = CircularBuffer::new(FIR_UPSAMPLING_DEG);
            for i in 0..FIR_UPSAMPLING_DEG {
                circ_buffer.append(samples_clone[c][i]);
            }

            let mut upsampled_ch: Vec<f32> = Vec::with_capacity(samples_clone[c].len() * filters_clone.len());
            for i in 0..samples_clone[c].len() {
                
                circ_buffer.append(samples_clone[c][i]);
                let history = circ_buffer.get_ordered();

                for filter in filters_clone.iter() {
                    upsampled_ch.push(filter.process(&history));
                }
            }
            return upsampled_ch;
        });
        handles.push(handle);
    }

    let mut upsampled = Vec::with_capacity(channels);
    for h in handles {
        upsampled.push(h.join().unwrap());
    }

    upsampled
}

// helpers for calculate_file_loudness
fn loudness(squared_mean: f64) -> f64 {
    -0.691 + 10. * f64::log10(squared_mean)
}

fn filter_blocks(mean_squares: Vec<Vec<f64>>, threshold: f64) -> Vec<Vec<f64>> {
    let mut out = vec![Vec::with_capacity(mean_squares[0].len()); mean_squares.len()];
    for i in 0..mean_squares[0].len() {
        let mut squares_sum = 0.;
        for c in 0..mean_squares.len() {
            squares_sum += mean_squares[c][i];
        }
        if loudness(squares_sum) > threshold {
            for c in 0..mean_squares.len() {
                out[c].push(mean_squares[c][i]);
            }
        }
    }
    out
}
