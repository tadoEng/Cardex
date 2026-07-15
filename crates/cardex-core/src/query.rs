use std::collections::BTreeSet;

use crate::model::ApiCard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryPlan {
    pub(crate) original_query: String,
    pub(crate) normalized_terms: Vec<String>,
    pub(crate) expanded_terms: Vec<String>,
    pub(crate) expanded_query: String,
    pub(crate) version_scope: VersionScope,
    pub(crate) wants_preference: bool,
    pub(crate) wants_overwrite: bool,
    pub(crate) wants_count: bool,
}

impl QueryPlan {
    pub(crate) fn has_expansion(&self) -> bool {
        self.expanded_terms.len() > self.normalized_terms.len()
    }

    pub(crate) fn version_scope_label(&self) -> Option<String> {
        match &self.version_scope {
            VersionScope::None => None,
            VersionScope::Explicit { requested, .. } => Some(requested.clone()),
            VersionScope::Family(_) => Some("all_aci318_versions".to_string()),
        }
    }

    pub(crate) fn allows_symbol(&self, symbol: &str) -> bool {
        let Some(version) = aci318_version_token(symbol) else {
            return true;
        };

        match &self.version_scope {
            VersionScope::None => true,
            VersionScope::Explicit { allowed, .. } | VersionScope::Family(allowed) => {
                allowed.contains(&version)
            }
        }
    }

