use std::{fs::File, io::BufReader};

use crate::file_io::{read_data, read_wav_meta, WavInfo};
use crate::parametric_eq::{EqNode, FilterType, ParametricEq};

#[derive(Debug)]
pub struct FileResults {
    pub metadata: WavInfo,
    pub lkfs_m: f32,
}

pub fn analyze_file(path: String) -> Option<FileResults> {
    let file = File::open(path);
    match file {
        Ok(_) => {}
        Err(_) => return None
    }
    let mut reader = BufReader::new(file.unwrap());
    let metadata = read_wav_meta(&mut reader);
    let mut samples = match read_data(&mut reader, metadata.clone(), 0., metadata.audio_duration) {
        Some(data) => data,
        None => return None,
    };

    // LKFS Measurements are based on: 
    // Recommendation ITU-R BS.1770-5 (11/2023) Algorithms to measure audio programme loudness and true-peak audio level

    // K-Weighting filters:
    let stage1_filter = EqNode { filter_type: FilterType::HIGHSHELF, freq: 1500.2, gain: 4., q: 1. };
    let mut stage1 = ParametricEq::new(vec![stage1_filter], metadata.sample_rate);
    let stage2_filter = EqNode { filter_type: FilterType::HPF, freq: 50.42, gain: 0., q: 1. };
    let mut stage2 = ParametricEq::new(vec![stage2_filter], metadata.sample_rate);

    // process all channels through 2-stage K-weighting filters
    for c in 0..samples.len() {
        stage1.process(&mut samples[c]);
        stage2.process(&mut samples[c]);
    }

    let samples_squared = samples.iter().map(|v| v.iter().map(|&x| x.powi(2)).collect::<Vec<f32>>()).collect::<Vec<Vec<f32>>>();

    let samples_per_block = (0.4 * metadata.sample_rate as f32).round() as usize;
    let step_size = (0.25 * samples_per_block as f32).round() as usize;

    let mut mean_squares = vec![vec![]; metadata.channels as usize];
    for c in 0..metadata.channels as usize {
        let mut i = 0;
        while i + samples_per_block < samples_squared[0].len() {
            let mean = samples_squared[c][i..i+samples_per_block].iter().sum::<f32>() / samples_per_block as f32;
            mean_squares[c].push(mean);
            i += step_size;
        }
    }

    //first gating stage
    println!("{}, {}", mean_squares.len(), mean_squares[0].len());
    let gamma_a = -70.;
    mean_squares = filter_blocks(mean_squares, gamma_a);
    println!("{}, {}", mean_squares.len(), mean_squares[0].len());

    let mut gamma_r = 0.;
    for c in 0..metadata.channels as usize {
        let ch_sum = mean_squares[c].iter().sum::<f32>();
        gamma_r += ch_sum / mean_squares[c].len() as f32;
    }
    gamma_r = loudness(gamma_r) - 10.;

    //second gating stage
    println!("{}", gamma_r);
    mean_squares = filter_blocks(mean_squares, gamma_r);
    println!("{}, {}", mean_squares.len(), mean_squares[0].len());

    let mut final_loudness_sum = 0.;
    for c in 0..metadata.channels as usize {
        let ch_sum = mean_squares[c].iter().sum::<f32>();
        final_loudness_sum += ch_sum / mean_squares[c].len() as f32;
    }
    let final_loudness = loudness(final_loudness_sum);
    println!("{}", final_loudness);

    Some(
        FileResults {
            metadata,
            lkfs_m: 0.,
        }
    )
}

fn loudness(squared_mean: f32) -> f32 {
    -0.691 + 10. * f32::log10(squared_mean)
}

fn filter_blocks(mean_squares: Vec<Vec<f32>>, threshold: f32) -> Vec<Vec<f32>> {
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
