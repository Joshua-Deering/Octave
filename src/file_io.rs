use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Seek, SeekFrom, Write};

use crate::lookup_tables::*;

#[derive(Clone, Copy, Debug)]
pub enum SpeakerPos {
    FrontLeft = 0x1,
    FrontRight = 0x2,
    BackLeft = 0x10,
    BackRight = 0x20,
    FrontLeftOfCenter = 0x40,
    FrontRightOfCenter = 0x80,
    BackCenter = 0x100,
    SideLeft = 0x200,
    SideRight = 0x400,
    TopCenter = 0x800,
    TopFrontLeft = 0x1000,
    TopFrontCenter = 0x2000,
    TopFrontRight = 0x4000,
    TopBackLeft = 0x8000,
    TopBackCenter = 0x10000,
    TopBackRight = 0x20000,
    Reserved = 0x80000000,
}

impl SpeakerPos {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
    
    pub fn short_name(&self) -> String {
        match *self {
            Self::FrontLeft => "FL",
            Self::FrontRight => "FR",
            Self::BackLeft => "BL",
            Self::BackRight => "BR",
            Self::FrontLeftOfCenter => "FLC",
            Self::FrontRightOfCenter => "FRC",
            Self::BackCenter => "BC",
            Self::SideLeft => "SL",
            Self::SideRight => "SR",
            Self::TopCenter => "TC",
            Self::TopFrontLeft => "TFL",
            Self::TopFrontCenter => "TFC",
            Self::TopFrontRight => "TFR",
            Self::TopBackLeft => "TBL",
            Self::TopBackCenter => "TBC",
            Self::TopBackRight => "TBR",
            Self::Reserved => "RES",
        }.to_string()
    }
    
    pub fn std_location(&self) -> u32 {
        match *self {
            Self::FrontLeft => 0,
            Self::FrontRight => 1,
            Self::BackLeft => 2,
            Self::BackRight => 3,
            Self::FrontLeftOfCenter => 4,
            Self::FrontRightOfCenter => 5,
            Self::BackCenter => 6,
            Self::SideLeft => 7,
            Self::SideRight => 8,
            Self::TopCenter => 9,
            Self::TopFrontLeft => 10,
            Self::TopFrontCenter => 11,
            Self::TopFrontRight => 12,
            Self::TopBackLeft => 13,
            Self::TopBackCenter => 14,
            Self::TopBackRight => 15,
            Self::Reserved => 16,
        }
    }
}

