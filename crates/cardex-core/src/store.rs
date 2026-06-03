use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::model::{
    ApiCard, CardexError, DocGraph, Result, SearchExplanation, SearchHit, SearchPromotion,
};
use crate::query::{QueryPlan, plan_query};
use crate::search::{QueryMode, search_cards};

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
        Ok(self.search_explained(query, limit)?.hits)
    }

    pub fn search_explained(&self, query: &str, limit: usize) -> Result<SearchExplanation> {
        let limit = limit.max(1);
        let seed_limit = limit.max(20);
        let plan = plan_query(query, &self.cards);
        let strict = search_cards(
            &self.root,
            query,
            seed_limit,
            &self.by_page_id,
            &self.cards,
            QueryMode::Strict,
        )?;
        let mut seed_hits = strict
            .into_iter()
            .filter(|hit| {
                hit.symbol
                    .as_deref()
                    .is_none_or(|symbol| plan.allows_symbol(symbol))
            })
            .collect::<Vec<_>>();
        let mut stage = "strict".to_string();

        if plan.has_expansion() {
            let expanded = search_cards(
                &self.root,
                &plan.expanded_query,
                seed_limit,
                &self.by_page_id,
                &self.cards,
                QueryMode::Relaxed,
            )?
            .into_iter()
            .filter(|hit| {
                hit.symbol
                    .as_deref()
                    .is_none_or(|symbol| plan.allows_symbol(symbol))
            })
            .collect();
            if seed_hits.is_empty() {
                stage = "expanded".to_string();
            } else {
                stage = "strict+expanded".to_string();
            }
            merge_hits(&mut seed_hits, expanded);
        }

        let mut seed_symbols = seed_hits
            .iter()
            .filter_map(|hit| hit.symbol.clone())
            .collect::<Vec<_>>();
        for symbol in self.graph_seed_symbols(&plan) {
            if !seed_symbols.contains(&symbol) {
                seed_symbols.push(symbol);
            }
        }
        let (hits, promotions) = self.rank_with_graph(&plan, seed_hits, limit);
        let version_scope = plan.version_scope_label();

        Ok(SearchExplanation {
            original_query: plan.original_query,
            expanded_query: plan.expanded_query,
            normalized_terms: plan.normalized_terms,
            expanded_terms: plan.expanded_terms,
            version_scope,
            stage,
            seed_symbols,
            promotions,
            hits,
        })
    }

    fn rank_with_graph(
        &self,
        plan: &QueryPlan,
        seed_hits: Vec<SearchHit>,
        limit: usize,
    ) -> (Vec<SearchHit>, Vec<SearchPromotion>) {
        let mut by_key: HashMap<String, SearchHit> = HashMap::new();
        let mut work = VecDeque::new();

        for hit in seed_hits {
            let score = hit.score + self.intent_boost(plan, &hit);
            if let Some(symbol) = hit.symbol.clone() {
                work.push_back((
                    symbol,
                    hit.symbol.clone().unwrap_or_default(),
                    score,
                    0usize,
                ));
            }
            insert_best_hit(&mut by_key, SearchHit { score, ..hit });
        }

        for symbol in self.graph_seed_symbols(plan) {
            if let Some(card) = self.card_by_symbol(&symbol) {
                let mut hit = self.hit_from_card(card, 120.0);
                hit.score += self.intent_boost(plan, &hit);
                insert_best_hit(&mut by_key, hit);
            }
            work.push_back((symbol.clone(), symbol, 120.0, 0usize));
        }

        let mut promotions = Vec::new();
        let mut seen_promotions = HashSet::new();
        let mut visited = HashSet::new();

        while let Some((symbol, seed_symbol, seed_score, depth)) = work.pop_front() {
            if depth >= 3 || !visited.insert((symbol.clone(), depth)) {
                continue;
            }

            for (target, reason) in self.graph_neighbors(&symbol) {
                if target == seed_symbol {
                    continue;
                }
                if !plan.allows_symbol(&target) {
                    continue;
                }
                let Some(card) = self.card_by_symbol(&target) else {
                    continue;
                };
                let promotion_key = format!("{target}|{reason}");
                if seen_promotions.insert(promotion_key) {
                    promotions.push(SearchPromotion {
                        symbol: target.clone(),
                        seed_symbol: seed_symbol.clone(),
                        reason: reason.clone(),
                    });
                }

                let base = if depth == 0 { 0.72 } else { 0.52 };
                let mut promoted_hit = self.hit_from_card(card, seed_score * base);
                promoted_hit.score += self.intent_boost(plan, &promoted_hit);
                insert_best_hit(&mut by_key, promoted_hit);

                if depth < 2 {
                    work.push_back((target, seed_symbol.clone(), seed_score * base, depth + 1));
                }
            }
        }

        let mut hits = by_key.into_values().collect::<Vec<_>>();
        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.symbol.cmp(&right.symbol))
        });
        hits.truncate(limit);
        let final_symbols = hits
            .iter()
            .filter_map(|hit| hit.symbol.as_deref())
            .collect::<HashSet<_>>();
        promotions.retain(|promotion| final_symbols.contains(promotion.symbol.as_str()));
        promotions.truncate(50);
        (hits, promotions)
    }

    fn graph_neighbors(&self, symbol: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(interface) = self.graph.returns_interface.get(symbol) {
            out.push((interface.clone(), format!("returns_interface:{symbol}")));
        }
        if let Some(members) = self.graph.members.get(symbol) {
            out.extend(
                members
                    .iter()
                    .cloned()
                    .map(|member| (member, format!("member_of:{symbol}"))),
            );
        }
        if let Some(related) = self.graph.related.get(symbol) {
            out.extend(
                related
                    .iter()
                    .cloned()
                    .map(|target| (target, format!("related_to:{symbol}"))),
            );
        }
        out
    }

    fn graph_seed_symbols(&self, plan: &QueryPlan) -> Vec<String> {
        let mut seeds = Vec::new();
        for term in &plan.expanded_terms {
            if (self.graph.members.contains_key(term)
                || self.graph.returns_interface.contains_key(term)
                || self.by_symbol.contains_key(term))
                && !seeds.contains(term)
            {
                seeds.push(term.clone());
            }
        }
        seeds
    }

    fn card_by_symbol(&self, symbol: &str) -> Option<&ApiCard> {
        self.by_symbol
            .get(symbol)
            .and_then(|index| self.cards.get(*index))
    }

    fn hit_from_card(&self, card: &ApiCard, score: f32) -> SearchHit {
        SearchHit {
            page_id: card.page_id.clone(),
            title: card.title.clone(),
            kind: card.kind.clone(),
            interface: card.interface.clone(),
            symbol: card.symbol.clone(),
            summary: card.summary.clone(),
            score,
        }
    }

    fn intent_boost(&self, plan: &QueryPlan, hit: &SearchHit) -> f32 {
        let symbol = hit.symbol.as_deref().unwrap_or_default();
        let title = hit.title.as_str();
        let mut boost = 0.0;

        if plan.wants_preference && (symbol.contains("Preference") || title.contains("Preference"))
        {
            boost += 45.0;
        }
        if plan.wants_overwrite && (symbol.contains("Overwrite") || title.contains("Overwrite")) {
            boost += 45.0;
        }
        if plan.wants_count {
            if symbol == "eFramePropType" {
                boost += 160.0;
            } else if symbol.ends_with(".Count") {
                boost += 35.0;
            }
        }
        if plan.matches_version_scope(symbol) {
            boost += 20.0;
        }

        boost
    }
}

fn merge_hits(existing: &mut Vec<SearchHit>, extra: Vec<SearchHit>) {
    for hit in extra {
        if let Some(old_hit) = existing
            .iter_mut()
            .find(|old| hit_key(old).as_str() == hit_key(&hit).as_str())
        {
            if hit.score > old_hit.score {
                *old_hit = hit;
            }
        } else {
            existing.push(hit);
        }
    }
}

fn insert_best_hit(by_key: &mut HashMap<String, SearchHit>, hit: SearchHit) {
    let key = hit_key(&hit);
    match by_key.get_mut(&key) {
        Some(existing) if existing.score >= hit.score => {}
        Some(existing) => *existing = hit,
        None => {
            by_key.insert(key, hit);
        }
    }
}

fn hit_key(hit: &SearchHit) -> String {
    hit.symbol.clone().unwrap_or_else(|| hit.page_id.clone())
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
