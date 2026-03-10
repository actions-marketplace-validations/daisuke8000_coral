# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8] - 2026-03-10

### Changed
- Modularize `analyzer.rs` into `analyzer/` submodules (mod.rs, core.rs, node_builder.rs, edge_builder.rs, util.rs)
- Modularize `diff.rs` into `diff/` submodules (mod.rs, core.rs, markdown.rs)
- Separate test files into dedicated `tests.rs` files

### Security
- Add stdin read size limit (256 MiB) to prevent OOM (H1)
- Log canonical path of `--static-dir` at startup (H2)
- Escape pipe and HTML chars in Markdown table cells (M1)
- Add recursion depth limit to nested message registration (H3)

## [0.1.7] - 2026-03-09

### Fixed
- Resolve mobile GPU crash and autolayout overlap (#7)

## [0.1.6] - 2025-12-23

### Changed
- Clean up comments in Cargo.toml

## [0.1.5] - 2025-12-23

### Fixed
- Show only diff in PR comments instead of full analysis (#6)

## [0.1.4] - 2025-12-17

### Added
- Release drafter and autolabeler (#4, #5)

### Changed
- Sync README inputs/outputs with action.yml

## [0.1.3] - 2025-12-17

### Added
- MIT License
- Package metadata (license, repository, description) in Cargo.toml

## [0.1.2] - 2025-12-17

### Added
- Markdown reports and diff output for GitHub Actions PR comments (#2)

### Changed
- Update dependencies to latest versions (#3)
  - `prost`: 0.13 → 0.14
  - `thiserror`: 1.0 → 2.0
  - `axum`: 0.7 → 0.8
  - `tower-http`: 0.5 → 0.6

## [0.1.1] - 2025-12-16

### Added
- Responsive mobile UI support (#1)

## [0.1.0] - 2025-12-14

### Added
- Initial release of Coral CLI tool
- Parse `FileDescriptorSet` binary from stdin (`buf build -o -`)
- Axum-based JSON API server (`/api/graph`, `/health`)
- React Flow frontend with Neon dark theme
- Node types: Service (magenta), Message (cyan), Enum (yellow), External (gray)
- Package grouping with expand/collapse
- Dagre auto-layout for graph visualization
- Expandable RPC type display in DetailPanel
- Resizable drawer and horizontal scroll
- GitHub Pages deployment workflow

### Security
- Prevent script injection via process.env

[Unreleased]: https://github.com/daisuke8000/coral/compare/v0.1.8...HEAD
[0.1.8]: https://github.com/daisuke8000/coral/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/daisuke8000/coral/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/daisuke8000/coral/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/daisuke8000/coral/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/daisuke8000/coral/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/daisuke8000/coral/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/daisuke8000/coral/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/daisuke8000/coral/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/daisuke8000/coral/releases/tag/v0.1.0
