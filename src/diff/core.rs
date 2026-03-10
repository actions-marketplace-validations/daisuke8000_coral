//! Diff computation for comparing proto dependency graphs.
//!
//! Compares two GraphModels and generates a report showing added, modified, and removed items.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::domain::node::{EnumValue, FieldInfo, MethodSignature};
use crate::domain::{GraphModel, Node, NodeDetails, NodeType};

/// Represents changes between two GraphModel snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub added: DiffItems,
    pub removed: DiffItems,
    pub modified: Vec<ModifiedItem>,
}

/// Collection of items by type (services, messages, enums).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffItems {
    pub services: Vec<DiffNode>,
    pub messages: Vec<DiffNode>,
    pub enums: Vec<DiffNode>,
}

/// Simplified node representation for diff output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNode {
    pub id: String,
    pub label: String,
    pub package: String,
}

impl From<&Node> for DiffNode {
    fn from(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            label: node.label.clone(),
            package: node.package.clone(),
        }
    }
}

/// Represents a modified item with its changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedItem {
    pub node_id: String,
    pub label: String,
    pub node_type: NodeType,
    pub package: String,
    pub changes: Vec<Change>,
}

/// Individual change within a modified item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Change {
    FieldAdded { field: FieldInfo },
    FieldRemoved { field: FieldInfo },
    MethodAdded { method: MethodSignature },
    MethodRemoved { method: MethodSignature },
    EnumValueAdded { value: EnumValue },
    EnumValueRemoved { value: EnumValue },
}

impl DiffReport {
    /// Compute differences between base and head GraphModels.
    #[must_use]
    pub fn compute(base: &GraphModel, head: &GraphModel) -> Self {
        let base_nodes: HashMap<&str, &Node> =
            base.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let head_nodes: HashMap<&str, &Node> =
            head.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let base_ids: HashSet<&str> = base_nodes.keys().copied().collect();
        let head_ids: HashSet<&str> = head_nodes.keys().copied().collect();

        let added = Self::collect_diff_items(head_ids.difference(&base_ids).copied(), &head_nodes);
        let removed =
            Self::collect_diff_items(base_ids.difference(&head_ids).copied(), &base_nodes);

        let mut modified: Vec<ModifiedItem> = base_ids
            .intersection(&head_ids)
            .filter_map(|id| {
                let base_node = base_nodes.get(id)?;
                let head_node = head_nodes.get(id)?;
                Self::compute_node_changes(base_node, head_node)
            })
            .collect();

        modified.sort_by(|a, b| a.node_id.cmp(&b.node_id));

        Self {
            added,
            removed,
            modified,
        }
    }

    /// Check if there are any changes.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.modified.is_empty()
    }

    fn collect_diff_items<'a>(
        ids: impl Iterator<Item = &'a str>,
        nodes: &HashMap<&str, &Node>,
    ) -> DiffItems {
        let mut items = DiffItems::default();

        for id in ids {
            if let Some(node) = nodes.get(id) {
                let diff_node = DiffNode::from(*node);
                match node.node_type {
                    NodeType::Service => items.services.push(diff_node),
                    NodeType::Message => items.messages.push(diff_node),
                    NodeType::Enum => items.enums.push(diff_node),
                    NodeType::External => {}
                }
            }
        }

        items.services.sort_by(|a, b| a.id.cmp(&b.id));
        items.messages.sort_by(|a, b| a.id.cmp(&b.id));
        items.enums.sort_by(|a, b| a.id.cmp(&b.id));

        items
    }

    fn compute_node_changes(base: &Node, head: &Node) -> Option<ModifiedItem> {
        let changes = match (&base.details, &head.details) {
            (
                NodeDetails::Service {
                    methods: base_methods,
                    ..
                },
                NodeDetails::Service {
                    methods: head_methods,
                    ..
                },
            ) => Self::compute_item_changes(
                base_methods,
                head_methods,
                |m| m.name.as_str(),
                |m| Change::MethodAdded { method: m },
                |m| Change::MethodRemoved { method: m },
            ),

            (
                NodeDetails::Message {
                    fields: base_fields,
                },
                NodeDetails::Message {
                    fields: head_fields,
                },
            ) => Self::compute_item_changes(
                base_fields,
                head_fields,
                |f| f.name.as_str(),
                |f| Change::FieldAdded { field: f },
                |f| Change::FieldRemoved { field: f },
            ),

            (
                NodeDetails::Enum {
                    values: base_values,
                },
                NodeDetails::Enum {
                    values: head_values,
                },
            ) => Self::compute_item_changes(
                base_values,
                head_values,
                |v| v.name.as_str(),
                |v| Change::EnumValueAdded { value: v },
                |v| Change::EnumValueRemoved { value: v },
            ),

            _ => vec![],
        };

        if changes.is_empty() {
            None
        } else {
            Some(ModifiedItem {
                node_id: head.id.clone(),
                label: head.label.clone(),
                node_type: head.node_type.clone(),
                package: head.package.clone(),
                changes,
            })
        }
    }

    /// Generic diff computation for named items.
    fn compute_item_changes<T: Clone>(
        base_items: &[T],
        head_items: &[T],
        get_name: fn(&T) -> &str,
        make_added: fn(T) -> Change,
        make_removed: fn(T) -> Change,
    ) -> Vec<Change> {
        let base_set: HashSet<&str> = base_items.iter().map(get_name).collect();
        let head_set: HashSet<&str> = head_items.iter().map(get_name).collect();

        let mut changes = Vec::new();

        for name in head_set.difference(&base_set) {
            if let Some(item) = head_items.iter().find(|i| get_name(i) == *name) {
                changes.push(make_added(item.clone()));
            }
        }

        for name in base_set.difference(&head_set) {
            if let Some(item) = base_items.iter().find(|i| get_name(i) == *name) {
                changes.push(make_removed(item.clone()));
            }
        }

        changes
    }
}

