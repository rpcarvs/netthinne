pub mod model {
    include!(concat!(env!("OUT_DIR"), "/ml/mobilenet_v2_1_4_fp32.rs"));
}

mod labels {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_in1k.rs"));
}

mod labels_norsk {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_in1k_norsk.rs"));
}

use std::cell::RefCell;

use burn::backend::NdArray;
use burn::tensor::Tensor;
use model::Model;

type Backend = NdArray<f32>;

thread_local! {
    static MODEL: RefCell<Option<Model<Backend>>> = const { RefCell::new(None) };
}

/// Runs MobileNetV2 inference on preprocessed NCHW float data.
/// Returns (english_label, norwegian_label).
pub fn recognize(float_data: Vec<f32>) -> (String, String) {
    let device = Default::default();

    MODEL.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(Model::from_embedded(&device));
        }
    });

    let input =
        Tensor::<Backend, 1>::from_floats(float_data.as_slice(), &device).reshape([1, 3, 224, 224]);

    let output = MODEL.with(|cell| {
        let guard = cell.borrow();
        guard.as_ref().unwrap().forward(input)
    });

    let class_idx = output.squeeze::<1>().argmax(0).into_scalar() as usize;

    let english = labels::LABELS_IN1K
        .get(class_idx)
        .unwrap_or(&"unknown")
        .to_string();
    let norwegian = labels_norsk::LABELS_IN1K_NORSK
        .get(class_idx)
        .unwrap_or(&"ukjent")
        .to_string();

    (english, norwegian)
}
