import re

content = open('desktop/src/ui/reader_state.rs', encoding='utf-8').read()
content = content.replace('pub(crate) type BlockGalleyEntry = (usize, Arc<egui::Galley>, egui::Rect, String);', 'pub(crate) type BlockGalleyEntry = (usize, Arc<egui::Galley>, egui::Rect, String);\n\npub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);')
content = content.replace('    pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n    static CSC_CHAR_MAP_CACHE: RefCell<Option<CscCharMapCacheData>>', '    static CSC_CHAR_MAP_CACHE: RefCell<Option<CscCharMapCacheData>>')
content = content.replace('    pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n    static CSC_CHAR_MAP_CACHE: RefCell<Option<(u64, u32, u32, Vec<usize>)>>', '    static CSC_CHAR_MAP_CACHE: RefCell<Option<CscCharMapCacheData>>')
open('desktop/src/ui/reader_state.rs', 'w', encoding='utf-8').write(content)