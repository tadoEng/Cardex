use std::fs;
use std::path::Path;

use cardex_core::{BuildOptions, CardStore, Manifest, build_corpus};

#[test]
fn build_corpus_writes_cards_docgraph_and_search_index() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    write_fixture_corpus(&source);

    let report = build_corpus(BuildOptions {
        source_dir: source.clone(),
        out_dir: out.clone(),
        corpus: "etabs-api-23.3".to_string(),
        product_name: "ETABS".to_string(),
        source_docs_version: "23.3".to_string(),
        source_docs_build: "synthetic".to_string(),
    })
    .expect("corpus builds");

    assert_eq!(report.pages, 3);
    assert!(out.join("pages.jsonl").exists());
    assert!(out.join("docgraph.json").exists());
    assert!(out.join("manifest.json").exists());
    assert!(out.join("tantivy").exists());
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(out.join("manifest.json")).expect("manifest reads"),
    )
    .expect("manifest json");
    assert_eq!(manifest["schema_version"], 4);
    assert_eq!(manifest["corpus"], "etabs-api-23.3");
    assert_eq!(manifest["product_name"], "ETABS");
    assert_eq!(manifest["source_docs_version"], "23.3");
    assert_eq!(manifest["source_docs_build"], "synthetic");
    assert_eq!(manifest["source_dir_sha256"], report.source_dir_sha256);
    assert_eq!(manifest["corpus_sha256"], report.corpus_sha256);

    let store = CardStore::open(&out).expect("store opens");
    let hits = store.search("frame force", 5).expect("search succeeds");
    assert_eq!(
        hits.first().and_then(|hit| hit.symbol.as_deref()),
        Some("cAnalysisResults.FrameForce")
    );

    let card = store
        .get("cAnalysisResults.FrameForce")
        .expect("get succeeds")
        .expect("card exists");
    assert_eq!(
        card.returns.as_deref(),
        Some("Returns zero if successful; otherwise it returns a nonzero value.")
    );

    let members = store.members("cAnalysisResults").expect("members succeed");
    assert_eq!(
        members,
        vec![
            "cAnalysisResults.BaseReact".to_string(),
            "cAnalysisResults.FrameForce".to_string()
        ]
    );

    let related = store
        .related("cAnalysisResults.FrameForce")
        .expect("related succeeds");
    assert_eq!(
        related,
        vec!["cAnalysisResultsSetup.SetCaseSelectedForOutput".to_string()]
    );
}

#[test]
fn search_prefers_camel_case_symbol_match_over_body_noise() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    fs::create_dir_all(source.join("html")).expect("create fixture dirs");
    fs::write(
        source.join("CSI API ETABS v1.hhc"),
        r#"
        <html><body>
          <ul>
            <li><object type="text/sitemap"><param name="Name" value="CSI API ETABS v1"></object>
              <ul>
                <li><object type="text/sitemap"><param name="Name" value="cAnalysisResults Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="FrameForce Method"><param name="Local" value="html/frame_force.htm"></object></li>
                  </ul>
                </li>
                <li><object type="text/sitemap"><param name="Name" value="cFrameObj Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="SetTCLimits Method"><param name="Local" value="html/set_tc_limits.htm"></object></li>
                  </ul>
                </li>
              </ul>
            </li>
          </ul>
        </body></html>
        "#,
    )
    .expect("write hhc");
    fs::write(
        source.join("html/frame_force.htm"),
        r#"<html><body><h1>FrameForce Method</h1><pre>int FrameForce()</pre></body></html>"#,
    )
    .expect("write frame force");
    fs::write(
        source.join("html/set_tc_limits.htm"),
        r#"
        <html><body>
          <h1>SetTCLimits Method</h1>
          <p>Sets frame force limit values for nonlinear analysis objects.</p>
        </body></html>
        "#,
    )
    .expect("write noise page");

    build_corpus(BuildOptions {
        source_dir: source,
        out_dir: out.clone(),
        corpus: "etabs-api".to_string(),
        product_name: "ETABS".to_string(),
        source_docs_version: "test".to_string(),
        source_docs_build: "synthetic".to_string(),
    })
    .expect("corpus builds");

    let store = CardStore::open(&out).expect("store opens");
    let hits = store.search("frame force", 5).expect("search succeeds");

    assert_eq!(
        hits.first().and_then(|hit| hit.symbol.as_deref()),
        Some("cAnalysisResults.FrameForce")
    );
}

