use std::{io::BufReader, fs::File};

//use image::{Pixel, Rgb, RgbImage, RgbaImage, Rgba};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};

use crate::{audio::{FreqData, ShortTimeDftData}, file_io::{read_data, read_wav_meta}, util::hue_to_rgb, ParametricEq};

pub fn generate_waveform_preview(audio_file: SharedString, imgx: f32, imgy: f32) -> Image {
    if audio_file.trim().is_empty() {
        return Image::default();
    }

    let mut file = BufReader::new(File::open(format!("./res/audio/{}", audio_file)).unwrap());
    let file_info = read_wav_meta(&mut file);

    let duration = file_info.audio_duration;
    let channels = file_info.channels as usize;

    let samples = read_data(&mut file, file_info, 0., duration).unwrap();
    let total_samples = samples[0].len();
    let samples_per_pixel = total_samples as f32 / imgx;

    let middle = (imgy as u32 / 2) as f32;

    let mut shared_buf = SharedPixelBuffer::new(imgx as u32, imgy as u32);
    let buf = shared_buf.make_mut_slice();
    for x in 0..(imgx as usize) {
        // Determine the range of sample indices that fall into this x column.
        let start = (x as f32 * samples_per_pixel).floor() as usize;
        let end = ((x as f32 + 1.0) * samples_per_pixel).ceil() as usize;

        // Set initial min/max values
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;

        // Process every sample in this bin
        for i in (start.min(total_samples)..end.min(total_samples)).step_by(4) {
            // Average across all channels
            let mut sum = 0.0;
            for ch in 0..channels {
                sum += samples[ch][i];
            }
            sum /= channels as f32;

            if sum < min_val { min_val = sum; }
            if sum > max_val { max_val = sum; }
        }

        // Map the amplitude values to y coordinates
        let y_min = (middle + min_val * middle).round() as usize;
        let y_max = (middle + max_val * middle).round() as usize;

        // Draw a vertical line in this column from y_min to y_max
        // (Ensure y_min is not greater than y_max)
        for y in y_min.min(y_max)..=y_min.max(y_max) {
            // Ensure we don't go out of bounds
            if y < imgy as usize {
                buf[y * (imgx as usize) + x] = Rgba8Pixel::new(0, 255, 0, 255);
            }
        }
    } 
    Image::from_rgba8(shared_buf)
}

pub fn generate_rta_line(
    imgx: u32, imgy: u32,
    min_freq: f32, max_freq: f32,
    min_level: f32, max_level: f32,
    octave_bandwidth: f32,
    fft: Vec<FreqData>,
) -> SharedString {
    let band_multiplier = 2f32.powf(octave_bandwidth);

    let mut bins = vec![];

    let mut low_bound = min_freq;
    let mut fft_i = 0;
    while low_bound < max_freq {
        let mut bin_sum = 0.;
        let upper_bound = low_bound * band_multiplier;

        while fft_i < fft.len() && fft[fft_i].frequency < upper_bound  {
            bin_sum += fft[fft_i].amplitude;
            fft_i += 1;
        }

        bins.push(((low_bound + upper_bound) / 2., 20. * bin_sum.log10()));
        low_bound *= band_multiplier;

        if fft_i >= fft.len() {
            break;
        }
    }

    let freq_to_x = | f: f32 | -> u32 {
        ((f.log10() - min_freq.log10()) / (max_freq.log10() - min_freq.log10()) * imgx as f32) as u32
    };
    let level_to_y = | level: f32 | -> u32 {
        ((imgy / 2) as f32 - (level - min_level) / (max_level - min_level) * imgy as f32) as u32
    };

    let mut svg_string_cmds = vec![(0, 0); bins.len()];
    for i in 0..bins.len() {
        svg_string_cmds[i] = (freq_to_x(bins[i].0), level_to_y(bins[i].1));
    }

    (format!("M 0 {}", imgy) + svg_string_cmds.into_iter().map(|(x, y)| format!("L {} {}", x, y.min(imgy))).collect::<Vec<String>>().join("").as_str()).into()
}

