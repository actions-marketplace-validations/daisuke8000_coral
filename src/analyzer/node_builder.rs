//! Node creation logic for protobuf descriptors.

use std::collections::HashSet;

use crate::domain::{
    EnumValue, FieldInfo, MessageDef, MethodSignature, Node, NodeDetails, NodeType,
};

use super::Analyzer;
use super::util;

/// Maximum nesting depth for recursive message registration to prevent stack overflow.
const MAX_NESTING_DEPTH: usize = 32;

impl Analyzer {
    pub(crate) fn create_service_node(
        &mut self,
        service: &prost_types::ServiceDescriptorProto,
        package: &str,
        file_name: &str,
    ) -> Option<Node> {
        let name = service.name.as_ref()?;
        let id = util::generate_node_id(package, name);
        let fq_type = util::generate_fq_type(package, name);
        self.type_to_node_id.insert(fq_type, id.clone());

        let methods: Vec<MethodSignature> = service
            .method
            .iter()
            .map(|m| MethodSignature {
                name: m.name.clone().unwrap_or_default(),
                input_type: util::extract_short_type(m.input_type.as_ref()),
                output_type: util::extract_short_type(m.output_type.as_ref()),
            })
            .collect();

        // Collect message definitions for input/output types (for expandable RPC fields)
        let mut seen_types = HashSet::new();
        let mut messages = Vec::new();
        for method in &service.method {
            for type_name in [&method.input_type, &method.output_type]
                .into_iter()
                .flatten()
            {
                if seen_types.insert(type_name.clone())
                    && let Some(msg_def) = self.type_to_message_def.get(type_name)
                {
                    messages.push((*msg_def).clone());
                }
            }
        }

        Some(Node::new(
            id,
            NodeType::Service,
            package.to_string(),
            name.clone(),
            file_name.to_string(),
            NodeDetails::Service { methods, messages },
        ))
    }

    pub(crate) fn create_message_node(
        &mut self,
        message: &prost_types::DescriptorProto,
        package: &str,
        file_name: &str,
    ) -> Option<Node> {
        let name = message.name.as_ref()?;
        let id = util::generate_node_id(package, name);
        let fq_type = util::generate_fq_type(package, name);
        self.type_to_node_id.insert(fq_type.clone(), id.clone());

        // Also register nested types
        for nested in &message.nested_type {
            self.register_nested_message(nested, &fq_type, 0);
        }
        for nested_enum in &message.enum_type {
            self.register_nested_enum(nested_enum, &fq_type);
        }

        let fields: Vec<FieldInfo> = message
            .field
            .iter()
            .map(|f| FieldInfo {
                name: f.name.clone().unwrap_or_default(),
                number: f.number.unwrap_or(0),
                type_name: util::type_to_string(f.r#type, f.type_name.as_ref()),
                label: util::label_to_string(f.label),
            })
            .collect();

        // Register MessageDef for expandable RPC method fields
        self.type_to_message_def.insert(
            fq_type,
            MessageDef {
                name: name.clone(),
                fields: fields.clone(),
            },
        );

        Some(Node::new(
            id,
            NodeType::Message,
            package.to_string(),
            name.clone(),
            file_name.to_string(),
            NodeDetails::Message { fields },
        ))
    }

    pub(crate) fn create_enum_node(
        &mut self,
        enum_type: &prost_types::EnumDescriptorProto,
        package: &str,
        file_name: &str,
    ) -> Option<Node> {
        let name = enum_type.name.as_ref()?;
        let id = util::generate_node_id(package, name);
        let fq_type = util::generate_fq_type(package, name);
        self.type_to_node_id.insert(fq_type, id.clone());

        let values = enum_type
            .value
            .iter()
            .map(|v| EnumValue {
                name: v.name.clone().unwrap_or_default(),
                number: v.number.unwrap_or(0),
            })
            .collect();

        Some(Node::new(
            id,
            NodeType::Enum,
            package.to_string(),
            name.clone(),
            file_name.to_string(),
            NodeDetails::Enum { values },
        ))
    }

    /// Register an external definition (type or enum) and return its fully-qualified type name.
    fn register_external_definition(&mut self, name: &str, package: &str) -> String {
        let id = util::generate_node_id(package, name);
        let fq_type = util::generate_fq_type(package, name);
        self.type_to_node_id.insert(fq_type.clone(), id);
        self.external_packages.insert(package.to_string());
        fq_type
    }

    pub(crate) fn register_external_type(
        &mut self,
        message: &prost_types::DescriptorProto,
        package: &str,
    ) {
        if let Some(name) = &message.name {
            let fq_type = self.register_external_definition(name, package);

            for nested in &message.nested_type {
                self.register_nested_message(nested, &fq_type, 0);
            }
        }
    }

    pub(crate) fn register_external_enum(
        &mut self,
        enum_type: &prost_types::EnumDescriptorProto,
        package: &str,
    ) {
        if let Some(name) = &enum_type.name {
            self.register_external_definition(name, package);
        }
    }

    /// Register a nested type and return its fully-qualified type name.
    fn register_nested_type(&mut self, name: &str, parent_fq: &str) -> String {
        let fq_type = format!("{parent_fq}.{name}");
        let id = fq_type.trim_start_matches('.').to_string();
        self.type_to_node_id.insert(fq_type.clone(), id);
        fq_type
    }

    pub(crate) fn register_nested_message(
        &mut self,
        message: &prost_types::DescriptorProto,
        parent_fq: &str,
        depth: usize,
    ) {
        if depth >= MAX_NESTING_DEPTH {
            return;
        }

        if let Some(name) = &message.name {
            let fq_type = self.register_nested_type(name, parent_fq);

            for nested in &message.nested_type {
                self.register_nested_message(nested, &fq_type, depth + 1);
            }
        }
    }

    pub(crate) fn register_nested_enum(
        &mut self,
        enum_type: &prost_types::EnumDescriptorProto,
        parent_fq: &str,
    ) {
        if let Some(name) = &enum_type.name {
            self.register_nested_type(name, parent_fq);
        }
    }
}
