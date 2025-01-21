use image::{Pixel, Rgb, RgbImage};

use crate::{hue_to_rgb, audio::ShortTimeDftData};

pub fn generate_img(
    target_dir: String,
    imgx: u32,
    imgy: u32,
    stdft: ShortTimeDftData,
    is_log_scale: bool,
) -> Result<(), image::ImageError> {
    let bottom_freq_loc =
        (((imgy + 1) as f32 / f32::log2(stdft.sample_rate as f32 / 2.)) * f32::log2(128.)) as u32;
    let y_scale = if is_log_scale {
        (imgy - 1 + bottom_freq_loc) as f32 / f32::log2(stdft.sample_rate as f32 / 2.)
    } else {
        (imgy as f32 - 1.) / (stdft.sample_rate as f32 / 2.)
    };
    let get_px_y = |f: f32| -> u32 {
        if is_log_scale {
            let n = y_scale * f32::log2(f);
            if (n - bottom_freq_loc as f32) <= 0. {
                return 0;
            }
            n as u32 - bottom_freq_loc
        } else {
            (y_scale * f) as u32
        }
    };

    let x_step = imgx as f32 / stdft.num_dfts as f32;
    let x_step_rounded = x_step.round() as usize;
    let mut px_x: f32 = 0.;

    let max_amp = find_max_amplitude(&stdft);

    let mut imgbuf = RgbImage::new(imgx, imgy);
    let mut written_pixels = vec![vec![false; imgx as usize]; imgy as usize];

    if x_step < 1. {
        for i in 0..stdft.num_dfts as usize {
            for j in 0..stdft.num_freq as usize - 1 {
                let cur_freq = &stdft.dft_data[0][i][j];

                let px_y = imgy - (get_px_y(stdft.dft_data[0][i][j].frequency) + 1);
                let cur_x = px_x.round() as u32;
                if cur_x as usize >= written_pixels[0].len()
                    || px_y as usize >= written_pixels.len()
                {
                    continue;
                }
                if written_pixels[px_y as usize][cur_x as usize] {
                    let mut px = Rgb(rgb_from_range(cur_freq.amplitude, max_amp));
                    px.blend(&imgbuf.get_pixel(cur_x, px_y).to_rgb());

                    imgbuf.put_pixel(cur_x, px_y, px);
                } else {
                    imgbuf.put_pixel(
                        cur_x,
                        px_y,
                        Rgb(rgb_from_range(cur_freq.amplitude, max_amp)),
                    );
                    written_pixels[px_y as usize][cur_x as usize] = true;
                }
            }
            px_x += x_step;
        }
    } else {
        for i in 0..stdft.num_dfts as usize {
            for j in 0..stdft.num_freq as usize {
                let cur_freq = &stdft.dft_data[0][i][j];

                let px_y = imgy - (get_px_y(stdft.dft_data[0][i][j].frequency) + 1);
                for i in 0..x_step_rounded {
                    let cur_x = (px_x + i as f32).round() as u32;
                    if written_pixels[px_y as usize][cur_x as usize] {
                        let mut px = Rgb(rgb_from_range(cur_freq.amplitude, max_amp));
                        px.blend(&imgbuf.get_pixel(cur_x, px_y).to_rgb());

                        imgbuf.put_pixel(cur_x, px_y, px);
                    } else {
                        imgbuf.put_pixel(
                            cur_x,
                            px_y,
                            Rgb(rgb_from_range(cur_freq.amplitude, max_amp)),
                        );
                        written_pixels[px_y as usize][cur_x as usize] = true;
                    }
                }
            }
            px_x += x_step;
        }
    }

    //fill in the undrawn pixels on y-axis
    let mut last_col = imgbuf.get_pixel(0, imgy - 1).clone();
    for x in 0..imgx as usize {
        for y in (0..imgy as usize).rev() {
            if written_pixels[y][x] {
                last_col = imgbuf.get_pixel(x as u32, y as u32).clone();
                continue;
            }
            imgbuf.put_pixel(x as u32, y as u32, last_col);
        }
    }

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
    let col_val = (amp_scaled * 360. + 250.) % 360.;

    hue_to_rgb(col_val, 0.6, (amp_scaled / 0.3) + 0.7)
}
