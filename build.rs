use burn_onnx::ModelGen;
use std::{env, fs, io::Write, path::Path};

fn main() {
    ModelGen::new()
        .input("src/ml/mobilenet_v2_1_4_fp32.onnx")
        .out_dir("ml/")
        .embed_states(true)
        .run_from_script();

    fix_padding_compat();
    generate_labels("src/ml/labels.txt", "ml/labels.rs", "LABELS");
    generate_labels("src/ml/labels_norsk.txt", "ml/labels_norsk.rs", "LABELS_NORSK");
}

/// burn-onnx 0.21 emits PaddingConfig2d::Explicit(top, left, bottom, right) but
/// burn-nn 0.20 expects Explicit(height, width). Rewrite 4-arg calls to 2-arg
/// using ceiling division: h = (top + bottom + 1) / 2, w = (left + right + 1) / 2.
fn fix_padding_compat() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("ml/mobilenet_v2_1_4_fp32.rs");
    if !path.exists() {
        return;
    }
    let src = fs::read_to_string(&path).unwrap();
    let fixed = rewrite_explicit_padding(&src)
        .replace("shape1_out1[actual_idx] as i64", "shape1_out1[actual_idx]");
    fs::write(&path, fixed).unwrap();
}

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

/// Reads a line-per-entry text file and emits a Rust static array in OUT_DIR.
fn generate_labels(src_path: &str, out_file: &str, const_name: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join(out_file);

    let text = fs::read_to_string(src_path)
        .unwrap_or_else(|_| panic!("{src_path} not found"));

    let entries: Vec<String> = text
        .lines()
        .map(|line| format!("    \"{}\"", line.trim()))
        .collect();

    let mut f = fs::File::create(&out_path).unwrap();
    writeln!(f, "pub static {const_name}: &[&str] = &[").unwrap();
    for entry in &entries {
        writeln!(f, "{entry},").unwrap();
    }
    writeln!(f, "];").unwrap();

    println!("cargo:rerun-if-changed={src_path}");
}
