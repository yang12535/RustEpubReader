//! 章节识别 — 规则引擎。
//!
//! 三级优先级匹配 + 可选启发式 + 用户自定义正则。

use regex::Regex;
use std::sync::OnceLock;

/// 识别出的原始章节。
#[derive(Debug, Clone, serde::Serialize)]
pub struct RawChapter {
    pub title: String,
    pub content: String,
    pub line_start: usize,
}

/// 章节预览信息（供 UI 展示）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterPreview {
    pub title: String,
    pub line_start: usize,
    pub char_count: usize,
}

/// 拆分配置。
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// 最小章节字数（低于此值的章节合并到上一章），默认 100。
    pub min_chapter_chars: usize,
    /// 是否启用启发式检测，默认 false。
    pub use_heuristic: bool,
    /// 用户自定义正则表达式（匹配整行作为章节标题）。
    pub custom_regex: Option<String>,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            min_chapter_chars: 100,
            use_heuristic: false,
            custom_regex: None,
        }
    }
}

// ── 规则正则 ────────────────────────────────────────────────────

/// 高优先级：中文数字 第X章/节/回/卷/部/篇/集/幕
fn re_chinese_numeral_chapter() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^[\s　]*第[零〇一二三四五六七八九十百千万壹贰叁肆伍陆柒捌玖拾佰仟]+[章节回卷部篇集幕]").unwrap()
    })
}

/// 高优先级：阿拉伯数字 第X章/节/回/卷/部/篇/集/幕
fn re_arabic_numeral_chapter() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^[\s　]*第\d+[章节回卷部篇集幕]").unwrap())
}

/// 高优先级：英文 Chapter/CHAPTER
fn re_english_chapter() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?im)^[\s]*(?:chapter|part|book|volume)\s+\d+").unwrap())
}

/// 高优先级：卷X
fn re_volume_prefix() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^[\s　]*卷[零〇一二三四五六七八九十百千万\d]+").unwrap())
}

/// 高优先级：带括号的卷名 【第X卷...】
fn re_bracketed_volume() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^[\s　]*【第[零〇一二三四五六七八九十百千万壹贰叁肆伍陆柒捌玖拾佰仟\d]+[卷部篇集].*】\s*$").unwrap()
    })
}

/// 中优先级：起始特殊章节
fn re_special_start() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^[\s　]*(序章|序言|楔子|引子|前言|引言|开篇|Prologue|Foreword|Preface|Introduction)\s*$").unwrap()
    })
}

/// 中优先级：结尾特殊章节
fn re_special_end() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^[\s　]*(尾声|终章|后记|番外|完结感言|完本感言|作品相关|Epilogue|Afterword|Conclusion)").unwrap()
    })
}

/// 低优先级：数字编号行（带分隔符）
fn re_numbered_with_sep() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^[\s　]*\d{1,4}[\.、]\s*\S").unwrap())
}

/// 低优先级：纯数字行
fn re_pure_number() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^[\s　]*\d{1,4}\s*$").unwrap())
}

// ── 核心拆分逻辑 ────────────────────────────────────────────────

