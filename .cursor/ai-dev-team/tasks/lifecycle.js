#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { validateTask, STATUSES } = require("./validate.js");

const TRANSITIONS = {
  inbox: ["analyzing", "clarification", "planned", "cancelled"],
  analyzing: ["clarification", "planned", "cancelled"],
  clarification: ["analyzing", "planned", "cancelled"],
  planned: ["approved", "ready", "cancelled"],
  approved: ["ready", "cancelled"],
  ready: ["in_progress", "cancelled"],
  in_progress: ["blocked", "testing", "review", "failed", "cancelled"],
  blocked: ["in_progress", "cancelled"],
  failed: ["rework", "cancelled"],
  testing: ["review", "rework", "failed", "blocked"],
  review: ["verified", "rework", "failed", "blocked"],
  rework: ["in_progress", "ready", "cancelled"],
  verified: ["completed", "rework"],
  completed: [],
  cancelled: [],
};

function canTransition(from, to) {
  return Boolean(TRANSITIONS[from] && TRANSITIONS[from].includes(to));
}

function transitionTask(task, toStatus, meta) {
  const errors = [];
  if (!task || typeof task !== "object") return { ok: false, errors: ["task required"] };
  const from = task.status;
  if (!STATUSES.includes(toStatus)) {
    return { ok: false, errors: [`unknown status ${toStatus}`] };
  }
  if (!canTransition(from, toStatus)) {
    errors.push(`illegal transition ${from} -> ${toStatus}`);
  }
  if (toStatus === "verified") {
    const reportPath = meta && meta.verificationReportPath;
    if (!reportPath || !fs.existsSync(reportPath)) {
      errors.push("verified requires verification-report.json path that exists");
    } else {
      try {
        const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
        if (report.status !== "passed" && report.status !== "skipped") {
          errors.push(`verification report status is ${report.status}`);
        }
        if (report.status === "skipped" && !(meta && meta.allowSkip)) {
          errors.push("skipped verification needs explicit allowSkip");
        }
        if (report.task_id && task.id && report.task_id !== task.id) {
          errors.push("verification report task_id mismatch");
        }
      } catch (error) {
        errors.push(`cannot read verification report: ${error.message}`);
      }
    }
  }
  if (toStatus === "completed" && from !== "verified") {
    errors.push("completed requires status verified first");
  }
  if (errors.length) return { ok: false, errors };

  const next = { ...task, status: toStatus };
  next.history = Array.isArray(task.history) ? task.history.slice() : [];
  next.history.push({
    at: new Date().toISOString(),
    from,
    to: toStatus,
    by: (meta && meta.by) || "team-lead",
    note: (meta && meta.note) || "",
  });
  if (meta && meta.contractVersion) {
    next.contract_version = meta.contractVersion;
  } else if (next.contract && next.contract_version == null) {
    next.contract_version = 1;
  }
  const shapeErrors = validateTask(next);
  if (shapeErrors.length) return { ok: false, errors: shapeErrors };
  return { ok: true, task: next };
}

function loadTask(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function saveTask(file, task) {
  fs.writeFileSync(file, JSON.stringify(task, null, 2) + "\n", "utf8");
}

function main() {
  const file = process.argv[2];
  const to = process.argv[3];
  const report = process.argv.includes("--report")
    ? process.argv[process.argv.indexOf("--report") + 1]
    : null;
  const allowSkip = process.argv.includes("--allow-skip");
  if (!file || !to) {
    process.stderr.write("usage: node lifecycle.js <TASK.json> <toStatus> [--report path] [--allow-skip]\n");
    process.exit(2);
  }
  const task = loadTask(file);
  const result = transitionTask(task, to, {
    verificationReportPath: report,
    allowSkip,
    by: "cli",
  });
  if (!result.ok) {
    process.stderr.write(JSON.stringify({ ok: false, errors: result.errors }, null, 2) + "\n");
    process.exit(1);
  }
  saveTask(file, result.task);
  try {
    const { appendEvent } = require("../observability/events.js");
    // Prefer cwd (CLI run from project root); fall back to walking up from the task file.
    let root = process.cwd();
    const abs = path.resolve(file);
    const marker = path.join(".cursor", "ai-dev-team");
    if (!fs.existsSync(path.join(root, marker))) {
      let dir = path.dirname(abs);
      for (let i = 0; i < 8; i++) {
        if (fs.existsSync(path.join(dir, marker))) {
          root = dir;
          break;
        }
        const parent = path.dirname(dir);
        if (parent === dir) break;
        dir = parent;
      }
    }
    appendEvent(root, {
      type: "task.transition",
      task_id: result.task.id,
      status: to,
      meta: { from: task.status, to },
    });
  } catch {
    // optional
  }
  process.stdout.write(JSON.stringify({ ok: true, from: task.status, to, file }, null, 2) + "\n");
}

if (require.main === module) main();

module.exports = { TRANSITIONS, canTransition, transitionTask };