impl DiffItems {
    /// Check if there are no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.services.is_empty() && self.messages.is_empty() && self.enums.is_empty()
    }

    /// Get total count of all items.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.services.len() + self.messages.len() + self.enums.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_base_model() -> GraphModel {
        GraphModel {
            nodes: vec![
                Node::new(
                    "user.v1.UserService".to_string(),
                    NodeType::Service,
                    "user.v1".to_string(),
                    "UserService".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Service {
                        methods: vec![MethodSignature {
                            name: "GetUser".to_string(),
                            input_type: "GetUserRequest".to_string(),
                            output_type: "User".to_string(),
                        }],
                        messages: vec![],
                    },
                ),
                Node::new(
                    "user.v1.User".to_string(),
                    NodeType::Message,
                    "user.v1".to_string(),
                    "User".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Message {
                        fields: vec![FieldInfo {
                            name: "id".to_string(),
                            number: 1,
                            type_name: "string".to_string(),
                            label: "optional".to_string(),
                        }],
                    },
                ),
                Node::new(
                    "user.v1.OldMessage".to_string(),
                    NodeType::Message,
                    "user.v1".to_string(),
                    "OldMessage".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Message { fields: vec![] },
                ),
            ],
            edges: vec![],
            packages: vec![],
        }
    }

    fn create_head_model() -> GraphModel {
        GraphModel {
            nodes: vec![
                Node::new(
                    "user.v1.UserService".to_string(),
                    NodeType::Service,
                    "user.v1".to_string(),
                    "UserService".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Service {
                        methods: vec![
                            MethodSignature {
                                name: "GetUser".to_string(),
                                input_type: "GetUserRequest".to_string(),
                                output_type: "User".to_string(),
                            },
                            MethodSignature {
                                name: "CreateUser".to_string(),
                                input_type: "CreateUserRequest".to_string(),
                                output_type: "User".to_string(),
                            },
                        ],
                        messages: vec![],
                    },
                ),
                Node::new(
                    "user.v1.User".to_string(),
                    NodeType::Message,
                    "user.v1".to_string(),
                    "User".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Message {
                        fields: vec![
                            FieldInfo {
                                name: "id".to_string(),
                                number: 1,
                                type_name: "string".to_string(),
                                label: "optional".to_string(),
                            },
                            FieldInfo {
                                name: "email".to_string(),
                                number: 2,
                                type_name: "string".to_string(),
                                label: "optional".to_string(),
                            },
                        ],
                    },
                ),
                Node::new(
                    "user.v1.NewMessage".to_string(),
                    NodeType::Message,
                    "user.v1".to_string(),
                    "NewMessage".to_string(),
                    "user/v1/user.proto".to_string(),
                    NodeDetails::Message { fields: vec![] },
                ),
            ],
            edges: vec![],
            packages: vec![],
        }
    }

    #[test]
    fn test_no_changes() {
        let model = create_base_model();
        let diff = DiffReport::compute(&model, &model);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_added_detection() {
        let base = create_base_model();
        let head = create_head_model();
        let diff = DiffReport::compute(&base, &head);

        assert_eq!(diff.added.messages.len(), 1);
        assert_eq!(diff.added.messages[0].label, "NewMessage");
    }

    #[test]
    fn test_removed_detection() {
        let base = create_base_model();
        let head = create_head_model();
        let diff = DiffReport::compute(&base, &head);

        assert_eq!(diff.removed.messages.len(), 1);
        assert_eq!(diff.removed.messages[0].label, "OldMessage");
    }

    #[test]
    fn test_modified_detection() {
        let base = create_base_model();
        let head = create_head_model();
        let diff = DiffReport::compute(&base, &head);

        assert_eq!(diff.modified.len(), 2); // UserService and User

        let service_mod = diff
            .modified
            .iter()
            .find(|m| m.label == "UserService")
            .expect("UserService should be modified");
        assert!(
            service_mod.changes.iter().any(
                |c| matches!(c, Change::MethodAdded { method } if method.name == "CreateUser")
            )
        );

        let user_mod = diff
            .modified
            .iter()
            .find(|m| m.label == "User")
            .expect("User should be modified");
        assert!(
            user_mod
                .changes
                .iter()
                .any(|c| matches!(c, Change::FieldAdded { field } if field.name == "email"))
        );
    }

    #[test]
    fn test_to_markdown_no_changes() {
        let model = create_base_model();
        let diff = DiffReport::compute(&model, &model);
        let markdown = diff.to_markdown();
        assert!(markdown.contains("No Changes Detected"));
    }

    #[test]
    fn test_to_markdown_with_changes() {
        let base = create_base_model();
        let head = create_head_model();
        let diff = DiffReport::compute(&base, &head);
        let markdown = diff.to_markdown();

        assert!(markdown.contains("### Changes from Base"));
        assert!(markdown.contains("✅ Added"));
        assert!(markdown.contains("⚠️ Modified"));
        assert!(markdown.contains("❌ Removed"));
        assert!(markdown.contains("NewMessage"));
        assert!(markdown.contains("OldMessage"));
    }

    #[test]
    fn test_diff_items_is_empty() {
        let items = DiffItems::default();
        assert!(items.is_empty());
        assert_eq!(items.total_count(), 0);
    }
}
