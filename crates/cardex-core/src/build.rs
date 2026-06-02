use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cards::build_card_from_html;
use crate::hhc::parse_hhc;
use crate::model::{BuildOptions, BuildReport, CardexError, DocGraph, Manifest, Result};
use crate::search::build_search_index;

pub fn build_corpus(options: BuildOptions) -> Result<BuildReport> {
    fs::create_dir_all(&options.out_dir)?;
    let hhc_path = find_first_with_extension(&options.source_dir, "hhc")?
        .ok_or_else(|| CardexError::MissingArtifact("no .hhc file found in source dir".into()))?;
    let hhc = fs::read_to_string(&hhc_path)?;
    let toc = parse_hhc(&hhc)?;
    let mut cards = Vec::new();

    for entry in &toc.entries {
        let Some(local) = &entry.local else {
            continue;
        };
        let page_path = join_local_path(&options.source_dir, local);
        if !page_path.exists() {
            continue;
        }
        let html = fs::read_to_string(page_path)?;
        cards.push(build_card_from_html(entry, &html)?);
    }

    let graph = DocGraph {
        members: build_members(&cards),
        related: cards
            .iter()
            .filter_map(|card| {
                card.symbol
                    .clone()
                    .filter(|_| !card.related.is_empty())
                    .map(|symbol| (symbol, card.related.clone()))
            })
            .collect(),
    };

    write_pages_jsonl(&options.out_dir.join("pages.jsonl"), &cards)?;
    write_json(&options.out_dir.join("docgraph.json"), &graph)?;
    write_json(
        &options.out_dir.join("manifest.json"),
        &Manifest {
            corpus: options.corpus.clone(),
            schema_version: 2,
            pages: cards.len(),
            generated_by: "cardex-core".to_string(),
        },
    )?;
    build_search_index(&options.out_dir, &cards)?;

    Ok(BuildReport {
        corpus: options.corpus,
        pages: cards.len(),
        hhc_entries: toc.entries.len(),
        output_dir: options.out_dir,
    })
}

fn build_members(
    cards: &[crate::model::ApiCard],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut members: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for card in cards {
        let (Some(interface), Some(symbol)) = (&card.interface, &card.symbol) else {
            continue;
        };
        if symbol == interface {
            continue;
        }
        members
            .entry(interface.clone())
            .or_default()
            .push(symbol.clone());
    }
    for symbols in members.values_mut() {
        symbols.sort();
        symbols.dedup();
    }
    members
}

fn write_pages_jsonl(path: &Path, cards: &[crate::model::ApiCard]) -> Result<()> {
    let mut file = File::create(path)?;
    for card in cards {
        serde_json::to_writer(&mut file, card)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

fn join_local_path(source_dir: &Path, local: &str) -> PathBuf {
    local
        .replace('\\', "/")
        .split('/')
        .fold(source_dir.to_path_buf(), |path, component| {
            path.join(component)
        })
}

fn find_first_with_extension(root: &Path, extension: &str) -> Result<Option<PathBuf>> {
    if !root.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_first_with_extension(&path, extension)? {
                return Ok(Some(found));
            }
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            return Ok(Some(path));
        }
    }

    Ok(None)
}