/// 拆分文本为章节列表。
pub fn split_chapters(text: &str, config: &SplitConfig) -> Vec<RawChapter> {
    let lines: Vec<&str> = text.lines().collect();

    // 1. 用户自定义正则优先
    if let Some(ref pattern) = config.custom_regex {
        if let Ok(re) = Regex::new(pattern) {
            let matches = find_matches_by_regex(&lines, &re);
            if matches.len() >= 2 {
                return build_chapters_from_matches(&lines, &matches, config.min_chapter_chars);
            }
            if matches.len() == 1 {
                return build_chapters_from_matches(&lines, &matches, config.min_chapter_chars);
            }
        }
    }

    // 2. 高优先级规则
    let high_regexes: Vec<&Regex> = vec![
        re_chinese_numeral_chapter(),
        re_arabic_numeral_chapter(),
        re_english_chapter(),
        re_volume_prefix(),
        re_bracketed_volume(),
    ];
    // 特殊标记（序章/番外等）始终参与合并
    let special_regexes: Vec<&Regex> = vec![re_special_start(), re_special_end()];

    for re in &high_regexes {
        let matches = find_matches_by_regex(&lines, re);
        if matches.len() >= 2 && is_spacing_reasonable(&matches, lines.len()) {
            // 高优先级命中后，追加特殊标记（番外/序章等）
            let merged = merge_special_markers(&lines, matches, &special_regexes);
            return build_chapters_from_matches(&lines, &merged, config.min_chapter_chars);
        }
    }

    // 合并所有高优先级结果看看
    let mut all_high: Vec<(usize, String)> = Vec::new();
    for re in &high_regexes {
        all_high.extend(find_matches_by_regex(&lines, re));
    }
    all_high.sort_by_key(|(idx, _)| *idx);
    all_high.dedup_by_key(|(idx, _)| *idx);
    if all_high.len() >= 2 && is_spacing_reasonable(&all_high, lines.len()) {
        let merged = merge_special_markers(&lines, all_high, &special_regexes);
        return build_chapters_from_matches(&lines, &merged, config.min_chapter_chars);
    }

    // 3. 中优先级：特殊章节标记（与高优先级合并后尝试）
    let all_mid = {
        let mut v: Vec<(usize, String)> = Vec::new();
        for re in &high_regexes {
            v.extend(find_matches_by_regex(&lines, re));
        }
        for re in &special_regexes {
            v.extend(find_matches_by_regex(&lines, re));
        }
        v.sort_by_key(|(idx, _)| *idx);
        v.dedup_by_key(|(idx, _)| *idx);
        v
    };
    if all_mid.len() >= 2 && is_spacing_reasonable(&all_mid, lines.len()) {
        return build_chapters_from_matches(&lines, &all_mid, config.min_chapter_chars);
    }

    // 4. 低优先级：数字编号（仅在高/中无结果时）
    if all_mid.is_empty() {
        let low_regexes: Vec<&Regex> = vec![re_numbered_with_sep(), re_pure_number()];
        for re in &low_regexes {
            let matches = find_matches_by_regex(&lines, re);
            if matches.len() >= 2 && is_spacing_reasonable(&matches, lines.len()) {
                return build_chapters_from_matches(&lines, &matches, config.min_chapter_chars);
            }
        }
    }

    // 5. 启发式（需用户开启）
    if config.use_heuristic {
        let heuristic_matches = heuristic_detect(&lines);
        if heuristic_matches.len() >= 2 && is_spacing_reasonable(&heuristic_matches, lines.len()) {
            return build_chapters_from_matches(
                &lines,
                &heuristic_matches,
                config.min_chapter_chars,
            );
        }
    }

    // 6. 如果有单个匹配点（来自高/中），构建两章
    if all_mid.len() == 1 {
        return build_chapters_from_matches(&lines, &all_mid, config.min_chapter_chars);
    }

    // 7. 回退：机械分割
    fallback_split(&lines)
}

/// 在每行上运行正则，收集匹配行号和标题文本。
fn find_matches_by_regex(lines: &[&str], re: &Regex) -> Vec<(usize, String)> {
    let mut matches = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && re.is_match(trimmed) {
            matches.push((i, trimmed.to_string()));
        }
    }
    matches
}

/// 将特殊标记（番外/序章等）追加到已有匹配中，去重排序。
fn merge_special_markers(
    lines: &[&str],
    mut matches: Vec<(usize, String)>,
    special_regexes: &[&Regex],
) -> Vec<(usize, String)> {
    for re in special_regexes {
        matches.extend(find_matches_by_regex(lines, re));
    }
    matches.sort_by_key(|(idx, _)| *idx);
    matches.dedup_by_key(|(idx, _)| *idx);
    matches
}

/// 检查候选匹配的间距是否合理。
fn is_spacing_reasonable(matches: &[(usize, String)], total_lines: usize) -> bool {
    if matches.len() < 2 {
        return false;
    }

    // 间距列表
    let spacings: Vec<usize> = matches
        .windows(2)
        .map(|w| w[1].0.saturating_sub(w[0].0))
        .collect();

    // 中位数
    let mut sorted = spacings.clone();
    sorted.sort();
    let median = sorted[sorted.len() / 2];

    // 间距中位数合理性：10~10000 行
    if !(10..=10000).contains(&median) {
        return false;
    }

    // 数量合理性：候选数 / 总行数 < 5%
    if total_lines > 0 && (matches.len() as f64 / total_lines as f64) > 0.05 {
        return false;
    }

    true
}

/// 从匹配结果构建章节列表。
fn build_chapters_from_matches(
    lines: &[&str],
    matches: &[(usize, String)],
    min_chars: usize,
) -> Vec<RawChapter> {
    let mut chapters = Vec::new();

    // 第一个匹配点之前的内容（如果有足够内容）
    if matches[0].0 > 0 {
        let content: String = lines[..matches[0].0].join("\n");
        let trimmed = content.trim();
        if !trimmed.is_empty() && trimmed.chars().count() >= min_chars {
            chapters.push(RawChapter {
                title: "前言".to_string(),
                content: trimmed.to_string(),
                line_start: 0,
            });
        }
    }

    // 每个匹配点到下一个匹配点之间的内容
    for (i, (line_idx, title)) in matches.iter().enumerate() {
        let content_start = line_idx + 1;
        let content_end = if i + 1 < matches.len() {
            matches[i + 1].0
        } else {
            lines.len()
        };

        let content = if content_start < content_end {
            lines[content_start..content_end].join("\n")
        } else {
            String::new()
        };

        chapters.push(RawChapter {
            title: title.clone(),
            content: content.trim().to_string(),
            line_start: *line_idx,
        });
    }

    // 合并过短章节到上一章
    merge_short_chapters(&mut chapters, min_chars);

    chapters
}

