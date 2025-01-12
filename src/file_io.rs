use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::collections::HashMap;
use std::fmt;
use std::error;
#[allow(unused)]

pub struct WavInfo {
    pub sample_type: u8,
    pub channels: u8,
    pub sample_rate: u32,
    pub data_rate: u32,
    pub data_block_size: u32,
    pub bit_depth: u32,
    pub chunks: HashMap<String, (u64, u32)>,
    pub file_size: u32,
}

impl WavInfo {
    pub fn new(sample_type: u8, channels: u8, sample_rate: u32, bit_depth: u32, file_size: u32, chunks: HashMap<String, (u64, u32)>) -> Self {
        let byte_depth = bit_depth / 8;
        WavInfo {
            sample_type,
            channels,
            sample_rate,
            data_rate: sample_rate * byte_depth * channels as u32,
            data_block_size: byte_depth * channels as u32,
            bit_depth,
            chunks,
            file_size
        }
    }
}

impl fmt::Display for WavInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("Sample rate: {}, Bit depth: {}, Channels: {}, File size: {}", self.sample_rate, self.bit_depth, self.channels, self.file_size).as_str())
    }
}

pub fn read_wav_meta(f: &mut BufReader<File>) -> WavInfo {
    f.seek(SeekFrom::Start(0)).unwrap();
    
    f.seek_relative(4).unwrap();
    let f_size: u32 = read_uint(f, 4);
    f.seek_relative(12).unwrap();
    let fmt_code = read_uint(f, 2) as u8;
    let channels = read_uint(f, 2) as u8;
    let sample_rate = read_uint(f, 4);
    let _data_rate = read_uint(f, 4); // are not used since WavInfo::new calculates these
    let _data_block_size = read_uint(f, 2); // are not used since WavInfo::new calculates these
    let bit_depth = read_uint(f, 2);

    f.seek(SeekFrom::Start(0)).unwrap();
    let mut chunks: HashMap<String, (u64, u32)> = HashMap::new();
    f.seek_relative(12).unwrap(); //skip 'RIFF' and 'WAVE' tags

    while f.stream_position().unwrap() < f_size as u64 {
        let title = read_str(f, 4);
        let size = read_uint(f, 4);
        chunks.insert(title, (f.stream_position().unwrap(), size));
        f.seek_relative(size as i64).unwrap();
    }
    
    f.seek(SeekFrom::Start(0)).unwrap();
    WavInfo::new(fmt_code, channels, sample_rate, bit_depth, f_size, chunks)
}

pub fn read_data(f: &mut BufReader<File>, file_info: WavInfo, start_pos: f32, duration: f32, ) -> Option<Vec<Vec<f32>>> {
    let sample_size = (file_info.bit_depth/8) as usize;
    let channels = file_info.channels as usize;
    let samples_per_channel = (duration * file_info.sample_rate as f32) as usize;
    let total_samples =  samples_per_channel * channels;

    f.seek(SeekFrom::Start(file_info.chunks.get("data".into()).unwrap().0)).unwrap();
    //skip to start_pos in the file
    f.seek_relative((start_pos * file_info.sample_rate as f32 * file_info.channels as f32) as i64).unwrap();

    let mut data = vec![0; total_samples * sample_size];
    f.read_exact(&mut data).unwrap();

    let mut output = vec![vec![0.; samples_per_channel]; channels];
    
    match file_info.bit_depth {
        16 => {
            for i in 0..samples_per_channel {
                let idx = i*sample_size*channels;
                for j in 0..channels {
                    let ch_offset = j*sample_size + idx;
                    output[j][i] = (((data[ch_offset + 1] as i32) << 24 | (data[ch_offset] as i32) << 16) >> 16) as f32 / 0xFFFF as f32;
                }
            }
        },

        24 => {
            for i in 0..samples_per_channel {
                let idx = i*sample_size*channels;
                for j in 0..channels {
                    let ch_idx = j*sample_size + idx;
                    output[j][i] = (((data[ch_idx + 2] as i32) << 24 | (data[ch_idx + 1] as i32) << 16 | (data[ch_idx] as i32) << 8) >> 8) as f32 / 0xFFFFFF as f32;
                }
            }
        },
        
        32 => {
            for i in 0..samples_per_channel {
                let idx = i*sample_size*channels;
                for j in 0..channels {
                    let ch_offset = j*sample_size + idx;
                    output[j][j] = (((data[ch_offset + 3] as i32) << 24 | (data[ch_offset + 2] as i32) << 16 | (data[ch_offset + 1] as i32) << 8) | (data[ch_offset] as i32)) as f32 / (i32::MAX) as f32;
                }
            }
        },

        _ => return None
    }
    
    Some(output)
}

pub fn read_data_to_arr(f: &mut BufReader<File>, bit_depth: usize, output: &mut [f32]) {
    let byte_depth = bit_depth / 8;
    let mut data = vec![0; output.len() * byte_depth];
    match f.read_exact(&mut data) {
        Ok(_) => (),
        Err(_) => (),
    };

    for i in 0..output.len() {
        let idx = i*byte_depth;
        output[i] = (((data[idx + 2] as i32) << 24 | (data[idx + 1] as i32) << 16 | (data[idx] as i32) << 8) >> 8) as f32 / 0xFFFFFF as f32;
    }
}

pub fn read_channel_to_arr(f: &mut BufReader<File>, bit_depth: usize, output: &mut [f32], channel_count: usize, channel: usize) {
    let byte_depth = bit_depth / 8;
    let mut data = vec![0; output.len() * byte_depth * channel_count];
    match f.read_exact(&mut data) {
        Ok(_) => (),
        Err(_) => (),
    };

    for i in (0..output.len()*2).step_by(channel_count) {
        let idx = i*byte_depth + channel;
        output[i/2] = (((data[idx + 2] as i32) << 24 | (data[idx + 1] as i32) << 16 | (data[idx] as i32) << 8) >> 8) as f32 / 0xFFFFFF as f32;
    }
}


pub fn read_str(f: &mut BufReader<File>, bytes: usize) -> String {
    let mut buf = vec![0; bytes];
    let _ = f.read_exact(&mut buf);
    buf.iter().map(|&e| e as char).collect::<String>()
}

pub fn read_uint(f: &mut BufReader<File>, bytes: usize) -> u32 {
    if bytes > 4 {
        return 0;
    }
    let mut buf = vec![0 as u8; bytes];
    let _ = f.read_exact(&mut buf);
    buf_to_int(&mut buf, bytes)
}

pub fn buf_to_int(buf: &[u8], bytes: usize) -> u32 {
    let mut out: u32 = 0;
    for i in (1..bytes+1).rev() {
        out |= (buf[i-1] as u32) << ((i-1)*8);
    }
    out
}