#[test]
fn search_promotes_docgraph_members_with_explicit_aci_version_scope() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    write_aci_fixture_corpus(&source);

    build_corpus(BuildOptions {
        source_dir: source,
        out_dir: out.clone(),
        corpus: "etabs-api".to_string(),
        product_name: "ETABS".to_string(),
        source_docs_version: "test".to_string(),
        source_docs_build: "synthetic".to_string(),
    })
    .expect("corpus builds");

    let graph_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("docgraph.json")).expect("graph reads"))
            .expect("graph json");
    assert_eq!(
        graph_json["returns_interface"]["cDesignConcrete.ACI318_14"],
        "cDCoACI318_14"
    );

    let store = CardStore::open(&out).expect("store opens");
    let hits = store
        .search("ACI 318-14 concrete frame design requirement", 8)
        .expect("search succeeds");
    let symbols = hit_symbols(&hits);

    assert!(
        symbols
            .iter()
            .any(|symbol| symbol == "cDCoACI318_14.GetPreference"),
        "expected GetPreference in {symbols:?}"
    );
    assert!(
        symbols
            .iter()
            .any(|symbol| symbol == "cDCoACI318_14.SetPreference"),
        "expected SetPreference in {symbols:?}"
    );
    assert!(
        !symbols.iter().any(|symbol| symbol.contains("ACI318_19")),
        "explicit ACI318_14 query leaked another ACI version: {symbols:?}"
    );

    let explained = store
        .search_explained("ACI 318-14 concrete frame design requirement", 8)
        .expect("explained search succeeds");
    assert_eq!(explained.version_scope.as_deref(), Some("ACI318_14"));
    assert!(
        explained
            .promotions
            .iter()
            .any(|promotion| promotion.symbol == "cDCoACI318_14.GetPreference"),
        "expected graph promotion in {:?}",
        explained.promotions
    );
}

#[test]
fn bare_aci318_query_keeps_all_available_versions_in_scope() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    write_aci_fixture_corpus(&source);

    build_corpus(BuildOptions {
        source_dir: source,
        out_dir: out.clone(),
        corpus: "etabs-api".to_string(),
        product_name: "ETABS".to_string(),
        source_docs_version: "test".to_string(),
        source_docs_build: "synthetic".to_string(),
    })
    .expect("corpus builds");

    let store = CardStore::open(&out).expect("store opens");
    let explained = store
        .search_explained("ACI 318 concrete frame design", 12)
        .expect("explained search succeeds");
    let symbols = hit_symbols(&explained.hits);

    assert_eq!(
        explained.version_scope.as_deref(),
        Some("all_aci318_versions")
    );
    assert!(
        symbols.iter().any(|symbol| symbol.contains("ACI318_14")),
        "expected ACI318_14 in {symbols:?}"
    );
    assert!(
        symbols.iter().any(|symbol| symbol.contains("ACI318_19")),
        "expected ACI318_19 in {symbols:?}"
    );
}

#[test]
fn section_definition_question_finds_frame_property_type_enum() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    write_section_fixture_corpus(&source);

    build_corpus(BuildOptions {
        source_dir: source,
        out_dir: out.clone(),
        corpus: "etabs-api".to_string(),
        product_name: "ETABS".to_string(),
        source_docs_version: "test".to_string(),
        source_docs_build: "synthetic".to_string(),
    })
    .expect("corpus builds");

    let store = CardStore::open(&out).expect("store opens");
    let hits = store
        .search("how many section does etabs api support to define", 5)
        .expect("search succeeds");

    assert_eq!(
        hits.first().and_then(|hit| hit.symbol.as_deref()),
        Some("eFramePropType")
    );
}

