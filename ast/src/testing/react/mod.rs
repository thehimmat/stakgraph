use crate::lang::graphs::{EdgeType, NodeType};
use crate::lang::Graph;
use crate::utils::get_use_lsp;
use crate::{lang::Lang, repo::Repo};
use std::str::FromStr;

pub async fn test_react_typescript_generic<G: Graph>() -> Result<(), anyhow::Error> {
    let use_lsp = get_use_lsp();
    let repo = Repo::new(
        "src/testing/react",
        Lang::from_str("tsx").unwrap(),
        use_lsp,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let graph = repo.build_graph_inner::<G>().await?;

    let (num_nodes, num_edges) = graph.get_graph_size();
    if use_lsp == true {
        assert_eq!(num_nodes, 56, "Expected 56 nodes!");
        assert_eq!(num_edges, 77, "Expected 77 edges");
    } else {
        assert_eq!(num_nodes, 50, "Expected 50 nodes");
        assert_eq!(num_edges, 61, "Expected 61 edges");
    }

    fn normalize_path(path: &str) -> String {
        path.replace("\\", "/")
    }

    let language_nodes = graph.find_nodes_by_type(NodeType::Language);
    assert_eq!(language_nodes.len(), 1, "Expected 1 language node");
    assert_eq!(
        language_nodes[0].name, "react",
        "Language node name should be 'react'"
    );
    assert_eq!(
        normalize_path(&language_nodes[0].file),
        "src/testing/react/",
        "Language node file path is incorrect"
    );

    let pkg_files = graph.find_nodes_by_name(NodeType::File, "package.json");
    assert_eq!(pkg_files.len(), 1, "Expected 1 package.json file");
    assert_eq!(
        pkg_files[0].name, "package.json",
        "Package file name is incorrect"
    );

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 4, "Expected 4 imports");

    let functions = graph.find_nodes_by_type(NodeType::Function);
    if use_lsp == true {
        assert_eq!(functions.len(), 17, "Expected 17 functions/components");
    } else {
        assert_eq!(functions.len(), 11, "Expected 11 functions/components");
    }

    let mut sorted_functions = functions.clone();
    sorted_functions.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(
        sorted_functions[0].name, "App",
        "App component name is incorrect"
    );
    assert_eq!(
        normalize_path(&sorted_functions[0].file),
        "src/testing/react/src/App.tsx",
        "App component file path is incorrect"
    );

    assert_eq!(
        sorted_functions[1].name, "FormContainer",
        "FormContainer component name is incorrect"
    );
    assert_eq!(
        normalize_path(&sorted_functions[1].file),
        "src/testing/react/src/components/NewPerson.tsx",
        "FormContainer component file path is incorrect"
    );

    let submit_button = functions
        .iter()
        .find(|f| f.name == "SubmitButton")
        .expect("SubmitButton component not found");
    assert_eq!(
        submit_button.name, "SubmitButton",
        "SubmitButton component name is incorrect"
    );
    assert_eq!(
        normalize_path(&submit_button.file),
        "src/testing/react/src/components/NewPerson.tsx",
        "SubmitButton component file path is incorrect"
    );

    let requests = graph.find_nodes_by_type(NodeType::Request);
    assert_eq!(requests.len(), 3, "Expected 3 requests");

    let calls_edges_count = graph.count_edges_of_type(EdgeType::Calls(Default::default()));
    assert_eq!(calls_edges_count, 14, "Expected 14 calls edges");

    let pages = graph.find_nodes_by_type(NodeType::Page);
    assert_eq!(pages.len(), 2, "Expected 2 pages");

    let renders_edges_count = graph.count_edges_of_type(EdgeType::Renders);
    assert_eq!(renders_edges_count, 2, "Expected 2 renders edges");

    let people_page = pages
        .iter()
        .find(|p| p.name == "/people")
        .expect("Expected '/people' page not found");
    assert_eq!(people_page.name, "/people", "Page name should be '/people'");
    assert_eq!(
        normalize_path(&people_page.file),
        "src/testing/react/src/App.tsx",
        "Page file path is incorrect"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_react_typescript() {
    use crate::lang::graphs::ArrayGraph;
    test_react_typescript_generic::<ArrayGraph>().await.unwrap();
}

// #[test(tokio::test)]
// async fn test_react_typescript_btree() {
//     use crate::lang::graphs::BTreeMapGraph;
//     test_react_typescript_generic::<BTreeMapGraph>().await.unwrap();
// }
