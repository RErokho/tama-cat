#!/usr/bin/env node
"use strict";

/**
 * Stronger kit harness: next-mono discover → contract validate → dispatch →
 * run-gates → lifecycle verified/completed. Still a fixture, not a customer repo.
 */

const fs = require("fs");
const path = require("path");
const { writeProjectGates } = require("../quality/discover-gates.js");
const { validateShape } = require("../quality/validate-gates.js");
const { runGates, validateReport } = require("../quality/run-gates.js");
const { validateTask } = require("../tasks/validate.js");
const { planDispatch } = require("../execution/dispatch.js");
const { transitionTask } = require("../tasks/lifecycle.js");
const { decide: decideOwnership } = require("../../hooks/guard-allowed-files.js");

const root = path.resolve(__dirname, "..", "..", "..");
const fixture = path.join(root, ".cursor", "ai-dev-team", "fixtures", "next-mono");
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
  ok(
    "mono-gates",
    discovered.doc.gates.some((g) => String(g.id).includes("packages/ui") && g.command),
    "missing package gates"
  );
  ok("gates-shape", validateShape(discovered.doc, "mono").length === 0);

  const task = {
    id: "TASK-HARNESS-1",
    status: "in_progress",
    priority: "medium",
    title: "harness",
    request: "e2e",
    interpretation: "e2e",
    acceptance: ["gates pass"],
    assignees: ["frontend-developer", "backend-developer"],
    history: [],
    budget: { effortCap: "medium", maxAttempts: 2 },
    execution: { mode: "shared", isolation: "disabled" },
    contract: {
      goal: "harness",
      context: "fixture",
      inputs: ["fixtures/next-mono"],
      constraints: [],
      allowed_files: ["apps/web/package.json"],
      forbidden_files: [],
      acceptance: ["gates pass"],
      report_format: "writer-brief",
      risks: [],
      unfinished: [],
      evidence: [],
      max_reply: "short",
    },
    packets: [
      { id: "PKT-1", assignee: "frontend-developer", depends_on: [], allowed_files: ["apps/web/package.json"] },
      { id: "PKT-2", assignee: "backend-developer", depends_on: ["PKT-1"], allowed_files: ["packages/ui/package.json"] },
    ],
  };
  ok("task-valid", validateTask(task).length === 0, validateTask(task).join("; "));

  const plan = planDispatch(task, []);
  ok("dispatch-serial-deps", plan.ready.includes("PKT-1") && !plan.ready.includes("PKT-2"));

  const tasksDir = path.join(root, ".cursor", "ai-dev-team", "tasks");
  fs.mkdirSync(tasksDir, { recursive: true });
  const tmpTask = path.join(tasksDir, "TASK-HARNESS-TMP.json");
  fs.writeFileSync(tmpTask, JSON.stringify(task, null, 2));
  try {
    const blocked = decideOwnership(
      { tool_input: { file_path: path.join(root, "README.md") } },
      root
    );
    const allowed = decideOwnership(
      { tool_input: { file_path: path.join(root, "apps/web/package.json") } },
      root
    );
    ok("ownership-deny-outside", blocked.permission === "deny", JSON.stringify(blocked));
    ok("ownership-allow-inside", allowed.permission === "allow", JSON.stringify(allowed));
  } finally {
    try {
      fs.unlinkSync(tmpTask);
    } catch {
      /* ignore */
    }
  }

  const only = discovered.doc.gates.filter((g) => g.command && g.required).map((g) => g.id).slice(0, 4);
  const ran = runGates({
    root: fixture,
    projectGatesPath: path.join(outQuality, "gates.project.json"),
    taskId: "TASK-HARNESS-1",
    outDir: outQuality,
    only: only.length ? only : null,
  });
  ok("run-gates", ran.report.status === "passed", `status=${ran.report.status}`);
  ok("report-valid", validateReport(ran.report).length === 0);

  task.status = "review";
  const v = transitionTask(task, "verified", {
    verificationReportPath: ran.reportPath,
    by: "harness-e2e",
  });
  ok("verified", v.ok, (v.errors || []).join("; "));
  const c = transitionTask(v.task, "completed", { by: "harness-e2e" });
  ok("completed", c.ok, (c.errors || []).join("; "));

  if (failed) {
    process.stdout.write(`\n${failed} failed\n`);
    process.exit(1);
  }
  process.stdout.write("\nharness-e2e passed\n");
}

main();
