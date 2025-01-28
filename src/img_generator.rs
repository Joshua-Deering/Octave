use image::{Pixel, Rgb, RgbImage};

use crate::{audio::ShortTimeDftData, hue_to_rgb};

pub fn generate_img(
    target_dir: String,
    imgx: u32,
    imgy: u32,
    stdft: ShortTimeDftData,
) -> Result<(), image::ImageError> {
    let x_scale = stdft.num_dfts as f32 / imgx as f32;
    let y_scale = stdft.num_freq as f32 / imgy as f32;

    let max_amplitude = find_max_amplitude(&stdft);

    let mut imgbuf = RgbImage::new(imgx, imgy);
    let mut written_px = vec![vec![false; imgx as usize]; imgy as usize];

    for y in 0..imgy {
        for x in 0..imgx {
            let input_x = (x as f32 * x_scale).floor() as usize;
            let input_y = (y as f32 * y_scale).floor() as usize;

            if !written_px[y as usize][x as usize] {
                imgbuf.put_pixel(x, y, *Rgb::from_slice(&rgb_from_range(stdft.dft_data[input_x][stdft.num_freq as usize - input_y - 1].amplitude, max_amplitude)));
                written_px[y as usize][x as usize] = true;
            } else { //if we have already written to this pixel, blend between the two rgb values
                let cur_px = rgb_from_range(stdft.dft_data[input_x][stdft.num_freq as usize - input_y - 1].amplitude, max_amplitude);
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
    [(col1[0] + col2[0]) / 2, (col1[1] + col2[1]) / 2, (col1[2] + col2[2]) / 2] 
}

fn rgb_from_range(amplitude: f32, max_amplitude: f32) -> [u8; 3] {
    let amp_scaled = f32::powf(amplitude / max_amplitude, 0.3);
    let col_val = (amp_scaled * 360. + 200.) % 360.;

    hue_to_rgb(col_val, 0.8, 1.)
}