/// 合并内容过短的章节到前一章。
fn merge_short_chapters(chapters: &mut Vec<RawChapter>, min_chars: usize) {
    if chapters.len() <= 1 {
        return;
    }

    let mut i = 1;
    while i < chapters.len() {
        if chapters[i].content.chars().count() < min_chars && i > 0 {
            let merged_content = {
                let prev = &chapters[i - 1];
                let curr = &chapters[i];
                format!("{}\n\n{}\n{}", prev.content, curr.title, curr.content)
            };
            chapters[i - 1].content = merged_content;
            chapters.remove(i);
        } else {
            i += 1;
        }
    }
}

/// 回退：按 200 行机械分割。
fn fallback_split(lines: &[&str]) -> Vec<RawChapter> {
    let chunk_size = 200;
    let mut chapters = Vec::new();

    for (i, chunk) in lines.chunks(chunk_size).enumerate() {
        let content = chunk.join("\n");
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            chapters.push(RawChapter {
                title: format!("第{}部分", i + 1),
                content: trimmed.to_string(),
                line_start: i * chunk_size,
            });
        }
    }

    if chapters.is_empty() {
        chapters.push(RawChapter {
            title: "全文".to_string(),
            content: lines.join("\n").trim().to_string(),
            line_start: 0,
        });
    }

    chapters
}

// ── 启发式检测 ──────────────────────────────────────────────────

/// 启发式检测候选章节标题。
fn heuristic_detect(lines: &[&str]) -> Vec<(usize, String)> {
    let mut candidates: Vec<(usize, String)> = Vec::new();

    // 句末标点
    let end_puncts = ['。', '！', '？', '；', '…', '」', '』', '"', ')', '）'];
    // 对话引号
    let dialog_chars = ['"', '「', '『', '"'];
    // 人称代词
    let pronouns = ["我", "你", "他", "她", "它", "们"];
    // 语气词
    let particles = ["吗", "呢", "啊", "哦", "嗯", "吧", "了", "的"];

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // 基本长度约束
        if trimmed.is_empty() || trimmed.chars().count() > 30 {
            continue;
        }

        // 不以句末标点结尾
        if let Some(last) = trimmed.chars().last() {
            if end_puncts.contains(&last) {
                continue;
            }
        }

        // 不含对话引号
        if trimmed.chars().any(|c| dialog_chars.contains(&c)) {
            continue;
        }

        // 不含人称代词
        if pronouns.iter().any(|p| trimmed.contains(p)) {
            continue;
        }

        // 不含常见语气词
        if particles.iter().any(|p| trimmed.contains(p)) {
            continue;
        }

        // 非纯标点/符号
        let has_content = trimmed.chars().any(|c| c.is_alphanumeric() || c > '\x7F');
        if !has_content {
            continue;
        }

        // 前后均有空行（首尾行特殊处理）
        let prev_blank = i == 0 || lines[i - 1].trim().is_empty();
        let next_blank = i >= lines.len() - 1 || lines[i + 1].trim().is_empty();
        if !prev_blank || !next_blank {
            continue;
        }

        candidates.push((i, trimmed.to_string()));
    }

    // 全局校验
    if candidates.len() < 2 {
        return candidates;
    }

    let total = lines.len();

    // 数量合理性
    if total > 0 && (candidates.len() as f64 / total as f64) > 0.02 {
        return Vec::new();
    }

    // 间距分析
    let spacings: Vec<usize> = candidates
        .windows(2)
        .map(|w| w[1].0.saturating_sub(w[0].0))
        .collect();

    if spacings.is_empty() {
        return Vec::new();
    }

    let mut sorted_spacings = spacings.clone();
    sorted_spacings.sort();
    let median = sorted_spacings[sorted_spacings.len() / 2];

    // 间距中位数合理性
    if !(50..=5000).contains(&median) {
        return Vec::new();
    }

    // 间距规律性（标准差/均值 ≤ 0.5）
    let mean = spacings.iter().sum::<usize>() as f64 / spacings.len() as f64;
    if mean > 0.0 {
        let variance = spacings
            .iter()
            .map(|&s| (s as f64 - mean).powi(2))
            .sum::<f64>()
            / spacings.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev / mean > 0.5 {
            return Vec::new();
        }
    }

    // 二次过滤：标题后跟长段落 + 标题前有标点结尾行
    candidates.retain(|(idx, _)| {
        // 后面 3 行中至少 1 行 >30 字
        let has_following_paragraph = (1..=3).any(|offset| {
            let next_idx = idx + offset;
            next_idx < lines.len() && lines[next_idx].trim().chars().count() > 30
        });

        // 前面最近的非空行以标点结尾
        let has_preceding_end = (1..=3).any(|offset| {
            if *idx < offset {
                return false;
            }
            let prev_line = lines[idx - offset].trim();
            if prev_line.is_empty() {
                return false;
            }
            let end_puncts = ['。', '！', '？', '；', '…', '」', '』', '"'];
            prev_line
                .chars()
                .last()
                .is_some_and(|c| end_puncts.contains(&c))
        });

        has_following_paragraph && has_preceding_end
    });

    candidates
}
