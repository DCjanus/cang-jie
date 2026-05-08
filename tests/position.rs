use std::sync::Arc;

use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions},
    Index, SnippetGenerator, TantivyDocument,
};

#[test]
fn test_tokenizer_position() -> tantivy::Result<()> {
    let mut schema_builder = SchemaBuilder::default();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE) // Set custom tokenizer
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    let title = schema_builder.add_text_field("title", text_options);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    index.tokenizers().register(CANG_JIE, tokenizer()); // Build cang-jie Tokenizer

    let mut index_writer = index.writer(50 * 1024 * 1024)?;
    index_writer.add_document(doc! { title => "南京大桥" })?;
    index_writer.add_document(doc! { title => "这个是长江" })?;
    index_writer.add_document(doc! { title => "这个是南京长" })?;
    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query = QueryParser::for_index(&index, vec![title]).parse_query("南京")?;
    let top_docs = searcher.search(query.as_ref(), &TopDocs::with_limit(10000))?;

    let snippet = SnippetGenerator::create(&searcher, &query, title).unwrap();
    for doc in top_docs.iter() {
        let s = snippet.snippet_from_doc(&searcher.doc::<TantivyDocument>(doc.1).unwrap());
        dbg!(s.to_html());
    }
    Ok(())
}

fn tokenizer() -> CangJieTokenizer {
    CangJieTokenizer {
        worker: Arc::new(Jieba::new()),
        option: TokenizerOption::All,
    }
}
