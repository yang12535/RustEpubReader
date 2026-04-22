import re

content = open('desktop/src/ui/reader_state.rs', encoding='utf-8').read()
content = content.replace('    type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n    static CSC_CHAR_MAP_CACHE:', '    static CSC_CHAR_MAP_CACHE:')
content = content.replace('thread_local! {\n', 'pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\n\nthread_local! {\n')
open('desktop/src/ui/reader_state.rs', 'w', encoding='utf-8').write(content)