# cang-jie([仓颉](https://en.wikipedia.org/wiki/Cangjie))

[![Crates.io](https://img.shields.io/crates/v/cang-jie.svg)](https://crates.io/crates/cang-jie)
[![latest document](https://img.shields.io/badge/latest-document-ff69b4.svg)](https://docs.rs/cang-jie/)
[![dependency status](https://deps.rs/repo/github/dcjanus/cang-jie/status.svg)](https://deps.rs/repo/github/dcjanus/cang-jie)

Chinese tokenizer integration for [Tantivy](https://github.com/quickwit-oss/tantivy), backed by [jieba-rs](https://github.com/messense/jieba-rs).

## Usage

```rust
use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use std::sync::Arc;
use tantivy::{
    doc,
    schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions},
    Index,
};

fn main() -> tantivy::Result<()> {
    let mut schema_builder = SchemaBuilder::default();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();
    let title = schema_builder.add_text_field("title", text_options);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let tokenizer = CangJieTokenizer {
        worker: Arc::new(Jieba::new()),
        option: TokenizerOption::Default { hmm: false },
    };
    index.tokenizers().register(CANG_JIE, tokenizer);

    let mut index_writer = index.writer(50 * 1024 * 1024)?;
    index_writer.add_document(doc! { title => "南京长江大桥" })?;
    index_writer.commit()?;

    Ok(())
}
```

See [unicode_split.rs](./tests/unicode_split.rs) for a complete searchable example.

## Maintenance

This crate uses Rust 2024 and requires Rust 1.88 or newer.

Install [prek](https://prek.j178.dev/installation/), then run the local checks with:

```console
prek run --all-files
```
