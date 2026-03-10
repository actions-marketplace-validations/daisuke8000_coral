//! Edge creation logic for dependency relationships between protobuf definitions.

use std::collections::HashSet;

use crate::domain::{Edge, Node, NodeDetails, NodeType};

use super::Analyzer;
use super::util;

impl Analyzer {
    pub(crate) fn create_service_edges(
        &self,
        service: &prost_types::ServiceDescriptorProto,
        package: &str,
    ) -> Vec<Edge> {
        let service_name = match &service.name {
            Some(n) => n,
            None => return Vec::new(),
        };
        let source_id = util::generate_node_id(package, service_name);

        let mut edges = Vec::new();
        for method in &service.method {
            // Edge to input type
            if let Some(input_type) = &method.input_type
                && let Some(target_id) = self.type_to_node_id.get(input_type)
            {
                edges.push(Edge::new(source_id.clone(), target_id.clone()));
            }
            // Edge to output type
            if let Some(output_type) = &method.output_type
                && let Some(target_id) = self.type_to_node_id.get(output_type)
            {
                edges.push(Edge::new(source_id.clone(), target_id.clone()));
            }
        }
        edges
    }

    pub(crate) fn create_message_edges(
        &self,
        message: &prost_types::DescriptorProto,
        package: &str,
        nodes: &mut Vec<Node>,
    ) -> Vec<Edge> {
        let message_name = match &message.name {
            Some(n) => n,
            None => return Vec::new(),
        };
        let source_id = util::generate_node_id(package, message_name);

        let mut edges = Vec::new();
        for field in &message.field {
            if let Some(type_name) = &field.type_name
                && let Some(target_id) = self.type_to_node_id.get(type_name)
            {
                // Create External node if referenced type is from external package
                if self.is_external_type(type_name) {
                    self.ensure_external_node(target_id, type_name, nodes);
                }
                edges.push(Edge::new(source_id.clone(), target_id.clone()));
            }
        }
        edges
    }

    fn is_external_type(&self, fq_type: &str) -> bool {
        // Check if type starts with external packages
        let type_without_dot = fq_type.trim_start_matches('.');
        type_without_dot.starts_with("google.") || type_without_dot.starts_with("buf.")
    }

    fn ensure_external_node(&self, id: &str, fq_type: &str, nodes: &mut Vec<Node>) {
        // Check if External node already exists
        if nodes.iter().any(|n| n.id == id) {
            return;
        }

        let type_without_dot = fq_type.trim_start_matches('.');
        let parts: Vec<&str> = type_without_dot.rsplitn(2, '.').collect();
        let (label, package) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (type_without_dot.to_string(), String::new())
        };

        // Determine file path from package
        let file = format!("{}.proto", package.replace('.', "/"));

        nodes.push(Node::new(
            id.to_string(),
            NodeType::External,
            package,
            label,
            file,
            NodeDetails::External,
        ));
    }

    pub(crate) fn deduplicate_edges(edges: Vec<Edge>) -> Vec<Edge> {
        let mut seen = HashSet::new();
        edges
            .into_iter()
            .filter(|e| seen.insert((e.source.clone(), e.target.clone())))
            .collect()
    }
}
