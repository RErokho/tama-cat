#!/usr/bin/env node
"use strict";

/**
 * Battle smoke against a real project root.
 * Usage: node smoke-project.js <project-root>
 * Discovers gates, validates shape, optionally runs required gates with --run.
 * Does not adapt rules or edit the project (unless --run executes project scripts).
 */

const fs = require("fs");
const path = require("path");
const { discover } = require("../quality/discover-gates.js");
const { validateShape } = require("../quality/validate-gates.js");
const { runGates } = require("../quality/run-gates.js");

function main() {
  const args = process.argv.slice(2);
  const doRun = args.includes("--run");
  const target = path.resolve(args.find((a) => !a.startsWith("-")) || "");
  if (!target || !fs.existsSync(target)) {
    process.stderr.write("usage: node smoke-project.js <project-root> [--run]\n");
    process.exit(2);
  }

  const outDir = path.join(target, ".cursor", "ai-dev-team", "quality");
  fs.mkdirSync(outDir, { recursive: true });
  const doc = discover(target);
  const shape = validateShape(doc, "smoke");
  const out = path.join(outDir, "gates.project.smoke.json");
  fs.writeFileSync(out, JSON.stringify(doc, null, 2) + "\n", "utf8");

  const summary = {
    ok: shape.length === 0,
    root: target,
    gates: doc.gates.length,
    with_command: doc.gates.filter((g) => g.command).length,
    required: doc.gates.filter((g) => g.required && g.command).length,
    shape_errors: shape,
    gates_path: out,
  };

  if (!doRun) {
    process.stdout.write(JSON.stringify(summary, null, 2) + "\n");
    process.exit(summary.ok ? 0 : 1);
  }

  // write as gates.project.json for runner, then restore? Use explicit path.
  const ran = runGates({
    root: target,
    projectGatesPath: out,
    taskId: "TASK-SMOKE",
    outDir,
  });
  summary.run = { status: ran.report.status, report: ran.reportPath, checks: ran.report.checks.length };
  summary.ok = summary.ok && (ran.report.status === "passed" || ran.report.status === "skipped");
  process.stdout.write(JSON.stringify(summary, null, 2) + "\n");
  process.exit(summary.ok ? 0 : 1);
}

main();
