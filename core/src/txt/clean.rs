//! 文本清洗。
//!
//! 对解码后的 TXT 内容做格式标准化，清除常见杂质。

use regex::Regex;
use std::sync::OnceLock;

fn re_html_tags() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)</?(?:br|p|div|span|a|b|i|em|strong|ul|ol|li|h[1-6])[^>]*>").unwrap()
    })
}

fn re_html_entities() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"&(?:nbsp|lt|gt|amp|quot|#\d{1,5}|#x[0-9a-fA-F]{1,4});").unwrap())
}

fn re_control_chars() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]").unwrap())
}

fn re_consecutive_blank_lines() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\n{3,}").unwrap())
}

/// 清洗原始文本：统一换行、去除杂质、标准化空白。
pub fn clean_text(raw: &str) -> String {
    // 1. 统一换行符
    let text = raw.replace("\r\n", "\n").replace('\r', "\n");

    // 2. 去除 HTML 标签残留
    let text = re_html_tags().replace_all(&text, "");

    // 3. 替换常见 HTML 实体
    let text = decode_html_entities(&text);

    // 4. 移除控制字符（保留 \n \t）
    let text = re_control_chars().replace_all(&text, "");

    // 5. 全角空格标准化：\u{3000} → 两个半角空格
    let text = text.replace('\u{3000}', "  ");

    // 6. 去除每行首尾多余空白（保留缩进意图：最多保留 2 个空格）
    let text: String = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_end();
            // 保留行首最多 2 个空格的缩进（中文小说常用两格缩进）
            let leading_spaces = trimmed.len() - trimmed.trim_start().len();
            if leading_spaces > 2 {
                format!("  {}", trimmed.trim_start())
            } else {
                trimmed.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 7. 合并连续空行（>2 → 2）
    let text = re_consecutive_blank_lines().replace_all(&text, "\n\n");

    // 8. 去除首尾空白
    text.trim().to_string()
}

/// 解码常见 HTML 实体。
fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    re_html_entities()
        .replace_all(s, |caps: &regex::Captures| {
            match &caps[0] {
                "&nbsp;" => " ".to_string(),
                "&lt;" => "<".to_string(),
                "&gt;" => ">".to_string(),
                "&amp;" => "&".to_string(),
                "&quot;" => "\"".to_string(),
                other => {
                    // 数字实体: &#123; 或 &#x1A;
                    if let Some(num_str) =
                        other.strip_prefix("&#x").and_then(|s| s.strip_suffix(';'))
                    {
                        u32::from_str_radix(num_str, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| c.to_string())
                            .unwrap_or_default()
                    } else if let Some(num_str) =
                        other.strip_prefix("&#").and_then(|s| s.strip_suffix(';'))
                    {
                        num_str
                            .parse::<u32>()
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| c.to_string())
                            .unwrap_or_default()
                    } else {
                        other.to_string()
                    }
                }
            }
        })
        .into_owned()
}
