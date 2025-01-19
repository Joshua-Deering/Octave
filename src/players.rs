use cpal::Data;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

use crate::audio::{do_fourier_transform, do_short_time_fourier_transform};
use crate::audio::{FreqData, ShortTimeDftData, WindowFunction};
use crate::file_io::WavInfo;
use crate::{read_data_interleaved_unchecked, read_wav_meta};

#[allow(unused)]
pub struct SignalPlayer {
    pub samples: Vec<Vec<f32>>,
    pub sample_rate: u32,
    channels: usize,
    duration: f32,
    pos: usize,
}

#[allow(unused)]
impl SignalPlayer {
    pub fn new(samples: Vec<Vec<f32>>, sample_rate: u32, channels: usize) -> Self {
        let duration = samples.len() as f32 / sample_rate as f32;
        Self {
            samples,
            sample_rate,
            channels,
            duration,
            pos: 0,
        }
    }
    pub fn next_chunk(&mut self, data: &mut Data) {
        let dat_slice = data.as_slice_mut().unwrap();
        let end = self.pos + (dat_slice.len() / self.channels);
        if end >= self.samples[0].len() {
            return;
        }
        for i in self.pos..end {
            for c in 0..self.channels {
                dat_slice[(i - self.pos) * self.channels + c] = self.samples[c][i];
            }
        }
        self.pos += data.len() / self.channels;
    }

    pub fn do_short_time_fourier_transform(
        &self,
        window_size: f32,
        overlap: f32,
        window_func: WindowFunction,
    ) -> ShortTimeDftData {
        let mut dft_data = vec![vec![]; self.channels];
        for i in 0..self.channels {
            dft_data[i] = do_short_time_fourier_transform(
                &self.samples[i],
                self.sample_rate,
                window_size,
                overlap,
                window_func,
            );
        }
        let ch = dft_data.len() as u32;
        let dfts = dft_data[0].len() as u32;
        let freqs = dft_data[0][0].len() as u32;
        ShortTimeDftData::new(
            dft_data,
            window_func,
            overlap,
            ch,
            dfts,
            freqs,
            self.sample_rate,
        )
    }

    pub fn do_fourier_transform(&self) -> Vec<Vec<FreqData>> {
        let mut out = vec![vec![]; self.channels];
        for i in 0..self.channels {
            out[i] = do_fourier_transform(&self.samples[i], self.sample_rate);
        }
        out
    }

    pub fn do_fourier_transform_on_channel(&self, channel: usize) -> Vec<FreqData> {
        if channel > self.channels {
            return vec![];
        }
        do_fourier_transform(&self.samples[channel - 1], self.sample_rate)
    }
}

pub struct FilePlayer {
    pub file_meta: WavInfo,
    pub finished: bool,
    reader: BufReader<File>,
    pos: usize,
    end_pos: usize,
}

impl FilePlayer {
    pub fn new(file_path: String) -> Self {
        let mut reader = BufReader::new(File::open(format!("./res/audio/{}", file_path)).unwrap());
        let file_meta = read_wav_meta(&mut reader);
        //advance reader to beginning of audio data
        reader
            .seek(SeekFrom::Start(file_meta.chunks.get("data").unwrap().0))
            .unwrap();
        let end_pos = (file_meta.file_size / (file_meta.bit_depth / 8)) as usize;

        Self {
            file_meta,
            finished: false,
            reader,
            pos: 0,
            end_pos,
        }
    }

    pub fn next_chunk(&mut self, data: &mut Data) {
        let dat_slice = data.as_slice_mut().unwrap();
        if self.pos + dat_slice.len() >= self.end_pos {
            self.finished = true;
            return;
        }

        let data =
            read_data_interleaved_unchecked(&mut self.reader, &self.file_meta, dat_slice.len());
        dat_slice[..].clone_from_slice(&data);

        self.pos += data.len();
    }
}
