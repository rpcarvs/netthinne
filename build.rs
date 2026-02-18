use burn_onnx::ModelGen;
use std::{env, fs, io::Write, path::Path};

fn main() {
    ModelGen::new()
        .input("src/ml/mobilenetv2-13.onnx")
        .out_dir("ml/")
        .embed_states(true)
        .run_from_script();

    fix_padding_compat();
    generate_labels();
    generate_labels_norsk();
}

/// burn-onnx 0.21.0-pre.1 emits PaddingConfig2d::Explicit(t, b, l, r) but
/// burn-nn 0.20.1 defines the variant as Explicit(height, width). Rewrite
/// the generated source so it compiles against the 0.20.1 API.
/// I'll have to check this later when updating Burn
fn fix_padding_compat() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("ml/mobilenetv2-13.rs");
    if !path.exists() {
        return;
    }
    let src = fs::read_to_string(&path).unwrap();
    let fixed = src
        .replace(
            "PaddingConfig2d::Explicit(1, 1, 1, 1)",
            "PaddingConfig2d::Explicit(1, 1)",
        )
        .replace("shape1_out1[actual_idx] as i64", "shape1_out1[actual_idx]");
    fs::write(&path, fixed).unwrap();
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
    println!("cargo:rerun-if-changed=src/ml/mobilenetv2-13.onnx");
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
