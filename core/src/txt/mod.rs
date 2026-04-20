//! TXT-to-EPUB 转换模块。
//!
//! 提供 TXT 文件的编码检测、文本清洗、章节识别、EPUB 生成全流程。

mod build;
mod clean;
mod detect;
pub mod split;

use std::path::{Path, PathBuf};

pub use build::TxtMeta;
pub use split::{ChapterPreview, RawChapter, SplitConfig};

// ── 错误类型 ────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum TxtError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Build error: {0}")]
    Build(String),
}

// ── 转换选项与结果 ──────────────────────────────────────────────

/// 转换选项。
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// 书名。None = 从文件名推导。
    pub title: Option<String>,
    /// 作者。
    pub author: Option<String>,
    /// 用户自定义正则。
    pub custom_regex: Option<String>,
    /// 启用启发式检测。默认 false。
    pub use_heuristic: bool,
    /// 语言代码。默认 "zh"。
    pub language: String,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            custom_regex: None,
            use_heuristic: false,
            language: "zh".to_string(),
        }
    }
}

/// 转换结果。
#[derive(Debug, Clone)]
pub struct ConvertResult {
    /// 生成的 EPUB 文件路径。
    pub epub_path: PathBuf,
    /// 书名。
    pub title: String,
    /// 章节数。
    pub chapter_count: usize,
}

// ── 顶层 API ────────────────────────────────────────────────────

/// 将 TXT 文件转换为 EPUB。
///
/// 流程：编码检测 → 文本清洗 → 章节识别 → EPUB 生成。
/// 输出文件存放在 `output_dir` 下，文件名为 `{uuid}.epub`。
pub fn convert_txt_to_epub(
    txt_path: &Path,
    output_dir: &Path,
    options: &ConvertOptions,
) -> Result<ConvertResult, TxtError> {
    // 1. 编码检测 + 读取
    let raw_text = detect::read_txt_file(txt_path)?;

    // 2. 文本清洗
    let cleaned = clean::clean_text(&raw_text);

    if cleaned.is_empty() {
        return Err(TxtError::Parse("文件内容为空".to_string()));
    }

    // 3. 章节识别
    let split_config = SplitConfig {
        min_chapter_chars: 100,
        use_heuristic: options.use_heuristic,
        custom_regex: options.custom_regex.clone(),
    };
    let chapters = split::split_chapters(&cleaned, &split_config);

    // 4. 推导标题
    let title = options
        .title
        .clone()
        .unwrap_or_else(|| derive_title_from_path(txt_path));

    // 5. EPUB 生成
    let epub_filename = format!("{}.epub", uuid::Uuid::new_v4());
    let epub_path = output_dir.join(&epub_filename);

    let meta = TxtMeta {
        title: title.clone(),
        author: options.author.clone(),
        language: options.language.clone(),
    };
    build::build_epub(&chapters, &meta, &epub_path)?;

    Ok(ConvertResult {
        epub_path,
        title,
        chapter_count: chapters.len(),
    })
}

/// 章节预览（不生成 EPUB，仅返回识别结果供 UI 展示）。
pub fn preview_chapters(
    txt_path: &Path,
    config: &SplitConfig,
) -> Result<Vec<ChapterPreview>, TxtError> {
    let raw_text = detect::read_txt_file(txt_path)?;
    let cleaned = clean::clean_text(&raw_text);

    if cleaned.is_empty() {
        return Ok(vec![]);
    }

    let chapters = split::split_chapters(&cleaned, config);

    Ok(chapters
        .iter()
        .map(|ch| ChapterPreview {
            title: ch.title.clone(),
            line_start: ch.line_start,
            char_count: ch.content.chars().count(),
        })
        .collect())
}

/// 从文件路径推导书名（去掉扩展名）。
fn derive_title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "未命名".to_string())
}
