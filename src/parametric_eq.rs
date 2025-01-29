use std::f32::consts::PI;

pub struct ParametricEq {
    nodes: Vec<Biquad>,
    sample_rate: u32,
}

impl ParametricEq {
    pub fn new(nodes: Vec<Biquad>, sample_rate: u32) -> Self {
        Self {
            nodes,
            sample_rate,
        }
    }

    pub fn add_node(&mut self, freq: f32, gain: f32, q: f32) {
        self.nodes.push(Biquad::new(freq, gain, q, self.sample_rate));
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        for i in 0..samples.len() {
            for filter in &mut self.nodes {
                samples[i] = filter.process(samples[i])
            }
        }
    }
}

pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn new(frequency: f32, gain: f32, q: f32, sample_rate: u32) -> Self {
        let omega = 2. * PI * frequency / sample_rate as f32;
        let alpha = omega.sin() / (2. * q);
        let a_gain = 10f32.powf(gain / 40.);

        let b0 = 1. + alpha * a_gain;
        let b1 = -2. * omega.cos();
        let b2 = 1. - alpha * a_gain;
        let a0 = 1. + alpha / a_gain;
        let a1 = -2. * omega.cos();
        let a2 = 1. - alpha / a_gain;

        Self {
            b0: b0/a0,
            b1: b1/a0,
            b2: b2/a0,
            a1: a1/a0,
            a2: a2/a0,
            z1: 0.,
            z2: 0.
        }
    }

    //process a single sample
    fn process(&mut self, sample: f32) -> f32 {
        let out = self.b0 * sample + self.z1;
        self.z1 = self.b1 * sample + self.z2 - self.a1 * out;
        self.z2 = self.b2 * sample - self.a2 * out;
        out
    }
}
