import re
content = open('desktop/src/ui/reader_state.rs', encoding='utf-8').read()
content = re.sub(r'pub\(crate\)\s*type CscCharMapCacheData = \(u64, u32, u32, Vec<usize>\);', '', content)
content = content.replace('pub(crate) type BlockGalleyEntry = (usize, Arc<egui::Galley>, egui::Rect, String);', 'pub(crate) type BlockGalleyEntry = (usize, Arc<egui::Galley>, egui::Rect, String);\n\npub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);')
open('desktop/src/ui/reader_state.rs', 'w', encoding='utf-8').write(content)