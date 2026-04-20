//! Contains image resolution, extraction, and parsing logic for EPUB files.
use std::collections::HashMap;
use std::path::Path;

use base64::Engine;
use epub::doc::EpubDoc;
use scraper::{Html, Selector};

/// Strip fragment/query from a src path, resolve it relative to chapter_dir,
/// and return `(clean, resolved)`.  Returns `None` when the cleaned path is empty.
pub(super) fn clean_and_resolve_src(src: &str, chapter_dir: &Path) -> Option<(String, String)> {
    let clean = src
        .split('#')
        .next()
        .unwrap_or(src)
        .split('?')
        .next()
        .unwrap_or(src)
        .trim();
    if clean.is_empty() {
        return None;
    }
    let resolved = if clean.starts_with('/') {
        clean.trim_start_matches('/').to_string()
    } else {
        chapter_dir.join(clean).to_string_lossy().to_string()
    };
    Some((clean.to_string(), resolved))
}

/// Look up image data by resolved path, falling back to filename match.
pub(super) fn lookup_image_by_path<'a>(
    clean: &str,
    resolved: &str,
    image_resources: &'a HashMap<String, Vec<u8>>,
) -> Option<&'a Vec<u8>> {
    if let Some(data) = image_resources.get(resolved) {
        return Some(data);
    }
    let file_name = Path::new(clean)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())?;
    image_resources.iter().find_map(|(k, v)| {
        let kn = Path::new(k)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if kn == file_name {
            Some(v)
        } else {
            None
        }
    })
}

/// Load only the images referenced by `<img>`/`<image>` tags in this chapter's HTML.
/// This avoids eagerly loading all images in the EPUB into memory.
pub(super) fn load_referenced_images(
    html: &str,
    chapter_path: &str,
    image_path_index: &HashMap<String, std::path::PathBuf>,
    image_resources: &mut HashMap<String, Vec<u8>>,
    doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>,
) {
    let document = Html::parse_document(html);
    let img_sel = Selector::parse("img, image").expect("valid selector");
    let chapter_dir = Path::new(chapter_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    for elem in document.select(&img_sel) {
        let src = elem
            .value()
            .attr("src")
            .or_else(|| elem.value().attr("data-src"))
            .or_else(|| elem.value().attr("href"))
            .or_else(|| elem.value().attr("xlink:href"))
            .unwrap_or("");
        let (clean, resolved) = match clean_and_resolve_src(src, chapter_dir) {
            Some(pair) => pair,
            None => continue,
        };
        if src.trim().starts_with("data:") || image_resources.contains_key(&resolved) {
            continue;
        }
        // Try direct path match
        if let Some(epub_path) = image_path_index.get(&resolved) {
            if let Some(data) = doc.get_resource_by_path(epub_path) {
                image_resources.insert(resolved, data);
                continue;
            }
        }
        // Fallback: match by filename
        let file_name = Path::new(&clean)
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        if let Some(file_name) = file_name {
            if let Some((key, epub_path)) = image_path_index.iter().find(|(k, _)| {
                Path::new(k.as_str())
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .as_deref()
                    == Some(&file_name)
            }) {
                if let Some(data) = doc.get_resource_by_path(epub_path) {
                    image_resources.insert(key.clone(), data);
                }
            }
        }
    }
}

pub(super) fn resolve_image_data(
    raw_src: &str,
    chapter_path: &str,
    image_resources: &HashMap<String, Vec<u8>>,
) -> Option<std::sync::Arc<Vec<u8>>> {
    let src = raw_src.trim();
    if src.is_empty() {
        return None;
    }

    if src.starts_with("data:") {
        return decode_data_uri(src).map(std::sync::Arc::new);
    }

    let lowered = src.to_ascii_lowercase();
    if lowered.starts_with("http://") || lowered.starts_with("https://") {
        return None;
    }

    let chapter_dir = Path::new(chapter_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let (clean, resolved) = clean_and_resolve_src(src, chapter_dir)?;
    lookup_image_by_path(&clean, &resolved, image_resources)
        .cloned()
        .map(std::sync::Arc::new)
}

fn decode_data_uri(uri: &str) -> Option<Vec<u8>> {
    let comma = uri.find(',')?;
    if comma + 1 >= uri.len() {
        return None;
    }
    let (meta, payload) = uri.split_at(comma);
    let payload = &payload[1..];

    if meta.to_ascii_lowercase().contains(";base64") {
        base64::engine::general_purpose::STANDARD
            .decode(payload)
            .ok()
    } else {
        Some(payload.as_bytes().to_vec())
    }
}
