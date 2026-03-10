use super::*;

use crate::domain::node::{FieldInfo, MethodSignature};
use crate::domain::{GraphModel, Node, NodeDetails, NodeType};

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
        service_mod
            .changes
            .iter()
            .any(|c| matches!(c, Change::MethodAdded { method } if method.name == "CreateUser"))
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
