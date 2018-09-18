extern crate cang_jie;
extern crate chrono;
extern crate flexi_logger;
extern crate jieba_rs;
extern crate tantivy;

use cang_jie::{CangJieTokenizer, TokenizerOption};
use chrono::Local;
use flexi_logger::{Logger, Record};
use jieba_rs::Jieba;
use std::{collections::HashSet, io, iter::FromIterator, sync::Arc};
use tantivy::{
    collector::TopCollector, directory::RAMDirectory, query::QueryParser, schema::*, Index,
};

#[test]
fn full_test_unicode_split() -> tantivy::Result<()> {
    Logger::with_env_or_str("cang_jie=trace,error")
        .format(logger_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let mut schema_builder = SchemaBuilder::default();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("cang_jie")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    let title = schema_builder.add_text_field("title", text_options.clone());
    let schema = schema_builder.build();

    let index = Index::create(RAMDirectory::create(), schema.clone())?;
    index.tokenizers().register("cang_jie", tokenizer());

    let mut index_writer = index.writer(50 * 1024 * 1024)?;

    let mut doc = Document::default();
    doc.add_text(title, "南京长江大桥");
    index_writer.add_document(doc);

    let mut doc = Document::default();
    doc.add_text(title, "这个是长江");
    index_writer.add_document(doc);

    let mut doc = Document::default();
    doc.add_text(title, "这个是南京长");
    index_writer.add_document(doc);

    index_writer.commit()?;

    index.load_searchers()?;
    let searcher = index.searcher();

    let query = QueryParser::for_index(&index, vec![title]).parse_query("京长")?;
    let mut top_collector = TopCollector::with_limit(10000);
    searcher.search(query.as_ref(), &mut top_collector)?;

    let actual = top_collector
        .docs()
        .into_iter()
        .map(|x| {
            searcher
                .doc(x)
                .unwrap()
                .get_first(title)
                .unwrap()
                .text()
                .unwrap()
                .to_string()
        }).collect::<HashSet<_>>();

    let expect = HashSet::from_iter(vec![
        "这个是南京长".to_string(),
        "南京长江大桥".to_string(),
    ]);

    assert_eq!(actual, expect);

    Ok(())
}

fn tokenizer() -> CangJieTokenizer {
    CangJieTokenizer {
        worker: Arc::new(Jieba::empty()),
        option: TokenizerOption::Unicode,
    }
}

pub fn logger_format(w: &mut io::Write, record: &Record) -> Result<(), io::Error> {
    write!(
        w,
        "[{}] {} [{}:{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}