#[test]
fn repeated_builds_produce_identical_manifest_card_and_corpus_digests() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let first_out = temp.path().join("first-index");
    let second_out = temp.path().join("second-index");
    write_fixture_corpus(&source);

    let build = |out| {
        build_corpus(BuildOptions {
            source_dir: source.clone(),
            out_dir: out,
            corpus: "etabs-api-23.3".to_string(),
            product_name: "ETABS".to_string(),
            source_docs_version: "23.3".to_string(),
            source_docs_build: "23.3.0.1234".to_string(),
        })
        .expect("corpus builds")
    };

    let first_report = build(first_out.clone());
    let second_report = build(second_out.clone());
    assert_eq!(
        first_report.source_dir_sha256,
        second_report.source_dir_sha256
    );
    assert_eq!(first_report.corpus_sha256, second_report.corpus_sha256);

    let first_manifest: Manifest = serde_json::from_reader(
        fs::File::open(first_out.join("manifest.json")).expect("first manifest opens"),
    )
    .expect("first manifest reads");
    let second_manifest: Manifest = serde_json::from_reader(
        fs::File::open(second_out.join("manifest.json")).expect("second manifest opens"),
    )
    .expect("second manifest reads");
    assert_eq!(first_manifest, second_manifest);
    assert_eq!(first_manifest.schema_version, 4);
    assert_eq!(first_manifest.corpus, "etabs-api-23.3");
    assert_eq!(first_manifest.product_name, "ETABS");
    assert_eq!(first_manifest.source_docs_version, "23.3");
    assert_eq!(first_manifest.source_docs_build, "23.3.0.1234");
    assert_eq!(first_manifest.source_dir_sha256.len(), 64);
    assert_eq!(first_manifest.corpus_sha256.len(), 64);
    assert_eq!(
        fs::read(first_out.join("pages.jsonl")).expect("first pages read"),
        fs::read(second_out.join("pages.jsonl")).expect("second pages read")
    );

    let store = CardStore::open(&first_out).expect("store opens");
    let evidence = store
        .card_evidence("cAnalysisResults.FrameForce")
        .expect("evidence reads")
        .expect("evidence exists");
    assert_eq!(evidence.card_sha256.len(), 64);
    assert_eq!(evidence.card_sha256, evidence.card.content_sha256);
    assert_eq!(evidence.corpus_sha256, first_manifest.corpus_sha256);
    assert_eq!(evidence.manifest, first_manifest);

    let raw = store
        .bounded_raw_text("cAnalysisResults.FrameForce", 24)
        .expect("raw text reads")
        .expect("raw text exists");
    assert_eq!(raw.page_id, evidence.card.page_id);
    assert_eq!(raw.card_sha256, evidence.card_sha256);
    assert_eq!(raw.text.chars().count(), 24);
    assert!(raw.truncated);
}

