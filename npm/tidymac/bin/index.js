#!/usr/bin/env node

const { spawnSync } = require("child_process");
const os = require("os");
const path = require("path");

const arch = os.arch();
const platform = os.platform();

if (platform !== "darwin") {
  console.error("tidymac: only macOS is supported");
  process.exit(1);
}

let binPath;
try {
  binPath = require.resolve(`@tidymac/${platform}-${arch}/bin/tidymac`);
} catch {
  console.error(
    `tidymac: no prebuilt binary available for ${platform}-${arch}\n` +
    `Supported: darwin-arm64 (Apple Silicon), darwin-x64 (Intel)`
  );
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 1);
