use std::{fs, io};

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
