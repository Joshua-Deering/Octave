use std::{fs, io};

use crate::fft::Fft;
use crate::audio::{FreqData, WindowFunction};

#[allow(unused)]
pub fn compare_signals(signal1: &Vec<Vec<f32>>, sample_rate1: u32, signal2: &Vec<Vec<f32>>, sample_rate2: u32) {
    assert!(signal1.len() == signal2.len());
    
    let fft1 = Fft::new(sample_rate1, signal1[0].len(), WindowFunction::Square);
    let fft2 = Fft::new(sample_rate2, signal2[0].len(), WindowFunction::Square);
    
    let mut fft_bins_1 = Vec::with_capacity(signal1.len());
    for c in 0..signal1.len() {
        fft_bins_1.push(fft_to_freq_bins(fft1.process(&signal1[c]), u32::min(sample_rate1, sample_rate2), 1./12.));
    }

    let mut fft_bins_2 = Vec::with_capacity(signal2.len());
    for c in 0..signal2.len() {
        fft_bins_2.push(fft_to_freq_bins(fft2.process(&signal2[c]), u32::min(sample_rate1, sample_rate2), 1./12.));
    }

    print!("Frequency 1\t");
    for c in 0..signal1.len() {
        print!("S1 Ch{}   \t S2 Ch{}  \t Abs. Diff    \t", c, c);
    }
    println!("");

    for i in 0..fft_bins_1[0].len() {
        print!("{:.3}  \t", fft_bins_1[0][i].0);
        for c in 0..fft_bins_1.len() {
            print!("{:.3} \t {:.3} \t {:.3} \t     ", fft_bins_1[c][i].1, fft_bins_2[c][i].1, (fft_bins_1[c][i].1.abs() - fft_bins_2[c][i].1.abs()).abs())
        }
        println!("");
    }
}

fn fft_to_freq_bins(fft: Vec<FreqData>, sample_rate: u32, octave_bandwidth: f32) -> Vec<(f32, f32)> {
    let band_multiplier = 2f32.powf(octave_bandwidth);

    let mut bins = vec![];

    let mut low_bound = 20.;
    let mut fft_i = 0;
    let mut last_fft_i = 0;
    while low_bound < (sample_rate / 2) as f32 {
        let mut bin_sum = 0.;
        let upper_bound = low_bound * band_multiplier;

        while fft_i < fft.len() && fft[fft_i].frequency < upper_bound  {
            bin_sum += fft[fft_i].amplitude;
            fft_i += 1;
        }
        if fft_i == last_fft_i && fft_i != 0 {
            bins.push(bins[bins.len()-1]);
        } else {
            bins.push(((low_bound + upper_bound) / 2., 20. * bin_sum.log10()));
        }

        low_bound *= band_multiplier;

        if fft_i >= fft.len() {
            break;
        }
        last_fft_i = fft_i;
    }

    bins
}

pub fn query_directory(dir: &str) -> impl Iterator<Item = String> {
    let mut entries = fs::read_dir(dir)
        .unwrap()
        .map(|res| res.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.retain(|e| !e.eq_ignore_ascii_case(".DS_store"));

    entries.into_iter().map(|e| e.to_string_lossy().to_string())
}

pub fn logspace(min: f32, max: f32, num_points: usize) -> impl Iterator<Item = f32> {
    let log_min = min.ln();
    let log_max = max.ln();
    let step = (log_max - log_min) / ((num_points - 1) as f32);
    
    (0..num_points).map(move |i| (log_min + step * (i as f32)).exp())
}

pub fn hue_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h = h / 60.;
    let x = c * (1. - f32::abs(h % 2. - 1.));
    let m = v - c;

    let rgb1: (f32, f32, f32);

    if h <= 1. {
        rgb1 = (c, x, 0.);
    } else if h <= 2. {
        rgb1 = (x, c, 0.);
    } else if h <= 3. {
        rgb1 = (0., c, x);
    } else if h <= 4. {
        rgb1 = (0., x, c);
    } else if h <= 5. {
        rgb1 = (x, 0., c);
    } else {
        rgb1 = (c, 0., x);
    }

    (
        ((rgb1.0 + m) * 255.) as u8,
        ((rgb1.1 + m) * 255.) as u8,
        ((rgb1.2 + m) * 255.) as u8,
    )
}
