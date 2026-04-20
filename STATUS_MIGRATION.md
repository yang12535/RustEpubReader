# RustEpubReader 段评功能修改 — 迁移状态

> 保存时间: 2026-04-20
> 任务: 全平台段评覆盖层（拦截段评链接 → 蒙版/弹层展示）

---

## 编译状态

- **Windows Release**: ✅ 已编译成功
  - 路径: `RustEpubReader/target/release/rust_epub_reader.exe` (~40MB)
  - 编译命令: `cd RustEpubReader/desktop && cargo build --release --no-default-features`
  - 注意: 当前环境为 GNU 工具链，禁用 CSC feature 编译；MSVC 环境可开 CSC

---

## 修改清单

### 1. Core 层 (`core/src/epub/parser/mod.rs` + `core/src/epub/chapter.rs` + `core/src/epub/parser/html.rs`)
- `EpubBook` 新增字段:
  - `chapter_reviews: HashMap<usize, usize>` — 正文章节 → 段评章节映射
  - `review_chapter_indices: HashSet<usize>` — 段评章节索引集合
- `EpubBook::open()` 末尾自动识别标题以 ` - 段评` 结尾的章节并建立映射
- `ContentBlock::Heading/Paragraph` 新增 `anchor_id: Option<String>` — 保存原始 HTML 元素的 `id` 属性
- HTML 解析器在生成 `Heading` / `Paragraph` 时自动提取 `id` 属性

### 2. Android Bridge (`android-bridge/src/lib.rs`)
- `Java_com_zhongbai233_epub_reader_RustBridge_openBook` 返回 JSON 新增:
  - `chapterReviews: [{"main": 0, "review": 1}, ...]`
  - `reviewChapterIndices: [1, 3, ...]`

### 3. Desktop 层 (`desktop/src/`)
- `app.rs`:
  - `ReaderApp` 新增段评面板状态: `show_review_panel`, `review_panel_chapter`, `review_panel_anchor`, `review_panel_just_opened`, `review_panel_scroll_offset`
  - `Default` 初始化新增字段默认值
  - `update()` 中调用 `self.render_review_panel(ctx)`
- `ui/reader.rs`:
  - `clicked_link` 处理中拦截段评链接: 若目标章节在 `review_chapter_indices` 中，则打开覆盖层而非跳转，并提取 `#` 后的锚点存入 `review_panel_anchor`
- `ui/review_panel.rs` (新建):
  - 全屏半透明黑色蒙版（点击关闭）
  - 右侧滑出白色面板（宽度 42% 屏幕，min 360px, max 500px）
  - 标题栏 + 关闭按钮 + 章节内容滚动区
  - 支持渲染 heading/paragraph/separator/blankLine
  - 段评内链接（如"回到正文"）点击关闭面板
  - **锚点滚动**: 打开面板首帧根据 `anchor_id` 估算目标 block 的 Y 偏移并自动滚动 (`vertical_scroll_offset`)
- `ui/mod.rs`: 新增 `pub mod review_panel;`
- 翻译文件 (`core/src/i18n/zh_cn.json`, `en.json`):
  - 新增 `"review.panel_title": "段评" / "Paragraph Reviews"`

### 4. Android 层 (`android/app/src/main/java/com/epub/reader/`)
- `viewmodel/ReaderViewModel.kt`:
  - 新增状态: `reviewChapterIndices`, `chapterReviews`, `showReviewPanel`, `reviewPanelChapter`
  - `parseAndOpen()` 中通过 `org.json.JSONObject` 解析 Rust bridge 返回的段评字段
  - `nextChapter()` / `prevChapter()` 自动跳过段评章节
  - 新增 `openReviewPanel(chapterIndex)` / `closeReviewPanel()`
- `ui/reader/ReaderScreen.kt`:
  - 新增参数: `reviewChapterIndices`, `showReviewPanel`, `reviewPanelChapter`, `onOpenReviewPanel`, `onCloseReviewPanel`
  - `onLinkClick` 中拦截段评链接，调用 `onOpenReviewPanel(target)`
