use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::cards::build_card_from_html;
use crate::hhc::parse_hhc;
use crate::model::{
    ApiCard, BuildOptions, BuildReport, CardexError, DocGraph, Manifest, PageKind, Result,
};
use crate::search::build_search_index;

pub fn build_corpus(options: BuildOptions) -> Result<BuildReport> {
    fs::create_dir_all(&options.out_dir)?;
    let hhc_path = find_first_with_extension(&options.source_dir, "hhc")?
        .ok_or_else(|| CardexError::MissingArtifact("no .hhc file found in source dir".into()))?;
    let hhc = fs::read_to_string(&hhc_path)?;
    let toc = parse_hhc(&hhc)?;
    let source_dir_sha256 = digest_source_directory(&options.source_dir)?;
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
        let mut card = build_card_from_html(entry, &html)?;
        card.content_sha256 = digest_card(&card)?;
        cards.push(card);
    }

    let corpus_sha256 = digest_corpus(&cards);

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
        returns_interface: build_returns_interface(&cards),
    };

    write_pages_jsonl(&options.out_dir.join("pages.jsonl"), &cards)?;
    write_json(&options.out_dir.join("docgraph.json"), &graph)?;
    write_json(
        &options.out_dir.join("manifest.json"),
        &Manifest {
            corpus: options.corpus.clone(),
            schema_version: 4,
            pages: cards.len(),
            generated_by: "cardex-core".to_string(),
            product_name: options.product_name,
            source_docs_version: options.source_docs_version,
            source_docs_build: options.source_docs_build,
            source_dir_sha256: source_dir_sha256.clone(),
            corpus_sha256: corpus_sha256.clone(),
        },
    )?;
    build_search_index(&options.out_dir, &cards)?;

    Ok(BuildReport {
        corpus: options.corpus,
        pages: cards.len(),
        hhc_entries: toc.entries.len(),
        output_dir: options.out_dir,
        source_dir_sha256,
        corpus_sha256,
    })
}

fn digest_card(card: &ApiCard) -> Result<String> {
    let mut canonical = card.clone();
    canonical.content_sha256.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(hex_digest(&bytes))
}

fn digest_corpus(cards: &[ApiCard]) -> String {
    let mut identities = cards
        .iter()
        .map(|card| (card.page_id.as_str(), card.content_sha256.as_str()))
        .collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = Sha256::new();
    for (page_id, digest) in identities {
        update_length_prefixed(&mut hasher, page_id.as_bytes());
        update_length_prefixed(&mut hasher, digest.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn digest_source_directory(root: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_source_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = Sha256::new();
    for (relative, path) in files {
        update_length_prefixed(&mut hasher, relative.as_bytes());
        update_length_prefixed(&mut hasher, &fs::read(path)?);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_source_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|err| CardexError::Parse(err.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, path));
        }
    }
    Ok(())
}

fn update_length_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn build_members(cards: &[ApiCard]) -> BTreeMap<String, Vec<String>> {
    let mut members: BTreeMap<String, Vec<String>> = BTreeMap::new();
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

fn build_returns_interface(cards: &[ApiCard]) -> BTreeMap<String, String> {
    let known_interfaces = known_interfaces(cards);
    cards
        .iter()
        .filter(|card| matches!(card.kind, PageKind::Property))
        .filter_map(|card| {
            let symbol = card.symbol.as_ref()?;
            let interface = first_known_return_interface(card, &known_interfaces)?;
            Some((symbol.clone(), interface))
        })
        .collect()
}

fn known_interfaces(cards: &[ApiCard]) -> BTreeSet<String> {
    let mut interfaces = BTreeSet::new();
    for card in cards {
        if let Some(interface) = &card.interface {
            interfaces.insert(interface.clone());
        }
        if card
            .symbol
            .as_ref()
            .is_some_and(|symbol| card.interface.as_ref() == Some(symbol))
        {
            interfaces.insert(card.symbol.clone().unwrap_or_default());
        }
    }
    interfaces
}

fn first_known_return_interface(
    card: &ApiCard,
    known_interfaces: &BTreeSet<String>,
) -> Option<String> {
    if let Some(token) = card
        .signature_cs
        .as_deref()
        .and_then(first_identifier_token)
        .filter(|token| known_interfaces.contains(token))
    {
        return Some(token);
    }

    if let Some(token) = card
        .signature_vb
        .as_deref()
        .and_then(first_vb_as_type)
        .filter(|token| known_interfaces.contains(token))
    {
        return Some(token);
    }

    None
}

fn first_identifier_token(source: &str) -> Option<String> {
    source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|token| !token.is_empty())
        .map(ToString::to_string)
}

fn first_vb_as_type(source: &str) -> Option<String> {
    let tokens = source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens
        .windows(2)
        .find(|window| window[0].eq_ignore_ascii_case("as"))
        .map(|window| window[1].to_string())
}

fn write_pages_jsonl(path: &Path, cards: &[ApiCard]) -> Result<()> {
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

    let mut entries = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
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