impl From<u32> for SpeakerPos {
    fn from(value: u32) -> Self {
        match value {
            0x1 => Self::FrontLeft,
            0x2 => Self::FrontRight,
            0x10 => Self::BackLeft,
            0x20 => Self::BackRight,
            0x40 => Self::FrontLeftOfCenter,
            0x80 => Self::FrontRightOfCenter,
            0x100 => Self::BackCenter,
            0x200 => Self::SideLeft,
            0x400 => Self::SideRight,
            0x800 => Self::TopCenter,
            0x1000 => Self::TopFrontLeft,
            0x2000 => Self::TopFrontCenter,
            0x4000 => Self::TopFrontRight,
            0x8000 => Self::TopBackLeft,
            0x10000 => Self::TopBackCenter,
            0x20000 => Self::TopBackRight,
            _ => Self::Reserved,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WavInfo {
    pub sample_type: u8,
    pub sample_type_str: String,
    pub channels: u8,
    pub sample_rate: u32,
    pub data_rate: u32,
    pub data_block_size: u32,
    pub bit_depth: u32,
    pub byte_depth: u32,
    pub chunks: HashMap<String, (u64, u32)>, // {chunk_name: (position, chunk_size)}
    pub file_size: u32,
    pub audio_duration: f32,
    pub channel_map: Vec<(u8, SpeakerPos)>,
}

impl WavInfo {
    pub fn new(
        sample_type: u8,
        channels: u8,
        sample_rate: u32,
        bit_depth: u32,
        file_size: u32,
        chunks: HashMap<String, (u64, u32)>,
        channel_map: Vec<(u8, SpeakerPos)>,
    ) -> Self {
        let byte_depth = bit_depth / 8;
        let audio_duration = chunks.get("data").unwrap().1 as f32
            / (byte_depth * channels as u32 * sample_rate) as f32;

        let sample_type_str = match sample_type {
            1 => "PCM".to_string(),
            3 => "IEEE Float".to_string(),
            6 => "8-bit ITU-T G.711 A-law".to_string(),
            7 => "8-bit ITU-T G.711 µ-law".to_string(),
            254 => "Wav Extensible Format".to_string(),
            _ => "Unsupported".to_string(),
        };
        WavInfo {
            sample_type,
            sample_type_str,
            channels,
            sample_rate,
            data_rate: sample_rate * byte_depth * channels as u32,
            data_block_size: byte_depth * channels as u32,
            bit_depth,
            byte_depth: bit_depth / 8,
            chunks,
            file_size,
            audio_duration,
            channel_map
        }
    }
}

impl fmt::Display for WavInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sample_type = match self.sample_type {
            1 => "PCM",
            3 => "IEEE Float",
            6 => "8-bit ITU-T G.711 A-law",
            7 => "8-bit ITU-T G.711 µ-law",
            254 => "Wav Extensible Format",
            _ => "Unsupported",
        };
        f.write_str(format!("Wav Info:\nSample Type: {}\nSample rate: {} Hz\nSample size: {} bit\nBlock Size: {} bytes\nData Rate: {} bytes/sec\nChannels: {}\nDuration: {} secs\nFile size: {} bytes", sample_type, self.sample_rate, self.bit_depth, self.data_block_size, self.data_rate, self.channels, self.audio_duration, self.file_size).as_str())
    }
}

pub fn read_wav_sample_rate(f: String) -> u32 {
    let mut r = BufReader::new(File::open(format!("./res/audio/{}", f)).unwrap());
    r.seek_relative(24).unwrap();
    read_le_uint(&mut r, 4)
}

pub fn read_wav_meta(f: &mut BufReader<File>) -> WavInfo {
    f.seek(SeekFrom::Start(0)).unwrap();

    f.seek_relative(4).unwrap();
    let f_size: u32 = read_le_uint(f, 4);
    f.seek_relative(12).unwrap();
    let mut fmt_code = read_le_uint(f, 2) as u8;
    let channels = read_le_uint(f, 2) as u8;
    let sample_rate = read_le_uint(f, 4);
    let _data_rate = read_le_uint(f, 4); // not used since WavInfo::new calculates these
    let _data_block_size = read_le_uint(f, 2); // not used since WavInfo::new calculates these
    let bit_depth = read_le_uint(f, 2);

    //only really reading this stuff for potential future use, its not used at the moment
    let ext_size: u8;
    let _v_bits_per_sample: u8; // information about the precision of IEEE floats in file
    let _subformat: String;
    
    // mapping from channels to physical speakers
    let mut channel_mask_num: u32 = 0xFFFFFFFF; // default is all 1s, for a direct mapping

    // bit depths of 8 or less in PCM use offset binary instead of 
    // 2's complement which idk how to parse so ..
    if fmt_code == 1 && bit_depth <= 8 {
        panic!("Unsupported bit depth (bit depth 8 or lower)");
    }

    match fmt_code {
        1 => {}, //no extra parsing needed for PCM data
        3 | 6 | 7 | 0xFE => { //non-PCM data should always have the ext_size field
            ext_size = read_le_uint(f, 2) as u8;
            if ext_size > 0 {
                _v_bits_per_sample = read_le_uint(f, 2) as u8;
                channel_mask_num = read_le_uint(f, 4);
                if channel_mask_num == 0 {
                    // channel mask of 0 actually indicates the default mapping
                    channel_mask_num = 0xFFFFFFFF;
                }
                // files with extension data store the actual format code
                // later in the file so now we read it in again ...
                fmt_code = read_le_uint(f, 2) as u8;
            }
        },
        _ => {
            panic!("Unknown format code!");
        }
    }

    //assign the channel map
    let mut cur_map = channel_mask_num;
    let mut cur_ch = 0;
    let mut channel_map = Vec::new();
    let mut i = 0;
    while cur_ch < channels && i < 32 {
        if cur_map & 1 != 0 {
            channel_map.push((cur_ch, SpeakerPos::from(2f32.powi(i as i32) as u32))); 
            cur_ch += 1;
        }
        cur_map >>= 1;
        i += 1;
    }

    f.seek(SeekFrom::Start(0)).unwrap();
    f.seek_relative(12).unwrap(); //skip 'RIFF' and 'WAVE' tags
    let mut chunks: HashMap<String, (u64, u32)> = HashMap::new();

    while f.stream_position().unwrap() < f_size as u64 {
        let title = read_str(f, 4);
        let size = read_le_uint(f, 4);
        chunks.insert(title, (f.stream_position().unwrap(), size));
        f.seek_relative(size as i64).unwrap();
    }

    f.seek(SeekFrom::Start(0)).unwrap();
    WavInfo::new(fmt_code, channels, sample_rate, bit_depth, f_size, chunks, channel_map)
}

pub fn read_data(
    f: &mut BufReader<File>,
    file_info: &WavInfo,
    start_time: f32,
    duration: f32,
) -> Option<Vec<Vec<f32>>> {
    let sample_size = (file_info.bit_depth / 8) as usize;
    let channels = file_info.channels as usize;
    let mut samples_per_channel = (duration * file_info.sample_rate as f32) as usize;
    let total_samples = samples_per_channel * channels;

    f.seek(SeekFrom::Start(
        file_info.chunks.get("data".into()).unwrap().0,
    ))
    .unwrap();
    let file_start_pos = (start_time * file_info.sample_rate as f32 * file_info.channels as f32) as i64;
    //skip to start_pos in the file
    f.seek_relative(file_start_pos)
        .unwrap();

    let mut data: Vec<u8>;
    let (data_start, data_size) = *file_info.chunks.get("data").unwrap();
    //either read the amount of data requested, or read to EOF
    if f.stream_position().unwrap() + total_samples as u64 * sample_size as u64 > (data_start + data_size as u64) {
        data = vec![0; (data_start + data_size as u64) as usize - f.stream_position().unwrap() as usize];
        f.read_exact(&mut data).unwrap();
        samples_per_channel = data.len() / channels / sample_size;
    } else {
        data = vec![0; total_samples * sample_size];
        f.read_exact(&mut data).unwrap();
    }

    let mut output = vec![vec![0.; samples_per_channel]; channels];

    match file_info.sample_type {
        1 => { //Regular PCM data
            match file_info.bit_depth {
                16 => {
                    for i in 0..samples_per_channel {
                        let idx = i * sample_size * channels;
                        for j in 0..channels {
                            let ch_offset = j * sample_size + idx;
                            output[j][i] = (((data[ch_offset + 1] as i32) << 24
                                | (data[ch_offset] as i32) << 16)
                                >> 16) as f32;
                            output[j][i] /= if output[j][i] < 0. { PCM_16BIT_NEG_MAX } else { PCM_16BIT_POS_MAX };
                        }
                    }
                }

                24 => {
                    for i in 0..samples_per_channel {
                        let idx = i * sample_size * channels;
                        for j in 0..channels {
                            let ch_idx = j * sample_size + idx;
                            output[j][i] = (((data[ch_idx + 2] as i32) << 24
                                | (data[ch_idx + 1] as i32) << 16
                                | (data[ch_idx] as i32) << 8)
                                >> 8) as f32;
                            output[j][i] /= if output[j][i] < 0. { PCM_24BIT_NEG_MAX } else { PCM_24BIT_POS_MAX };
                        }
                    }
                }

                32 => {
                    for i in 0..samples_per_channel {
                        let idx = i * sample_size * channels;
                        for j in 0..channels {
                            let ch_offset = j * sample_size + idx;
                            output[j][j] = (((data[ch_offset + 3] as i32) << 24
                                | (data[ch_offset + 2] as i32) << 16
                                | (data[ch_offset + 1] as i32) << 8)
                                | (data[ch_offset] as i32)) as f32;
                            output[j][i] /= if output[j][i] < 0. { PCM_32BIT_NEG_MAX } else { PCM_32BIT_POS_MAX };
                        }
                    }
                }

                _ => return None,
            }
        },
        3 => { // IEEE float data
            // Wav supports 64-bit float so may implement this in future but it is very uncommon
            if file_info.bit_depth > 32 {
                panic!("Unsupported bit depth!");
            }
            for i in 0..samples_per_channel {
                let idx = i * sample_size * channels;
                for j in 0..channels {
                    let ch_idx = j * sample_size + idx;
                    let dat: &[u8] = &data[ch_idx..ch_idx+sample_size];
                    output[j][i] = f32::from_le_bytes(dat.try_into().unwrap());
                }
            }
        }
        6 => {
            for i in 0..samples_per_channel {
                let idx = i * sample_size * channels;
                for j in 0..channels {
                    let ch_idx = j * sample_size + idx;
                    output[j][i] = ALAW_TO_PCM[data[ch_idx] as usize];
                }
            }
        }
        7 => {
            for i in 0..samples_per_channel {
                let idx = i * sample_size * channels;
                for j in 0..channels {
                    let ch_idx = j * sample_size + idx;
                    output[j][i] = ULAW_TO_PCM[data[ch_idx] as usize];
                }
            }
        }
        _ => {
            panic!("Unsupported file format!");
        },
    }


    Some(output)
}

pub struct WavWriteInfo {
    pub sample_type: u8,
    pub channels: u8,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channel_mapping: Vec<(u8, SpeakerPos)>
}

pub fn write_wav_file(target_file: String, target_wav_format: &WavWriteInfo, samples: &Vec<Vec<f32>>) -> std::io::Result<()> {
    let sample_rate = target_wav_format.sample_rate;
    let sample_type = target_wav_format.sample_type;
    let bit_depth = target_wav_format.bit_depth;
    let channels = target_wav_format.channels;
    let channel_mapping = target_wav_format.channel_mapping.clone();
    let samples_per_channel = samples[0].len() as u32;

    //check input validity
    if samples.len() != channels as usize {return Err(Error::new(ErrorKind::InvalidInput, "Wav Format must match given samples!"));}
    for c in 0..samples.len() {
        if samples[c].len() != samples_per_channel as usize {
            return Err(Error::new(ErrorKind::InvalidInput, "All channels must have the same number of samples!"));
        }
    }

    let mut is_std_channel_map = true;
    for i in 0..channels{
        if channel_mapping[i as usize].0 != i {
            is_std_channel_map = false;
        }
    }
    let channel_mask: u32 = get_channel_mask(&channel_mapping);

    let is_extended_fmt = sample_type != 1 || channels > 2 || !is_std_channel_map;
    let ext_size: u16 = if is_extended_fmt && (sample_type == 1 || !is_std_channel_map) {
        //need to specify subformat or channel mapping
        22u16
    } else { 0u16 };

    let has_fact_chunk = sample_type == 6 || sample_type == 7;

    let data_block_size: u16 = (bit_depth / 8) * channels as u16;
    let data_rate: u32 = sample_rate * data_block_size as u32;

    let fmt_chunk_size: u32 = 16 + if is_extended_fmt {2 + ext_size as u32} else {0};
    let data_chunk_size: u32 = data_block_size as u32 * samples_per_channel;

    let file_size: u32 = 4 + fmt_chunk_size as u32 + data_chunk_size + (if has_fact_chunk {4} else {0});

    let mut file = BufWriter::new(File::create_new(format!("./res/audio/{}", target_file))?);

    // Write RIFF Header
    file.write_all("RIFF".as_bytes())?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all("WAVE".as_bytes())?;

    // Write fmt header
    file.write_all("fmt ".as_bytes())?;
    file.write_all(&fmt_chunk_size.to_le_bytes())?;

    // if the extended chunk exists, write this as "Wav Extensible Format" (254)
    if ext_size > 0 {
        file.write_all(&254u16.to_le_bytes())?;
    } else { //otherwise just write the sample type here
        file.write_all(&(sample_type as u16).to_le_bytes())?;
    }
    file.write_all(&(channels as u16).to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&data_rate.to_le_bytes())?;
    file.write_all(&data_block_size.to_le_bytes())?;
    file.write_all(&bit_depth.to_le_bytes())?;

    // write the ext chunk if needed
    if fmt_chunk_size > 16 {
        file.write_all(&ext_size.to_le_bytes())?;

        if fmt_chunk_size > 18 {
            //this is wValidBitsPerSample, which is really just redundant
            file.write_all(&bit_depth.to_le_bytes())?;
            // channel mask
            file.write_all(&channel_mask.to_le_bytes())?;

            //this is SubFormat, which includes the sample type as well as a fixed string 
            // "\x00\x00\x00\x00\x10\x00\x80\x00\x00\xAA\x00\x38\x9B\x71"
            file.write_all(&(sample_type as u16).to_le_bytes())?;
            file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71])?;
        }
    }

    // write fact chunk if needed
    if has_fact_chunk {
        file.write_all("fact".as_bytes())?;
        // fact chunk will always have a size of 4
        file.write_all(&4u32.to_le_bytes())?;
        file.write_all(&samples_per_channel.to_le_bytes())?;
    }

    // data chunk
    file.write_all("data".as_bytes())?;
    file.write_all(&data_chunk_size.to_le_bytes())?;
    match sample_type {
        1 => {
            match bit_depth {
                16 => {
                    for i in 0..samples[0].len() {
                        for c in 0..samples.len() {
                            let sample_scaled: i16 = (samples[c][i] * if samples[c][i] > 0. {PCM_16BIT_POS_MAX} else {PCM_16BIT_NEG_MAX}) as i16;
                            file.write_all(&sample_scaled.to_le_bytes())?;
                        }
                    }
                }
                24 => {
                    for i in 0..samples[0].len() {
                        for c in 0..samples.len() {
                            let sample_scaled: i32 = (samples[c][i] * if samples[c][i] > 0. {PCM_24BIT_POS_MAX} else {PCM_24BIT_NEG_MAX}) as i32;
                            file.write_all(&sample_scaled.to_le_bytes()[0..3])?;
                        }
                    }
                }
                32 => {
                    for i in 0..samples[0].len() {
                        for c in 0..samples.len() {
                            let sample_scaled: i32 = (samples[c][i] * if samples[c][i] > 0. {PCM_32BIT_POS_MAX} else {PCM_32BIT_NEG_MAX}) as i32;
                            file.write_all(&sample_scaled.to_le_bytes())?;
                        }
                    }
                }
                _ => {

                }
            }
        },
        3 => {
            // idk how to parse f32s into IEEE floats of 16 or 24 bits so..
            return Err(Error::new(ErrorKind::Unsupported, "Unsupported sample format!"));
        }
        _ => {

        }
    }

    file.flush()?;

    Ok(())
}