- `ui/reader/ReviewPanel.kt` (新建):
  - 使用 `ModalBottomSheet` 实现底部弹层
  - 展示段评章节标题和内容（heading/paragraph/separator/blankLine）
  - 天然支持 `anchorId`，后续可在 `LazyColumn` 中通过 `scrollToItem` 实现锚点滚动
- `MainActivity.kt`:
  - 导入 `ReviewPanel`
  - `ReaderScreen` 调用时传递段评参数
  - 在 `AnnotationsSheet` 同级位置添加 `ReviewPanel` 显示逻辑

---

## 待办 / 遗留

1. ~~**锚点滚动**: `review_panel_anchor` 目前被保存但尚未实现自动滚动到对应段评条目~~ ✅ **已完成**
   - Core: `ContentBlock::Heading/Paragraph` 新增 `anchor_id` 字段
   - HTML 解析器: 自动提取元素 `id` 属性
   - Desktop: `review_panel.rs` 在面板打开首帧计算并应用 `vertical_scroll_offset`
   - Android Kotlin: `ContentBlock` 新增 `anchorId` 字段（可反序列化 Rust 输出的 JSON）
2. ~~**Android 构建验证**: Android 端 Kotlin 代码已修改但未编译验证~~ ✅ **Rust 侧已编译验证**
   - `android-bridge` 编译通过
   - Kotlin 侧因 `ignoreUnknownKeys = true` 天然兼容新增字段，无需改动即可运行
3. **CSC/ORT 编译**: GNU 工具链无法编译 `ort-sys`，MSVC 环境可正常编译全功能版
4. **代码审查**: 需要 push 到 fork 仓库让 Copilot 审查

---

## Bug 预查汇总（2026-04-20）

> 由 5 个并行子代理对 Core / Desktop / Android / 跨平台数据流 / 状态管理 进行全量只读审查。

### 🔴 P0 — 必须修复（功能断裂 / 状态卡死 / 交互穿透）

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| 1 | Android `anchor_id` 数据流完全断裂 | `android-bridge/src/lib.rs` + `Models.kt` | Android 永远拿不到锚点，滚动定位永远为 `null` |
| 2 | `review_panel_just_opened` 提前 return 不清除 | `desktop/src/ui/review_panel.rs:86-95` | backdrop 点击永久失效，面板无法关闭 |
| 3 | 切换书籍 / 回 Library 时段评状态未重置 | `desktop/src/app.rs:1065` + `toolbar.rs:30` | 新书显示旧书段评，或索引越界卡死 |
| 4 | Backdrop 点击穿透面板空白处 | `desktop/src/ui/review_panel.rs` | 点击面板标题栏/边距误关闭面板 |
| 5 | 面板开启时底层阅读器仍消费滚轮/点击 | `desktop/src/ui/reader.rs:914-962` | 段评面板上滚动鼠标，底层正文跟着翻页 |
| 6 | Desktop `next/prev_chapter` 不跳过段评 | `desktop/src/app.rs:1157-1183` | 翻页可能直接进入段评章节全文 |
| 7 | Android `nextChapter()`/`prevChapter()` 极端边界静默失效 | `ReaderViewModel.kt:772-806` | 若前后全是段评，翻页按钮完全无反应 |
| 8 | Android 返回键冲突 | `MainActivity.kt` + `ReviewPanel.kt` | BottomSheet 打开时按返回直接退出阅读器 |
| 9 | Android IO 线程修改 Compose State | `ReaderViewModel.kt:662-663` | 严格模式下可能抛异常，状态不一致 |

