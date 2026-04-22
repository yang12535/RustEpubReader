content = open('desktop/src/ui/reader_state.rs', encoding='utf-8').read()
content = content.replace('pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);\npub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);', 'pub(crate) type CscCharMapCacheData = (u64, u32, u32, Vec<usize>);')
open('desktop/src/ui/reader_state.rs', 'w', encoding='utf-8').write(content)