fn write_fixture_corpus(source: &Path) {
    fs::create_dir_all(source.join("html")).expect("create fixture dirs");
    fs::write(
        source.join("CSI API ETABS v1.hhc"),
        r#"
        <html><body>
          <ul>
            <li><object type="text/sitemap"><param name="Name" value="CSI API ETABS v1"></object>
              <ul>
                <li><object type="text/sitemap"><param name="Name" value="cAnalysisResults Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="BaseReact Method"><param name="Local" value="html/base_react.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="FrameForce Method"><param name="Local" value="html/frame_force.htm"></object></li>
                  </ul>
                </li>
                <li><object type="text/sitemap"><param name="Name" value="cAnalysisResultsSetup Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="SetCaseSelectedForOutput Method"><param name="Local" value="html/set_case.htm"></object></li>
                  </ul>
                </li>
              </ul>
            </li>
          </ul>
        </body></html>
        "#,
    )
    .expect("write hhc");

    fs::write(
        source.join("html/frame_force.htm"),
        api_page(
            "FrameForce",
            "int FrameForce(string Name, eItemTypeElm ItemTypeElm, ref int NumberResults)",
            "Frame force results for line elements.",
        ),
    )
    .expect("write frame force page");
    fs::write(
        source.join("html/base_react.htm"),
        api_page(
            "BaseReact",
            "int BaseReact(ref int NumberResults)",
            "Base reaction results.",
        ),
    )
    .expect("write base react page");
    fs::write(
        source.join("html/set_case.htm"),
        api_page(
            "SetCaseSelectedForOutput",
            "int SetCaseSelectedForOutput(string Name, bool Selected)",
            "Selects a case for output before reading results.",
        ),
    )
    .expect("write setup page");
}

fn write_section_fixture_corpus(source: &Path) {
    fs::create_dir_all(source.join("html")).expect("create fixture dirs");
    fs::write(
        source.join("CSI API ETABS v1.hhc"),
        r#"
        <html><body>
          <ul>
            <li><object type="text/sitemap"><param name="Name" value="CSI API ETABS v1"></object>
              <ul>
                <li><object type="text/sitemap"><param name="Name" value="eFramePropType Enumeration"><param name="Local" value="html/e_frame_prop_type.htm"></object></li>
                <li><object type="text/sitemap"><param name="Name" value="cPropFrame Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="SetRectangle Method"><param name="Local" value="html/set_rectangle.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="SetCircle Method"><param name="Local" value="html/set_circle.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="Count Method"><param name="Local" value="html/count.htm"></object></li>
                  </ul>
                </li>
              </ul>
            </li>
          </ul>
        </body></html>
        "#,
    )
    .expect("write hhc");

    fs::write(
        source.join("html/e_frame_prop_type.htm"),
        r#"
        <html><body>
          <h1>eFramePropType Enumeration</h1>
          <table>
            <tr><th>Member name</th><th>Value</th></tr>
            <tr><td>I</td><td>1</td></tr>
            <tr><td>Channel</td><td>2</td></tr>
            <tr><td>Rectangular</td><td>8</td></tr>
          </table>
          <p>The possible frame section property types.</p>
        </body></html>
        "#,
    )
    .expect("write enum page");
    fs::write(
        source.join("html/set_rectangle.htm"),
        api_page(
            "SetRectangle",
            "int SetRectangle(string Name, string MatProp, double T3, double T2)",
            "Initializes a rectangular frame section property.",
        ),
    )
    .expect("write set rectangle");
    fs::write(
        source.join("html/set_circle.htm"),
        api_page(
            "SetCircle",
            "int SetCircle(string Name, string MatProp, double T3)",
            "Initializes a circular frame section property.",
        ),
    )
    .expect("write set circle");
    fs::write(
        source.join("html/count.htm"),
        api_page(
            "Count",
            "int Count()",
            "Returns the total number of defined frame section properties in the model.",
        ),
    )
    .expect("write count");
}

