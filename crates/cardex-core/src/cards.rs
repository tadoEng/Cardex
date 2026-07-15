use scraper::{ElementRef, Html, Selector};

use crate::hhc::normalize_space;
use crate::model::{
    ApiCard, ApiExample, CardexError, ExampleLanguage, Parameter, Result, TocEntry,
};

pub fn build_card_from_html(entry: &TocEntry, html: &str) -> Result<ApiCard> {
    let document = Html::parse_document(html);
    let title = first_text(&document, "h1, title")?.unwrap_or_else(|| entry.title.clone());
    let raw_text = normalize_space(&document.root_element().text().collect::<Vec<_>>().join(" "));
    let code_blocks = all_text(&document, "pre, code")?;
    let parameters = extract_parameters(&document)?;
    let returns = extract_return_value(&raw_text).or_else(|| {
        all_text(&document, "p, div, li")
            .ok()?
            .into_iter()
            .find(|text| looks_like_return_text(text))
    });
    let remarks = extract_named_section(&raw_text, "Remarks", &["See Also", "Example", "Examples"]);
    let related = extract_related(&document)?;
    let examples = extract_examples(&document)?;
    let method_name = entry
        .overload_of
        .as_deref()
        .or(entry.symbol.as_deref())
        .unwrap_or(&entry.title);
    let signature_cs = code_blocks
        .iter()
        .find(|text| looks_like_csharp_signature(text, method_name))
        .cloned();
    let signature_vb = code_blocks
        .iter()
        .find(|text| looks_like_vb_signature(text, method_name))
        .cloned();
    let summary = remarks
        .as_deref()
        .and_then(first_sentence)
        .or_else(|| returns.as_deref().and_then(first_sentence))
        .or_else(|| first_sentence(&raw_text));

    Ok(ApiCard {
        page_id: entry.local.clone().unwrap_or_else(|| entry.title.clone()),
        title,
        kind: entry.kind.clone(),
        interface: entry.interface.clone(),
        symbol: entry.symbol.clone(),
        overload_of: entry.overload_of.clone(),
        signature_cs,
        signature_vb,
        parameters,
        returns,
        remarks,
        related,
        examples,
        summary,
        raw_text,
        content_sha256: String::new(),
    })
}

fn extract_examples(document: &Html) -> Result<Vec<ApiExample>> {
    let section_selector = selector(
        "h1, h2, h3, h4, h5, h6, .collapsibleRegionTitle, .language, .codeSnippetContainer, pre, code",
    )?;
    let mut in_examples = false;
    let mut language = ExampleLanguage::Unknown;
    let mut examples = Vec::new();

    for element in document.select(&section_selector) {
        let tag = element.value().name();
        let text = normalize_space(&element.text().collect::<Vec<_>>().join(" "));
        if text.is_empty() {
            continue;
        }

        if has_class(&element, "codeSnippetContainer") {
            if in_examples {
                for example in extract_tabbed_examples(&element)? {
                    if !examples
                        .iter()
                        .any(|existing: &ApiExample| existing.code == example.code)
                    {
                        examples.push(example);
                    }
                }
            }
            continue;
        }

        if tag == "pre" || tag == "code" {
            if in_examples
                && !examples
                    .iter()
                    .any(|example: &ApiExample| example.code == text)
            {
                examples.push(ApiExample {
                    language: infer_example_language(&text, &language),
                    code: text,
                });
            }
            continue;
        }

        if is_examples_heading(&text) {
            in_examples = true;
            language = ExampleLanguage::Unknown;
        } else if in_examples {
            if let Some(example_language) = parse_example_language(&text) {
                language = example_language;
            } else if is_section_heading(tag, element.value().attr("class")) {
                in_examples = false;
            }
        }
    }

    Ok(examples)
}

