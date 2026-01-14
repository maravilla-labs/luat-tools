# Contributing to Luat Tools

Thank you for your interest in contributing to Luat Tools!

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v20+)
- [VSCode](https://code.visualstudio.com/) (for extension development)

### Building

```bash
# Clone the repository
git clone https://github.com/maravilla-labs/luat-tools.git
cd luat-tools

# Build the language server
cargo build --release

# Build the VSCode extension
cd editors/vscode
npm install
npm run compile
```

### Running Tests

```bash
# Run Rust tests
cargo test --all

# Run linting
cargo clippy --all -- -D warnings
cargo fmt --all -- --check
```

### Testing the Extension Locally

1. Build the language server: `cargo build --release`
2. Build the extension: `cd editors/vscode && npm run compile`
3. Open the `editors/vscode` folder in VSCode
4. Press F5 to launch a new VSCode window with the extension loaded
5. Open a `.luat` file to test

## Release Process

Releases are automated via GitHub Actions. Here's how to create a new release:

### 1. Update Version Numbers

Update the version in these files:

- `Cargo.toml` (workspace version)
- `crates/luat-lsp/Cargo.toml`
- `editors/vscode/package.json`

### 2. Update Changelog

Add release notes to `CHANGELOG.md` following the existing format.

### 3. Create a Git Tag

```bash
# Commit version changes
git add -A
git commit -m "chore: bump version to vX.Y.Z"

# Create and push tag
git tag vX.Y.Z
git push origin main --tags
```

### 4. Automated Release

Once you push the tag, GitHub Actions will automatically:

1. Create a draft GitHub release
2. Build `luat-lsp` binaries for all platforms:
   - Linux x64 (`x86_64-unknown-linux-gnu`)
   - Linux ARM64 (`aarch64-unknown-linux-gnu`)
   - macOS Intel (`x86_64-apple-darwin`)
   - macOS Apple Silicon (`aarch64-apple-darwin`)
   - Windows x64 (`x86_64-pc-windows-msvc`)
3. Build and attach the VSCode extension (`.vsix`)
4. Publish the release

### 5. Marketplace Publishing (Coming Soon)

VSCode Marketplace publishing will be enabled once the publisher account is set up. The workflow is prepared in `.github/workflows/publish.yml`.

To enable marketplace publishing:

1. Create an Azure DevOps organization at https://dev.azure.com
2. Generate a Personal Access Token (PAT) with "Manage" marketplace permissions
3. Create a publisher at https://marketplace.visualstudio.com/manage
4. Add the PAT as a GitHub repository secret: `VSCE_PAT`
5. (Optional) For Open VSX Registry, add token as: `OVSX_PAT`
6. Uncomment the publishing steps in `.github/workflows/publish.yml`

## Code Style

- Rust code follows standard Rust conventions (enforced by `rustfmt`)
- TypeScript code should pass TypeScript compilation without errors
- All source files must include the license header:
  ```
  // Copyright 2026 Maravilla Labs
  // SPDX-License-Identifier: MIT OR Apache-2.0
  ```

## Pull Requests

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all tests pass
5. Submit a pull request

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
