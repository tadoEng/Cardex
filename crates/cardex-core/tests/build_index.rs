use std::fs;
use std::path::Path;

use cardex_core::{BuildOptions, CardStore, build_corpus};

#[test]
fn build_corpus_writes_cards_docgraph_and_search_index() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let out = temp.path().join("index");
    write_fixture_corpus(&source);

    let report = build_corpus(BuildOptions {
        source_dir: source.clone(),
        out_dir: out.clone(),
        corpus: "etabs-api".to_string(),
    })
    .expect("corpus builds");

    assert_eq!(report.pages, 3);
    assert!(out.join("pages.jsonl").exists());
    assert!(out.join("docgraph.json").exists());
    assert!(out.join("manifest.json").exists());
    assert!(out.join("tantivy").exists());

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
    })
    .expect("corpus builds");

    let store = CardStore::open(&out).expect("store opens");
    let hits = store.search("frame force", 5).expect("search succeeds");

    assert_eq!(
        hits.first().and_then(|hit| hit.symbol.as_deref()),
        Some("cAnalysisResults.FrameForce")
    );
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