pub fn generate_eq_response(
    param_eq: &ParametricEq,
    min_freq: f32, max_freq: f32,
    min_gain: f32, max_gain: f32,
    imgx: u32, imgy: u32) -> SharedString {

    let freq_to_x = | f: f32 | -> u32 {
        ((f.log10() - min_freq.log10()) / (max_freq.log10() - min_freq.log10()) * imgx as f32) as u32
    };
    let gain_to_y = | gain: f32 | -> u32 {
        ((imgy / 2) as f32 - gain / (max_gain - min_gain) * imgy as f32) as u32
    };

    let eq_points = param_eq.get_freq_response_log(min_freq as u32, max_freq as u32, (imgx / 2) as usize);

    let mut svg_string_cmds: Vec<(u32, u32)> = Vec::with_capacity(eq_points.len());
    for i in 0..eq_points.len() {
        svg_string_cmds.push((freq_to_x(eq_points[i].0), gain_to_y(eq_points[i].1) + 1));
    }
    
    (format!("M {} {} ", freq_to_x(min_freq), gain_to_y(eq_points[0].1)) + svg_string_cmds.into_iter().map(|(x, y)| format!("L {} {} ", x, y)).collect::<Vec<String>>().join("").as_str()).into()
}

pub fn generate_spectrogram_img(
    imgx: u32,
    imgy: u32,
    stdft: ShortTimeDftData,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let num_dfts = stdft.num_dfts;
    let num_freq = stdft.num_freq;

    let x_scale = num_dfts as f32 / imgx as f32;
    let y_scale = num_freq as f32 / imgy as f32;

    let max_amplitude = find_max_amplitude(&stdft);

    let mut img = SharedPixelBuffer::new(imgx, imgy);
    let imgbuf = img.make_mut_slice();
    let mut written_px = vec![vec![false; imgx as usize]; imgy as usize];

    for input_y in 0..num_freq {
        for input_x in 0..num_dfts {
            let x = (input_x as f32 / x_scale).floor() as u32;
            let y = (input_y as f32 / y_scale).floor() as u32;

            if !written_px[y as usize][x as usize] {
                let (r, g, b) = rgb_from_range(stdft.dft_data[input_x as usize][stdft.num_freq as usize - input_y as usize - 1].amplitude, max_amplitude);
                imgbuf[(y * imgx + x) as usize] = Rgba8Pixel::new(r, g, b, 255);
                written_px[y as usize][x as usize] = true;
            } else { //if we have already written to this pixel, blend between the two rgb values
                let cur_col = rgb_from_range(stdft.dft_data[input_x as usize][stdft.num_freq as usize - input_y as usize - 1].amplitude, max_amplitude);
                let other_col: (u8, u8, u8) = imgbuf[(y * imgx + x) as usize].rgb().into();
                let blended = blend_rgb(cur_col, other_col);
                imgbuf[(y * imgx + x) as usize] = Rgba8Pixel::new(blended.0, blended.1, blended.2, 255);
            }
        }
    }

    img
}

pub fn generate_waveform_img(
    imgx: u32,
    imgy: u32,
    samples: Vec<Vec<f32>>
) -> SharedPixelBuffer<Rgba8Pixel> {
    let imgx = imgx as usize;
    
    let channels = samples.len();
    
    let x_scale = imgx as f32 / samples[0].len() as f32;
    let center_y = (imgy / 2) as usize;

    let mut img = SharedPixelBuffer::new(imgx as u32, imgy);
    let imgbuf = img.make_mut_slice();

    for i in 0..samples[0].len() {
        let mut sum = 0.;
        for j in 0..channels {
            sum += samples[j][i];
        }
        sum /= channels as f32;
        
        let x = (i as f32 * x_scale).floor() as usize;
        let y = ((sum * center_y as f32).floor() + center_y as f32) as usize;
        imgbuf[y * imgx + x] = Rgba8Pixel::new(0, 255, 0, 255);
    }

    img
}

fn find_max_amplitude(stdft: &ShortTimeDftData) -> f32 {
    let mut max = 0.;
    for dft in &stdft.dft_data {
        for freq_data in dft {
            if freq_data.amplitude.abs() > max {
                max = freq_data.amplitude.abs();
            }
        }
    }
    max
}

fn blend_rgb(col1: (u8, u8, u8), col2: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        ((col1.0 as u32 + col2.0 as u32) / 2) as u8,
        ((col1.1 as u32 + col2.1 as u32) / 2) as u8,
        ((col1.2 as u32 + col2.2 as u32) / 2) as u8,
    )
}

fn rgb_from_range(amplitude: f32, max_amplitude: f32) -> (u8, u8, u8) {
    let amp_scaled = f32::powf(amplitude / max_amplitude, 0.3);
    let col_val = (amp_scaled * 360. + 200.) % 360.;

    hue_to_rgb(col_val, 0.8, 1.)
}
