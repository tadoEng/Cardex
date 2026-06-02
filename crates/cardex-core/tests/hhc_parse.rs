use cardex_core::{PageKind, parse_hhc};

#[test]
fn parse_hhc_recovers_fully_qualified_method_symbols() {
    let hhc = r#"
    <html><body>
      <ul>
        <li><object type="text/sitemap">
          <param name="Name" value="CSI API ETABS v1">
          <param name="Local" value="index.htm">
        </object>
          <ul>
            <li><object type="text/sitemap">
              <param name="Name" value="cAnalysisResults Interface">
            </object>
              <ul>
                <li><object type="text/sitemap">
                  <param name="Name" value="FrameForce Method">
                  <param name="Local" value="html/frame_force.htm">
                </object></li>
                <li><object type="text/sitemap">
                  <param name="Name" value="cAnalysisResults Methods">
                  <param name="Local" value="html/analysis_results_methods.htm">
                </object></li>
              </ul>
            </li>
            <li><object type="text/sitemap">
              <param name="Name" value="cAnalyze Interface">
            </object>
              <ul>
                <li><object type="text/sitemap">
                  <param name="Name" value="RunAnalysis Method">
                  <param name="Local" value="html/run_analysis.htm">
                </object></li>
              </ul>
            </li>
          </ul>
        </li>
      </ul>
    </body></html>
    "#;

    let toc = parse_hhc(hhc).expect("hhc parses");
    let frame_force = toc
        .entries
        .iter()
        .find(|entry| entry.local.as_deref() == Some("html/frame_force.htm"))
        .expect("FrameForce entry exists");

    assert_eq!(frame_force.title, "FrameForce Method");
    assert_eq!(frame_force.kind, PageKind::Method);
    assert_eq!(frame_force.interface.as_deref(), Some("cAnalysisResults"));
    assert_eq!(
        frame_force.symbol.as_deref(),
        Some("cAnalysisResults.FrameForce")
    );
    assert_eq!(frame_force.overload_of.as_deref(), Some("FrameForce"));
    assert_eq!(
        frame_force.ancestors,
        vec![
            "CSI API ETABS v1".to_string(),
            "cAnalysisResults Interface".to_string()
        ]
    );

    let methods_group = toc
        .entries
        .iter()
        .find(|entry| entry.local.as_deref() == Some("html/analysis_results_methods.htm"))
        .expect("methods group entry exists");

    assert_eq!(methods_group.kind, PageKind::Page);
    assert_eq!(methods_group.symbol, None);

    let run_analysis = toc
        .entries
        .iter()
        .find(|entry| entry.local.as_deref() == Some("html/run_analysis.htm"))
        .expect("RunAnalysis entry exists");

    assert_eq!(run_analysis.symbol.as_deref(), Some("cAnalyze.RunAnalysis"));
}

#[test]
fn parse_hhc_gives_overloads_unique_symbols_and_shared_family() {
    let hhc = r#"
    <html><body><ul>
      <li><object type="text/sitemap">
        <param name="Name" value="CSI API ETABS v1">
        <param name="Local" value="index.htm"></object>
        <ul>
          <li><object type="text/sitemap">
            <param name="Name" value="cAnalysisResults Interface"></object>
            <ul>
              <li><object type="text/sitemap">
                <param name="Name" value="AssembledJointMass Method">
                <param name="Local" value="html/ajm.htm"></object></li>
              <li><object type="text/sitemap">
                <param name="Name" value="AssembledJointMass_1 Method">
                <param name="Local" value="html/ajm1.htm"></object></li>
            </ul>
          </li>
        </ul>
      </li>
    </ul></body></html>
    "#;
    let toc = parse_hhc(hhc).expect("hhc parses");
    let base = toc
        .entries
        .iter()
        .find(|entry| entry.local.as_deref() == Some("html/ajm.htm"))
        .expect("base overload exists");
    let overload = toc
        .entries
        .iter()
        .find(|entry| entry.local.as_deref() == Some("html/ajm1.htm"))
        .expect("suffix overload exists");

    assert_eq!(
        base.symbol.as_deref(),
        Some("cAnalysisResults.AssembledJointMass")
    );
    assert_eq!(
        overload.symbol.as_deref(),
        Some("cAnalysisResults.AssembledJointMass_1")
    );
    assert_eq!(base.overload_of.as_deref(), Some("AssembledJointMass"));
    assert_eq!(overload.overload_of.as_deref(), Some("AssembledJointMass"));
    assert_ne!(base.symbol, overload.symbol);
}
