# RustEpubReader 段评功能 — 交接状态（2026-04-20）

## 已完成

1. ✅ **Issue 已发** — https://github.com/zhongbai2333/RustEpubReader/issues/12
   - 标题：段评覆盖层 Feature Request / Discussion
   - 等待作者回复是否接受该方向

2. ✅ **Fork 远程已添加**
   ```bash
   git remote add fork https://github.com/yang12535/RustEpubReader.git
   ```

3. ✅ **桌面端代码** — 编译通过（`cargo build --release --no-default-features`）
   - 右侧滑出面板、半透明蒙版、锚点自动滚动、段评内链接支持

4. ✅ **Android 代码** — 已修改但未编译验证
   - ModalBottomSheet、ViewModel 状态、链接拦截、跳过段评翻页

## 待修复 P0 Bug（优先级排序）

### Desktop

| # | 问题 | 文件 | 修复思路 |
|---|------|------|---------|
| 1 | `next_chapter()` / `prev_chapter()` 不跳过段评 | `desktop/src/app.rs:1157-1201` | 加减章节时检查 `review_chapter_indices.contains(&idx)`，跳过段评 |
| 2 | `go_to_chapter()` 目录/搜索不跳过段评 | `desktop/src/app.rs` 相关 | 若目标为段评，自动调到下一个非段评章节 |
| 3 | 切换书籍 / 回 Library 时段评状态未重置 | `desktop/src/app.rs:1065+` | 在 `open_book_from_path` 和返回 library 时重置 `show_review_panel` 等状态 |
| 4 | `review_panel_just_opened` 提前 return 不清除 | `desktop/src/ui/review_panel.rs:86-95` | 提前 return 时设置 `self.review_panel_just_opened = false` |
| 5 | 面板开启时底层消费滚轮/键盘 | `desktop/src/app.rs:1989+` | 段评面板打开时屏蔽左右箭头翻页和滚轮事件 |
| 6 | Backdrop 点击穿透 | `desktop/src/ui/review_panel.rs` | 用 `panel_rect.contains(pos)` 判断，只有点击 backdrop 才关闭 |

### Android

| # | 问题 | 文件 | 修复思路 |
|---|------|------|---------|
| 7 | `ReviewPanel.kt` 包名不匹配 | `android/.../ui/reader/ReviewPanel.kt` | 包名 `com.zhongbai233...` → `com.epub.reader.ui.reader` |
| 8 | `MainActivity.kt` import 路径错误 | `android/.../MainActivity.kt` | import `com.epub.reader.ui.reader.ReviewPanel` |
| 9 | `nextChapter()`/`prevChapter()` 边界失效 | `ReaderViewModel.kt:772-806` | 若全为段评则原地不动，加日志 |

## 关键上下文

- **编译命令**: `cd desktop && cargo build --release --no-default-features`
- **Android Bridge**: `cargo build -p android-bridge`
- **远程**: origin = zhongbai2333, fork = yang12535
- **未提交修改**: 19 文件修改 + 2 新增（review_panel.rs + ReviewPanel.kt）

## 下一步建议

1. 修复上述 P0 Bug
2. `git add . && git commit -m "feat: 段评覆盖层"`
3. `git push fork main`
4. 等 Issue #12 作者回复后决定是否发 PR
