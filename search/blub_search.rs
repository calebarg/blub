//
// blub_search.rs
//
// Caleb Barger
// 12/02/2024
//

use serde::Serialize;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::snippet::*;
use tantivy::{Index, IndexReader, ReloadPolicy};

static mut TANTIFY_INDEX: MaybeUninit<Index> = MaybeUninit::<Index>::uninit();
static mut TANTIFY_READER: MaybeUninit<IndexReader> = MaybeUninit::<IndexReader>::uninit();

#[derive(Serialize)]
struct BlubSearchResult {
    url: String,
    title: String,
    body_snippet_html: String,
}

#[unsafe(no_mangle)]
pub extern "C" fn blub_search_init() -> u8 {
    let index = match Index::open_in_dir("blub2-data") {
        Ok(v) => v,
        Err(_) => return u8::from(0),
    };

    let reader = match index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
    {
        Ok(v) => v,
        Err(_) => return u8::from(0),
    };
    unsafe {
        TANTIFY_INDEX.write(index);
        TANTIFY_READER.write(reader);
    }
    return u8::from(1);
}

#[unsafe(no_mangle)]
pub extern "C" fn blub_search(query_strz: *const i8) -> *mut i8 {
    let index = unsafe { TANTIFY_INDEX.assume_init_mut() };
    let reader = unsafe { TANTIFY_READER.assume_init_mut() };
    let searcher = reader.searcher();

    let schema = searcher.schema();

    let url = schema.get_field("url").unwrap();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    let query_cstr = unsafe { CStr::from_ptr(query_strz) };
    let query_str = query_cstr.to_str().unwrap();
    let query = query_parser.parse_query(query_str).unwrap();

    let mut search_results = Vec::<BlubSearchResult>::new();

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
        let body_snippet_generator = SnippetGenerator::create(&searcher, &*query, body).unwrap();
        let body_snippet = body_snippet_generator.snippet_from_doc(&retrieved_doc);

        let url_str = retrieved_doc
            .get_first(url)
            .unwrap()
            .as_str()
            .unwrap_or_default();
        let title_str = retrieved_doc
            .get_first(title)
            .unwrap()
            .as_str()
            .unwrap_or_default();
        let search_result = BlubSearchResult {
            url: url_str.to_owned(),
            title: title_str.to_owned(),
            body_snippet_html: body_snippet.to_html(),
        };
        search_results.push(search_result);
    }

    let search_results_json_str = serde_json::to_string(&search_results).unwrap();
    return CString::new(search_results_json_str).unwrap().into_raw();
}

#[unsafe(no_mangle)]
pub extern "C" fn blub_search_give_back(search_result_strz: *mut i8) {
    let _ = unsafe { CString::from_raw(search_result_strz) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let status = blub_search_init();
        assert_eq!(status, 1);
        blub_search(c"c fopen".as_ptr());
    }
}
