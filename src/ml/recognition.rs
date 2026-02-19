pub mod model {
    include!(concat!(env!("OUT_DIR"), "/ml/model.rs"));
}

mod labels {
    include!(concat!(env!("OUT_DIR"), "/ml/labels.rs"));
}

mod labels_norsk {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_norsk.rs"));
}

#[allow(dead_code)]
mod model_config {
    include!(concat!(env!("OUT_DIR"), "/ml/model_config.rs"));
}
use model_config::{IMG_H, IMG_W, IS_NHWC};

use std::cell::RefCell;

use burn::backend::NdArray;
use burn::tensor::Tensor;
use model::Model;

type Backend = NdArray<f32>;

thread_local! {
    /// Model weights are deserialized once and reused across inference calls.
    /// Model::default() copies ~50 MB of weights into NdArray tensors, so calling
    /// it on every shutter press would take several minutes in WASM.
    static MODEL: RefCell<Option<Model<Backend>>> = const { RefCell::new(None) };
}

/// Runs model inference on preprocessed float tensor data.
/// Layout and dimensions are determined by the active model's config.
/// Returns (english_label, norwegian_label).
pub fn recognize(float_data: Vec<f32>) -> (String, String) {
    let device = Default::default();

    MODEL.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(Model::from_embedded(&device));
        }
    });

    let input = if IS_NHWC {
        Tensor::<Backend, 1>::from_floats(float_data.as_slice(), &device).reshape([
            1,
            IMG_H as i32,
            IMG_W as i32,
            3,
        ])
    } else {
        Tensor::<Backend, 1>::from_floats(float_data.as_slice(), &device).reshape([
            1,
            3,
            IMG_H as i32,
            IMG_W as i32,
        ])
    };

    let output = MODEL.with(|cell| {
        let guard = cell.borrow();
        guard.as_ref().unwrap().forward(input)
    });

    let class_idx = output.squeeze::<1>().argmax(0).into_scalar() as usize;

    let english = labels::LABELS
        .get(class_idx)
        .unwrap_or(&"unknown")
        .to_string();
    let norwegian = labels_norsk::LABELS_NORSK
        .get(class_idx)
        .unwrap_or(&"ukjent")
        .to_string();

    (english, norwegian)
}