fn write_aci_fixture_corpus(source: &Path) {
    fs::create_dir_all(source.join("html")).expect("create fixture dirs");
    fs::write(
        source.join("CSI API ETABS v1.hhc"),
        r#"
        <html><body>
          <ul>
            <li><object type="text/sitemap"><param name="Name" value="CSI API ETABS v1"></object>
              <ul>
                <li><object type="text/sitemap"><param name="Name" value="cDesignConcrete Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="ACI318_14 Property"><param name="Local" value="html/aci318_14_property.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="ACI318_19 Property"><param name="Local" value="html/aci318_19_property.htm"></object></li>
                  </ul>
                </li>
                <li><object type="text/sitemap"><param name="Name" value="cDCoACI318_14 Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="GetPreference Method"><param name="Local" value="html/get_preference_14.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="SetPreference Method"><param name="Local" value="html/set_preference_14.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="GetOverwrite Method"><param name="Local" value="html/get_overwrite_14.htm"></object></li>
                  </ul>
                </li>
                <li><object type="text/sitemap"><param name="Name" value="cDCoACI318_19 Interface"></object>
                  <ul>
                    <li><object type="text/sitemap"><param name="Name" value="GetPreference Method"><param name="Local" value="html/get_preference_19.htm"></object></li>
                  </ul>
                </li>
              </ul>
            </li>
          </ul>
        </body></html>
        "#,
    )
    .expect("write hhc");

    fs::write(
        source.join("html/aci318_14_property.htm"),
        property_page(
            "ACI318_14",
            "cDCoACI318_14 ACI318_14 { get ; }",
            "ReadOnly Property ACI318_14 As cDCoACI318_14",
        ),
    )
    .expect("write ACI318_14 property");
    fs::write(
        source.join("html/aci318_19_property.htm"),
        property_page(
            "ACI318_19",
            "cDCoACI318_19 ACI318_19 { get ; }",
            "ReadOnly Property ACI318_19 As cDCoACI318_19",
        ),
    )
    .expect("write ACI318_19 property");
    fs::write(
        source.join("html/get_preference_14.htm"),
        api_page(
            "GetPreference",
            "int GetPreference(int Item, ref double Value)",
            "Retrieves the value of a concrete design preference item.",
        ),
    )
    .expect("write GetPreference 14");
    fs::write(
        source.join("html/set_preference_14.htm"),
        api_page(
            "SetPreference",
            "int SetPreference(int Item, double Value)",
            "Sets the value of a concrete design preference item.",
        ),
    )
    .expect("write SetPreference 14");
    fs::write(
        source.join("html/get_overwrite_14.htm"),
        api_page(
            "GetOverwrite",
            "int GetOverwrite(string Name, int Item, ref double Value, ref bool ProgDet)",
            "Retrieves the value of a concrete frame design overwrite item.",
        ),
    )
    .expect("write GetOverwrite 14");
    fs::write(
        source.join("html/get_preference_19.htm"),
        api_page(
            "GetPreference",
            "int GetPreference(int Item, ref double Value)",
            "Retrieves the value of an ACI 318-19 concrete design preference item.",
        ),
    )
    .expect("write GetPreference 19");
}

fn property_page(name: &str, signature_cs: &str, signature_vb: &str) -> String {
    format!(
        r#"
        <html><body>
          <h1>{name} Property</h1>
          <pre>{signature_cs}</pre>
          <pre>{signature_vb}</pre>
          <p>Concrete frame design code property.</p>
          <h2>See Also</h2>
          <p><a href="design_concrete.htm">cDesignConcrete Interface</a></p>
        </body></html>
        "#
    )
}

fn hit_symbols(hits: &[cardex_core::SearchHit]) -> Vec<String> {
    hits.iter()
        .filter_map(|hit| hit.symbol.clone())
        .collect::<Vec<_>>()
}

fn api_page(name: &str, signature: &str, remarks: &str) -> String {
    format!(
        r#"
        <html><body>
          <h1>{name} Method</h1>
          <pre>{signature}</pre>
          <table>
            <tr><th>Parameter</th><th>Type</th><th>Description</th></tr>
            <tr><td>Name</td><td>string</td><td>Object or case name.</td></tr>
          </table>
          <p>Returns zero if successful; otherwise it returns a nonzero value.</p>
          <h2>Remarks</h2>
          <p>{remarks}</p>
          <h2>See Also</h2>
          <p><a href="set_case.htm">cAnalysisResultsSetup.SetCaseSelectedForOutput Method</a></p>
        </body></html>
        "#
    )
}
