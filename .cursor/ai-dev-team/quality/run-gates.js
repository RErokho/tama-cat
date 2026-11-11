#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
const { validateShape } = require("./validate-gates.js");

function loadJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function selectGates(doc, only) {
  const list = (doc.gates || []).filter((g) => g.kind === "command" && g.command);
  if (!only || !only.length) return list.filter((g) => g.required);
  const set = new Set(only);
  return list.filter((g) => set.has(g.id) || set.has(g.id.split(":")[0]));
}

function runOne(gate, root, logsDir) {
  const started = Date.now();
  const cwd = path.resolve(root, gate.cwd || ".");
  const logFile = path.join(logsDir, `${gate.id.replace(/[^\w.-]+/g, "_")}.log`);
  const result = spawnSync(gate.command, {
    cwd,
    shell: true,
    encoding: "utf8",
    timeout: gate.timeout || 300000,
    env: process.env,
  });
  const output = `${result.stdout || ""}${result.stderr || ""}`;
  fs.writeFileSync(logFile, output || `(no output)\nexit=${result.status}\n`, "utf8");
  const exitCode = result.status === null ? 1 : result.status;
  return {
    gate: gate.id,
    status: exitCode === 0 ? "passed" : "failed",
    exit_code: exitCode,
    duration_ms: Date.now() - started,
    command: gate.command,
    evidence: path.relative(root, logFile).replace(/\\/g, "/"),
    required: Boolean(gate.required),
    summary: summarizeOutput(output, exitCode),
  };
}

function summarizeOutput(output, exitCode) {
  const text = String(output || "").replace(/\r/g, "");
  const lines = text.split("\n").filter((l) => l.trim());
  if (exitCode === 0) {
    return {
      ok: true,
      lines: lines.length,
      headline: lines.slice(0, 2).join(" | ").slice(0, 200) || "ok",
    };
  }
  const errorish = lines.filter((l) => /error|fail|ERR!|TS\d+|✖|×/i.test(l)).slice(0, 8);
  const head = (errorish.length ? errorish : lines.slice(0, 8)).map((l) => l.slice(0, 200));
  return {
    ok: false,
    lines: lines.length,
    headline: head[0] || `exit ${exitCode}`,
    first_errors: head,
  };
}

function buildReport({ taskId, checks, skipReason }) {
  const requiredFailed = checks.some((c) => c.required && c.status !== "passed" && c.status !== "skipped");
  const status = skipReason ? "skipped" : requiredFailed ? "failed" : checks.length ? "passed" : "incomplete";
  return {
    version: 1,
    task_id: taskId || "TASK-UNKNOWN",
    status,
    created_at: new Date().toISOString(),
    skip_reason: skipReason || null,
    checks,
    review: {
      agent: "pending",
      status: "pending",
      findings: [],
    },
  };
}

function validateReport(report) {
  const errors = [];
  if (!report || report.version !== 1) errors.push("report.version must be 1");
  if (!report.task_id) errors.push("report.task_id required");
  if (!["passed", "failed", "skipped", "incomplete"].includes(report.status)) errors.push("bad report.status");
  if (!Array.isArray(report.checks)) errors.push("report.checks must be array");
  if (report.status === "passed") {
    const required = (report.checks || []).filter((c) => c.required);
    if (!required.length) errors.push("passed report needs at least one required check");
    for (const check of required) {
      if (check.status !== "passed") errors.push(`required check ${check.gate} not passed`);
      if (!check.evidence) errors.push(`required check ${check.gate} missing evidence`);
    }
  }
  if (report.status === "skipped" && !report.skip_reason) errors.push("skipped report needs skip_reason");
  return errors;
}

function emitObs(rootDir, event) {
  try {
    const { appendEvent } = require("../observability/events.js");
    appendEvent(rootDir, event);
  } catch {
    // observability is optional; never fail gates on log errors
  }
}

function runGates({ root, projectGatesPath, taskId, only, skipReason, outDir }) {
  const rootDir = path.resolve(root || process.cwd());
  const gatesFile =
    projectGatesPath ||
    path.join(rootDir, ".cursor", "ai-dev-team", "quality", "gates.project.json");
  if (!fs.existsSync(gatesFile)) {
    throw new Error(`missing ${gatesFile}; run discover-gates first`);
  }
  const doc = loadJson(gatesFile);
  const shapeErrors = validateShape(doc, "gates.project.json");
  if (shapeErrors.length) throw new Error(shapeErrors.join("; "));

  const logsDir = path.join(outDir || path.join(rootDir, ".cursor", "ai-dev-team", "quality", "logs"), "runs");
  ensureDir(logsDir);
  const started = Date.now();

  let checks = [];
  if (skipReason) {
    checks = selectGates(doc, only).map((gate) => ({
      gate: gate.id,
      status: "skipped",
      exit_code: null,
      duration_ms: 0,
      command: gate.command,
      evidence: null,
      required: Boolean(gate.required),
      summary: { ok: true, lines: 0, headline: "skipped" },
    }));
  } else {
    for (const gate of selectGates(doc, only)) {
      checks.push(runOne(gate, rootDir, logsDir));
    }
  }

  const report = buildReport({ taskId, checks, skipReason });
  const reportPath = path.join(
    outDir || path.join(rootDir, ".cursor", "ai-dev-team", "quality"),
    "verification-report.json"
  );
  ensureDir(path.dirname(reportPath));
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2) + "\n", "utf8");
  const errors = validateReport(report);
  emitObs(rootDir, {
    type: "gates.run",
    task_id: taskId || "TASK-UNKNOWN",
    status: report.status,
    duration_ms: Date.now() - started,
    meta: { checks: checks.length, only: only || null },
  });
  return { report, reportPath, errors };
}

function main() {
  const args = process.argv.slice(2);
  const get = (flag) => {
    const i = args.indexOf(flag);
    return i >= 0 ? args[i + 1] : null;
  };
  const onlyRaw = get("--only");
  const only = onlyRaw ? onlyRaw.split(",").map((s) => s.trim()).filter(Boolean) : null;
  try {
    const result = runGates({
      root: get("--root") || process.cwd(),
      taskId: get("--task") || "TASK-UNKNOWN",
      only,
      skipReason: get("--skip-reason"),
      projectGatesPath: get("--gates"),
      outDir: get("--out"),
    });
    if (result.errors.length) {
      process.stderr.write(JSON.stringify({ ok: false, errors: result.errors, report: result.reportPath }, null, 2) + "\n");
      process.exit(1);
    }
    process.stdout.write(JSON.stringify({ ok: true, report: result.reportPath, status: result.report.status }, null, 2) + "\n");
    process.exit(result.report.status === "passed" || result.report.status === "skipped" ? 0 : 1);
  } catch (error) {
    process.stderr.write(JSON.stringify({ ok: false, error: String(error.message || error) }, null, 2) + "\n");
    process.exit(1);
  }
}

if (require.main === module) main();

module.exports = { runGates, buildReport, validateReport, selectGates, summarizeOutput };
