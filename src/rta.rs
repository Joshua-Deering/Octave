use std::sync::{Arc, Mutex};

use cpal::{default_host, traits::{DeviceTrait, HostTrait, StreamTrait}, InputCallbackInfo, SampleRate, Stream, SupportedStreamConfigRange};

use crate::{circular_buffer::CircularBuffer, fft::Fft};
use crate::audio::{FreqData, WindowFunction};

pub struct RTA {
    cached_samples: CircularBuffer,
    fft: Fft,
}

impl RTA {
    pub fn new(num_samples: usize, sample_rate: u32) -> Self {
        let fft = Fft::new(sample_rate, num_samples, WindowFunction::Square);
        Self {
            cached_samples: CircularBuffer::new(num_samples),
            fft,
        }
    }

    pub fn update(&mut self, data: &[f32]) {
        self.cached_samples.append_slice(data);
    }

    pub fn get_fft(&self) -> Vec<FreqData> {
        self.fft.process(self.cached_samples.get_ordered().as_slice())
    }
}

pub struct ExternalRta {
    stream: Stream,
    rta: Arc<Mutex<RTA>>,
    pub cache_size: usize,
}

impl ExternalRta {
    pub fn new(cache_size: usize) -> Self {

        let host = default_host();
        let device = host.default_input_device().expect("No input device available!");
        let supported_config_range = device.supported_input_configs().expect("Error querying input configs!");
        let supported_configs = supported_config_range.into_iter().collect::<Vec<SupportedStreamConfigRange>>();

        //find a sample rate at or under 48kHz
        let mut config_opt = None;
        for c in supported_configs.iter().rev() {
            if c.max_sample_rate() == SampleRate(48000) || c.max_sample_rate() < SampleRate(48000) {
                config_opt = Some(c.with_max_sample_rate().config());
                break;
            }
        }
        if config_opt == None {
            panic!("No supported input configs!");
        }

        let config = config_opt.unwrap();

        let rta = Arc::new(Mutex::new(RTA::new(cache_size, config.sample_rate.0)));
        
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