    pub(crate) fn matches_version_scope(&self, symbol: &str) -> bool {
        let Some(version) = aci318_version_token(symbol) else {
            return false;
        };

        match &self.version_scope {
            VersionScope::None => true,
            VersionScope::Explicit { allowed, .. } | VersionScope::Family(allowed) => {
                allowed.contains(&version)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VersionScope {
    None,
    Explicit {
        requested: String,
        allowed: BTreeSet<String>,
    },
    Family(BTreeSet<String>),
}

pub(crate) fn plan_query(query: &str, cards: &[ApiCard]) -> QueryPlan {
    let available_versions = collect_aci318_versions(cards);
    let normalized_terms = normalized_terms(query);
    let lower = query.to_ascii_lowercase();
    let wants_preference = contains_any(
        &lower,
        &[
            "requirement",
            "requirements",
            "setting",
            "settings",
            "option",
            "options",
            "preference",
            "preferences",
        ],
    );
    let wants_overwrite = contains_any(&lower, &["overwrite", "overwrites", "frame-specific"]);
    let wants_concrete_design = lower.contains("concrete") && lower.contains("design");
    let wants_section_definition = lower.contains("section")
        && contains_any(&lower, &["define", "defined", "support", "supports"]);
    let wants_count = contains_any(&lower, &["how many", "count", "number of", "total"]);
    let version_scope = detect_aci318_scope(query, &available_versions);

    let mut expanded_terms = normalized_terms.clone();
    match &version_scope {
        VersionScope::None => {}
        VersionScope::Explicit { requested, allowed } => {
            push_unique(&mut expanded_terms, requested);
            for version in allowed {
                push_unique(&mut expanded_terms, version);
                push_unique(&mut expanded_terms, &format!("cDCo{version}"));
            }
        }
        VersionScope::Family(versions) => {
            for version in versions {
                push_unique(&mut expanded_terms, version);
                push_unique(&mut expanded_terms, &format!("cDCo{version}"));
            }
        }
    }

    if wants_concrete_design {
        push_unique(&mut expanded_terms, "cDesignConcrete");
        push_unique(&mut expanded_terms, "cDCo");
    }
    if wants_preference {
        push_unique(&mut expanded_terms, "Preference");
        push_unique(&mut expanded_terms, "GetPreference");
        push_unique(&mut expanded_terms, "SetPreference");
    }
    if wants_overwrite {
        push_unique(&mut expanded_terms, "Overwrite");
        push_unique(&mut expanded_terms, "GetOverwrite");
        push_unique(&mut expanded_terms, "SetOverwrite");
    }
    if wants_section_definition {
        push_unique(&mut expanded_terms, "eFramePropType");
        push_unique(&mut expanded_terms, "cPropFrame");
        push_unique(&mut expanded_terms, "PropFrame");
        push_unique(&mut expanded_terms, "frame");
        push_unique(&mut expanded_terms, "property");
        push_unique(&mut expanded_terms, "section");
        push_unique(&mut expanded_terms, "Set");
    }
    if wants_count {
        push_unique(&mut expanded_terms, "Count");
        push_unique(&mut expanded_terms, "Enumeration");
        push_unique(&mut expanded_terms, "Members");
    }

    let expanded_query = expanded_terms.join(" ");
    QueryPlan {
        original_query: query.to_string(),
        normalized_terms,
        expanded_terms,
        expanded_query,
        version_scope,
        wants_preference,
        wants_overwrite,
        wants_count,
    }
}

pub(crate) fn aci318_version_token(value: &str) -> Option<String> {
    let upper = value.to_ascii_uppercase();
    let start = upper.find("ACI318_")?;
    let suffix = &value[start + "ACI318_".len()..];
    let token_suffix = suffix
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    if token_suffix.is_empty() {
        return None;
    }
    Some(format!("ACI318_{token_suffix}"))
}

fn collect_aci318_versions(cards: &[ApiCard]) -> BTreeSet<String> {
    let mut versions = BTreeSet::new();
    for card in cards {
        for value in [
            card.symbol.as_deref(),
            card.interface.as_deref(),
            Some(card.title.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(version) = aci318_version_token(value) {
                versions.insert(version);
            }
        }
    }
    versions
}

fn detect_aci318_scope(query: &str, available_versions: &BTreeSet<String>) -> VersionScope {
    let Some(requested) = explicit_aci318_version(query) else {
        return if has_bare_aci318(query) && !available_versions.is_empty() {
            VersionScope::Family(available_versions.clone())
        } else {
            VersionScope::None
        };
    };

    let mut allowed = available_versions
        .iter()
        .filter(|version| version.starts_with(&requested))
        .cloned()
        .collect::<BTreeSet<_>>();
    if allowed.is_empty() {
        allowed.insert(requested.clone());
    }
    VersionScope::Explicit { requested, allowed }
}

fn explicit_aci318_version(query: &str) -> Option<String> {
    let compact = compact_alphanumeric(query);
    let start = compact.find("ACI318")?;
    let suffix = compact[start + "ACI318".len()..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if suffix.len() < 2 {
        return None;
    }

    let year = if suffix.len() >= 4 {
        &suffix[suffix.len() - 2..]
    } else {
        &suffix[..2]
    };
    Some(format!("ACI318_{year}"))
}

fn has_bare_aci318(query: &str) -> bool {
    compact_alphanumeric(query).contains("ACI318")
}

fn normalized_terms(query: &str) -> Vec<String> {
    let mut terms = query
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if let Some(version) = explicit_aci318_version(query) {
        push_unique(&mut terms, &version);
    }
    terms
}

fn compact_alphanumeric(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn push_unique(terms: &mut Vec<String>, term: &str) {
    if !terms.iter().any(|existing| existing == term) {
        terms.push(term.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PageKind;

    #[test]
    fn explicit_aci318_version_locks_to_requested_year() {
        let cards = vec![card("cDCoACI318_14"), card("cDCoACI318_19")];

        let plan = plan_query("ACI 318-14 concrete frame design requirement", &cards);

        assert_eq!(plan.version_scope_label().as_deref(), Some("ACI318_14"));
        assert!(plan.allows_symbol("cDCoACI318_14.GetPreference"));
        assert!(!plan.allows_symbol("cDCoACI318_19.GetPreference"));
    }

    #[test]
    fn bare_aci318_keeps_all_available_versions() {
        let cards = vec![card("cDCoACI318_14"), card("cDCoACI318_19")];

        let plan = plan_query("ACI 318 concrete design", &cards);

        assert_eq!(
            plan.version_scope_label().as_deref(),
            Some("all_aci318_versions")
        );
        assert!(plan.allows_symbol("cDCoACI318_14.GetPreference"));
        assert!(plan.allows_symbol("cDCoACI318_19.GetPreference"));
    }

    fn card(symbol: &str) -> ApiCard {
        ApiCard {
            page_id: format!("{symbol}.htm"),
            title: symbol.to_string(),
            kind: PageKind::Interface,
            interface: Some(symbol.to_string()),
            symbol: Some(symbol.to_string()),
            overload_of: None,
            signature_cs: None,
            signature_vb: None,
            parameters: Vec::new(),
            returns: None,
            remarks: None,
            related: Vec::new(),
            examples: Vec::new(),
            summary: None,
            raw_text: String::new(),
            content_sha256: String::new(),
        }
    }
}
