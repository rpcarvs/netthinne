use burn_onnx::ModelGen;
use std::{env, fs, io::Write, path::Path};

fn main() {
    ModelGen::new()
        .input("src/ml/model.onnx")
        .out_dir("ml/")
        .embed_states(true)
        .run_from_script();

    fix_padding_compat();
    generate_model_config();
    generate_labels();
    generate_labels_norsk();
}

/// burn-onnx 0.21.0-pre.1 emits PaddingConfig2d::Explicit(top, bottom, left, right) but
/// burn-nn 0.20.1 defines the variant as Explicit(height, width). Rewrite any 4-arg
/// Explicit call to 2-arg by taking (top, left) = (height, width).
/// TODO: revisit when updating Burn past 0.20.1
fn fix_padding_compat() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("ml/model.rs");
    if !path.exists() {
        return;
    }
    let src = fs::read_to_string(&path).unwrap();
    let fixed = rewrite_explicit_padding(&src)
        .replace("shape1_out1[actual_idx] as i64", "shape1_out1[actual_idx]");
    fs::write(&path, fixed).unwrap();
}

/// Rewrites PaddingConfig2d::Explicit(a, b, c, d) to Explicit(h, w) for any values.
/// burn-onnx emits padding in ONNX order: (top, left, bottom, right).
/// burn-nn Explicit(h, w) applies h symmetrically to top+bottom and w to left+right.
/// For asymmetric SAME padding we use ceiling of total/2 to preserve the output tensor size.
fn rewrite_explicit_padding(src: &str) -> String {
    let marker = "PaddingConfig2d::Explicit(";
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(pos) = rest.find(marker) {
        out.push_str(&rest[..pos + marker.len()]);
        rest = &rest[pos + marker.len()..];
        if let Some(end) = rest.find(')') {
            let args: Vec<&str> = rest[..end].split(", ").collect();
            if args.len() == 4 {
                let top: i64 = args[0].trim().parse().unwrap_or(0);
                let left: i64 = args[1].trim().parse().unwrap_or(0);
                let bottom: i64 = args[2].trim().parse().unwrap_or(0);
                let right: i64 = args[3].trim().parse().unwrap_or(0);
                let h = (top + bottom + 1) / 2;
                let w = (left + right + 1) / 2;
                out.push_str(&h.to_string());
                out.push_str(", ");
                out.push_str(&w.to_string());
            } else {
                out.push_str(&rest[..end]);
            }
            out.push(')');
            rest = &rest[end + 1..];
        }
    }
    out.push_str(rest);
    out
}

/// Reads src/ml/model.cfg and emits OUT_DIR/ml/model_config.rs with compile-time constants.
fn generate_model_config() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("ml/model_config.rs");

    let cfg = fs::read_to_string("src/ml/model.cfg").expect("src/ml/model.cfg not found");

    let mut map = std::collections::HashMap::new();
    for line in cfg.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    let is_nhwc = map["format"] == "NHWC";
    let is_bgr: bool = map["bgr"].parse().unwrap();
    let w: u32 = map["width"].parse().unwrap();
    let h: u32 = map["height"].parse().unwrap();
    let mean: Vec<f32> = map["mean"]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    let std: Vec<f32> = map["std"]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    let mut f = fs::File::create(&out_path).unwrap();
    writeln!(f, "pub const IMG_W: usize = {w};").unwrap();
    writeln!(f, "pub const IMG_H: usize = {h};").unwrap();
    writeln!(f, "pub const IS_NHWC: bool = {is_nhwc};").unwrap();
    writeln!(f, "pub const IS_BGR: bool = {is_bgr};").unwrap();
    writeln!(
        f,
        "pub const MEAN: [f32; 3] = [{}, {}, {}];",
        mean[0], mean[1], mean[2]
    )
    .unwrap();
    writeln!(
        f,
        "pub const STD:  [f32; 3] = [{}, {}, {}];",
        std[0], std[1], std[2]
    )
    .unwrap();

    println!("cargo:rerun-if-changed=src/ml/model.cfg");
}

/// Reads src/ml/labels.txt and emits OUT_DIR/ml/labels.rs with a LABELS static array.
fn generate_labels() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("ml/labels.rs");

    let labels_txt = fs::read_to_string("src/ml/labels.txt").expect("src/ml/labels.txt not found");

    let entries: Vec<String> = labels_txt
        .lines()
        .map(|line| format!("    \"{}\"", line.trim()))
        .collect();

    let mut f = fs::File::create(&out_path).unwrap();
    writeln!(f, "pub static LABELS: &[&str] = &[").unwrap();
    for entry in &entries {
        writeln!(f, "{entry},").unwrap();
    }
    writeln!(f, "];").unwrap();

    println!("cargo:rerun-if-changed=src/ml/labels.txt");
    println!("cargo:rerun-if-changed=src/ml/model.onnx");
}

/// Reads src/ml/labels_norsk.txt and emits OUT_DIR/ml/labels_norsk.rs with a LABELS_NORSK static array.
fn generate_labels_norsk() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("ml/labels_norsk.rs");

    let labels_txt =
        fs::read_to_string("src/ml/labels_norsk.txt").expect("src/ml/labels_norsk.txt not found");

    let entries: Vec<String> = labels_txt
        .lines()
        .map(|line| format!("    \"{}\"", line.trim()))
        .collect();

    let mut f = fs::File::create(&out_path).unwrap();
    writeln!(f, "pub static LABELS_NORSK: &[&str] = &[").unwrap();
    for entry in &entries {
        writeln!(f, "{entry},").unwrap();
    }
    writeln!(f, "];").unwrap();

    println!("cargo:rerun-if-changed=src/ml/labels_norsk.txt");
}
