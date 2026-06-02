use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::model::{ApiCard, CardexError, DocGraph, Result, SearchHit};
use crate::search::search_cards;

pub struct CardStore {
    root: PathBuf,
    cards: Vec<ApiCard>,
    by_page_id: HashMap<String, usize>,
    by_symbol: HashMap<String, usize>,
    graph: DocGraph,
}

impl CardStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let pages_path = root.join("pages.jsonl");
        if !pages_path.exists() {
            return Err(CardexError::MissingArtifact(format!(
                "missing {}",
                pages_path.display()
            )));
        }

        let cards = load_cards(&pages_path)?;
        let graph_path = root.join("docgraph.json");
        let graph = if graph_path.exists() {
            serde_json::from_reader(File::open(graph_path)?)?
        } else {
            DocGraph::default()
        };
        let mut by_page_id = HashMap::new();
        let mut by_symbol = HashMap::new();
        for (index, card) in cards.iter().enumerate() {
            by_page_id.insert(card.page_id.clone(), index);
            if let Some(symbol) = &card.symbol {
                by_symbol.insert(symbol.clone(), index);
            }
        }

        Ok(Self {
            root,
            cards,
            by_page_id,
            by_symbol,
            graph,
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<ApiCard>> {
        if let Some(index) = self.by_symbol.get(key).or_else(|| self.by_page_id.get(key)) {
            return Ok(Some(self.cards[*index].clone()));
        }

        let key_lower = key.to_ascii_lowercase();
        Ok(self
            .cards
            .iter()
            .find(|card| {
                card.symbol
                    .as_deref()
                    .is_some_and(|symbol| symbol.eq_ignore_ascii_case(&key_lower))
                    || card.page_id.eq_ignore_ascii_case(key)
            })
            .cloned())
    }

    pub fn members(&self, interface: &str) -> Result<Vec<String>> {
        Ok(self
            .graph
            .members
            .get(interface)
            .cloned()
            .unwrap_or_default())
    }

    pub fn related(&self, symbol: &str) -> Result<Vec<String>> {
        if let Some(list) = self.graph.related.get(symbol) {
            return Ok(list.clone());
        }

        if let Some(card) = self.get(symbol)? {
            return Ok(card.related);
        }

        Ok(Vec::new())
    }

    /// All cards whose canonical family equals the canonical family of `key`.
    pub fn overloads(&self, key: &str) -> Result<Vec<String>> {
        let Some(target) = self.get(key)? else {
            return Ok(Vec::new());
        };
        let Some(family) = target.overload_of else {
            return Ok(Vec::new());
        };
        let interface = target.interface;

        let mut out: Vec<String> = self
            .cards
            .iter()
            .filter(|card| {
                card.overload_of.as_ref() == Some(&family)
                    && card.interface.as_ref() == interface.as_ref()
            })
            .filter_map(|card| card.symbol.clone())
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        search_cards(&self.root, query, limit, &self.by_page_id, &self.cards)
    }
}

fn load_cards(path: &Path) -> Result<Vec<ApiCard>> {
    let reader = BufReader::new(File::open(path)?);
    let mut cards = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        cards.push(serde_json::from_str(&line)?);
    }
    Ok(cards)
}

#[allow(dead_code)]
fn read_to_string(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
