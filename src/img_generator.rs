use image::{Pixel, Rgb, RgbImage, RgbaImage, Rgba};

use crate::{audio::ShortTimeDftData, hue_to_rgb};

pub fn generate_waveform_img(
    target_dir: String,
    imgx: u32,
    imgy: u32,
    samples: Vec<Vec<f32>>, 
) -> Result<(), image::ImageError> {
    if samples.len() == 0 || samples[0].len() == 0 {
        return Ok(())
    }
    let channels = samples.len();

    let x_scale = imgx as f32 / samples[0].len() as f32;
    let middle = (imgy / 2) as f32;

    let mut imgbuf = RgbaImage::new(imgx, imgy);
    
    for i in 0..samples[0].len() {
        let mut sum = 0.;
        for j in 0..channels {
            sum += samples[j][i];
        }
        sum /= channels as f32;

        imgbuf.put_pixel(((i as f32 * x_scale).round() as u32).min(imgx - 1), (middle + sum * middle).round() as u32, *Rgba::from_slice(&[0u8, 255u8, 0u8, 255u8]));
    }
    
    imgbuf.save(target_dir)
}

pub fn generate_spectrogram_img(
    target_dir: String,
    imgx: u32,
    imgy: u32,
    stdft: ShortTimeDftData,
) -> Result<(), image::ImageError> {
    let num_dfts = stdft.num_dfts;
    let num_freq = stdft.num_freq;

    let x_scale = num_dfts as f32 / imgx as f32;
    let y_scale = num_freq as f32 / imgy as f32;

    let max_amplitude = find_max_amplitude(&stdft);

    let mut imgbuf = RgbImage::new(imgx, imgy);
    let mut written_px = vec![vec![false; imgx as usize]; imgy as usize];

    for input_y in 0..num_freq {
        for input_x in 0..num_dfts {
            let x = (input_x as f32 / x_scale).floor() as u32;
            let y = (input_y as f32 / y_scale).floor() as u32;

            if !written_px[y as usize][x as usize] {
                imgbuf.put_pixel(x, y, *Rgb::from_slice(&rgb_from_range(stdft.dft_data[input_x as usize][stdft.num_freq as usize - input_y as usize - 1].amplitude, max_amplitude)));
                written_px[y as usize][x as usize] = true;
            } else { //if we have already written to this pixel, blend between the two rgb values
                let cur_px = rgb_from_range(stdft.dft_data[input_x as usize][stdft.num_freq as usize - input_y as usize - 1].amplitude, max_amplitude);
                let other_px = imgbuf.get_pixel(x, y).channels();
                imgbuf.put_pixel(x, y, *Rgb::from_slice(&blend_rgb(&cur_px, &other_px)));
            }
        }
    }

    imgbuf.save(target_dir)
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

fn blend_rgb(col1: &[u8], col2: &[u8]) -> [u8; 3] {
    [
        ((col1[0] as u32 + col2[0] as u32) / 2) as u8,
        ((col1[1] as u32 + col2[1] as u32) / 2) as u8,
        ((col1[2] as u32 + col2[2] as u32) / 2) as u8,
    ]
}

fn rgb_from_range(amplitude: f32, max_amplitude: f32) -> [u8; 3] {
    let amp_scaled = f32::powf(amplitude / max_amplitude, 0.3);
    let col_val = (amp_scaled * 360. + 200.) % 360.;

    hue_to_rgb(col_val, 0.8, 1.)
}
