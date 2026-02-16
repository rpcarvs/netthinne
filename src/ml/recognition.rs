/// Stub: recognizes objects in image data.
/// Returns a hardcoded label until a real model is integrated.
///
/// TODO: Future flow:
/// 1. Load ONNX model (via Burn)
/// 2. Preprocess image (resize, normalize)
/// 3. Run forward pass
/// 4. Argmax on output tensor
/// 5. Map index to ImageNet label
pub fn recognize(_image_data: &[u8]) -> String {
    "dog".to_string()
}
