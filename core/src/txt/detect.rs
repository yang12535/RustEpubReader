//! 编码检测与文件读取。
//!
//! 使用 chardetng（Mozilla 编码检测库）探测 TXT 文件编码，
//! 再用 encoding_rs 完成实际转码。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use chardetng::EncodingDetector;

use super::TxtError;

/// 读取 TXT 文件并自动检测编码，返回 UTF-8 字符串。
///
/// 探测策略：读取文件前 8KB 喂给 chardetng，然后用 encoding_rs 转码全部内容。
/// 支持 UTF-8 / GBK / GB18030 / BIG5 / Shift-JIS / EUC-KR 等常见编码。
pub fn read_txt_file(path: &Path) -> Result<String, TxtError> {
    let mut file = File::open(path).map_err(TxtError::Io)?;
    let mut raw = Vec::new();
    file.read_to_end(&mut raw).map_err(TxtError::Io)?;

    if raw.is_empty() {
        return Ok(String::new());
    }

    // 去除 UTF-8 BOM（EF BB BF）
    let content = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw
    };

    // 探测编码：取前 8KB
    let probe_len = content.len().min(8192);
    let mut detector = EncodingDetector::new();
    detector.feed(&content[..probe_len], probe_len == content.len());
    let encoding = detector.guess(None, true);

    // 使用 encoding_rs 转码
    let (decoded, _, had_errors) = encoding.decode(content);
    if had_errors {
        // 如果推测编码解码失败，尝试用 UTF-8 有损解码
        Ok(String::from_utf8_lossy(content).into_owned())
    } else {
        Ok(decoded.into_owned())
    }
}
