//! Analyzer module for converting FileDescriptorSet to GraphModel.

mod edge_builder;
mod node_builder;
mod util;

use std::collections::{HashMap, HashSet};

use prost_types::FileDescriptorSet;

use crate::domain::{GraphModel, MessageDef};

/// Analyzer creates definition-level nodes (Service, Message, Enum) from protobuf descriptors.
/// Each Service, Message, and Enum definition becomes its own graph node.
/// Edges are created based on field type references between definitions.
pub struct Analyzer {
    /// Maps fully-qualified type name to node ID (e.g., ".user.v1.User" → "user.v1.User")
    type_to_node_id: HashMap<String, String>,
    /// Maps fully-qualified type name to MessageDef for expandable RPC method fields
    type_to_message_def: HashMap<String, MessageDef>,
    /// Tracks external packages (google.*, buf.*) for External node creation
    external_packages: HashSet<String>,
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_to_node_id: HashMap::new(),
            type_to_message_def: HashMap::new(),
            external_packages: HashSet::new(),
        }
    }

    #[must_use]
    pub fn analyze(&mut self, fds: &FileDescriptorSet) -> GraphModel {
        let mut model = GraphModel::default();

        // First pass: Create Message/Enum nodes and build type mappings
        // (Service nodes need message definitions, so messages must be processed first)
        for file in &fds.file {
            let file_name = file.name.as_deref().unwrap_or("");
            let package = file.package.as_deref().unwrap_or("");
            let is_external = util::is_external_file(file_name);

            // Create Message nodes (skip external files - just track their types)
            for message in &file.message_type {
                if is_external {
                    self.register_external_type(message, package);
                } else if let Some(node) = self.create_message_node(message, package, file_name) {
                    model.nodes.push(node);
                }
            }

            // Create Enum nodes (skip external files - just track their types)
            for enum_type in &file.enum_type {
                if is_external {
                    self.register_external_enum(enum_type, package);
                } else if let Some(node) = self.create_enum_node(enum_type, package, file_name) {
                    model.nodes.push(node);
                }
            }
        }

        // Second pass: Create Service nodes (now message definitions are available)
        for file in &fds.file {
            let file_name = file.name.as_deref().unwrap_or("");
            let package = file.package.as_deref().unwrap_or("");

            for service in &file.service {
                if let Some(node) = self.create_service_node(service, package, file_name) {
                    model.nodes.push(node);
                }
            }
        }

        // Third pass: Create edges based on field type references
        for file in &fds.file {
            let file_name = file.name.as_deref().unwrap_or("");
            if util::is_external_file(file_name) {
                continue;
            }

            let package = file.package.as_deref().unwrap_or("");

            // Edges from Service RPC methods
            for service in &file.service {
                model
                    .edges
                    .extend(self.create_service_edges(service, package));
            }

            // Edges from Message fields
            for message in &file.message_type {
                model
                    .edges
                    .extend(self.create_message_edges(message, package, &mut model.nodes));
            }
        }

        // Deduplicate edges
        model.edges = Self::deduplicate_edges(model.edges);

        model.packages = util::group_packages(&model.nodes);
        model
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use prost_types::field_descriptor_proto::Type;
    use prost_types::{
        DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
        FileDescriptorProto, MethodDescriptorProto, ServiceDescriptorProto,
    };

    use crate::domain::NodeType;

    use super::*;

    #[test]
    fn test_definition_level_nodes() {
        let fds = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("user/v1/user.proto".to_string()),
                package: Some("user.v1".to_string()),
                service: vec![ServiceDescriptorProto {
                    name: Some("UserService".to_string()),
                    method: vec![MethodDescriptorProto {
                        name: Some("GetUser".to_string()),
                        input_type: Some(".user.v1.GetUserRequest".to_string()),
                        output_type: Some(".user.v1.User".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                message_type: vec![
                    DescriptorProto {
                        name: Some("GetUserRequest".to_string()),
                        field: vec![FieldDescriptorProto {
                            name: Some("user_id".to_string()),
                            number: Some(1),
                            r#type: Some(Type::String as i32),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    DescriptorProto {
                        name: Some("User".to_string()),
                        field: vec![
                            FieldDescriptorProto {
                                name: Some("id".to_string()),
                                number: Some(1),
                                r#type: Some(Type::String as i32),
                                ..Default::default()
                            },
                            FieldDescriptorProto {
                                name: Some("status".to_string()),
                                number: Some(2),
                                r#type: Some(Type::Enum as i32),
                                type_name: Some(".user.v1.UserStatus".to_string()),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    },
                ],
                enum_type: vec![EnumDescriptorProto {
                    name: Some("UserStatus".to_string()),
                    value: vec![
                        EnumValueDescriptorProto {
                            name: Some("UNKNOWN".to_string()),
                            number: Some(0),
                            ..Default::default()
                        },
                        EnumValueDescriptorProto {
                            name: Some("ACTIVE".to_string()),
                            number: Some(1),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let mut analyzer = Analyzer::new();
        let graph = analyzer.analyze(&fds);

        // Should have 4 nodes: 1 Service + 2 Messages + 1 Enum
        assert_eq!(graph.nodes.len(), 4);

        // Check Service node
        let service = graph
            .nodes
            .iter()
            .find(|n| n.id == "user.v1.UserService")
            .expect("Service node should exist");
        assert_eq!(service.node_type, NodeType::Service);
        assert_eq!(service.label, "UserService");

        // Check Message nodes
        let request = graph
            .nodes
            .iter()
            .find(|n| n.id == "user.v1.GetUserRequest")
            .expect("Request message should exist");
        assert_eq!(request.node_type, NodeType::Message);

        let user = graph
            .nodes
            .iter()
            .find(|n| n.id == "user.v1.User")
            .expect("User message should exist");
        assert_eq!(user.node_type, NodeType::Message);

        // Check Enum node
        let status = graph
            .nodes
            .iter()
            .find(|n| n.id == "user.v1.UserStatus")
            .expect("Enum node should exist");
        assert_eq!(status.node_type, NodeType::Enum);
        assert_eq!(status.label, "UserStatus");

        // Check edges (Service → Request, Service → User, User → UserStatus)
        assert_eq!(graph.edges.len(), 3);
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.source == "user.v1.UserService" && e.target == "user.v1.GetUserRequest")
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.source == "user.v1.UserService" && e.target == "user.v1.User")
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.source == "user.v1.User" && e.target == "user.v1.UserStatus")
        );
    }

    #[test]
    fn test_external_dependencies() {
        let fds = FileDescriptorSet {
            file: vec![
                FileDescriptorProto {
                    name: Some("google/protobuf/timestamp.proto".to_string()),
                    package: Some("google.protobuf".to_string()),
                    message_type: vec![DescriptorProto {
                        name: Some("Timestamp".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                FileDescriptorProto {
                    name: Some("user/v1/user.proto".to_string()),
                    package: Some("user.v1".to_string()),
                    message_type: vec![DescriptorProto {
                        name: Some("User".to_string()),
                        field: vec![FieldDescriptorProto {
                            name: Some("created_at".to_string()),
                            number: Some(1),
                            r#type: Some(Type::Message as i32),
                            type_name: Some(".google.protobuf.Timestamp".to_string()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
        };

        let mut analyzer = Analyzer::new();
        let graph = analyzer.analyze(&fds);

        // Should have 2 nodes: User + External Timestamp
        assert_eq!(graph.nodes.len(), 2);

        let user = graph
            .nodes
            .iter()
            .find(|n| n.id == "user.v1.User")
            .expect("User should exist");
        assert_eq!(user.node_type, NodeType::Message);

        let timestamp = graph
            .nodes
            .iter()
            .find(|n| n.id == "google.protobuf.Timestamp")
            .expect("External timestamp should exist");
        assert_eq!(timestamp.node_type, NodeType::External);

        // Edge from User to Timestamp
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "user.v1.User");
        assert_eq!(graph.edges[0].target, "google.protobuf.Timestamp");
    }

    #[test]
    fn test_analyze_empty() {
        let fds = FileDescriptorSet { file: vec![] };
        let mut analyzer = Analyzer::new();
        let graph = analyzer.analyze(&fds);

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.packages.is_empty());
    }

    #[test]
    fn test_multiple_services_same_file() {
        let fds = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("api/v1/api.proto".to_string()),
                package: Some("api.v1".to_string()),
                service: vec![
                    ServiceDescriptorProto {
                        name: Some("UserService".to_string()),
                        ..Default::default()
                    },
                    ServiceDescriptorProto {
                        name: Some("OrderService".to_string()),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
        };

        let mut analyzer = Analyzer::new();
        let graph = analyzer.analyze(&fds);

        // Should have 2 Service nodes from the same file
        assert_eq!(graph.nodes.len(), 2);
        assert!(graph.nodes.iter().any(|n| n.id == "api.v1.UserService"));
        assert!(graph.nodes.iter().any(|n| n.id == "api.v1.OrderService"));
        assert!(
            graph
                .nodes
                .iter()
                .all(|n| n.file == "api/v1/api.proto" && n.node_type == NodeType::Service)
        );
    }
}
