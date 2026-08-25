mod activations;
mod impls;
mod layer;
mod vectors;
mod parse;
pub mod nnue_conf;

// https://github.com/glinscott/nnue-pytorch/blob/master/docs/nnue.md

use std::mem::MaybeUninit;
use nnue_conf::*;
use features::NUM_FEATURES;

use crate::eval::nnue::layer::{FeatureLayer, Layer};

include!(env!("NNUE_PATH"));

#[repr(align(64))]
pub struct NNUE {
    pub ft: FeatureLayer<NUM_FEATURES, L1>,
    pub layer_1: Layer<{ L1 * 2 }, L2>,
    pub output: Layer<L2, 1>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(align(64))]
pub struct NNUEAccumulator {
    pub v: [[i16; L1]; 2],
}