fn extract_tabbed_examples(container: &ElementRef<'_>) -> Result<Vec<ApiExample>> {
    let tab_selector = selector(".codeSnippetContainerTab a, .codeSnippetContainerTabSingle a")?;
    let code_selector = selector(".codeSnippetContainerCode")?;
    let languages = container
        .select(&tab_selector)
        .map(|tab| normalize_space(&tab.text().collect::<Vec<_>>().join(" ")))
        .map(|label| parse_example_language(&label))
        .collect::<Vec<_>>();
    let codes = container
        .select(&code_selector)
        .map(|code| normalize_space(&code.text().collect::<Vec<_>>().join(" ")))
        .filter(|code| !code.is_empty())
        .collect::<Vec<_>>();

    Ok(languages
        .into_iter()
        .zip(codes)
        .filter_map(|(language, code)| language.map(|language| ApiExample { language, code }))
        .collect())
}

fn has_class(element: &ElementRef<'_>, expected: &str) -> bool {
    element
        .value()
        .attr("class")
        .is_some_and(|value| value.split_whitespace().any(|class| class == expected))
}

fn is_examples_heading(text: &str) -> bool {
    text.eq_ignore_ascii_case("example") || text.eq_ignore_ascii_case("examples")
}

fn parse_example_language(text: &str) -> Option<ExampleLanguage> {
    let normalized = text.to_ascii_lowercase().replace([' ', '-', '.'], "");
    match normalized.as_str() {
        "c#" | "csharp" => Some(ExampleLanguage::CSharp),
        "vb" | "vbnet" | "visualbasic" | "visualbasicnet" => Some(ExampleLanguage::VisualBasic),
        _ => None,
    }
}

fn infer_example_language(code: &str, current: &ExampleLanguage) -> ExampleLanguage {
    if current != &ExampleLanguage::Unknown {
        return current.clone();
    }
    if code.trim_start().starts_with("Function ")
        || code.trim_start().starts_with("Sub ")
        || code.contains(" As ")
    {
        ExampleLanguage::VisualBasic
    } else if code.contains(';')
        || code.contains("using ")
        || code.contains("public ")
        || code.contains("private ")
    {
        ExampleLanguage::CSharp
    } else {
        ExampleLanguage::Unknown
    }
}

fn is_section_heading(tag: &str, class: Option<&str>) -> bool {
    matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
        || class.is_some_and(|value| {
            value
                .split_whitespace()
                .any(|name| name == "collapsibleRegionTitle")
        })
}

