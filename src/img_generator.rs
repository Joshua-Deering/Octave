use fastapprox::fast::log2;
use image::{RgbImage, Rgb};

use crate::ShortTimeDftData;

pub fn generate_img(target_dir: &str, imgx: u32, imgy: u32, stdft: ShortTimeDftData, is_log_scale: bool) -> Result<(), image::ImageError> {
    let bottom_freq_loc = (((imgy + 1) as f32 / log2(stdft.sample_rate as f32 / 2.)) * log2(128.)) as u32;
    let y_scale = if is_log_scale {(imgy - 1 + bottom_freq_loc) as f32 / log2(stdft.sample_rate as f32 / 2.)} else {imgy as f32 / (stdft.sample_rate as f32 / 2.)};
    let get_px_y = | f: f32 | -> u32 {
        if is_log_scale {
            (y_scale * log2(f)) as u32 - bottom_freq_loc
        } else {
            (y_scale * f) as u32
        }
    };
    //TODO fix so that if this is < 0, it does something useful
    let x_step = imgx / stdft.num_dfts;
    let mut px_x = 0;
    let mut next_px_y = 0;

    let max_amp = find_max_amplitude(&stdft);

    let mut imgbuf = RgbImage::new(imgx, imgy);
    for i in 0..stdft.num_dfts {
        for j in 0..stdft.num_freq-1 {
            let cur_freq = &stdft.dft_data[0][i as usize][j as usize];
            if cur_freq.frequency <= 27. {
                continue;
            }
            let px_y = next_px_y;
            next_px_y = get_px_y(stdft.dft_data[0][i as usize][j as usize + 1].frequency);
            if next_px_y > px_y + 1 {
                for y in 0..next_px_y-px_y {
                    for i in 0..x_step {
                        imgbuf.put_pixel(px_x + i, imgy - (px_y + y + 1), Rgb(rgb_from_range(cur_freq.amplitude, max_amp)));
                    }
                }
            } else {
                for i in 0..x_step {
                    imgbuf.put_pixel(px_x + i, imgy - (px_y + 1), Rgb(rgb_from_range(cur_freq.amplitude, max_amp)));
                }
            }
        }
        px_x += x_step;
    }

    ////draw lines at powers of 2 starting from 32
    //for i in 5..15 {
    //    let freq = u32::pow(2, i);
    //    let y = get_px_y(freq as f32);
    //    for x in 0..imgx {
    //        imgbuf.put_pixel(x, imgy - 1 - y, Rgb([255, 255, 255]));
    //    }
    //}

    imgbuf.save(target_dir)
}

fn find_max_amplitude(stdft: &ShortTimeDftData) -> f32 {
    let mut max = 0.;
    for ch in &stdft.dft_data {
        for dft in ch {
            for freq_data in dft {
                if freq_data.amplitude.abs() > max {
                    max = freq_data.amplitude.abs();
                }
            }
        }
    }
    max
}

fn rgb_from_range(amplitude: f32, max_amplitude: f32) -> [u8; 3] {
    let amp_scaled = amplitude / max_amplitude;
    let col_val = (amp_scaled * 255. * 3.) as u32; //3*256 bc 256 for each color channel
    let r = if col_val > 255 {255} else {col_val as u8};
    let g = if r < 255 {0} else {(col_val % 256) as u8};
    let b = if g < 255 {0} else {(col_val % 512) as u8};
    [r, g, b]
}
