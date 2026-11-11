#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const OPEN = new Set(["in_progress", "review", "testing", "rework"]);

function tasksDir(cwd) {
  const dir = path.join(cwd, ".cursor", "ai-dev-team", "tasks");
  return fs.existsSync(dir) ? dir : null;
}

function listOpen(cwd) {
  const dir = tasksDir(cwd);
  if (!dir) return [];
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
    .filter((task) => task && OPEN.has(task.status));
}

function decide(payload) {
  const status = String((payload && payload.status) || "completed");
  const loopCount = Number((payload && payload.loop_count) || 0);
  if (status !== "completed") return {};
  if (loopCount >= 2) return {};

  const open = listOpen(process.cwd());
  if (!open.length) return {};

  const task = open[0];
  return {
    followup_message:
      `Open task ${task.id} is still "${task.status}". ` +
      `Close gates via skill quality-gates / run-gates, or set status blocked/completed with a fact. ` +
      `Do not start unrelated work. (auto loop ${loopCount + 1}/2)`,
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
  process.stdout.write(JSON.stringify(decide(payload)));
}

if (require.main === module) {
  main().catch(() => process.stdout.write("{}"));
}

module.exports = { decide };
