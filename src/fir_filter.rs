
#[derive(Clone)]
pub struct FIRFilter {
    degree: usize,
    coefficients: Vec<f32>,
}

impl FIRFilter {
    pub fn new(degree: usize, coefficients: Vec<f32>) -> Self {
        assert!(degree == coefficients.len(), "Must provide N coefficients for an N degree filter!");

        Self {
            degree,
            coefficients
        }
    }

    //this assumes the history array has length of 'self.degree'
    pub fn process(&self, history: &[f32]) -> f32 {
        debug_assert!(history.len() == self.degree, "History buffer must match degree of FIR filter!");

        let mut out_sample = 0.;
        for i in 0..self.degree {
            out_sample += self.coefficients[i] * history[self.degree - i - 1];
        }
        
        out_sample
    }
}
