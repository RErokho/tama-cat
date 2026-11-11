#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const STATUSES = [
  "inbox",
  "analyzing",
  "clarification",
  "planned",
  "approved",
  "ready",
  "in_progress",
  "blocked",
  "failed",
  "testing",
  "review",
  "rework",
  "verified",
  "completed",
  "cancelled",
];

const PRIORITIES = ["low", "medium", "high", "critical"];
const EFFORTS = ["low", "medium", "high"];
const WRITERS = new Set([
  "frontend-developer",
  "backend-developer",
  "fullstack-developer",
  "mobile-developer",
  "desktop-developer",
  "refactoring-specialist",
  "debugging-specialist",
  "technical-writer",
]);

const CONTRACT_ACTIVE = new Set([
  "planned",
  "approved",
  "ready",
  "in_progress",
  "blocked",
  "failed",
  "testing",
  "review",
  "rework",
  "verified",
]);

function assigneeId(item) {
  if (typeof item === "string") return item;
  if (item && typeof item === "object") return String(item.id || item.role || item.cursorAgent || item.assignee || "");
  return "";
}

function normPath(p) {
  return String(p || "").replace(/\\/g, "/").replace(/\/+$/, "");
}

function entriesOverlap(a, b) {
  const x = normPath(a);
  const y = normPath(b);
  if (!x || !y) return false;
  if (x === y) return true;
  if (x.includes("*") || y.includes("*")) {
    const strip = (s) => s.replace(/\*\*/g, "").replace(/\*/g, "").replace(/\/+$/, "");
    const xs = strip(x);
    const ys = strip(y);
    return Boolean(xs && ys && (xs === ys || xs.startsWith(ys + "/") || ys.startsWith(xs + "/")));
  }
  return x.startsWith(y + "/") || y.startsWith(x + "/");
}

function findOverlaps(lists) {
  const errors = [];
  for (let i = 0; i < lists.length; i++) {
    for (let j = i + 1; j < lists.length; j++) {
      for (const a of lists[i].paths) {
        for (const b of lists[j].paths) {
          if (entriesOverlap(a, b)) {
            errors.push(`allowed_files overlap between ${lists[i].id} and ${lists[j].id}: ${a} ↔ ${b}`);
          }
        }
      }
    }
  }
  return errors;
}

