pub mod model {
    include!(concat!(env!("OUT_DIR"), "/ml/mobilenetv2-13.rs"));
}

mod labels {
    include!(concat!(env!("OUT_DIR"), "/ml/labels.rs"));
}

mod labels_norsk {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_norsk.rs"));
}

use burn::backend::NdArray;
use burn::tensor::Tensor;
use model::Model;

type Backend = NdArray<f32>;

/// Runs MobileNetV2 inference on preprocessed CHW float data.
/// Input must be 1x3x224x224 normalized with ImageNet mean/std.
/// Returns (english_label, norwegian_label).
pub fn recognize(float_data: Vec<f32>) -> (String, String) {
    let device = Default::default();
    let model: Model<Backend> = Model::default();

    let input =
        Tensor::<Backend, 1>::from_floats(float_data.as_slice(), &device).reshape([1, 3, 224, 224]);

    let output = model.forward(input);
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