fn get_channel_mask(mapping: &Vec<(u8, SpeakerPos)>) -> u32 {
    let mut mask_num: u32 = 0;
    for (_, pos) in mapping {
        mask_num |= *pos as u32;
    }

    mask_num
}

pub fn read_data_interleaved_unchecked<T: Read>(
    f: &mut BufReader<T>,
    file_info: &WavInfo,
    data_len: usize,
) -> Vec<f32> {
    let byte_depth = file_info.byte_depth as usize;
    let mut data = vec![0; data_len * byte_depth];

    match f.read_exact(&mut data) {
        Ok(_) => (),
        Err(err) => {
            panic!("Unexpected error while reading file: {}", err)
        }
    };
    let mut out_data = vec![0.; data_len];

    match byte_depth {
        2 => {
            for j in 0..out_data.len() {
                let idx = j * byte_depth;
                out_data[j] = (((data[idx + 1] as i32) << 24 | (data[idx] as i32) << 16) >> 16)
                    as f32
                    / 0xFFFF as f32;
            }
        }

        3 => {
            for j in 0..out_data.len() {
                let idx = j * byte_depth;
                out_data[j] = (((data[idx + 2] as i32) << 24
                    | (data[idx + 1] as i32) << 16
                    | (data[idx] as i32) << 8)
                    >> 8) as f32
                    / 0xFFFFFF as f32;
            }
        }

        4 => {
            for j in 0..out_data.len() {
                let idx = j * byte_depth;
                out_data[j] = (((data[idx + 3] as i32) << 24
                    | (data[idx + 2] as i32) << 16
                    | (data[idx + 1] as i32) << 8)
                    | (data[idx] as i32)) as f32
                    / (i32::MAX) as f32;
            }
        }

        _ => panic!("Unsupported byte depth"),
    }

    out_data
}

pub fn read_str(f: &mut BufReader<File>, bytes: usize) -> String {
    let mut buf = vec![0; bytes];
    let _ = f.read_exact(&mut buf);
    buf.iter().map(|&e| e as char).collect::<String>()
}

pub fn read_le_uint(f: &mut BufReader<File>, bytes: usize) -> u32 {
    if bytes > 4 {
        return 0;
    }
    let mut buf = vec![0 as u8; bytes];
    let _ = f.read_exact(&mut buf);
    buf_to_int(&mut buf, bytes)
}

pub fn buf_to_int(buf: &[u8], bytes: usize) -> u32 {
    let mut out: u32 = 0;
    for i in (1..bytes + 1).rev() {
        out |= (buf[i - 1] as u32) << ((i - 1) * 8);
    }
    out
}
