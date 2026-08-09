
#[derive(Debug)]
#[repr(C)]
pub struct SparseBatch {
    pub size: i32,
    pub max_active_features: i32,
    pub num_active_our_features: i32,
    pub num_active_their_features: i32,
    pub score: *mut f32,
    pub eval: *mut i32,
    pub our_feature_indices: *mut i32,
    pub their_feature_indices: *mut i32,
}

impl SparseBatch {
    pub fn new(data: &[TrainingDataEntry]) -> Self {
        // dbg!(data[0]);
        let size = data.len();

        let score: Vec<f32> = data.iter().map(|e| e.final_score as f32).collect();
        let eval: Vec<i32> = data.iter().map(|e| e.eval).collect();

        // let white_feature_indices: Vec<i32> = data
        //     .iter()
        //     .enumerate()
        //     .map(|(i, e)| ([i as u16; 30], e.our_features))
        //     .flat_map(|(i, e)| i.into_iter().zip(e.into_iter()))
        //     .filter(|(_, e)| *e != u16::MAX)
        //     .flat_map(|(i, e)| [i as i32, e as i32])
        //     .collect();

        // let black_feature_indices: Vec<i32> = data
        //     .iter()
        //     .enumerate()
        //     .map(|(i, e)| ([i as u16; 30], e.their_features))
        //     .flat_map(|(i, e)| i.into_iter().zip(e.into_iter()))
        //     .filter(|(_, e)| *e != u16::MAX)
        //     .flat_map(|(i, e)| [i as i32, e as i32])
        //     .collect();

        // dbg!(size);
        // dbg!(data);
        let mut white_feature_indices = vec![0; 2 * size * CurrentFeatures::FEATURES_PER_SIDE];
        let mut black_feature_indices = vec![0; 2 * size * CurrentFeatures::FEATURES_PER_SIDE];

        let mut white_index = 0;
        let mut black_index = 0;
        for (index, item) in data.iter().enumerate() {
            for feature in item.our_features {
                if feature == u16::MAX {
                    break;
                }
                debug_assert!(
                    feature < 768,
                    "feature is {} from index {} | data: {:?}",
                    feature,
                    index,
                    item
                );
                white_feature_indices[white_index] = index as i32;
                white_feature_indices[white_index + 1] = feature as i32;
                white_index += 2
            }

            for feature in item.their_features {
                if feature == u16::MAX {
                    break;
                }
                debug_assert!(
                    feature < 768,
                    "feature is {} from index {} | data: {:?}",
                    feature,
                    index,
                    item
                );
                black_feature_indices[black_index] = index as i32;
                black_feature_indices[black_index + 1] = feature as i32;
                black_index += 2
            }
        }

        let num_active_white_features = white_index / 2; // Indices contain batch index and value
        let num_active_black_features = black_index / 2;

        Self {
            size: size as i32,
            max_active_features: 30 * size as i32,
            num_active_our_features: num_active_white_features as i32,
            num_active_their_features: num_active_black_features as i32,
            score: score.leak().as_mut_ptr(),
            eval: eval.leak().as_mut_ptr(),
            our_feature_indices: white_feature_indices.leak().as_mut_ptr(),
            their_feature_indices: black_feature_indices.leak().as_mut_ptr(),
        }
    }

    pub unsafe fn drop_batch(ptr: *mut SparseBatch) {
        let batch = std::ptr::read_unaligned(ptr);

        macro_rules! drop {
            ($($name: ident),*) => {
                $(let _ = Box::from_raw(batch.$name);)*
            };
        }

        drop!(score, eval, our_feature_indices, their_feature_indices);
        // let _ = Box::from_raw(batch.stm as *mut bool);
    }
}
