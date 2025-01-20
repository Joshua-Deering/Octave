
pub trait DFT {
    fn process(&self);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Complex {
    pub r: f64,
    pub i: f64,
}
