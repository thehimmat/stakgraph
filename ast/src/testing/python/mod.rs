use crate::lang::graphs::{EdgeType, NodeType};
use crate::lang::{Graph, Node};
use crate::{lang::Lang, repo::Repo};
use std::str::FromStr;

pub async fn test_python_generic<G: Graph>() -> Result<(), anyhow::Error> {
    let repo = Repo::new(
        "src/testing/python",
        Lang::from_str("python").unwrap(),
        false,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let graph = repo.build_graph_inner::<G>().await?;

    graph.analysis();

    let (num_nodes, num_edges) = graph.get_graph_size();
    assert_eq!(num_nodes, 86, "Expected 86 nodes");
    assert_eq!(num_edges, 118, "Expected 118 edges");

    let language_nodes = graph.find_nodes_by_type(NodeType::Language);
    assert_eq!(language_nodes.len(), 1, "Expected 1 language node");
    assert_eq!(
        language_nodes[0].name, "python",
        "Language node name should be 'python'"
    );
    assert_eq!(
        language_nodes[0].file, "src/testing/python/",
        "Language node file path is incorrect"
    );

    let files = graph.find_nodes_by_type(NodeType::File);
    assert_eq!(files.len(), 16, "Expected 16 files");

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 12, "Expected 12 imports");

    let calls = graph.count_edges_of_type(EdgeType::Calls);
    assert_eq!(calls, 12, "Expected 12 call edges");

    let contains = graph.count_edges_of_type(EdgeType::Contains);
    assert_eq!(contains, 93, "Expected 93 contains edges");

    let main_import_body = format!(
        r#"import os
import signal
import subprocess
import sys
from fastapi import FastAPI
from flask import Flask
from fastapi_app.routes import router
from database import Base, engine
from flask_app.routes import flask_bp"#
    );
    let main = imports
        .iter()
        .find(|i| i.file == "src/testing/python/main.py")
        .unwrap();

    assert_eq!(
        main.body, main_import_body,
        "Model import body is incorrect"
    );
    let classes = graph.find_nodes_by_type(NodeType::Class);
    assert_eq!(classes.len(), 3, "Expected 3 classes");

    let vars = graph.find_nodes_by_type(NodeType::Var);
    assert_eq!(vars.len(), 25, "Expected 25 variables");

    let mut sorted_classes = classes.clone();
    sorted_classes.sort_by(|a, b| a.name.cmp(&b.name));

    assert!(
        classes
            .iter()
            .any(|c| c.name == "Person" && c.file == "src/testing/python/model.py"),
        "Expected Person class not found"
    );

    let class_function_edges =
        graph.find_nodes_with_edge_type(NodeType::Class, NodeType::Function, EdgeType::Operand);
    assert_eq!(class_function_edges.len(), 2, "Expected 2 methods");

    let data_models = graph.find_nodes_by_type(NodeType::DataModel);
    assert_eq!(data_models.len(), 3, "Expected 3 data models");

    let endpoints = graph.find_nodes_by_type(NodeType::Endpoint);
    assert_eq!(endpoints.len(), 6, "Expected 6 endpoints");

    let imported_edges = graph.count_edges_of_type(EdgeType::Imports);
    assert_eq!(imported_edges, 7, "Expected 7 import edges");

    let person_class = graph
        .find_nodes_by_name(NodeType::Class, "Person")
        .into_iter()
        .find(|n| n.file == "src/testing/python/model.py")
        .map(|n| Node::new(NodeType::Class, n))
        .expect("Person class not found in model.py");

    let create_or_edit_person_dm = graph
        .find_nodes_by_name(NodeType::DataModel, "CreateOrEditPerson")
        .into_iter()
        .find(|n| n.file == "src/testing/python/model.py")
        .map(|n| Node::new(NodeType::DataModel, n))
        .expect("CreateOrEditPerson DataModel not found in model.py");

    let model_py_file = graph
        .find_nodes_by_name(NodeType::File, "model.py")
        .into_iter()
        .find(|n| n.file == "src/testing/python/model.py")
        .map(|n| Node::new(NodeType::File, n))
        .expect("model.py file node not found");

    let fastapi_post_endpoint = graph
        .find_nodes_by_name(NodeType::Endpoint, "/person/")
        .into_iter()
        .find(|n| {
            n.file == "src/testing/python/fastapi_app/routes.py"
                && n.meta.get("verb") == Some(&"POST".to_string())
        })
        .map(|n| Node::new(NodeType::Endpoint, n))
        .expect("FastAPI /person/ POST endpoint not found");

    let create_person_fn = graph
        .find_nodes_by_name(NodeType::Function, "create_person")
        .into_iter()
        .find(|n| n.file == "src/testing/python/fastapi_app/routes.py")
        .map(|n| Node::new(NodeType::Function, n))
        .expect("create_person function not found in fastapi_app/routes.py");

    assert!(
        graph.has_edge(&model_py_file, &person_class, EdgeType::Contains),
        "Expected 'model.py' file to contain 'Person' class"
    );
    assert!(
        graph.has_edge(
            &model_py_file,
            &create_or_edit_person_dm,
            EdgeType::Contains
        ),
        "Expected 'model.py' file to contain 'CreateOrEditPerson' DataModel"
    );
    assert!(
        graph.has_edge(&fastapi_post_endpoint, &create_person_fn, EdgeType::Handler),
        "Expected FastAPI '/person/' POST endpoint to be handled by 'create_person'"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_python() {
    use crate::lang::graphs::{ArrayGraph, BTreeMapGraph};
    test_python_generic::<ArrayGraph>().await.unwrap();
    test_python_generic::<BTreeMapGraph>().await.unwrap();

    #[cfg(feature = "neo4j")]
    {
        use crate::lang::graphs::Neo4jGraph;
        let mut graph = Neo4jGraph::default();
        graph.clear().await.unwrap();
        test_python_generic::<Neo4jGraph>().await.unwrap();
    }
}
