use {
    similar::TextDiff,
    std::{fs, path::PathBuf},
    symo::Model,
};

#[tokio::test]
async fn self_doc() {
    let repodir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template = repodir.join("docs-template");
    let model = repodir.join("model");
    let tempdir = tempfile::tempdir().unwrap();
    let output = tempdir.path().to_owned();
    symo::run_one_time(&model, &template, &output).unwrap();
    let readme_diff = TextDiff::from_lines(
        &String::from_utf8(fs::read(&repodir.join("README.md")).unwrap()).unwrap(),
        &String::from_utf8(fs::read(&output.join("README.md")).unwrap()).unwrap(),
    )
    .unified_diff()
    .to_string();
    assert_eq!(readme_diff, "");
}

#[tokio::test]
async fn sample_generate() {
    let testdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let template = testdir.join("data-template");
    let model = testdir.join("data-model");
    let output = testdir.join("data-output");
    let tempdir = tempfile::tempdir().unwrap();
    let tempoutput = tempdir.path().to_owned();
    symo::run_one_time(&model, &template, &tempoutput).unwrap();
    let sample1_diff = TextDiff::from_lines(
        &String::from_utf8(fs::read(&tempoutput.join("sample1.md")).unwrap()).unwrap(),
        &String::from_utf8(fs::read(&output.join("sample1.md")).unwrap()).unwrap(),
    )
    .unified_diff()
    .to_string();
    assert_eq!(sample1_diff, "");

    let sample2_diff = TextDiff::from_lines(
        &String::from_utf8(fs::read(&tempoutput.join("sample2.md")).unwrap()).unwrap(),
        &String::from_utf8(fs::read(&output.join("sample2.md")).unwrap()).unwrap(),
    )
    .unified_diff()
    .to_string();
    assert_eq!(sample2_diff, "");
}

#[tokio::test]
async fn sample_model() {
    let model = Model::new(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data-model"),
    )
    .unwrap();
    assert!(model.nodes.contains_key("template0"));
    assert!(model.nodes.contains_key("template1"));
    assert!(model.nodes.contains_key("node0"));
    assert!(model.nodes.contains_key("node1"));
    assert!(model.nodes.contains_key("node2"));
    assert!(model.nodes.contains_key("node0-0"));
    assert!(model.nodes.contains_key("node1-0"));
    assert!(model.relations.contains_key("base-tag0"));
    assert!(model.relations.contains_key("base-tag1"));
    assert!(model.diagrams.contains_key("diagram0"));
    assert!(model.diagrams.contains_key("diagram1"));
    assert!(model.diagrams.contains_key("diagram2"));
}
