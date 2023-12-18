use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use petgraph::{
    algo::toposort,
    csr::Csr,
    dot::Dot,
    graphmap::UnGraphMap,
    visit::{Dfs, DfsPostOrder, EdgeRef, Walker},
    Graph,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use said::{
    derivation::{HashFunction, HashFunctionCode},
    SelfAddressingIdentifier,
};
use serde::{ser::SerializeSeq, Serialize};

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Debug)]
pub struct ACDC {
    text: String,
    digest: SelfAddressingIdentifier,
}

impl Display for ACDC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap()).unwrap();
        Ok(())
    }
}

impl ACDC {
    pub fn new(text: String) -> Self {
        ACDC {
            digest: HashFunction::from(HashFunctionCode::Blake3_256).derive(&text.as_bytes()),
            text,
        }
    }
}
#[derive(Serialize, Debug)]
enum Edges {
    Value,
    Reference,
}

struct References {
    graph: Graph<SelfAddressingIdentifier, Edges>,
    dict: HashMap<SelfAddressingIdentifier, ACDC>,
}

impl References {
    fn new() -> Self {
        References {
            graph: Graph::<SelfAddressingIdentifier, Edges>::new(),
            dict: HashMap::new(),
        }
    }

    fn setup(&mut self) {
        let acdc1 = ACDC::new("hello1".to_string());
        let acdc2 = ACDC::new("hello2".to_string());
        let acdc3 = ACDC::new("hello3".to_string());
        let acdc4 = ACDC::new("hello4".to_string());

        self.dict.insert(acdc1.digest.clone(), acdc1.clone());
        let origin = self.graph.add_node(acdc1.digest);
        self.dict.insert(acdc2.digest.clone(), acdc2.clone());
        let destination_1 = self.graph.add_node(acdc2.digest);
        self.dict.insert(acdc3.digest.clone(), acdc3.clone());
        let destination_2 = self.graph.add_node(acdc3.digest);
        self.dict.insert(acdc4.digest.clone(), acdc4.clone());
        let destination_3 = self.graph.add_node(acdc4.digest);

        self.graph.add_edge(origin, destination_1, Edges::Value);
        self.graph.add_edge(origin, destination_2, Edges::Reference);
        self.graph
            .add_edge(destination_1, destination_3, Edges::Value);
        let nodes = self.graph.raw_nodes();
        // for node in nodes {
        // 	let acdc = self.dict.get(&node.weight);
        // 	println!("{:#?}", acdc)
        // };
    }

    fn print_graph(&self) {
        println!("{}", serde_json::to_string_pretty(&self.graph).unwrap());
    }

    // Function to perform a traversal based on edge weights
    fn visit_nodes_based_on_weight(&self, start_node: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = vec![];
        let mut stack = vec![start_node];

        while let Some(current_node) = stack.pop() {
            if visited.contains(&current_node) {
                continue;
            }

            visited.push(current_node);

            let mut neighbors_with_weights: Vec<_> = self
                .graph
                .edges(current_node)
                .map(|edge| {
                    let nodes = self.graph.raw_nodes();
                    match edge.weight() {
                        Edges::Value => {
                            let index = edge.target();
                            println!(
                                "node: {:#?}",
                                self.dict.get(&nodes[index.index()].weight).unwrap()
                            );
                            stack.push(edge.target());
                        }
                        Edges::Reference => {
                            let index = edge.target();
                            println!("node: {:#?}", &nodes[index.index()].weight);
                        }
                    };
                })
                .collect();

            // Sort neighbors by edge weights
            // neighbors_with_weights.sort_by(|a, b| a.1.cmp(&b.1));

            // Add neighbors to the stack based on edge weights
            // for neighbor in neighbors_with_weights.into_iter().rev() {
            //     stack.push(neighbor.0);
            // }
        }

        visited
    }

    fn enocde(&self) {
        // 	let g = &self.graph;
        // 	g.edge_weight(e)
        // 	match toposort(g, None){
        //     Ok(order) => {
        //         print!("Sorted: ");
        //         for i in order {
        //              g.node_weight(i).map(|weight| {
        //                  print!("{}, ", weight);
        //                  weight
        //              });
        //          }
        //     },
        //     Err(err) => {
        //         g.node_weight(err.node_id()).map(|weight|
        //             println!("Error graph has cycle at node {}", weight));
        //     }
        // }

        for start in self.graph.node_indices() {
            // let mut dfs = Dfs::new(&self.graph, start);
            print!("\n[{}] ", start.index());
            let nodes = self.graph.raw_nodes();
            for edge in self.graph.edges(start) {
                match edge.weight() {
                    Edges::Value => {
                        let index = edge.target();
                        println!(
                            "node: {:#?}",
                            self.dict.get(&nodes[index.index()].weight).unwrap()
                        );
                    }
                    Edges::Reference => {
                        let index = edge.target();
                        println!("node: {:#?}", &nodes[index.index()].weight);
                    }
                }
            }
            // let edges = self.graph.raw_edges();
            // // println!("node: {:#?}", nodes[start.index()]);
            // println!("node: {:#?}", self.dict.get(&nodes[start.index()].weight).unwrap());

            // while let Some(visited) = dfs.next(&self.graph) {
            // 	println!("\tsubnode : {:#?}", self.dict.get(&nodes[visited.index()].weight).unwrap());
            // 	// print!(" {:#?}", nodes[visited.index()]);
            // }

            // println!();
        }
    }
}

