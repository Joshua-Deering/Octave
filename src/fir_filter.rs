
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

    //this assumes the slices have a combined length of 'self.degree'
    pub fn process_slices(&self, slice1: &[f32], slice2: &[f32]) -> f32 {
        debug_assert!(slice1.len() + slice2.len() == self.degree, "History buffer must match degree of FIR filter!");

        let mut cur_slice: &[f32] = slice2;
        if cur_slice.len() == 0 { cur_slice = slice1; }

        let mut slice_idx = cur_slice.len();

        let mut out_sample = 0.;
        for i in 0..self.degree {
            slice_idx -= 1;
            out_sample += self.coefficients[i] * cur_slice[slice_idx];

            if slice_idx == 0 {
                cur_slice = slice1;
                slice_idx = slice1.len();
            }
        }
        
        out_sample
    }
}
