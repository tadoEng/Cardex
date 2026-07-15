use cardex_core::{ExampleLanguage, PageKind, TocEntry, build_card_from_html};

#[test]
fn build_card_from_html_extracts_compact_api_fields() {
    let entry = TocEntry {
        title: "FrameForce Method".to_string(),
        local: Some("html/frame_force.htm".to_string()),
        depth: 2,
        ancestors: vec![
            "CSI API ETABS v1".to_string(),
            "cAnalysisResults Interface".to_string(),
        ],
        kind: PageKind::Method,
        interface: Some("cAnalysisResults".to_string()),
        symbol: Some("cAnalysisResults.FrameForce".to_string()),
        overload_of: Some("FrameForce".to_string()),
    };
    let html = r#"
    <html>
      <head><title>FrameForce Method</title></head>
      <body>
        <h1>FrameForce Method</h1>
        <pre>int FrameForce(string Name, eItemTypeElm ItemTypeElm, ref int NumberResults)</pre>
        <pre>Function FrameForce(Name As String, ItemTypeElm As eItemTypeElm, ByRef NumberResults As Integer) As Integer</pre>
        <table>
          <tr><th>Parameter</th><th>Type</th><th>Description</th></tr>
          <tr><td>Name</td><td>string</td><td>Object name or group name.</td></tr>
          <tr><td>ItemTypeElm</td><td>eItemTypeElm</td><td>Selection type.</td></tr>
        </table>
        <p>Returns zero if successful; otherwise it returns a nonzero value.</p>
        <h2>Remarks</h2>
        <p>Use cAnalysisResultsSetup before reading results.</p>
        <h2>See Also</h2>
        <p><a href="setup.htm">cAnalysisResultsSetup.SetCaseSelectedForOutput</a></p>
      </body>
    </html>
    "#;

    let card = build_card_from_html(&entry, html).expect("card builds");

    assert_eq!(card.page_id, "html/frame_force.htm");
    assert_eq!(card.title, "FrameForce Method");
    assert_eq!(card.kind, PageKind::Method);
    assert_eq!(card.interface.as_deref(), Some("cAnalysisResults"));
    assert_eq!(card.symbol.as_deref(), Some("cAnalysisResults.FrameForce"));
    assert_eq!(
        card.signature_cs.as_deref(),
        Some("int FrameForce(string Name, eItemTypeElm ItemTypeElm, ref int NumberResults)")
    );
    assert_eq!(
        card.signature_vb.as_deref(),
        Some(
            "Function FrameForce(Name As String, ItemTypeElm As eItemTypeElm, ByRef NumberResults As Integer) As Integer"
        )
    );
    assert_eq!(card.parameters.len(), 2);
    assert_eq!(card.parameters[0].name, "Name");
    assert_eq!(card.parameters[0].type_name.as_deref(), Some("string"));
    assert_eq!(
        card.returns.as_deref(),
        Some("Returns zero if successful; otherwise it returns a nonzero value.")
    );
    assert_eq!(
        card.remarks.as_deref(),
        Some("Use cAnalysisResultsSetup before reading results.")
    );
    assert_eq!(
        card.related,
        vec!["cAnalysisResultsSetup.SetCaseSelectedForOutput".to_string()]
    );
}

#[test]
fn build_card_from_sandcastle_html_keeps_return_and_related_compact() {
    let entry = TocEntry {
        title: "FrameForce Method".to_string(),
        local: Some("html/frame_force.htm".to_string()),
        depth: 2,
        ancestors: vec!["cAnalysisResults Interface".to_string()],
        kind: PageKind::Method,
        interface: Some("cAnalysisResults".to_string()),
        symbol: Some("cAnalysisResults.FrameForce".to_string()),
        overload_of: Some("FrameForce".to_string()),
    };
    let html = r##"
    <html><body>
      <h1>cAnalysisResults.FrameForce Method</h1>
      <div class="summary">Reports frame forces.</div>
      <h4 class="subHeading">Return Value</h4>
      Type: <a href="https://example.invalid/int32">Int32</a><br />
      Returns zero when successful; otherwise nonzero.
      <h4 class="subHeading">Parameters</h4>
      <dl>
        <dt><span class="parameter">Name</span></dt>
        <dd>Type: <a href="https://example.invalid/string">String</a><br />Object or group name.</dd>
      </dl>
      <div class="collapsibleAreaRegion">
        <span class="collapsibleRegionTitle">Remarks</span>
      </div>
      <div id="ID3RBSection" class="collapsibleSection">
        Select output cases before reading results.
        <div class="collapsibleAreaRegion">
          <span class="collapsibleRegionTitle">Examples</span>
        </div>
        <div id="ID2RBSection" class="collapsibleSection">
          <pre>Example code should not become remarks.</pre>
        </div>
      </div>
      <div class="collapsibleAreaRegion" id="seeAlsoSection">
        <span class="collapsibleRegionTitle">See Also</span>
      </div>
      <div id="ID4RBSection" class="collapsibleSection">
        <div class="seeAlsoStyle"><a href="results.htm">cAnalysisResults Interface</a></div>
        <div class="seeAlsoStyle"><a href="namespace.htm">ETABSv1 Namespace</a></div>
      </div>
      <div id="pageFooter"><a href="mailto:support@example.invalid">support@example.invalid</a></div>
    </body></html>
    "##;

    let card = build_card_from_html(&entry, html).expect("card builds");

    assert_eq!(
        card.returns.as_deref(),
        Some("Returns zero when successful; otherwise nonzero.")
    );
    assert_eq!(
        card.remarks.as_deref(),
        Some("Select output cases before reading results.")
    );
    assert_eq!(
        card.related,
        vec!["ETABSv1".to_string(), "cAnalysisResults".to_string()]
    );
    assert_eq!(card.parameters.len(), 1);
    assert_eq!(card.parameters[0].name, "Name");
    assert_eq!(card.parameters[0].type_name.as_deref(), Some("String"));
    assert_eq!(
        card.parameters[0].desc.as_deref(),
        Some("Object or group name.")
    );
}

