//! EPUB 生成器。
//!
//! 参考 Tomato-Novel-Downloader 的 epub_generator.rs 设计，
//! 使用 epub-builder crate 生成标准 EPUB 3 文件。

use std::fs;
use std::io::Cursor;
use std::path::Path;

use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ReferenceType, ZipLibrary};

use super::split::RawChapter;
use super::TxtError;

/// 书籍元信息。
#[derive(Debug, Clone)]
pub struct TxtMeta {
    pub title: String,
    pub author: Option<String>,
    pub language: String,
}

/// 默认嵌入 CSS。
const DEFAULT_CSS: &str = "\
body { font-family: serif; line-height: 1.6; margin: 1em; }
h2 { text-align: center; margin: 1.5em 0 1em; font-size: 1.3em; }
p { text-indent: 2em; margin: 0.5em 0; }
p.no-indent { text-indent: 0; }
";

/// 从章节列表生成 EPUB 文件。
pub fn build_epub(chapters: &[RawChapter], meta: &TxtMeta, output: &Path) -> Result<(), TxtError> {
    let zip = ZipLibrary::new().map_err(|e| TxtError::Build(e.to_string()))?;
    let mut builder = EpubBuilder::new(zip).map_err(|e| TxtError::Build(e.to_string()))?;

    // EPUB 3.0
    builder.epub_version(EpubVersion::V30);

    // 元信息
    builder
        .metadata("title", &meta.title)
        .map_err(|e| TxtError::Build(e.to_string()))?;
    builder
        .metadata("lang", &meta.language)
        .map_err(|e| TxtError::Build(e.to_string()))?;
    builder
        .metadata("toc_name", &meta.title)
        .map_err(|e| TxtError::Build(e.to_string()))?;

    if let Some(ref author) = meta.author {
        let author = author.trim();
        if !author.is_empty() {
            builder
                .metadata("author", author)
                .map_err(|e| TxtError::Build(e.to_string()))?;
        }
    }

    builder
        .metadata("generator", "RustEpubReader")
        .map_err(|e| TxtError::Build(e.to_string()))?;

    // 嵌入 CSS
    builder
        .stylesheet(Cursor::new(DEFAULT_CSS))
        .map_err(|e| TxtError::Build(e.to_string()))?;

    // 添加章节
    for (i, chapter) in chapters.iter().enumerate() {
        let file_name = format!("chapter_{:05}.xhtml", i);
        let xhtml = render_chapter_xhtml(&chapter.title, &chapter.content);

        builder
            .add_content(
                EpubContent::new(&file_name, Cursor::new(xhtml))
                    .title(&chapter.title)
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| TxtError::Build(e.to_string()))?;
    }

    // 写入文件
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(TxtError::Io)?;
    }

    let mut buffer = Vec::new();
    builder
        .generate(&mut buffer)
        .map_err(|e| TxtError::Build(e.to_string()))?;

    fs::write(output, buffer).map_err(TxtError::Io)?;

    Ok(())
}

/// 将章节标题和纯文本内容渲染为 XHTML。
fn render_chapter_xhtml(title: &str, content: &str) -> String {
    let escaped_title = escape_html(title);

    // 构建正文段落
    let mut body = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // 空行不输出（段落间距由 CSS margin 控制）
            continue;
        }
        body.push_str(&format!("    <p>{}</p>\n", escape_html(trimmed)));
    }

    if body.is_empty() {
        body = "    <p class=\"no-indent\">（本章内容为空）</p>\n".to_string();
    }

    format!(
        "<?xml version='1.0' encoding='utf-8'?>\n\
         <!DOCTYPE html>\n\
         <html xmlns=\"http://www.w3.org/1999/xhtml\" \
               xmlns:epub=\"http://www.idpf.org/2007/ops\" \
               lang=\"{lang}\" xml:lang=\"{lang}\">\n\
         <head>\n\
           <title>{title}</title>\n\
           <link href=\"stylesheet.css\" rel=\"stylesheet\" type=\"text/css\"/>\n\
         </head>\n\
         <body>\n\
           <h2>{title}</h2>\n\
         {body}\
         </body>\n\
         </html>",
        lang = "zh",
        title = escaped_title,
        body = body,
    )
}

use crate::escape_html;