function isStringList(value) {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function validateContract(contract, label) {
  const errors = [];
  const id = label || "contract";
  if (!contract || typeof contract !== "object" || Array.isArray(contract)) {
    return [`${id} must be an object`];
  }
  for (const key of ["goal", "context", "report_format"]) {
    if (typeof contract[key] !== "string" || !contract[key].trim()) {
      errors.push(`${id}.${key} must be a non-empty string`);
    }
  }
  for (const key of ["inputs", "constraints", "acceptance", "risks", "unfinished", "evidence"]) {
    if (!isStringList(contract[key])) {
      errors.push(`${id}.${key} must be a string list`);
    }
  }
  if (!isStringList(contract.allowed_files)) {
    errors.push(`${id}.allowed_files must be a string list`);
  }
  if (!isStringList(contract.forbidden_files)) {
    errors.push(`${id}.forbidden_files must be a string list`);
  }
  if (Array.isArray(contract.acceptance) && contract.acceptance.length === 0) {
    errors.push(`${id}.acceptance must not be empty`);
  }
  if (contract.max_reply !== undefined && !["short", "normal"].includes(contract.max_reply)) {
    errors.push(`${id}.max_reply must be short or normal`);
  }
  const rf = String(contract.report_format || "").trim();
  if (/^goal\s*\/\s*result$/i.test(rf) || /^goal:\s*$/im.test(rf)) {
    errors.push(
      `${id}.report_format must use a brief preset (writer-brief|reviewer-brief|analyst-brief|gate-brief) or explicit short headings — not Goal/Result`
    );
  }
  return errors;
}

function validatePacket(packet, index) {
  const errors = [];
  const id = packet && packet.id ? packet.id : `packets[${index}]`;
  if (!packet || typeof packet !== "object") return [`${id} must be an object`];
  if (typeof packet.id !== "string" || !packet.id.trim()) errors.push(`${id} missing id`);
  if (!Array.isArray(packet.allowed_files) || packet.allowed_files.length === 0) {
    errors.push(`${id} requires non-empty allowed_files`);
  } else if (packet.allowed_files.some((item) => typeof item !== "string")) {
    errors.push(`${id} allowed_files must be strings`);
  }
  if (packet.forbidden_files !== undefined) {
    if (!isStringList(packet.forbidden_files)) errors.push(`${id} forbidden_files must be a string list`);
  }
  if (packet.depends_on !== undefined && !Array.isArray(packet.depends_on)) {
    errors.push(`${id} depends_on must be a list`);
  }
  if (packet.kind !== undefined && typeof packet.kind !== "string") {
    errors.push(`${id} kind must be a string`);
  }
  return errors;
}

function validateTask(task) {
  const errors = [];
  if (!task || typeof task !== "object" || Array.isArray(task)) {
    return ["task must be an object"];
  }
  if (typeof task.id !== "string" || !/^TASK-[A-Za-z0-9-]+$/.test(task.id)) errors.push("id must look like TASK-...");
  if (!STATUSES.includes(task.status)) errors.push("unknown status");
  if (!PRIORITIES.includes(task.priority)) errors.push("unknown priority");
  if (typeof task.title !== "string" || !task.title.trim()) errors.push("empty title");
  if (typeof task.request !== "string" || !task.request.trim()) errors.push("empty request");
  if (typeof task.interpretation !== "string") errors.push("missing interpretation");
  if (!Array.isArray(task.acceptance) || task.acceptance.length === 0) errors.push("missing acceptance");
  if (!Array.isArray(task.assignees)) errors.push("assignees must be a list");
  if (!Array.isArray(task.history)) errors.push("history must be a list");
  if (!task.budget || !EFFORTS.includes(task.budget.effortCap)) errors.push("budget.effortCap must be low, medium, or high");
  if (!task.budget || task.budget.maxAttempts !== 2) errors.push("budget.maxAttempts must be 2");
  if (task.models && task.models.some((item) => /fast|xhigh|extra-high|500k|grok-4\.6/i.test(JSON.stringify(item)))) {
    errors.push("models must not use fast, extra high, or Grok 4.6");
  }
  if (task.allowed_files !== undefined && !isStringList(task.allowed_files)) {
    errors.push("allowed_files must be a string list");
  }
  if (task.forbidden_files !== undefined && !isStringList(task.forbidden_files)) {
    errors.push("forbidden_files must be a string list");
  }

  const writers = Array.isArray(task.assignees)
    ? task.assignees.map(assigneeId).filter((id) => WRITERS.has(id))
    : [];
  if (writers.length && (!Array.isArray(task.allowed_files) || task.allowed_files.length === 0) && !Array.isArray(task.packets)) {
    errors.push("writing assignees require non-empty allowed_files or packets");
  }

  if (CONTRACT_ACTIVE.has(task.status)) {
    if (!task.contract) {
      errors.push("active task requires contract object");
    } else {
      errors.push(...validateContract(task.contract, "contract"));
      if (
        writers.length &&
        Array.isArray(task.contract.allowed_files) &&
        task.contract.allowed_files.length === 0 &&
        !Array.isArray(task.packets)
      ) {
        errors.push("contract.allowed_files must be non-empty for writers");
      }
    }
  } else if (task.contract !== undefined) {
    errors.push(...validateContract(task.contract, "contract"));
  }

  if (task.packets !== undefined) {
    if (!Array.isArray(task.packets)) {
      errors.push("packets must be a list");
    } else {
      task.packets.forEach((packet, index) => {
        errors.push(...validatePacket(packet, index));
      });
      const lists = task.packets
        .filter((packet) => packet && Array.isArray(packet.allowed_files))
        .map((packet) => ({ id: packet.id || "packet", paths: packet.allowed_files }));
      errors.push(...findOverlaps(lists));
    }
  }

  if (task.execution !== undefined) {
    if (!task.execution || typeof task.execution !== "object" || Array.isArray(task.execution)) {
      errors.push("execution must be an object");
    } else {
      if (task.execution.mode !== undefined && !["shared", "worktree"].includes(task.execution.mode)) {
        errors.push("execution.mode must be shared or worktree");
      }
      if (
        task.execution.isolation !== undefined &&
        !["disabled", "preferred", "required"].includes(task.execution.isolation)
      ) {
        errors.push("execution.isolation must be disabled, preferred, or required");
      }
    }
  }

  return errors;
}

function validateDir(dir) {
  if (!fs.existsSync(dir)) return [];
  const problems = [];
  for (const name of fs.readdirSync(dir).filter((item) => item.endsWith(".json"))) {
    const full = path.join(dir, name);
    let task;
    try {
      task = JSON.parse(fs.readFileSync(full, "utf8"));
    } catch (error) {
      problems.push({ file: name, errors: [error.message] });
      continue;
    }
    const errors = validateTask(task);
    if (errors.length) problems.push({ file: name, errors });
  }
  return problems;
}

function main() {
  const dir = process.argv[2] || path.resolve(__dirname);
  const problems = validateDir(dir);
  if (problems.length) {
    process.stderr.write(JSON.stringify(problems, null, 2) + "\n");
    process.exit(1);
  }
  process.stdout.write(JSON.stringify({ ok: true, dir }) + "\n");
}

if (require.main === module) main();

module.exports = {
  validateTask,
  validateDir,
  validatePacket,
  validateContract,
  findOverlaps,
  STATUSES,
  WRITERS,
  CONTRACT_ACTIVE,
};
