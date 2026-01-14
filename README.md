# Luat Tools

Language tooling for [Luat](https://github.com/maravilla-labs/luat) template files.

## Components

- **luat-lsp** - Language server providing IDE features
- **editors/vscode** - Visual Studio Code extension

## Features

- Syntax highlighting for `.luat` files
- Completions for template syntax (`{#if}`, `{#each}`, `{@html}`, etc.)
- Lua language support in `<script>` blocks
- Go-to-definition for components
- Diagnostics for syntax errors
- Document symbols for outline view

## Installation

### VSCode Marketplace (Coming Soon)

The extension will be available on the VSCode Marketplace. Search for "Luat" in the extensions panel.

### Download from GitHub Releases

1. Download the `.vsix` file from the [latest release](https://github.com/maravilla-labs/luat-tools/releases/latest)
2. Install in VSCode:
   ```bash
   code --install-extension luat-*.vsix
   ```

The extension will automatically download the correct `luat-lsp` binary for your platform on first activation.

### Manual Installation

If you prefer to install the language server manually:

1. Download `luat-lsp` binary for your platform from [releases](https://github.com/maravilla-labs/luat-tools/releases/latest)
2. Place it in your PATH, or configure the path in VSCode settings:
   ```json
   {
     "luat.server.path": "/path/to/luat-lsp"
   }
   ```

### Build from Source

```bash
# Install the language server
cargo install --path crates/luat-lsp

# Build and install the VSCode extension
cd editors/vscode
npm install
npm run package
code --install-extension luat-*.vsix
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and release process.

## Supported Platforms

| Platform | Architecture | Binary |
|----------|-------------|--------|
| Linux | x64 | `luat-lsp-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `luat-lsp-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `luat-lsp-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `luat-lsp-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows | x64 | `luat-lsp-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

## License

This project is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Copyright 2026 Maravilla Labs
