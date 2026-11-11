#!/usr/bin/env node
"use strict";

/**
 * preToolUse Write|Delete — hard scope for open tasks.
 * No open writer task → allow. Open task with allowed_files → path must match.
 */

const fs = require("fs");
const path = require("path");

const ACTIVE = new Set(["ready", "in_progress", "testing", "review", "rework", "blocked"]);

const ALWAYS_ALLOW = [
  /^\.cursor\/ai-dev-team\/tasks\//,
  /^\.cursor\/ai-dev-team\/memory\/CHECKPOINT\.md$/,
  /^\.cursor\/ai-dev-team\/quality\/(verification-report\.json|logs\/|regression-report\.json)/,
  /^\.cursor\/ai-dev-team\/observability\/(events\.jsonl|observability-report\.json)$/,
];

function tasksDir(cwd) {
  return path.join(cwd, ".cursor", "ai-dev-team", "tasks");
}

function loadOpenTasks(cwd) {
  const dir = tasksDir(cwd);
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((n) => /^TASK-.*\.json$/i.test(n))
    .map((n) => {
      try {
        return JSON.parse(fs.readFileSync(path.join(dir, n), "utf8"));
      } catch {
        return null;
      }
    })
    .filter((t) => t && ACTIVE.has(t.status));
}

function collectAllows(task) {
  const set = new Set();
  const add = (list) => {
    if (!Array.isArray(list)) return;
    for (const p of list) if (typeof p === "string" && p.trim()) set.add(p.replace(/\\/g, "/"));
  };
  add(task.allowed_files);
  if (task.contract) add(task.contract.allowed_files);
  if (Array.isArray(task.packets)) {
    for (const packet of task.packets) add(packet.allowed_files);
  }
  return [...set];
}

function globMatch(rel, pattern) {
  const p = pattern.replace(/\\/g, "/");
  const r = rel.replace(/\\/g, "/");
  if (p === r) return true;
  if (!p.includes("*")) {
    return r === p || r.startsWith(p.replace(/\/$/, "") + "/");
  }
  // limited: ** and * only
  const esc = p
    .replace(/[.+^${}()|[\]\\]/g, "\\$&")
    .replace(/\*\*/g, "<<<DD>>>")
    .replace(/\*/g, "[^/]*")
    .replace(/<<<DD>>>/g, ".*");
  return new RegExp("^" + esc + "$").test(r) || new RegExp("^" + esc + "/").test(r + "/");
}

function pathAllowed(rel, allows) {
  const r = rel.replace(/\\/g, "/").replace(/^\.\//, "");
  if (ALWAYS_ALLOW.some((re) => re.test(r))) return true;
  if (!allows.length) return true;
  return allows.some((a) => globMatch(r, a));
}

function toolPath(payload) {
  const input = (payload && (payload.tool_input || payload.input || payload.args)) || {};
  return (
    input.file_path ||
    input.path ||
    input.target_file ||
    input.filePath ||
    (payload && payload.file_path) ||
    ""
  );
}

function decide(payload, cwd) {
  const root = path.resolve(cwd || process.cwd());
  const rawPath = String(toolPath(payload) || "");
  if (!rawPath) return { permission: "allow" };

  const abs = path.isAbsolute(rawPath) ? rawPath : path.resolve(root, rawPath);
  let rel;
  try {
    rel = path.relative(root, abs).replace(/\\/g, "/");
  } catch {
    return { permission: "allow" };
  }
  if (!rel || rel.startsWith("..")) {
    // outside workspace — deny when a scoped task is open
    const open = loadOpenTasks(root);
    const allows = open.flatMap(collectAllows);
    if (allows.length) {
      return {
        permission: "deny",
        user_message: "AI Dev Team blocked write outside the workspace while a scoped task is open.",
        agent_message: `Path ${rawPath} is outside the project root. Stay inside allowed_files.`,
      };
    }
    return { permission: "allow" };
  }

  const open = loadOpenTasks(root);
  const allows = [...new Set(open.flatMap(collectAllows))];
  if (!allows.length) return { permission: "allow" };

  if (pathAllowed(rel, allows)) return { permission: "allow" };

  try {
    const { appendEvent } = require("../ai-dev-team/observability/events.js");
    appendEvent(root, {
      type: "ownership.deny",
      note: rel,
      meta: { allows },
    });
  } catch {
    /* optional */
  }

  return {
    permission: "deny",
    user_message: `AI Dev Team blocked edit outside allowed_files (${rel}).`,
    agent_message:
      `Write/Delete blocked for ${rel}. Open task allowed_files: ${allows.join(", ")}. ` +
      `Open a repair packet or update the Task Contract — do not bypass the hook.`,
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
    process.stdout.write(JSON.stringify({ permission: "allow" }));
    return;
  }
  process.stdout.write(JSON.stringify(decide(payload, process.cwd())));
}

if (require.main === module) {
  main().catch(() => process.stdout.write(JSON.stringify({ permission: "allow" })));
}

module.exports = { decide, collectAllows, pathAllowed, globMatch, loadOpenTasks, ALWAYS_ALLOW };
