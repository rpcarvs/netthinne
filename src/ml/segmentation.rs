pub mod yolo_model {
    include!(concat!(env!("OUT_DIR"), "/ml/yolov8n.rs"));
}

mod labels_yolo {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_yolo.rs"));
}

mod labels_yolo_norsk {
    include!(concat!(env!("OUT_DIR"), "/ml/labels_yolo_norsk.rs"));
}

use std::cell::RefCell;

use burn::backend::NdArray;
use burn::tensor::Tensor;
use yolo_model::Model;

type Backend = NdArray<f32>;

const CONF_THRESHOLD: f32 = 0.25;
const INTERSECTION_OVER_UNION_THRESHOLD: f32 = 0.45;
const MAX_DETECTIONS: usize = 3;
const NUM_CLASSES: usize = 80;

pub struct Detection {
    pub bbox: [f32; 4],
    pub class_idx: usize,
    pub confidence: f32,
}

thread_local! {
    static YOLO: RefCell<Option<Model<Backend>>> = const { RefCell::new(None) };
}

/// Runs YOLOv8n object detection on preprocessed 640x640 NCHW float data.
/// Returns up to MAX_DETECTIONS sorted by confidence descending.
pub fn detect(float_data: Vec<f32>, orig_width: u32, orig_height: u32) -> Vec<Detection> {
    let device = Default::default();

    YOLO.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(Model::from_embedded(&device));
        }
    });

    let input =
        Tensor::<Backend, 1>::from_floats(float_data.as_slice(), &device).reshape([1, 3, 640, 640]);

    let output = YOLO.with(|cell| {
        let guard = cell.borrow();
        guard.as_ref().unwrap().forward(input)
    });

    // YOLOv8 output: [1, 84, 8400] -> squeeze to [84, 8400] -> transpose to [8400, 84]
    let raw: Vec<f32> = output.into_data().to_vec().unwrap();
    let candidates = decode_and_filter(&raw, orig_width, orig_height);
    non_maximum_suppression(candidates)
}

pub fn label_en(idx: usize) -> String {
    labels_yolo::LABELS_YOLO
        .get(idx)
        .unwrap_or(&"unknown")
        .to_string()
}

pub fn label_no(idx: usize) -> String {
    labels_yolo_norsk::LABELS_YOLO_NORSK
        .get(idx)
        .unwrap_or(&"ukjent")
        .to_string()
}

/// Decodes raw [84, 8400] output into filtered detections in original image coords.
fn decode_and_filter(raw: &[f32], orig_w: u32, orig_h: u32) -> Vec<Detection> {
    let scale_x = orig_w as f32 / 640.0;
    let scale_y = orig_h as f32 / 640.0;
    let mut detections = Vec::new();

    // raw is in [84, 8400] layout: row-major, so raw[row * 8400 + col]
    for col in 0..8400 {
        let cx = raw[col]; // raw[0 * 8400 + col];
        let cy = raw[8400 + col]; // raw[1 * 8400 + col];
        let w = raw[2 * 8400 + col];
        let h = raw[3 * 8400 + col];

        let mut max_score: f32 = 0.0;
        let mut max_class: usize = 0;
        for cls in 0..NUM_CLASSES {
            let score = raw[(4 + cls) * 8400 + col];
            if score > max_score {
                max_score = score;
                max_class = cls;
            }
        }

        if max_score < CONF_THRESHOLD {
            continue;
        }

        let x1 = (cx - w / 2.0) * scale_x;
        let y1 = (cy - h / 2.0) * scale_y;
        let x2 = (cx + w / 2.0) * scale_x;
        let y2 = (cy + h / 2.0) * scale_y;

        detections.push(Detection {
            bbox: [
                x1.max(0.0),
                y1.max(0.0),
                x2.min(orig_w as f32),
                y2.min(orig_h as f32),
            ],
            class_idx: max_class,
            confidence: max_score,
        });
    }

    detections
}

/// Keep top detections, suppress overlapping boxes of same class.
fn non_maximum_suppression(mut detections: Vec<Detection>) -> Vec<Detection> {
    detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let mut keep = Vec::new();
    let mut suppressed = vec![false; detections.len()];

    for i in 0..detections.len() {
        if suppressed[i] {
            continue;
        }
        keep.push(i);
        if keep.len() >= MAX_DETECTIONS {
            break;
        }
        for j in (i + 1)..detections.len() {
            if suppressed[j] || detections[j].class_idx != detections[i].class_idx {
                continue;
            }
            if intersection_over_union(&detections[i].bbox, &detections[j].bbox)
                > INTERSECTION_OVER_UNION_THRESHOLD
            {
                suppressed[j] = true;
            }
        }
    }

    keep.into_iter()
        .map(|i| {
            let d = &detections[i];
            Detection {
                bbox: d.bbox,
                class_idx: d.class_idx,
                confidence: d.confidence,
            }
        })
        .collect()
}

fn intersection_over_union(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let x1 = a[0].max(b[0]);
    let y1 = a[1].max(b[1]);
    let x2 = a[2].min(b[2]);
    let y2 = a[3].min(b[3]);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let area_a = (a[2] - a[0]) * (a[3] - a[1]);
    let area_b = (b[2] - b[0]) * (b[3] - b[1]);
    let union = area_a + area_b - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}
