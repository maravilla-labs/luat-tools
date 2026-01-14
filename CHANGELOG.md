# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-14

### Added
- Multi-line comment syntax highlighting: `{/* ... */}`
- Single-line comment syntax highlighting: `{-- ... --}`
- Auto-closing pairs for both comment styles

### Fixed
- LSP no longer reports false "unclosed bracket" errors inside comments

## [0.1.0] - 2026-01-14

### Added
- Initial release
- Luat Language Server (luat-lsp)
- VSCode extension with syntax highlighting
- Go to definition for components and modules
- Hover information for Luat syntax
- Basic diagnostics for unclosed braces and tags
