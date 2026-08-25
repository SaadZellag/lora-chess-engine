use crate::{NNUE, eval::nnue::layer::{FeatureLayer, Layer}, nnue_conf::{L1, L2}};

impl<const INPUT: usize, const OUTPUT: usize> FeatureLayer<INPUT, OUTPUT> {
    pub const fn number_of_bytes(&self) -> usize {
        INPUT * OUTPUT * std::mem::size_of::<i16>() + // weights
        OUTPUT * std::mem::size_of::<i16>() // bias
    }

    pub const fn read_from_bytes(&mut self, bytes: &'static [u8]) {
        let weights_size = INPUT * OUTPUT * std::mem::size_of::<i16>();
        let bias_size = OUTPUT * std::mem::size_of::<i16>();

        let (weights_bytes, bias_bytes) = bytes.split_at(weights_size);

        let mut i = 0;
        while i < INPUT {
            let mut j = 0;
            while j < OUTPUT {
                let index = i * OUTPUT + j;
                let weight1 = weights_bytes[index * 2];
                let weight2 = weights_bytes[index * 2 + 1];
                self.weights[i][j] = i16::from_le_bytes([weight1, weight2]);
                j += 1;
            }
            i += 1;
        }

        let mut j = 0;
        while j < OUTPUT {
            let bias1 = bias_bytes[j * 2];
            let bias2 = bias_bytes[j * 2 + 1];
            self.bias[j] = i16::from_le_bytes([bias1, bias2]);
            j += 1;
        }
    }
}

impl<const INPUT: usize, const OUTPUT: usize> Layer<INPUT, OUTPUT> {
    pub const fn number_of_bytes(&self) -> usize {
        INPUT * OUTPUT * std::mem::size_of::<i8>() + // weights
        OUTPUT * std::mem::size_of::<i32>() // bias
    }

    pub const fn read_from_bytes(&mut self, bytes: &'static [u8]) {
        let weights_size = INPUT * OUTPUT * std::mem::size_of::<i8>();
        let bias_size = OUTPUT * std::mem::size_of::<i32>();

        let (weights_bytes, bias_bytes) = bytes.split_at(weights_size);

        let mut i = 0;
        while i < OUTPUT {
            let mut j = 0;
            while j < INPUT {
                let index = i * INPUT + j;
                self.weights[i][j] = weights_bytes[index] as i8;
                j += 1;
            }
            i += 1;
        }

        let mut j = 0;
        while j < OUTPUT {
            let index = j * std::mem::size_of::<i32>();
            self.bias[j] = i32::from_le_bytes([
                bias_bytes[index],
                bias_bytes[index + 1],
                bias_bytes[index + 2],
                bias_bytes[index + 3],
            ]);
            j += 1;
        }
    }
}

impl NNUE {
    pub const fn read_from_bytes(bytes: &'static [u8]) -> Self {
        let mut ft = FeatureLayer::new();
        let mut layer_1 = Layer::new();
        let mut output = Layer::new();

        let (ft_slice, bytes) = bytes.split_at(ft.number_of_bytes());
        let (layer_1_slice, bytes) = bytes.split_at(layer_1.number_of_bytes());
        let (output_slice, _bytes) = bytes.split_at(output.number_of_bytes());

        ft.read_from_bytes(ft_slice);
        layer_1.read_from_bytes(layer_1_slice);
        output.read_from_bytes(output_slice);
        
        Self {
            ft,
            layer_1,
            output,
        }
    }
}