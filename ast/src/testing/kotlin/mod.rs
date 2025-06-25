use crate::lang::graphs::{EdgeType, NodeType};
use crate::lang::{Graph, Node};
use crate::{lang::Lang, repo::Repo};
use anyhow::Result;
use std::str::FromStr;

pub async fn test_kotlin_generic<G: Graph>() -> Result<(), anyhow::Error> {
    let repo = Repo::new(
        "src/testing/kotlin",
        Lang::from_str("kotlin").unwrap(),
        false,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let graph = repo.build_graph_inner::<G>().await?;

    let (num_nodes, num_edges) = graph.get_graph_size();

    graph.analysis();

    assert_eq!(num_nodes, 167, "Expected 167 nodes");
    assert_eq!(num_edges, 198, "Expected 198 edges");

    fn normalize_path(path: &str) -> String {
        path.replace("\\", "/")
    }

    let language_nodes = graph.find_nodes_by_type(NodeType::Language);
    assert_eq!(language_nodes.len(), 1, "Expected 1 language node");
    assert_eq!(
        language_nodes[0].name, "kotlin",
        "Language node name should be 'kotlin'"
    );
    assert_eq!(
        normalize_path(&language_nodes[0].file),
        "src/testing/kotlin/",
        "Language node file path is incorrect"
    );

    let build_gradle_files = graph.find_nodes_by_name(NodeType::File, "build.gradle.kts");
    assert_eq!(
        build_gradle_files.len(),
        2,
        "Expected 2 build.gradle.kts files"
    );
    assert_eq!(
        build_gradle_files[0].name, "build.gradle.kts",
        "Gradle file name is incorrect"
    );

    let libraries = graph.find_nodes_by_type(NodeType::Library);
    assert_eq!(libraries.len(), 58, "Expected 58 libraries");

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 9, "Expected 9 imports");

    let main_import_body = format!(
        r#"package com.kotlintestapp.sqldelight

import android.content.Context
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.android.AndroidSqliteDriver
import com.kotlintestapp.db.Person
import com.kotlintestapp.db.PersonDatabase"#
    );
    let main = imports
        .iter()
        .find(|i| i.file == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/sqldelight/DatabaseHelper.kt")
        .unwrap();

    assert_eq!(
        main.body, main_import_body,
        "Model import body is incorrect"
    );

    let classes = graph.find_nodes_by_type(NodeType::Class);
    assert_eq!(classes.len(), 6, "Expected 6 classes");

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 9, "Expected 9 imports");

    let variables = graph.find_nodes_by_type(NodeType::Var);
    assert_eq!(variables.len(), 9, "Expected 9 variables");

    let mut sorted_classes = classes.clone();
    sorted_classes.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(
        sorted_classes[1].name, "ExampleInstrumentedTest",
        "Class name is incorrect"
    );
    assert_eq!(
        normalize_path(&sorted_classes[1].file),
        "src/testing/kotlin/app/src/androidTest/java/com/kotlintestapp/ExampleInstrumentedTest.kt",
        "Class file path is incorrect"
    );

    let functions = graph.find_nodes_by_type(NodeType::Function);
    assert_eq!(functions.len(), 19, "Expected 19 functions");

    let data_models = graph.find_nodes_by_type(NodeType::DataModel);
    assert_eq!(data_models.len(), 1, "Expected 1 data model");

    let requests = graph.find_nodes_by_type(NodeType::Request);
    assert_eq!(requests.len(), 2, "Expected 2 requests");

    let calls_edges_count = graph.count_edges_of_type(EdgeType::Calls);
    assert_eq!(calls_edges_count, 15, "Expected at 15 calls edges");

    let import_edges_count = graph.count_edges_of_type(EdgeType::Imports);
    assert_eq!(import_edges_count, 6, "Expected at 6 import edges");

    let person_data_model = data_models
        .iter()
        .find(|dm| {
            dm.name == "Person"
                && normalize_path(&dm.file)
                    == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/models/Person.kt"
        })
        .map(|n| Node::new(NodeType::DataModel, n.clone()))
        .expect("Person DataModel not found");

    let database_helper_class = classes
    .iter()
    .find(|c| c.name == "DatabaseHelper" && normalize_path(&c.file) == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/sqldelight/DatabaseHelper.kt")
    .map(|n| Node::new(NodeType::Class, n.clone()))
    .expect("DatabaseHelper class not found");

    let insert_person_fn = functions
    .iter()
    .find(|f| f.name == "insertPerson" && normalize_path(&f.file) == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/sqldelight/DatabaseHelper.kt")
    .map(|n| Node::new(NodeType::Function, n.clone()))
    .expect("insertPerson function not found");

    let update_person_fn = functions
    .iter()
    .find(|f| f.name == "updatePerson" && normalize_path(&f.file) == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/sqldelight/DatabaseHelper.kt")
    .map(|n| Node::new(NodeType::Function, n.clone()))
    .expect("updatePerson function not found");

    let person_kt_file = graph
        .find_nodes_by_name(NodeType::File, "Person.kt")
        .into_iter()
        .find(|n| {
            normalize_path(&n.file)
                == "src/testing/kotlin/app/src/main/java/com/kotlintestapp/models/Person.kt"
        })
        .map(|n| Node::new(NodeType::File, n))
        .expect("Person.kt file node not found");

    assert!(
        graph.has_edge(&database_helper_class, &insert_person_fn, EdgeType::Operand),
        "Expected DatabaseHelper class to operand insertPerson function"
    );

    assert!(
        graph.has_edge(&database_helper_class, &update_person_fn, EdgeType::Operand),
        "Expected DatabaseHelper class to operand updatePerson function"
    );
    assert!(
        graph.has_edge(&person_kt_file, &person_data_model, EdgeType::Contains),
        "Expected Person.kt file to contain Person DataModel"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kotlin() {
    use crate::lang::graphs::{ArrayGraph, BTreeMapGraph};
    test_kotlin_generic::<ArrayGraph>().await.unwrap();
    test_kotlin_generic::<BTreeMapGraph>().await.unwrap();

    #[cfg(feature = "neo4j")]
    {
        use crate::lang::graphs::Neo4jGraph;
        let mut graph = Neo4jGraph::default();
        graph.clear().await.unwrap();
        test_kotlin_generic::<Neo4jGraph>().await.unwrap();
    }
}
