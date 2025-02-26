use std::sync::{Arc, Mutex};

use cpal::{default_host, traits::{DeviceTrait, HostTrait, StreamTrait}, InputCallbackInfo, SampleRate, Stream};

use crate::fft::Fft;
use crate::audio::{FreqData, WindowFunction};

pub struct RTA {
    cache_size: usize,
    cached_samples: Vec<f32>,
    samples_start: usize,
    fft: Fft,
}

impl RTA {
    pub fn new(num_samples: usize) -> Self {
        let fft = Fft::new(48000, num_samples, WindowFunction::Square);
        Self {
            cache_size: num_samples,
            cached_samples: vec![0.; num_samples],
            samples_start: 0,
            fft,
        }
    }

    pub fn update(&mut self, data: &[f32]) {
        if data.len() > self.cache_size {
            panic!("Cannot have larger data chunks than cache size!");
        }

        let start_idx = (self.samples_start + self.cache_size - data.len()) % self.cache_size;

        for i in 0..data.len() {
            self.cached_samples[(start_idx + i) % self.cache_size] = data[i];
        }
        self.samples_start += data.len();
        self.samples_start %= self.cache_size;
    }

    pub fn get_fft(&self) -> Vec<FreqData> {
        let mut in_order_buf = Vec::with_capacity(self.cache_size);
        for i in 0..self.cache_size {
            in_order_buf.push(self.cached_samples[(self.samples_start + i) % self.cache_size]);
        }
        self.fft.process(&in_order_buf)
    }
}

pub struct ExternalRta {
    stream: Stream,
    rta: Arc<Mutex<RTA>>,
    pub cache_size: usize,
}

impl ExternalRta {
    pub fn new(cache_size: usize) -> Self {
        let rta = Arc::new(Mutex::new(RTA::new(cache_size)));

        let host = default_host();
        let device = host.default_input_device().expect("No input device available!");
        let mut supported_config_range = device.supported_input_configs().expect("Error querying input configs!");

        let config = supported_config_range
            .find(|e| e.max_sample_rate() == SampleRate(48000)).unwrap()
            .with_max_sample_rate().config();
        
        let rta_copy = Arc::clone(&rta);
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &InputCallbackInfo| {
                rta_copy.lock().unwrap().update(data);
            },
            move |err| {
                panic!("something went bad {}", err);
            }, 
            None
        ).unwrap();
        stream.pause().unwrap();

        Self {
            stream,
            rta,
            cache_size,
        }
    }

    pub fn get_fft(&self) -> Vec<FreqData> {
        return self.rta.lock().unwrap().get_fft();
    }

    pub fn start(&mut self) {
        self.stream.play().unwrap();
    }

    pub fn stop(&mut self) {
        self.stream.pause().unwrap();
    }
}