#[test]
fn build_card_extracts_language_tagged_examples_without_changing_raw_text() {
    let entry = TocEntry {
        title: "SetWall Method".to_string(),
        local: Some("html/set_wall.htm".to_string()),
        depth: 2,
        ancestors: vec!["cPropArea Interface".to_string()],
        kind: PageKind::Method,
        interface: Some("cPropArea".to_string()),
        symbol: Some("cPropArea.SetWall".to_string()),
        overload_of: Some("SetWall".to_string()),
    };
    let html = r#"
    <html><body>
      <h1>SetWall Method</h1>
      <pre>int SetWall(string Name)</pre>
      <h2>Examples</h2>
      <h3>C#</h3>
      <pre>var result = sapModel.PropArea.SetWall("W1");</pre>
      <h3>Visual Basic</h3>
      <pre>Dim result As Integer = SapModel.PropArea.SetWall("W1")</pre>
      <h2>See Also</h2>
      <pre>This block is outside the examples section.</pre>
    </body></html>
    "#;

    let card = build_card_from_html(&entry, html).expect("card builds");

    assert_eq!(card.examples.len(), 2);
    assert_eq!(card.examples[0].language, ExampleLanguage::CSharp);
    assert_eq!(
        card.examples[0].code,
        "var result = sapModel.PropArea.SetWall(\"W1\");"
    );
    assert_eq!(card.examples[1].language, ExampleLanguage::VisualBasic);
    assert_eq!(
        card.examples[1].code,
        "Dim result As Integer = SapModel.PropArea.SetWall(\"W1\")"
    );
    assert!(card.raw_text.contains("Examples C#"));
    assert!(card.raw_text.contains("Dim result As Integer"));
    assert!(
        !card
            .examples
            .iter()
            .any(|example| example.code.contains("outside"))
    );
}

#[test]
fn build_card_extracts_csharp_and_vb_from_sandcastle_tabbed_examples() {
    let entry = TocEntry {
        title: "SetWall Method".to_string(),
        local: Some("html/set_wall.htm".to_string()),
        depth: 2,
        ancestors: vec!["cPropArea Interface".to_string()],
        kind: PageKind::Method,
        interface: Some("cPropArea".to_string()),
        symbol: Some("cPropArea.SetWall".to_string()),
        overload_of: Some("SetWall".to_string()),
    };
    let html = r##"
    <html><body>
      <h1>SetWall Method</h1>
      <div class="collapsibleAreaRegion">
        <span class="collapsibleRegionTitle">Examples</span>
      </div>
      <div class="collapsibleSection">
        <div class="codeSnippetContainer">
          <div class="codeSnippetContainerTabs">
            <div class="codeSnippetContainerTab"><a href="#">C#</a></div>
            <div class="codeSnippetContainerTab"><a href="#">Visual Basic</a></div>
            <div class="codeSnippetContainerTab"><a href="#">C++</a></div>
          </div>
          <div class="codeSnippetContainerCodeContainer">
            <div class="codeSnippetContainerCode">var result = SetWall("W1");</div>
            <div class="codeSnippetContainerCode">Dim result As Integer = SetWall("W1")</div>
            <div class="codeSnippetContainerCode">auto result = SetWall("W1");</div>
          </div>
        </div>
      </div>
      <div class="collapsibleAreaRegion">
        <span class="collapsibleRegionTitle">See Also</span>
      </div>
    </body></html>
    "##;

    let card = build_card_from_html(&entry, html).expect("card builds");

    assert_eq!(card.examples.len(), 2);
    assert_eq!(card.examples[0].language, ExampleLanguage::CSharp);
    assert_eq!(card.examples[0].code, "var result = SetWall(\"W1\");");
    assert_eq!(card.examples[1].language, ExampleLanguage::VisualBasic);
    assert_eq!(
        card.examples[1].code,
        "Dim result As Integer = SetWall(\"W1\")"
    );
}
