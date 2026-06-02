use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn cli_build_search_get_and_members_support_json_output() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let index = temp.path().join("index");
    write_fixture_corpus(&source);

    let build = run_cardex([
        "build",
        "--source",
        source.to_str().expect("utf-8 source"),
        "--out",
        index.to_str().expect("utf-8 index"),
        "--corpus",
        "etabs-api",
        "--json",
    ]);
    assert_success(&build);
    let build_json: Value = serde_json::from_slice(&build.stdout).expect("build json");
    assert_eq!(build_json["pages"], 4);

    let search = run_cardex([
        "search",
        "frame force",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--limit",
        "3",
        "--json",
    ]);
    assert_success(&search);
    let search_json: Value = serde_json::from_slice(&search.stdout).expect("search json");
    assert_eq!(search_json[0]["symbol"], "cAnalysisResults.FrameForce");

    let get = run_cardex([
        "get",
        "cAnalysisResults.FrameForce",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--json",
    ]);
    assert_success(&get);
    let get_json: Value = serde_json::from_slice(&get.stdout).expect("get json");
    assert_eq!(get_json["symbol"], "cAnalysisResults.FrameForce");
    assert_eq!(
        get_json["returns"],
        "Returns zero if successful; otherwise it returns a nonzero value."
    );

    let members = run_cardex([
        "members",
        "cAnalysisResults",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--json",
    ]);
    assert_success(&members);
    let members_json: Value = serde_json::from_slice(&members.stdout).expect("members json");
    assert_eq!(
        members_json,
        serde_json::json!([
            "cAnalysisResults.AssembledJointMass",
            "cAnalysisResults.AssembledJointMass_1",
            "cAnalysisResults.BaseReact",
            "cAnalysisResults.FrameForce"
        ])
    );

    let get_base = run_cardex([
        "get",
        "cAnalysisResults.AssembledJointMass",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--json",
    ]);
    assert_success(&get_base);
    let get_base_json: Value = serde_json::from_slice(&get_base.stdout).expect("get base json");
    assert_eq!(
        get_base_json["symbol"],
        "cAnalysisResults.AssembledJointMass"
    );

    let get_overload = run_cardex([
        "get",
        "cAnalysisResults.AssembledJointMass_1",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--json",
    ]);
    assert_success(&get_overload);
    let get_overload_json: Value =
        serde_json::from_slice(&get_overload.stdout).expect("get overload json");
    assert_eq!(
        get_overload_json["symbol"],
        "cAnalysisResults.AssembledJointMass_1"
    );
    assert_ne!(get_base_json["page_id"], get_overload_json["page_id"]);

    let related = run_cardex([
        "related",
        "cAnalysisResults.FrameForce",
        "--index",
        index.to_str().expect("utf-8 index"),
        "--json",
    ]);
    assert_success(&related);
    let related_json: Value = serde_json::from_slice(&related.stdout).expect("related json");
    assert_eq!(
        related_json,
        serde_json::json!(["cAnalysisResults.BaseReact"])
    );
}

#[test]
fn cli_build_then_query_with_default_index_paths() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    write_fixture_corpus(&source);

    let build = Command::new(env!("CARGO_BIN_EXE_cardex"))
        .current_dir(temp.path())
        .args([
            "build",
            "--source",
            source.to_str().expect("utf-8 source"),
            "--json",
        ])
        .output()
        .expect("build command runs");
    assert_success(&build);

    let search = Command::new(env!("CARGO_BIN_EXE_cardex"))
        .current_dir(temp.path())
        .args(["search", "frame force", "--json"])
        .output()
        .expect("search command runs");
    assert_success(&search);
    let search_json: Value = serde_json::from_slice(&search.stdout).expect("search json");
    assert_eq!(search_json[0]["symbol"], "cAnalysisResults.FrameForce");
}

fn run_cardex<const N: usize>(args: [&str; N]) -> std::process::Output {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_cardex"));
    Command::new(bin)
        .args(args)
        .output()
        .expect("cardex command runs")
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
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
                    <li><object type="text/sitemap"><param name="Name" value="AssembledJointMass Method"><param name="Local" value="html/assembled_joint_mass.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="AssembledJointMass_1 Method"><param name="Local" value="html/assembled_joint_mass_1.htm"></object></li>
                    <li><object type="text/sitemap"><param name="Name" value="FrameForce Method"><param name="Local" value="html/frame_force.htm"></object></li>
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
        api_page_with_related(
            "FrameForce",
            "int FrameForce(string Name, eItemTypeElm ItemTypeElm, ref int NumberResults)",
            "Frame force results for line elements.",
            "cAnalysisResults.BaseReact Method",
        ),
    )
    .expect("write frame force page");
    fs::write(
        source.join("html/assembled_joint_mass.htm"),
        api_page(
            "AssembledJointMass",
            "int AssembledJointMass(ref int NumberResults)",
            "Assembled joint mass results.",
        ),
    )
    .expect("write assembled joint mass page");
    fs::write(
        source.join("html/assembled_joint_mass_1.htm"),
        api_page(
            "AssembledJointMass",
            "int AssembledJointMass(string Name, ref int NumberResults)",
            "Assembled joint mass results for a named object.",
        ),
    )
    .expect("write assembled joint mass overload page");
    fs::write(
        source.join("html/base_react.htm"),
        api_page(
            "BaseReact",
            "int BaseReact(ref int NumberResults)",
            "Base reaction results.",
        ),
    )
    .expect("write base react page");
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

fn api_page_with_related(name: &str, signature: &str, remarks: &str, related: &str) -> String {
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
          <p><a href="base_react.htm">{related}</a></p>
        </body></html>
        "#
    )
}
