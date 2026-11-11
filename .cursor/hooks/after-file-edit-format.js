#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");

function whichFormatter(cwd, filePath) {
  const rel = filePath;
  const ext = path.extname(rel).toLowerCase();
  if (![".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".json", ".css", ".md", ".yml", ".yaml"].includes(ext)) {
    return null;
  }
  const prettierLocal = path.join(cwd, "node_modules", ".bin", process.platform === "win32" ? "prettier.cmd" : "prettier");
  if (fs.existsSync(prettierLocal)) {
    return { cmd: prettierLocal, args: ["--write", rel] };
  }
  return null;
}

function decide(payload) {
  const filePath = String((payload && payload.file_path) || "");
  if (!filePath || !fs.existsSync(filePath)) {
    return {};
  }
  const cwd = process.cwd();
  const formatter = whichFormatter(cwd, filePath);
  if (!formatter) return {};
  try {
    spawnSync(formatter.cmd, formatter.args, { cwd, stdio: "ignore", timeout: 15000 });
  } catch {
    // fail open
  }
  return {};
}

function readStdin() {
  return new Promise((resolve, reject) => {
    const chunks = [];
    process.stdin.on("data", (chunk) => chunks.push(chunk));
    process.stdin.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    process.stdin.on("error", reject);
  });
}

async function main() {
  const raw = await readStdin();
  let payload = {};
  try {
    payload = JSON.parse(raw || "{}");
  } catch {
    payload = {};
  }
  process.stdout.write(JSON.stringify(decide(payload)));
}

if (require.main === module) {
  main().catch(() => process.stdout.write("{}"));
}

module.exports = { decide };
