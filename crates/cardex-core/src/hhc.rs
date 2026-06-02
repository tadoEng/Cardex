use std::collections::BTreeMap;

use scraper::{ElementRef, Html, Selector};

use crate::model::{CardexError, PageKind, Result, Toc, TocEntry};

pub fn parse_hhc(input: &str) -> Result<Toc> {
    let document = Html::parse_document(input);
    let object_selector = selector("object")?;
    let param_selector = selector("param")?;
    let mut entries = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut root_ul_count: Option<usize> = None;

    for object in document.select(&object_selector) {
        let params = sitemap_params(object, &param_selector);
        let Some(title) = params.get("name").map(|value| normalize_space(value)) else {
            continue;
        };
        if title.is_empty() {
            continue;
        }

        let local = params
            .get("local")
            .map(|value| normalize_local(value))
            .filter(|value| !value.is_empty());
        let ul_count = object
            .ancestors()
            .filter_map(ElementRef::wrap)
            .filter(|element| element.value().name().eq_ignore_ascii_case("ul"))
            .count();
        let root_count = *root_ul_count.get_or_insert(ul_count);
        let depth = ul_count.saturating_sub(root_count);

        stack.truncate(depth);
        let ancestors = stack.clone();
        let kind = infer_kind(&title);
        let base = title_base(&title);
        let interface = infer_interface(&kind, &base, &ancestors);
        let overload_of = match kind {
            PageKind::Method | PageKind::Property => Some(base.clone()),
            _ => None,
        };
        let symbol = infer_symbol(&kind, &base, interface.as_deref());

        entries.push(TocEntry {
            title: title.clone(),
            local,
            depth,
            ancestors,
            kind,
            interface,
            symbol,
            overload_of,
        });

        stack.push(title);
    }

    Ok(Toc { entries })
}

pub(crate) fn infer_kind(title: &str) -> PageKind {
    let lower = title.to_ascii_lowercase();
    if lower.ends_with(" methods")
        || lower.ends_with(" properties")
        || lower.ends_with(" members")
        || lower.ends_with(" fields")
    {
        PageKind::Page
    } else if lower.contains(" method") {
        PageKind::Method
    } else if lower.contains(" interface") {
        PageKind::Interface
    } else if lower.contains(" enumeration") || lower.ends_with(" enum") {
        PageKind::Enum
    } else if lower.contains(" property") {
        PageKind::Property
    } else {
        PageKind::Page
    }
}

pub(crate) fn title_base(title: &str) -> String {
    let mut value = normalize_space(title);
    for suffix in [
        " Method",
        " Property",
        " Interface",
        " Enumeration",
        " Enum",
        " Object",
        " Class",
    ] {
        if let Some(stripped) = value.strip_suffix(suffix) {
            value = stripped.trim_end().to_string();
            break;
        }
    }
    if let Some(stripped) = value.strip_suffix("()") {
        value = stripped.trim_end().to_string();
    }
    value
}

#[allow(dead_code)] // Wired into overload grouping in the next hardening work order.
pub(crate) fn canonical_base(base: &str) -> String {
    if let Some(idx) = base.rfind('_') {
        let suffix = &base[idx + 1..];
        let is_overload =
            suffix.len() == 1 && suffix.as_bytes()[0].is_ascii_digit() && suffix != "0";
        if is_overload {
            return base[..idx].to_string();
        }
    }
    base.to_string()
}

pub(crate) fn normalize_space(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sitemap_params(object: ElementRef<'_>, param_selector: &Selector) -> BTreeMap<String, String> {
    object
        .select(param_selector)
        .filter_map(|param| {
            let name = param.attr("name")?.to_ascii_lowercase();
            let value = param.attr("value")?.to_string();
            Some((name, value))
        })
        .collect()
}

fn infer_interface(kind: &PageKind, base: &str, ancestors: &[String]) -> Option<String> {
    if matches!(kind, PageKind::Interface) {
        return Some(base.to_string());
    }

    ancestors.iter().rev().find_map(|ancestor| {
        if matches!(infer_kind(ancestor), PageKind::Interface) {
            Some(title_base(ancestor))
        } else {
            None
        }
    })
}

fn infer_symbol(kind: &PageKind, base: &str, interface: Option<&str>) -> Option<String> {
    match kind {
        PageKind::Method | PageKind::Property => {
            interface.map(|interface| format!("{interface}.{base}"))
        }
        PageKind::Interface | PageKind::Enum => Some(base.to_string()),
        PageKind::Page => None,
    }
}

fn normalize_local(value: &str) -> String {
    normalize_space(value).replace('\\', "/")
}

fn selector(css: &str) -> Result<Selector> {
    Selector::parse(css).map_err(|err| CardexError::Parse(format!("invalid selector {css}: {err}")))
}

#[cfg(test)]
mod title_base_tests {
    use super::{canonical_base, title_base};

    #[test]
    fn title_base_keeps_code_editions_and_overload_suffixes() {
        assert_eq!(title_base("FrameForce Method"), "FrameForce");
        assert_eq!(title_base("GetNTC2018 Method"), "GetNTC2018");
        assert_eq!(title_base("ACI318_14 Property"), "ACI318_14");
        assert_eq!(title_base("Eurocode_2_2004 Interface"), "Eurocode_2_2004");
        assert_eq!(
            title_base("AssembledJointMass_1 Method"),
            "AssembledJointMass_1"
        );
        assert_eq!(title_base("GetChannel_2 Method"), "GetChannel_2");
        assert_eq!(title_base("GetOffsets3 Method"), "GetOffsets3");
        assert_eq!(
            title_base("AssembledJointMass() Method"),
            "AssembledJointMass"
        );
    }

    #[test]
    fn canonical_base_strips_only_single_digit_overload_suffix() {
        assert_eq!(canonical_base("AssembledJointMass_1"), "AssembledJointMass");
        assert_eq!(canonical_base("GetChannel_2"), "GetChannel");
        assert_eq!(canonical_base("GetASCE716"), "GetASCE716");
        assert_eq!(canonical_base("ACI318_14"), "ACI318_14");
        assert_eq!(canonical_base("Eurocode_2_2004"), "Eurocode_2_2004");
        assert_eq!(canonical_base("FrameForce"), "FrameForce");
    }
}
