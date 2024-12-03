//
// blub_indexer.rs
//
// Caleb Barger
// 11/30/2024
//

use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::mem::{size_of};
use std::path::Path;
use std::str;
use std::time::{Instant};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter};

#[derive(Debug)]
#[repr(C, packed)]
struct Blub1Header {
    url_len: u16,
    title_len: u16,
    body_len: u32,
}

#[derive(Serialize)]
struct DomainIndexInfo {
    domain: String,
    pages_indexed: usize,
}

const MAX_PAGES_TO_INDEX: usize = 10000;

fn main() -> io::Result<()> {
    println!("Indexer start!");
    let indexer_start = Instant::now();

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("url", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | FAST | STORED);
    schema_builder.add_text_field("body", TEXT | FAST | STORED);
    let schema = schema_builder.build();

    let blub2_data_path = Path::new("blub2-data");
    if !blub2_data_path.exists() {
        fs::create_dir(blub2_data_path).unwrap();
    }
    let index_file_path = blub2_data_path.join("meta.json");
    if index_file_path.exists() {
        fs::remove_file(index_file_path).unwrap()
    }
    let index = Index::create_in_dir(blub2_data_path, schema.clone()).unwrap();
    let mut index_writer: IndexWriter = index.writer(1024 * 1024 * 100).unwrap();

    let url = schema.get_field("url").unwrap();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let mut indexed_domains_info = Vec::<DomainIndexInfo>::new();

    let blub1_data_path = "blub1-data/";
    for domain_entry in fs::read_dir(blub1_data_path)? {
        let domain_entry = domain_entry?;
        let domain_entry_path = domain_entry.path();
        let domain_name = domain_entry_path.file_name().unwrap();
        let mut pages_indexed: usize = 0;
        if domain_entry_path.is_dir() {
            for blub1_entry in fs::read_dir(&domain_entry_path)? {
                let blub1_entry = blub1_entry?;
                let blub1_entry_path = blub1_entry.path();

                let mut blub1_file = File::open(blub1_entry_path).unwrap();
                let mut blub1_file_contents: Vec<u8> = Vec::new();
                let _ = blub1_file.read_to_end(&mut blub1_file_contents);

                let (_, blub1_headers, _) =
                    unsafe { blub1_file_contents.align_to::<Blub1Header>() };
                if blub1_headers.len() > 0 {
                    let blub1_header = &blub1_headers[0];

                    let expected_file_size: usize = size_of::<Blub1Header>()
                        + usize::from(blub1_header.url_len)
                        + usize::from(blub1_header.title_len)
                        + usize::try_from(blub1_header.body_len).unwrap();
                    if blub1_file_contents.len() == expected_file_size {
                        let mut blub1_file_contents_offset: usize = size_of::<Blub1Header>();

                        let url_str = match str::from_utf8(
                            &blub1_file_contents[blub1_file_contents_offset
                                ..blub1_file_contents_offset + usize::from(blub1_header.url_len)],
                        ) {
                            Ok(v) => v,
                            Err(_) => "",
                        };
                        blub1_file_contents_offset += usize::from(blub1_header.url_len);

                        let mut end_title_idx =
                            blub1_file_contents_offset + usize::from(blub1_header.title_len);
                        if blub1_header.title_len > 1024 { // NOTE(calebarg): 1K is maybe to generous
                                                           // for the title...
                            end_title_idx = blub1_file_contents_offset + 1024;
                        }
                        let title_str = match str::from_utf8(
                            &blub1_file_contents[blub1_file_contents_offset..end_title_idx],
                        ) {
                            Ok(v) => v,
                            Err(_) => "",
                        };
                        blub1_file_contents_offset += usize::from(blub1_header.title_len);

                        let mut end_body_idx = blub1_file_contents_offset
                            + usize::try_from(blub1_header.body_len).unwrap();
                        if blub1_header.body_len > 1024 * 50 {
                            end_body_idx = blub1_file_contents_offset + 1024 * 50;
                        }
                        let body_str = match str::from_utf8(
                            &blub1_file_contents[blub1_file_contents_offset..end_body_idx],
                        ) {
                            Ok(v) => v,
                            Err(_) => "",
                        };

                        let mut doc = TantivyDocument::default();
                        doc.add_text(url, url_str);
                        doc.add_text(title, title_str);
                        doc.add_text(body, body_str);
                        let _ = index_writer.add_document(doc);

                        pages_indexed += 1;
                        if pages_indexed >= MAX_PAGES_TO_INDEX {
                            break
                        }
                    } else {
                        // TODO(calebarg): Clean this error up.
                        println!("WARN: Discarding blub1 file because it wasn't the expected size expected {:?} got {:?}", expected_file_size, blub1_file_contents.len());
                    }
                } else {
                    // TODO(calebarg): Clean this error up.
                    println!("WARN: Discarding blub1 file because couldn't deserialize header");
                }
            }
        } else {
            unreachable!("blub1-data should only contain directories of blub1 files.");
        }

        let index_info = DomainIndexInfo {
            domain: String::from(domain_name.to_str().unwrap()),
            pages_indexed: pages_indexed,
        };
        indexed_domains_info.push(index_info);
    }

    index_writer.commit().unwrap();

    let indexed_domains_info_json = serde_json::to_string(&indexed_domains_info).unwrap();
    let indexed_domains_info_file_path = blub2_data_path.join("indexed_domains_info.json");
    let mut indexed_domains_info_file = File::create(&indexed_domains_info_file_path).unwrap();
    let _ = indexed_domains_info_file.write(indexed_domains_info_json.as_bytes()).unwrap();

    let indexer_end = Instant::now();
    println!(
        "Indexing completed in {:?}",
        indexer_end.duration_since(indexer_start)
    );

    Ok(())
}
