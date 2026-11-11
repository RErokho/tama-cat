#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

function checkpointPath(cwd) {
  return path.join(cwd, ".cursor", "ai-dev-team", "memory", "CHECKPOINT.md");
}

function decide(cwd) {
  const root = cwd || process.cwd();
  const parts = [
    "AI Dev Team v0.8. Map: AGENTS.md. Registry → one agent card → compact handoff.",
    "Core roster default; optional on signal. Writers: allowed_files (hook). Gates: use check summary, logs on disk.",
    "No agent gets unused context. Procedures: when.md. Prefer CLI; Context7 on demand.",
  ];
  const file = checkpointPath(root);
  if (fs.existsSync(file)) {
    parts.push(
      `Runtime CHECKPOINT.md exists at ${file}. Read it (and skill compact-checkpoint) before continuing prior open work.`
    );
  }
  try {
    const { appendEvent } = require("../ai-dev-team/observability/events.js");
    appendEvent(root, { type: "session.start", note: "sessionStart hook" });
  } catch {
    // optional
  }
  return { additional_context: parts.join(" ") };
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
  let payload = {};
  try {
    payload = JSON.parse((await readStdin()) || "{}");
  } catch {
    payload = {};
  }
  const cwd = String((payload && (payload.cwd || (payload.workspace_roots && payload.workspace_roots[0]))) || process.cwd());
  process.stdout.write(JSON.stringify(decide(cwd)));
}

if (require.main === module) {
  main().catch(() => process.stdout.write(JSON.stringify(decide(process.cwd()))));
}

module.exports = { decide, checkpointPath };