impl Serialize for References {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut visited = vec![];
        let mut stack = vec![self.graph.node_indices().next().unwrap()];

        let mut seq = serializer.serialize_seq(None)?;

        let nodes = self.graph.raw_nodes();
        while let Some(current_node) = stack.pop() {
            if visited.contains(&current_node) {
                continue;
            }

            visited.push(current_node);
            // seq.serialize_element(self.dict.get(&nodes[current_node.index()].weight).unwrap()).unwrap();

            let mut neighbors_with_weights: Vec<_> = self
                .graph
                .edges(current_node)
                .map(|edge| {
                    let nodes = self.graph.raw_nodes();
                    match edge.weight() {
                        Edges::Value => {
                            let index = edge.target();
                            println!(
                                "node: {:#?}",
                                self.dict.get(&nodes[index.index()].weight).unwrap()
                            );
                            stack.push(edge.target());
                        }
                        Edges::Reference => {
                            let index = edge.target();
                            seq.serialize_element(&nodes[index.index()].weight).unwrap();
                            println!("node: {:#?}", &nodes[index.index()].weight);
                        }
                    };
                })
                .collect();

            // Sort neighbors by edge weights
            // neighbors_with_weights.sort_by(|a, b| a.1.cmp(&b.1));

            // Add neighbors to the stack based on edge weights
            // for neighbor in neighbors_with_weights.into_iter().rev() {
            //     stack.push(neighbor.0);
            // }
        }
        seq.end()
    }
}

#[test]
fn test() {
    let mut gr = References::new();
    gr.setup();

    gr.print_graph();
    println!("================================");
    // let start_node = gr.graph.node_indices().next().unwrap();
    // gr.visit_nodes_based_on_weight(start_node);

    let serialized = serde_json::to_string_pretty(&gr).unwrap();
    println!("\n\n{}", serialized);

    // println!("{}", Dot::new(&graph));
}

#[test]
fn ggg() {
    // Create a new graph
    let mut graph = Graph::<&str, ()>::new();

    // Add nodes with values
    let node_a = graph.add_node("Node A");
    let node_b = graph.add_node("Node B");
    let node_c = graph.add_node("Node C");
    let node_d = graph.add_node("Node D");

    // Add edges between nodes
    graph.add_edge(node_a, node_b, ());
    graph.add_edge(node_b, node_c, ());
    graph.add_edge(node_c, node_d, ());
    graph.add_edge(node_a, node_d, ());

    // Perform depth-first traversal starting from the first node
    let first_node = graph.node_indices().next(); // Get the first node index
    if let Some(start_node) = first_node {
        let dfs = Dfs::new(&graph, start_node);
        for node in dfs.iter(&graph) {
            println!("Visited Node {}: {}", node.index(), graph[node]);
            for neighbor in graph.neighbors(node) {
                println!("  Neighbor: {}: {}", neighbor.index(), graph[neighbor]);
            }
        }
    }
}

use petgraph::prelude::*;
use petgraph::visit::{IntoEdges, NodeIndexable};

// Function to perform a traversal based on edge weights
fn visit_nodes_based_on_weight<G>(graph: G, start_node: G::NodeId) -> Vec<G::NodeId>
where
    G: IntoEdges + NodeIndexable,
    G::NodeId: Ord + Debug,
    G::EdgeWeight: Ord + Debug,
{
    let mut visited = vec![];
    let mut stack = vec![start_node];

    while let Some(current_node) = stack.pop() {
        if visited.contains(&current_node) {
            continue;
        }

        visited.push(current_node);

        let mut neighbors_with_weights: Vec<_> = graph
            .edges(current_node)
            .map(|edge| {
                println!("{:?}", (edge.target().clone(), edge.weight()));
                stack.push(edge.target());
            })
            .collect();

        // Sort neighbors by edge weights
        // neighbors_with_weights.sort_by(|a, b| a.1.cmp(&b.1));

        // Add neighbors to the stack based on edge weights
        // for neighbor in neighbors_with_weights.into_iter().rev() {
        //     stack.push(neighbor.0);
        // }
    }

    visited
}

#[test]
fn gttt() {
    // Create a new graph
    let mut graph = Graph::<&str, i32>::new();

    // Add nodes with values
    let node_a = graph.add_node("Node A");
    let node_b = graph.add_node("Node B");
    let node_c = graph.add_node("Node C");
    let node_d = graph.add_node("Node D");

    // Add edges between nodes with weights
    graph.add_edge(node_a, node_b, 2);
    // graph.add_edge(node_b, node_c, 1);
    graph.add_edge(node_b, node_d, 3);
    // graph.add_edge(node_a, node_d, 4);

    // Perform traversal based on edge weights starting from node A
    let result = visit_nodes_based_on_weight(&graph, node_a);
    for node in result {
        println!("Visited Node {}: {}", node.index(), graph[node]);
    }
}
