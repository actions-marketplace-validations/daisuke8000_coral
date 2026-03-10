//! Markdown rendering for diff reports.

use super::core::{Change, DiffItems, DiffReport};
use crate::domain::NodeType;

impl DiffReport {
    /// Generate Markdown representation of the diff.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if !self.has_changes() {
            return "### No Changes Detected\n\n".to_string();
        }

        let mut output = String::from("### Changes from Base\n\n");

        if !self.added.is_empty() {
            output.push_str(&format!("#### ✅ Added (+{})\n", self.added.total_count()));
            render_diff_table(&self.added, &mut output);
        }

        if !self.modified.is_empty() {
            output.push_str(&format!("#### ⚠️ Modified ({})\n", self.modified.len()));
            output.push_str("| Type | Name | Changes |\n");
            output.push_str("|------|------|--------|\n");

            for item in &self.modified {
                let type_str = node_type_label(&item.node_type);
                let changes_summary = summarize_changes(&item.changes);
                output.push_str(&format!(
                    "| {} | {} | {} |\n",
                    type_str,
                    escape_markdown_cell(&item.label),
                    changes_summary,
                ));
            }
            output.push('\n');
        }

        if !self.removed.is_empty() {
            output.push_str(&format!(
                "#### ❌ Removed (-{})\n",
                self.removed.total_count()
            ));
            render_diff_table(&self.removed, &mut output);
        }

        output
    }
}

fn escape_markdown_cell(s: &str) -> String {
    s.replace('|', r"\|")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn node_type_label(node_type: &NodeType) -> &'static str {
    match node_type {
        NodeType::Service => "Service",
        NodeType::Message => "Message",
        NodeType::Enum => "Enum",
        NodeType::External => "External",
    }
}

fn render_diff_table(items: &DiffItems, output: &mut String) {
    output.push_str("| Type | Name | Package |\n");
    output.push_str("|------|------|--------|\n");

    for (type_name, nodes) in [
        ("Service", &items.services),
        ("Message", &items.messages),
        ("Enum", &items.enums),
    ] {
        for node in nodes {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                type_name,
                escape_markdown_cell(&node.label),
                escape_markdown_cell(&node.package),
            ));
        }
    }
    output.push('\n');
}

fn summarize_changes(changes: &[Change]) -> String {
    let counts: &[(&str, usize)] = &[
        (
            "+field",
            changes
                .iter()
                .filter(|c| matches!(c, Change::FieldAdded { .. }))
                .count(),
        ),
        (
            "-field",
            changes
                .iter()
                .filter(|c| matches!(c, Change::FieldRemoved { .. }))
                .count(),
        ),
        (
            "+method",
            changes
                .iter()
                .filter(|c| matches!(c, Change::MethodAdded { .. }))
                .count(),
        ),
        (
            "-method",
            changes
                .iter()
                .filter(|c| matches!(c, Change::MethodRemoved { .. }))
                .count(),
        ),
        (
            "+value",
            changes
                .iter()
                .filter(|c| matches!(c, Change::EnumValueAdded { .. }))
                .count(),
        ),
        (
            "-value",
            changes
                .iter()
                .filter(|c| matches!(c, Change::EnumValueRemoved { .. }))
                .count(),
        ),
    ];

    counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(label, n)| {
            let (sign, kind) = label.split_at(1);
            format!("{sign}{n} {kind}(s)")
        })
        .collect::<Vec<_>>()
        .join(", ")
}
