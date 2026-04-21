# RustEpubReader 段评功能 — 状态快照

> 生成时间: 2026-04-20
> 用于会话恢复，避免长对话卡顿。

---

## 分支状态

- **fork/main** (`yang12535/RustEpubReader`) 已同步最新
- PR #3 (Copilot 安全修复) 已合并到 fork/main
- 本地仓库在 `c:\Users\yang\Desktop\code\RustEpubReader`

---

## 已完成 ✅

| 功能 | 文件 | 状态 |
|------|------|------|
| 段评覆盖层面板 | `desktop/src/ui/review_panel.rs` | ✅ 卡片渲染、ESC/返回键关闭 |
| 段评筛选 | `desktop/src/ui/review_panel.rs` | ✅ 按 `anchor_id` 组匹配筛选 |
| 文字选择回归 | `desktop/src/ui/reader_block.rs` | ✅ `Sense::click()` |
| 段评链接拦截 | `desktop/src/ui/reader.rs` | ✅ 不跳转，开覆盖层 |
| 段评章节跳过 | `desktop/src/app.rs` | ✅ `next/prev/go_to` 跳过段评章节 |
| Dark 模式阅读背景 | `desktop/src/app.rs` | ✅ 临时覆盖 `reader_bg_color` 为暗色 |
| 安全修复 (Copilot) | `core/src/sharing/*` 等 6 文件 | ✅ 已合并到 fork/main |

---

## 待验证 ❓

| 项目 | 状态 |
|------|------|
| Android 段评面板 | 未测试 |
| Android ESC/返回键关闭 | 未测试 |
| Android Dark 模式适配 | 未测试 |

---

## 已知问题 / 注意事项

1. `windows_subsystem` 已注释（显示控制台），正式发布前需恢复
2. 段评卡片解析依赖固定文本格式：`"N. 内容 作者：xxx | 时间：xxx | 赞：52"`，格式变化会 fallback 到普通文本渲染
3. 段评面板 backdrop 为纯视觉遮罩（`interactable(false)`），关闭仅通过 ✕ 按钮或 ESC/返回键
4. `reader_bg_color` 默认米白色 `Color32::from_rgb(250, 246, 238)`，dark mode 下临时覆盖为 `#1e1e22`，不持久化

---

## 关键上下文

- **EPUB 段评结构**: `h3 id="para-N"` (段落标题) + `ol/li` (评论列表) → 解析为 `ContentBlock::Heading` + `ContentBlock::Paragraph`
- **段评章节识别**: 标题以 ` - 段评` 结尾
- **锚点匹配**: `review_panel_anchor` 从 URL `#para-N` 提取，匹配 `Heading { anchor_id: Some("para-N") }`
- **筛选逻辑**: 找到目标 heading 后，包含其后续所有非-heading blocks 直到下一个 heading
- **构建命令**: `cargo build --release --no-default-features`
- **工具链**: Windows, GNU 工具链
- **代理**: v2r `127.0.0.1:10808`
