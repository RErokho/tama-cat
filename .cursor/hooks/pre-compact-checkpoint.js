#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

function findTasksDir(cwd) {
  const candidates = [
    path.join(cwd, ".cursor", "ai-dev-team", "tasks"),
    path.join(cwd, "ai-dev-team", "tasks"),
  ];
  return candidates.find((dir) => fs.existsSync(dir)) || null;
}

function activeTasks(dir) {
  if (!dir) return [];
  const open = new Set(["in_progress", "review", "testing", "rework", "blocked"]);
  return fs
    .readdirSync(dir)
    .filter((name) => /^TASK-.*\.json$/i.test(name))
    .map((name) => {
      try {
        return JSON.parse(fs.readFileSync(path.join(dir, name), "utf8"));
      } catch {
        return null;
      }
    })
    .filter((task) => task && open.has(task.status));
}

function writeCheckpoint(cwd, tasks) {
  const memoryDir = path.join(cwd, ".cursor", "ai-dev-team", "memory");
  try {
    fs.mkdirSync(memoryDir, { recursive: true });
  } catch {
    return null;
  }
  const lines = [
    "# Checkpoint",
    `- Saved: ${new Date().toISOString()}`,
    "",
  ];
  for (const task of tasks) {
    lines.push(`## ${task.id}`);
    lines.push(`- Title: ${task.title || ""}`);
    lines.push(`- Status: ${task.status}`);
    lines.push(`- Allowed: ${(task.allowed_files || []).join(", ") || "(none)"}`);
    lines.push(`- Forbidden: ${(task.forbidden_files || []).join(", ") || "(none)"}`);
    lines.push(`- Acceptance: ${(task.acceptance || []).join("; ")}`);
    lines.push("");
  }
  if (!tasks.length) {
    lines.push("No open TASK-*.json found.");
    lines.push("");
  }
  lines.push("After compact: read this file and skill `compact-checkpoint` before new tool calls.");
  const file = path.join(memoryDir, "CHECKPOINT.md");
  fs.writeFileSync(file, lines.join("\n"), "utf8");
  return file;
}

function decide(payload) {
  const cwd = String((payload && (payload.cwd || payload.workspace_roots && payload.workspace_roots[0])) || process.cwd());
  const tasks = activeTasks(findTasksDir(cwd));
  const file = writeCheckpoint(cwd, tasks);
  const ids = tasks.map((task) => task.id).join(", ") || "none";
  return {
    // Cursor/Claude-compatible: stdout / message steers the compaction summary
    user_message: `Checkpoint saved (${file || "unavailable"}). Open tasks: ${ids}. Preserve goal, allowed_files, gates left, and next step.`,
  };
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
  const out = decide(payload);
  // Also print instructions on stdout for hosts that use stdout as compact guidance
  process.stdout.write(JSON.stringify(out));
}

if (require.main === module) {
  main().catch(() => {
    process.stdout.write(JSON.stringify({}));
  });
}

module.exports = { decide, activeTasks };
