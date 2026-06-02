use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT};
use tantivy::{Document, Index, TantivyDocument, doc};

use crate::model::{ApiCard, CardexError, Result, SearchHit};

struct SearchFields {
    page_id: Field,
    symbol: Field,
    title: Field,
    interface: Field,
    identifier: Field,
    text: Field,
}

pub(crate) fn build_search_index(out_dir: &Path, cards: &[ApiCard]) -> Result<()> {
    let index_dir = out_dir.join("tantivy");
    if index_dir.exists() {
        fs::remove_dir_all(&index_dir)?;
    }
    fs::create_dir_all(&index_dir)?;

    let (schema, fields) = search_schema();
    let index = Index::create_in_dir(&index_dir, schema)?;
    let mut writer = index.writer(50_000_000)?;

    for card in cards {
        let symbol = card.symbol.as_deref().unwrap_or_default();
        let interface = card.interface.as_deref().unwrap_or_default();
        let identifier = identifier_blob(card);
        let text = search_blob(card);
        writer.add_document(doc!(
            fields.page_id => card.page_id.as_str(),
            fields.symbol => symbol,
            fields.title => card.title.as_str(),
            fields.interface => interface,
            fields.identifier => identifier.as_str(),
            fields.text => text.as_str(),
        ))?;
    }
    writer.commit()?;
    Ok(())
}

pub(crate) fn search_cards(
    root: &Path,
    query: &str,
    limit: usize,
    by_page_id: &HashMap<String, usize>,
    cards: &[ApiCard],
) -> Result<Vec<SearchHit>> {
    let index_dir = root.join("tantivy");
    if !index_dir.exists() {
        return Err(CardexError::MissingArtifact(format!(
            "missing {}",
            index_dir.display()
        )));
    }

    let index = Index::open_in_dir(index_dir)?;
    let schema = index.schema();
    let fields = fields_from_schema(&schema)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let mut query_parser = QueryParser::for_index(
        &index,
        vec![
            fields.identifier,
            fields.symbol,
            fields.title,
            fields.interface,
            fields.text,
        ],
    );
    query_parser.set_conjunction_by_default();
    query_parser.set_field_boost(fields.identifier, 8.0);
    query_parser.set_field_boost(fields.symbol, 5.0);
    query_parser.set_field_boost(fields.title, 3.0);
    let parsed_query = query_parser
        .parse_query(query)
        .or_else(|_| query_parser.parse_query(&sanitize_query(query)))?;
    let top_docs = searcher.search(
        &parsed_query,
        &TopDocs::with_limit(limit.max(1)).order_by_score(),
    )?;
    let mut hits = Vec::new();

    for (score, address) in top_docs {
        let retrieved = searcher.doc::<TantivyDocument>(address)?;
        let json: Value = serde_json::from_str(&retrieved.to_json(&schema))?;
        let Some(page_id) = first_json_string(&json, "page_id") else {
            continue;
        };
        let Some(card) = by_page_id.get(page_id).and_then(|index| cards.get(*index)) else {
            continue;
        };
        hits.push(SearchHit {
            page_id: card.page_id.clone(),
            title: card.title.clone(),
            kind: card.kind.clone(),
            interface: card.interface.clone(),
            symbol: card.symbol.clone(),
            summary: card.summary.clone(),
            score,
        });
    }

    Ok(hits)
}

fn search_schema() -> (Schema, SearchFields) {
    let mut builder = Schema::builder();
    let page_id = builder.add_text_field("page_id", STRING | STORED);
    let symbol = builder.add_text_field("symbol", TEXT | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let interface = builder.add_text_field("interface", TEXT | STORED);
    let identifier = builder.add_text_field("identifier", TEXT);
    let text = builder.add_text_field("text", TEXT);
    let schema = builder.build();
    (
        schema,
        SearchFields {
            page_id,
            symbol,
            title,
            interface,
            identifier,
            text,
        },
    )
}

fn fields_from_schema(schema: &Schema) -> Result<SearchFields> {
    Ok(SearchFields {
        page_id: schema.get_field("page_id")?,
        symbol: schema.get_field("symbol")?,
        title: schema.get_field("title")?,
        interface: schema.get_field("interface")?,
        identifier: schema.get_field("identifier")?,
        text: schema.get_field("text")?,
    })
}

fn identifier_blob(card: &ApiCard) -> String {
    let parts = [
        Some(card.title.as_str()),
        card.symbol.as_deref(),
        card.interface.as_deref(),
        card.overload_of.as_deref(),
    ]
    .into_iter()
    .flatten()
    .flat_map(|value| [value.to_string(), split_identifier_words(value).join(" ")])
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" ");

    std::iter::repeat_n(parts, 4).collect::<Vec<_>>().join("\n")
}

fn search_blob(card: &ApiCard) -> String {
    [
        Some(card.title.as_str()),
        card.symbol.as_deref(),
        card.interface.as_deref(),
        card.signature_cs.as_deref(),
        card.signature_vb.as_deref(),
        card.returns.as_deref(),
        card.remarks.as_deref(),
        Some(card.raw_text.as_str()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n")
}

fn sanitize_query(query: &str) -> String {
    query
        .split(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_identifier_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();

    for segment in value.split(|ch: char| !(ch.is_ascii_alphanumeric())) {
        let mut current = String::new();
        let mut previous: Option<char> = None;

        for ch in segment.chars() {
            if previous.is_some_and(|prev| {
                (prev.is_ascii_lowercase() || prev.is_ascii_digit()) && ch.is_ascii_uppercase()
            }) && !current.is_empty()
            {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(ch);
            previous = Some(ch);
        }

        if !current.is_empty() {
            words.push(current.to_ascii_lowercase());
        }
    }

    words
}

fn first_json_string<'a>(json: &'a Value, field: &str) -> Option<&'a str> {
    match json.get(field)? {
        Value::String(value) => Some(value),
        Value::Array(values) => values.first().and_then(Value::as_str),
        _ => None,
    }
}
