use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use std::sync::Arc;
use tantivy::{
    collector::TopDocs, doc, query::QueryParser, schema::*, Index, ReloadPolicy, SnippetGenerator,
};

fn tokenizer() -> CangJieTokenizer {
    CangJieTokenizer {
        worker: Arc::new(Jieba::empty()), // empty dictionary
        option: TokenizerOption::Unicode,
    }
}
#[test]
fn baseline() {
    let mut schema_builder = Schema::builder();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE) // Set custom tokenizer
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    schema_builder.add_text_field("title", text_options);

    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());

    index.tokenizers().register(CANG_JIE, tokenizer());
    let mut index_writer = index.writer(50_000_000).unwrap();

    let title = schema.get_field("title").unwrap();
    index_writer.add_document(doc! {title => "Old Man and the Sea"});
    index_writer.add_document(doc! {title => "老人与海"});
    index_writer.commit().unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
        .unwrap();

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![title]);
    let query = query_parser.parse_query("Old").unwrap();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    let snip_gen = SnippetGenerator::create(&searcher, &*query, title).unwrap();

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        let s = snip_gen.snippet_from_doc(&retrieved_doc);
        println!("{:#?}", s.to_html());
    }
}