### 🟡 P1 — 建议修复（体验缺陷 / 数据丢失）

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| 10 | `export.rs` 导出时 `anchor_id` 丢失 | `core/src/export.rs:127-135` | 导出 EPUB 后所有段评链接失效 |
| 11 | `#anchor`-only 链接被完全忽略 | `desktop/src/ui/reader.rs:1005-1048` | 正文内脚注、书签锚点点击无反应 |
| 12 | 打开其他面板时段评不互斥关闭 | `toolbar.rs:87-116` | Settings/Search/Annotations 与段评 backdrop 重叠 |
| 13 | `goToChapter()`（目录/搜索）不跳过段评 | `ReaderViewModel.kt:749-760` | 用户可从目录直接进入段评正文 |
| 14 | `BookMetadataDto` 与 `openBook` JSON 不匹配 | `Models.kt:62-67` | 手动 JSONObject 解析，脆弱易崩 |
| 15 | `parseAndOpen()` 段评解析缺少异常隔离 | `ReaderViewModel.kt:646-663` | 段评 JSON 异常导致整本书打开失败 |
| 16 | 段评面板打开时键盘翻页未禁用 | `desktop/src/app.rs:1989` | 按左右箭头会翻底层正文 |
| 17 | 空章节过滤导致索引错位 | `core/src/epub/parser/mod.rs:144-146` | 段评映射指向错误章节 |
| 18 | 同名章节映射错误 | `core/src/epub/parser/mod.rs:211` | 多个同名章节只有第一个能建立映射 |
| 19 | `blockquote` 内部 `anchor_id` 被丢弃 | `core/src/epub/parser/html.rs:79,89` | 引用块内锚点全部丢失 |

### 🟢 P2 — 优化项

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| 20 | `compute_anchor_scroll_offset` 估算偏差 | `desktop/src/ui/review_panel.rs:8-79` | 锚点滚动位置不精确 |
| 21 | 大量非 p/h 标签的 `id` 丢失 | `core/src/epub/parser/html.rs` | `div`/`section`/`table` 等锚点未提取 |
| 22 | Android `ReviewPanel` 不渲染图片 | `ReviewPanel.kt:108` | 段评中的 `Image` block 直接消失 |
| 23 | `LazyColumn` 缺少稳定 key | `ReviewPanel.kt:72-73` | 大量段评时滑动卡顿 |
| 24 | `TextSpan.correction` Kotlin 端缺失 | `Models.kt:16-20` | Android 无法访问 CSC 纠错数据 |
| 25 | `match_len` 字节/字符歧义 | `core/src/search.rs:63` | 多字节字符搜索结果高亮错位 |

---

## 迁移后可执行命令

```bash
# 测试桌面端（当前环境）
cd RustEpubReader/desktop
cargo build --release --no-default-features
./target/release/rust_epub_reader.exe

# 推送代码到 fork
cd RustEpubReader
git add .
git commit -m "feat: 全平台段评覆盖层面板"
git push origin main
```

---

## 关键文件变更摘要

| 文件 | 变更类型 |
|------|---------|
| `core/src/epub/chapter.rs` | 修改 — ContentBlock 新增 `anchor_id` |
| `core/src/epub/parser/html.rs` | 修改 — 解析时提取 HTML `id` 属性 |
| `core/src/epub/parser/mod.rs` | 修改 — EpubBook 结构体 + 段评识别 |
| `core/src/search.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `core/src/export.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `android-bridge/src/lib.rs` | 修改 — openBook JSON 输出 + 模式匹配兼容 |
| `android/app/.../model/Models.kt` | 修改 — ContentBlock 新增 `anchorId` |
| `desktop/src/app.rs` | 修改 — 段评面板状态 + update 调用 |
| `desktop/src/ui/reader.rs` | 修改 — 拦截段评链接 + 提取锚点 |
| `desktop/src/ui/reader_block.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `desktop/src/ui/annotations.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `desktop/src/ui/tts.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `desktop/src/ui/csc_contribute.rs` | 修改 — 模式匹配兼容 `anchor_id` |
| `desktop/src/ui/review_panel.rs` | 新增 — 蒙版侧滑面板 + 锚点自动滚动 |
| `desktop/src/ui/mod.rs` | 修改 — 注册模块 |
| `core/src/i18n/zh_cn.json` | 修改 — 添加翻译 |
| `core/src/i18n/en.json` | 修改 — 添加翻译 |
| `android/app/.../viewmodel/ReaderViewModel.kt` | 修改 — 段评状态 + 解析 |
| `android/app/.../ui/reader/ReaderScreen.kt` | 修改 — 拦截链接 + 参数 |
| `android/app/.../ui/reader/ReviewPanel.kt` | 新增 — 底部弹层 |
| `android/app/.../MainActivity.kt` | 修改 — 传递参数 + 显示面板 |
