use std::fmt::Debug;

use crate::eval::nnue::vectors::fast_vdot;

#[derive(Debug, Clone)]
#[repr(align(64))]
pub struct Layer<const INPUT: usize, const OUTPUT: usize> {
    pub weights: [[i8; INPUT]; OUTPUT],
    pub bias: [i32; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Layer<INPUT, OUTPUT> {
    pub fn activate(&self, input: &[i8; INPUT]) -> [i32; OUTPUT] {
        let mut result = [0; OUTPUT];
        for i in 0..OUTPUT {
            result[i] = fast_vdot(&self.weights[i], input) + self.bias[i];
        }
        result
    }
}

#[derive(Debug, Clone)]
#[repr(align(64))]
pub struct FeatureLayer<const INPUT: usize, const OUTPUT: usize> {
    pub weights: [[i16; OUTPUT]; INPUT],
    pub bias: [i16; OUTPUT],
}
