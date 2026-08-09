mod activations;
mod impls;
mod layer;
mod vectors;
pub mod nnue_conf;

// https://github.com/glinscott/nnue-pytorch/blob/master/docs/nnue.md

use std::mem::MaybeUninit;
use nnue_conf::*;
use features::NUM_FEATURES;

use crate::eval::nnue::layer::{FeatureLayer, Layer};

pub static EVALUATOR: NNUE = unsafe { MaybeUninit::zeroed().assume_init() }; // TODO: Initialize this properly

#[repr(align(64))]
pub struct NNUE {
    ft: FeatureLayer<NUM_FEATURES, L1>,
    layer_1: Layer<{ L1 * 2 }, L2>,
    output: Layer<L2, 1>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(align(64))]
pub struct NNUEAccumulator {
    v: [[i16; L1]; 2],
}
