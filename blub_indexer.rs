//
// blub_indexer.rs
//
// Caleb Barger
// 11/30/2024
//

use std::fs::{self, File};
use std::io::{self, Read};
use std::mem::{size_of, transmute};
use std::path::Path;
use std::str;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

#[derive(Debug)]
#[repr(C)]
struct Blub1Header {
    url_len: u16,
    title_len: u16,
    body_len: u32,
}

fn main() -> io::Result<()> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("url", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | FAST | STORED);
    schema_builder.add_text_field("body", TEXT | FAST | STORED);
    let schema = schema_builder.build();

    let blub_index_path = Path::new("blub2-data");
    if !blub_index_path.exists() {
        fs::create_dir(blub_index_path).unwrap();
    }
    let index_file_path = blub_index_path.join("meta.json");
    if index_file_path.exists() {
        fs::remove_file(index_file_path).unwrap()
    }
    let index = Index::create_in_dir(blub_index_path, schema.clone()).unwrap();
    let mut index_writer: IndexWriter = index.writer(1024 * 1024 * 100).unwrap();

    let url = schema.get_field("url").unwrap();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let blub1_data_path = "blub1-data/";
    for domain_entry in fs::read_dir(blub1_data_path)? {
        let domain_entry = domain_entry?;
        let domain_entry_path = domain_entry.path();
        if domain_entry_path.is_dir() {
            for blub1_entry in fs::read_dir(domain_entry_path)? {
                let blub1_entry = blub1_entry?;
                let blub1_entry_path = blub1_entry.path();

                let mut blub1_file = File::open(blub1_entry_path)?;

                // TODO(calebarg): Figure out how to do this with file contents string instead.
                // IDK feels gross to do a read here and then read the entire string.
                let blub1_header: Blub1Header = {
                    let mut h = [0u8; size_of::<Blub1Header>()];
                    blub1_file.read_exact(&mut h[..])?;
                    unsafe { transmute(h) }
                };

                let mut blub1_file_contents: Vec<u8> = Vec::new();
                blub1_file.read_to_end(&mut blub1_file_contents);

                let expected_file_size: usize = size_of::<Blub1Header>()
                    + usize::from(blub1_header.url_len)
                    + usize::from(blub1_header.title_len)
                    + usize::try_from(blub1_header.body_len).unwrap();
                if blub1_file_contents.len() == expected_file_size {
                    println!("potato!");
                    let mut blub1_file_contents_offset: usize = size_of::<Blub1Header>();
                    println!("{:?}", blub1_header);
                    println!("{:?}", blub1_file_contents.len());

                    let url_str = match str::from_utf8(
                        &blub1_file_contents[blub1_file_contents_offset
                            ..blub1_file_contents_offset + usize::from(blub1_header.url_len)],
                    ) {
                        Ok(v) => v,
                        Err(_) => "",
                    };
                    blub1_file_contents_offset += usize::from(blub1_header.url_len);

                    let title_str = match str::from_utf8(
                        &blub1_file_contents[blub1_file_contents_offset
                            ..blub1_file_contents_offset + usize::from(blub1_header.title_len)],
                    ) {
                        Ok(v) => v,
                        Err(_) => "",
                    };
                    blub1_file_contents_offset += usize::from(blub1_header.title_len);

                    let body_str =
                        match str::from_utf8(&blub1_file_contents[blub1_file_contents_offset..]) {
                            Ok(v) => v,
                            Err(_) => "",
                        };

                    let mut doc = TantivyDocument::default();
                    doc.add_text(url, url_str);
                    doc.add_text(title, title_str);
                    doc.add_text(body, body_str);
                    let _ = index_writer.add_document(doc);
                } else {
                    // TODO(calebarg): Clean this error up.
                    println!("Ignoring blub1 file because it wasn't the expected size expected {:?} got {:?}", expected_file_size, blub1_file_contents.len());
                }
            }
        } else {
            unreachable!("blub1-data should only contain directories of blub1 files.");
        }
    }

    index_writer.commit().unwrap();

    Ok(())
}
