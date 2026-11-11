#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
const { writeProjectGates } = require("../quality/discover-gates.js");
const { validateShape } = require("../quality/validate-gates.js");
const { runGates, validateReport } = require("../quality/run-gates.js");
const { transitionTask } = require("../tasks/lifecycle.js");

const root = path.resolve(__dirname, "..", "..", "..");
const fixture = path.join(root, ".cursor", "ai-dev-team", "fixtures", "vite-ts");
const outQuality = path.join(fixture, ".cursor", "ai-dev-team", "quality");
let failed = 0;

function ok(name, cond, detail) {
  if (cond) process.stdout.write(`PASS ${name}\n`);
  else {
    failed += 1;
    process.stdout.write(`FAIL ${name}${detail ? ` — ${detail}` : ""}\n`);
  }
}

function main() {
  fs.rmSync(path.join(fixture, ".cursor"), { recursive: true, force: true });
  fs.mkdirSync(outQuality, { recursive: true });

  const discovered = writeProjectGates(fixture, path.join(outQuality, "gates.project.json"));
  ok("discover-gates", discovered.doc.gates.some((g) => g.id === "typecheck" && g.command), "typecheck not discovered");
  ok("no-invented-required-null", discovered.doc.gates.every((g) => !(g.required && !g.command)), "required null command");

  const shape = validateShape(discovered.doc, "fixture gates");
  ok("gates-shape", shape.length === 0, shape.join("; "));

  const ran = runGates({
    root: fixture,
    projectGatesPath: path.join(outQuality, "gates.project.json"),
    taskId: "TASK-E2E-1",
    outDir: outQuality,
  });
  ok("run-gates", ran.report.status === "passed", `status=${ran.report.status}`);
  ok("report-valid", validateReport(ran.report).length === 0, validateReport(ran.report).join("; "));
  ok("evidence-files", ran.report.checks.every((c) => c.evidence && fs.existsSync(path.join(fixture, c.evidence))), "missing logs");

  const task = {
    id: "TASK-E2E-1",
    status: "review",
    priority: "medium",
    title: "e2e",
    request: "fixture",
    interpretation: "harness",
    acceptance: ["gates pass"],
    assignees: ["frontend-developer"],
    history: [],
    budget: { effortCap: "low", maxAttempts: 2 },
    allowed_files: ["src/main.ts"],
    contract: {
      goal: "e2e",
      context: "fixture",
      inputs: ["src/main.ts"],
      constraints: [],
      allowed_files: ["src/main.ts"],
      forbidden_files: [],
      acceptance: ["gates pass"],
      report_format: "writer-brief",
      risks: [],
      unfinished: [],
      evidence: [],
    },
  };

  const toVerified = transitionTask(task, "verified", {
    verificationReportPath: ran.reportPath,
    by: "adapt-e2e",
  });
  ok("lifecycle-verified", toVerified.ok, (toVerified.errors || []).join("; "));

  const toCompleted = transitionTask(toVerified.task, "completed", { by: "adapt-e2e" });
  ok("lifecycle-completed", toCompleted.ok, (toCompleted.errors || []).join("; "));

  const illegal = transitionTask(task, "completed", { by: "adapt-e2e" });
  ok("lifecycle-blocks-skip-verified", !illegal.ok, "completed without verified should fail");

  if (failed) {
    process.stdout.write(`\n${failed} failed\n`);
    process.exit(1);
  }
  process.stdout.write("\nadapt-e2e passed\n");
}

main();
