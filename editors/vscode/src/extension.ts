// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

import * as path from "path";
import * as fs from "fs";
import * as https from "https";
import * as os from "os";
import {
  workspace,
  ExtensionContext,
  window,
  commands,
  StatusBarAlignment,
  ProgressLocation,
} from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

const GITHUB_REPO = "maravilla-labs/luat-tools";
const BINARY_NAME = process.platform === "win32" ? "luat-lsp.exe" : "luat-lsp";

export async function activate(context: ExtensionContext) {
  const serverPath = await getServerPath(context);

  if (!serverPath) {
    const choice = await window.showWarningMessage(
      "Luat language server not found. Would you like to download it?",
      "Download",
      "Cancel"
    );

    if (choice === "Download") {
      const downloaded = await downloadServer(context);
      if (!downloaded) {
        window.showErrorMessage(
          "Failed to download luat-lsp. Please install manually."
        );
        return;
      }
      // Restart activation with downloaded binary
      return activate(context);
    }
    return;
  }

  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
      options: {
        env: {
          ...process.env,
          RUST_LOG: "luat_lsp=debug",
        },
      },
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "luat" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.luat"),
    },
    outputChannelName: "Luat Language Server",
  };

  client = new LanguageClient(
    "luat",
    "Luat Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client
  await client.start();

  // Create status bar item
  const statusBarItem = window.createStatusBarItem(
    StatusBarAlignment.Right,
    100
  );
  statusBarItem.text = "$(file-code) Luat";
  statusBarItem.tooltip = "Luat Language Server";
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  // Register restart command
  context.subscriptions.push(
    commands.registerCommand("luat.restartServer", async () => {
      if (client) {
        await client.restart();
        window.showInformationMessage("Luat language server restarted");
      }
    })
  );

  // Register download command
  context.subscriptions.push(
    commands.registerCommand("luat.downloadServer", async () => {
      const downloaded = await downloadServer(context);
      if (downloaded) {
        window.showInformationMessage(
          "Luat language server downloaded. Restart to use it."
        );
      }
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}

async function getServerPath(
  context: ExtensionContext
): Promise<string | undefined> {
  // Check config first
  const config = workspace.getConfiguration("luat");
  const configPath = config.get<string>("server.path");

  if (configPath && fs.existsSync(configPath)) {
    return configPath;
  }

  // Check extension's server directory (downloaded binary)
  const extensionServerPath = path.join(
    context.globalStorageUri.fsPath,
    "server",
    BINARY_NAME
  );
  if (fs.existsSync(extensionServerPath)) {
    return extensionServerPath;
  }

  // Check if in PATH
  const pathDirs = (process.env.PATH || "").split(path.delimiter);
  for (const dir of pathDirs) {
    const candidate = path.join(dir, BINARY_NAME);
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  // Check cargo install location
  const cargoHome =
    process.env.CARGO_HOME || path.join(os.homedir(), ".cargo");
  const cargoPath = path.join(cargoHome, "bin", BINARY_NAME);
  if (fs.existsSync(cargoPath)) {
    return cargoPath;
  }

  return undefined;
}

function getPlatformTarget(): string | undefined {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "linux" && arch === "x64") {
    return "x86_64-unknown-linux-gnu";
  } else if (platform === "linux" && arch === "arm64") {
    return "aarch64-unknown-linux-gnu";
  } else if (platform === "darwin" && arch === "x64") {
    return "x86_64-apple-darwin";
  } else if (platform === "darwin" && arch === "arm64") {
    return "aarch64-apple-darwin";
  } else if (platform === "win32" && arch === "x64") {
    return "x86_64-pc-windows-msvc";
  }

  return undefined;
}

async function downloadServer(context: ExtensionContext): Promise<boolean> {
  const target = getPlatformTarget();
  if (!target) {
    window.showErrorMessage(
      `Unsupported platform: ${process.platform}-${process.arch}`
    );
    return false;
  }

  return window.withProgress(
    {
      location: ProgressLocation.Notification,
      title: "Downloading Luat language server...",
      cancellable: false,
    },
    async (progress) => {
      try {
        // Get latest release
        progress.report({ message: "Fetching latest release..." });
        const release = await getLatestRelease();
        if (!release) {
          window.showErrorMessage("Failed to fetch latest release");
          return false;
        }

        // Find matching asset
        const assetName = `luat-lsp-${release.tag_name}-${target}`;
        const asset = release.assets.find(
          (a: any) =>
            a.name.startsWith(assetName) &&
            (a.name.endsWith(".tar.gz") || a.name.endsWith(".zip"))
        );

        if (!asset) {
          window.showErrorMessage(
            `No binary found for ${target} in release ${release.tag_name}`
          );
          return false;
        }

        // Download asset
        progress.report({ message: `Downloading ${asset.name}...` });
        const serverDir = path.join(
          context.globalStorageUri.fsPath,
          "server"
        );
        fs.mkdirSync(serverDir, { recursive: true });

        const archivePath = path.join(serverDir, asset.name);
        await downloadFile(asset.browser_download_url, archivePath);

        // Extract archive
        progress.report({ message: "Extracting..." });
        await extractArchive(archivePath, serverDir);

        // Make executable on Unix
        if (process.platform !== "win32") {
          const binaryPath = path.join(serverDir, BINARY_NAME);
          fs.chmodSync(binaryPath, 0o755);
        }

        // Clean up archive
        fs.unlinkSync(archivePath);

        return true;
      } catch (error) {
        window.showErrorMessage(`Download failed: ${error}`);
        return false;
      }
    }
  );
}

async function getLatestRelease(): Promise<any> {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: "api.github.com",
      path: `/repos/${GITHUB_REPO}/releases/latest`,
      headers: {
        "User-Agent": "luat-vscode-extension",
      },
    };

    https
      .get(options, (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          try {
            resolve(JSON.parse(data));
          } catch {
            reject(new Error("Failed to parse release data"));
          }
        });
      })
      .on("error", reject);
  });
}

async function downloadFile(url: string, dest: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    const request = (url: string) => {
      https
        .get(url, { headers: { "User-Agent": "luat-vscode-extension" } }, (res) => {
          // Follow redirects
          if (res.statusCode === 302 || res.statusCode === 301) {
            request(res.headers.location!);
            return;
          }

          res.pipe(file);
          file.on("finish", () => {
            file.close();
            resolve();
          });
        })
        .on("error", (err) => {
          fs.unlink(dest, () => {});
          reject(err);
        });
    };

    request(url);
  });
}

async function extractArchive(archivePath: string, destDir: string): Promise<void> {
  const { exec } = require("child_process");
  const { promisify } = require("util");
  const execAsync = promisify(exec);

  if (archivePath.endsWith(".tar.gz")) {
    await execAsync(`tar -xzf "${archivePath}" -C "${destDir}"`);
  } else if (archivePath.endsWith(".zip")) {
    if (process.platform === "win32") {
      await execAsync(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`
      );
    } else {
      await execAsync(`unzip -o "${archivePath}" -d "${destDir}"`);
    }
  }
}