fn extract_return_value(raw_text: &str) -> Option<String> {
    let section = extract_named_section(
        raw_text,
        "Return Value",
        &["Parameters", "Remarks", "Examples", "See Also"],
    )?;
    let cleaned = if let Some(index) = find_case_insensitive(&section, "returns ") {
        section[index..].trim()
    } else {
        section.trim()
    };
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

fn extract_parameters(document: &Html) -> Result<Vec<Parameter>> {
    let row_selector = selector("tr")?;
    let cell_selector = selector("td")?;
    let mut parameters = Vec::new();

    for row in document.select(&row_selector) {
        let cells = row
            .select(&cell_selector)
            .map(|cell| normalize_space(&cell.text().collect::<Vec<_>>().join(" ")))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>();
        if cells.len() < 2 || cells[0].eq_ignore_ascii_case("parameter") {
            continue;
        }

        let (type_name, desc) = if cells.len() >= 3 {
            (Some(cells[1].clone()), Some(cells[2..].join(" ")))
        } else {
            (None, Some(cells[1].clone()))
        };

        parameters.push(Parameter {
            name: cells[0].clone(),
            type_name,
            desc,
        });
    }

    parameters.extend(extract_definition_list_parameters(document)?);

    Ok(parameters)
}

fn extract_definition_list_parameters(document: &Html) -> Result<Vec<Parameter>> {
    let dl_selector = selector("dl")?;
    let dt_selector = selector("dt")?;
    let dd_selector = selector("dd")?;
    let mut parameters = Vec::new();

    for dl in document.select(&dl_selector) {
        let names = dl
            .select(&dt_selector)
            .map(|dt| normalize_space(&dt.text().collect::<Vec<_>>().join(" ")))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>();
        let descriptions = dl.select(&dd_selector).collect::<Vec<_>>();
        if names.is_empty() || names.len() != descriptions.len() {
            continue;
        }

        for (name, description) in names.into_iter().zip(descriptions) {
            let (type_name, desc) = split_definition_description(&description.inner_html());
            parameters.push(Parameter {
                name,
                type_name,
                desc,
            });
        }
    }

    Ok(parameters)
}

fn split_definition_description(html: &str) -> (Option<String>, Option<String>) {
    let mut parts = html.splitn(2, "<br").collect::<Vec<_>>();
    let type_text = fragment_text(parts.first().copied().unwrap_or_default());
    let desc_html = parts
        .get_mut(1)
        .and_then(|part| part.split_once('>').map(|(_, rest)| rest))
        .unwrap_or_default();
    let desc = fragment_text(desc_html);
    let type_name = clean_type_text(&type_text);

    (
        (!type_name.is_empty()).then_some(type_name),
        (!desc.is_empty()).then_some(desc),
    )
}

fn fragment_text(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    normalize_space(&fragment.root_element().text().collect::<Vec<_>>().join(" "))
}

fn clean_type_text(text: &str) -> String {
    let text = text.strip_prefix("Type:").unwrap_or(text).trim();
    text.split_whitespace()
        .last()
        .unwrap_or_default()
        .to_string()
}

fn extract_related(document: &Html) -> Result<Vec<String>> {
    let scoped = related_from_selector(document, "#ID4RBSection a, .seeAlsoStyle a")?;
    if !scoped.is_empty() {
        return Ok(scoped);
    }

    related_from_selector(document, "a")
}

fn related_from_selector(document: &Html, css: &str) -> Result<Vec<String>> {
    let anchor_selector = selector(css)?;
    let mut related = Vec::new();
    for anchor in document.select(&anchor_selector) {
        let href = anchor.attr("href").unwrap_or_default();
        if href.starts_with("http") || href.starts_with("mailto:") || href == "#" {
            continue;
        }
        let text = normalize_space(&anchor.text().collect::<Vec<_>>().join(" "));
        if text.is_empty() || text.eq_ignore_ascii_case("copy") {
            continue;
        }
        related.push(normalize_related_text(&text));
    }
    related.sort();
    related.dedup();
    Ok(related)
}

fn normalize_related_text(text: &str) -> String {
    for suffix in [
        " Method",
        " Property",
        " Interface",
        " Enumeration",
        " Namespace",
    ] {
        if let Some(stripped) = text.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    text.to_string()
}

fn extract_named_section(raw_text: &str, heading: &str, stop_headings: &[&str]) -> Option<String> {
    let (_, after) = raw_text.split_once(heading)?;
    let mut section = after.trim();
    for stop in stop_headings {
        if let Some((candidate, _)) = section.split_once(stop) {
            section = candidate.trim();
        }
    }
    (!section.is_empty()).then(|| section.to_string())
}

fn first_sentence(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((sentence, _)) = text.split_once(". ") {
        return Some(format!("{}.", sentence.trim_end_matches('.')));
    }
    Some(text.to_string())
}

fn looks_like_return_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("return") && (lower.contains("zero") || lower.contains('0'))
}

fn looks_like_csharp_signature(text: &str, method_name: &str) -> bool {
    let trimmed = text.trim();
    trimmed.contains(method_name)
        && !looks_like_vb_signature(trimmed, method_name)
        && (trimmed.starts_with("int ")
            || trimmed.starts_with("void ")
            || trimmed.contains(" ref ")
            || trimmed.contains(" out ")
            || trimmed.ends_with(';'))
}

fn looks_like_vb_signature(text: &str, method_name: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.contains(method_name)
        && (trimmed.starts_with("Function ")
            || trimmed.starts_with("Sub ")
            || trimmed.contains(" As "))
}

fn first_text(document: &Html, css: &str) -> Result<Option<String>> {
    let selector = selector(css)?;
    Ok(document
        .select(&selector)
        .map(|element| normalize_space(&element.text().collect::<Vec<_>>().join(" ")))
        .find(|text| !text.is_empty()))
}

fn all_text(document: &Html, css: &str) -> Result<Vec<String>> {
    let selector = selector(css)?;
    Ok(document
        .select(&selector)
        .map(|element| normalize_space(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|text| !text.is_empty())
        .collect())
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

fn selector(css: &str) -> Result<Selector> {
    Selector::parse(css).map_err(|err| CardexError::Parse(format!("invalid selector {css}: {err}")))
}
