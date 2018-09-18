# cang-jie([仓颉](https://en.wikipedia.org/wiki/Cangjie))

[![Crates.io](https://img.shields.io/crates/v/cang-jie.svg)](https://crates.io/crates/cang-jie)
[![Build Status](https://travis-ci.org/DCjanus/cang-jie.svg?branch=master)](https://travis-ci.org/DCjanus/cang-jie)
[![latest document](https://img.shields.io/badge/latest-document-ff69b4.svg)](https://docs.rs/cang-jie/)
[![dependency status](https://deps.rs/repo/github/dcjanus/cang-jie/status.svg)](https://deps.rs/repo/github/dcjanus/cang-jie)

A Chinese tokenizer for [tantivy](https://github.com/tantivy-search/tantivy), based on [jieba-rs](https://github.com/messense/jieba-rs).

As of now, only support UTF-8.

## Example

```rust
    let mut schema_builder = SchemaBuilder::default();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE) // Set custom tokenizer
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();
    // ... Some code   
     let index = Index::create(RAMDirectory::create(), schema.clone())?;
     let tokenizer = CangJieTokenizer {
                        worker: Arc::new(Jieba::empty()), // empty dictionary
                        option: TokenizerOption::Unicode,
                     };
     index.tokenizers().register(CANG_JIE, tokenizer); 
    // ... Some code
```

[Full example](./tests/unicode_split.rs)
