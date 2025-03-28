use std::{fs::File, io::BufReader};

use crate::circular_buffer::CircularBuffer;
use crate::file_io::{read_data, read_wav_meta, WavInfo};
use crate::fir_filter::FIRFilter;
use crate::parametric_eq::{EqNode, FilterType, ParametricEq, Biquad};

#[derive(Debug)]
pub struct FileResults {
    pub metadata: WavInfo,
    pub lkfs_i: f64, //LKFS
    pub lkfs_s: f64, //LKFS
    pub lkfs_m: f64, //LKFS
    pub true_peaks: Vec<f32>, // dBTP (dB True-Peak)
}

// Stats for 11 Raining On Prom Night.wav:
// True Peak: -0.7 dBTP
// LKFS-M: -7.7 LKFS
// LKFS-S: -10.1 LKFS
// LKFS-I: -13.5 LKFS
// LRA: 8.6

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

    let (lkfs_i, lkfs_m, lkfs_s) = calculate_file_loudness(&samples, &metadata);
    let true_peaks = match calculate_true_peak(&samples, &metadata) {
        Some(d) => d,
        None => vec![]
    };

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
    if metadata.sample_rate != 48000 {
        return None
        //panic!("Only 48kHz is supported for true peak!");
    }
    
    // first upsample to 196kHz
    let upsampled = upsample(samples);

    // now find highest absolute magnitude(s)
    let mut ch_maxes = vec![];
    for c in 0..samples.len() {
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

const FIR_DEGREE: usize = 12;
const PHASE0_COEFF: [f32; 12] = [0.0017089843750, 0.0109863281250, -0.0196533203125, 0.0332031250000, -0.0594482421875, 0.1373291015625, 0.9721679687500, -0.1022949218750, 0.0476074218750, -0.0266113281250, 0.0148925781250, -0.0083007812500];
const PHASE1_COEFF: [f32; 12] = [-0.0291748046875, 0.0292968750000, -0.0517578125000, 0.0891113281250, -0.1665039062500, 0.4650878906250, 0.7797851562500, -0.2003173828125, 0.1015625000000, -0.0582275390625, 0.0330810546875, -0.0189208984375];
const PHASE2_COEFF: [f32; 12] = [-0.0189208984375, 0.0330810546875, -0.0582275390625, 0.1015625000000, -0.2003173828125, 0.7797851562500, 0.4650878906250, -0.1665039062500, 0.0891113281250, -0.0517578125000, 0.0292968750000, -0.0291748046875];
const PHASE3_COEFF: [f32; 12] = [-0.0083007812500, 0.0148925781250, -0.0266113281250, 0.0476074218750, -0.1022949218750, 0.9721679687500, 0.1373291015625, -0.0594482421875, 0.0332031250000, -0.0196533203125, 0.0109863281250, 0.0017089843750];
fn upsample(samples: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    //FIR Filters
    let phase0 = FIRFilter::new(FIR_DEGREE, PHASE0_COEFF.to_vec());
    let phase1 = FIRFilter::new(FIR_DEGREE, PHASE1_COEFF.to_vec());
    let phase2 = FIRFilter::new(FIR_DEGREE, PHASE2_COEFF.to_vec());
    let phase3 = FIRFilter::new(FIR_DEGREE, PHASE3_COEFF.to_vec());

    let mut upsampled = Vec::with_capacity(samples.len());
    for c in 0..samples.len() {
        let mut circ_buffer = CircularBuffer::new(FIR_DEGREE);
        for i in 0..FIR_DEGREE {
            circ_buffer.append(samples[c][i]);
        }

        let mut upsampled_ch: Vec<f32> = Vec::with_capacity(samples[c].len() * 4);
        for i in 0..samples[c].len() {
            
            circ_buffer.append(samples[c][i]);
            let history = circ_buffer.get_ordered();

            upsampled_ch.push(phase0.process(&history));
            upsampled_ch.push(phase1.process(&history));
            upsampled_ch.push(phase2.process(&history));
            upsampled_ch.push(phase3.process(&history));
        }
        upsampled.push(upsampled_ch);
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
