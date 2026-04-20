//! CSC ONNX model paths and configuration loading.
use std::path::PathBuf;

const MODEL_FILENAME: &str = "csc-macbert-int8.onnx";
const VOCAB_FILENAME: &str = "csc-vocab.txt";
const MANIFEST_FILENAME: &str = "csc-manifest.json";

const MODEL_URL: &str = "https://dl.zhongbai233.com/models/csc-macbert-int8.onnx";
const VOCAB_URL: &str = "https://dl.zhongbai233.com/models/csc-vocab.txt";
const MANIFEST_URL: &str = "https://dl.zhongbai233.com/models/csc-manifest.json";

/// SHA256 hash of the model file. Empty = skip verification (model not yet hosted).
const MODEL_SHA256: &str = "";

pub fn model_dir(data_dir: &str) -> PathBuf {
    PathBuf::from(data_dir).join("models")
}

pub fn model_path(data_dir: &str) -> PathBuf {
    model_dir(data_dir).join(MODEL_FILENAME)
}

pub fn vocab_path(data_dir: &str) -> PathBuf {
    model_dir(data_dir).join(VOCAB_FILENAME)
}

pub fn manifest_path(data_dir: &str) -> PathBuf {
    model_dir(data_dir).join(MANIFEST_FILENAME)
}

pub fn is_model_available(data_dir: &str) -> bool {
    model_path(data_dir).exists() && vocab_path(data_dir).exists()
}

pub fn model_url() -> &'static str {
    MODEL_URL
}

pub fn vocab_url() -> &'static str {
    VOCAB_URL
}

pub fn manifest_url() -> &'static str {
    MANIFEST_URL
}

/// All files needed for CSC: (url, local_filename)
pub fn required_files() -> Vec<(&'static str, &'static str)> {
    vec![
        (MODEL_URL, MODEL_FILENAME),
        (VOCAB_URL, VOCAB_FILENAME),
        (MANIFEST_URL, MANIFEST_FILENAME),
    ]
}

/// Verify model integrity via SHA256 hash.
/// Returns true if hash is not configured (empty) or matches.
#[allow(clippy::const_is_empty)]
pub fn verify_model(data_dir: &str) -> bool {
    if MODEL_SHA256.is_empty() {
        return true;
    }
    let path = model_path(data_dir);
    let Ok(data) = std::fs::read(path) else {
        return false;
    };
    use sha2::{Digest, Sha256};
    let hash = format!("{:x}", Sha256::digest(&data));
    hash == MODEL_SHA256
}
