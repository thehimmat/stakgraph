use crate::lang::graphs::{EdgeType, NodeType};
use crate::lang::{Graph, Node};

use crate::{lang::Lang, repo::Repo};
use std::str::FromStr;

pub async fn test_cpp_generic<G: Graph>() -> Result<(), anyhow::Error> {
    let repo = Repo::new(
        "src/testing/cpp",
        Lang::from_str("cpp").unwrap(),
        false,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let graph = repo.build_graph_inner::<G>().await?;

    graph.analysis();

    let (num_nodes, num_edges) = graph.get_graph_size();

    assert_eq!(num_nodes, 24, "Expected 24 nodes");
    assert_eq!(num_edges, 27, "Expected 27 edges");

    let language_nodes = graph.find_nodes_by_name(NodeType::Language, "cpp");
    assert_eq!(language_nodes.len(), 1, "Expected 1 language node");
    assert_eq!(
        language_nodes[0].name, "cpp",
        "Language node name should be 'cpp'"
    );
    assert!(
        "src/testing/cpp/".contains(language_nodes[0].file.as_str()),
        "Language node file path is incorrect"
    );

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 5, "Expected 5 imports");

    let main_import_body = format!(
        r#"#include "crow.h"

#include "routes.h"

#include "model.h"
"#
    );
    let main = imports
        .iter()
        .find(|i| i.file == "src/testing/cpp/main.cpp")
        .unwrap();

    assert_eq!(
        main.body, main_import_body,
        "Model import body is incorrect"
    );

    let classes = graph.find_nodes_by_type(NodeType::Class);
    assert_eq!(classes.len(), 1, "Expected 1 class");
    assert_eq!(
        classes[0].name, "Database",
        "Class name should be 'Database'"
    );
    assert_eq!(
        classes[0].file, "src/testing/cpp/model.h",
        "Class file path is incorrect"
    );

    let data_models = graph.find_nodes_by_type(NodeType::DataModel);
    assert_eq!(data_models.len(), 1, "Expected 1 data models");
    assert!(
        data_models
            .iter()
            .any(|dm| dm.name == "Person" && dm.file == "src/testing/cpp/model.h"),
        "Expected Person data model not found"
    );

    let endpoints = graph.find_nodes_by_type(NodeType::Endpoint);
    assert_eq!(endpoints.len(), 2, "Expected 2 endpoints");

    let get_endpoint = endpoints
        .iter()
        .find(|e| e.name == "/person/<int>" && e.meta.get("verb") == Some(&"ANY".to_string()))
        .expect("ANY endpoint not found");
    assert_eq!(get_endpoint.file, "src/testing/cpp/routes.cpp");

    let post_endpoint = endpoints
        .iter()
        .find(|e| e.name == "/person" && e.meta.get("verb") == Some(&"POST".to_string()))
        .expect("POST endpoint not found");
    assert_eq!(post_endpoint.file, "src/testing/cpp/routes.cpp");

    let handler_edges_count = graph.count_edges_of_type(EdgeType::Handler);
    assert_eq!(handler_edges_count, 2, "Expected 2 handler edges");

    let function_calls = graph.count_edges_of_type(EdgeType::Calls);
    assert_eq!(function_calls, 3, "Expected 3 function calls");

    let contains = graph.count_edges_of_type(EdgeType::Contains);
    assert_eq!(contains, 21, "Expected 21 contains edges");

    let variables = graph.find_nodes_by_type(NodeType::Var);
    assert_eq!(variables.len(), 1, "Expected 1 variables");

    let instances = graph.find_nodes_by_type(NodeType::Instance);
    assert_eq!(instances.len(), 1, "Expected 1 instances");

    let person_data_model = graph
        .find_nodes_by_name(NodeType::DataModel, "Person")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/model.h")
        .map(|n| Node::new(NodeType::DataModel, n))
        .expect("Person DataModel not found in model.h");

    let database_class = graph
        .find_nodes_by_name(NodeType::Class, "Database")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/model.h")
        .map(|n| Node::new(NodeType::Class, n))
        .expect("Database class not found in model.h");

    let main_fn = graph
        .find_nodes_by_name(NodeType::Function, "main")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/main.cpp")
        .map(|n| Node::new(NodeType::Function, n))
        .expect("main function not found in main.cpp");

    let setup_routes_fn = graph
        .find_nodes_by_name(NodeType::Function, "setup_routes")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/routes.cpp")
        .map(|n| Node::new(NodeType::Function, n))
        .expect("setup_routes function not found in routes.cpp");

    let post_endpoint = graph
        .find_nodes_by_name(NodeType::Endpoint, "/person")
        .into_iter()
        .find(|n| {
            n.file == "src/testing/cpp/routes.cpp"
                && n.meta.get("verb") == Some(&"POST".to_string())
        })
        .map(|n| Node::new(NodeType::Endpoint, n))
        .expect("POST /person endpoint not found in routes.cpp");
    let new_person_fn = graph
        .find_nodes_by_name(NodeType::Function, "new_person")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/routes.cpp")
        .map(|n| Node::new(NodeType::Function, n))
        .expect("new_person function not found in routes.cpp");

    let model_h_file = graph
        .find_nodes_by_name(NodeType::File, "model.h")
        .into_iter()
        .find(|n| n.file == "src/testing/cpp/model.h")
        .map(|n| Node::new(NodeType::File, n))
        .expect("model.h file node not found");

    assert!(
        graph.has_edge(&model_h_file, &person_data_model, EdgeType::Contains),
        "Expected 'Database' class to contain 'Person' DataModel"
    );
    assert!(
        graph.has_edge(&model_h_file, &database_class, EdgeType::Contains),
        "Expected 'Database' class to contain 'Person' DataModel"
    );
    assert!(
        graph.has_edge(&main_fn, &setup_routes_fn, EdgeType::Calls),
        "Expected 'main' function to call 'setup_routes' function"
    );
    assert!(
        graph.has_edge(&post_endpoint, &new_person_fn, EdgeType::Handler),
        "Expected '/person' endpoint to be handled by 'new_person'"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cpp() {
    use crate::lang::graphs::{ArrayGraph, BTreeMapGraph};
    test_cpp_generic::<ArrayGraph>().await.unwrap();
    test_cpp_generic::<BTreeMapGraph>().await.unwrap();

    #[cfg(feature = "neo4j")]
    {
        use crate::lang::graphs::Neo4jGraph;
        let mut graph = Neo4jGraph::default();
        graph.clear().await.unwrap();
        test_cpp_generic::<Neo4jGraph>().await.unwrap();
    }
}
