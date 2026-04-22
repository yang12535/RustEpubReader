import re

content = open('desktop/src/ui/reader_state.rs', encoding='utf-8').read()
content = content.replace('pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n\nthread_local! {\n', 'thread_local! {\n')

content = content.replace('    static CSC_CHAR_MAP_CACHE:', '    pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n    static CSC_CHAR_MAP_CACHE:')
open('desktop/src/ui/reader_state.rs', 'w', encoding='utf-8').write(content)