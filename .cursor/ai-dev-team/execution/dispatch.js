#!/usr/bin/env node
"use strict";

const fs = require("fs");

/**
 * Dependency-aware packet dispatch.
 * execution.mode: shared | worktree
 * execution.isolation: required | preferred | disabled
 */
function normalizeExecution(task) {
  const exec = (task && task.execution) || {};
  return {
    mode: exec.mode === "worktree" ? "worktree" : "shared",
    isolation: ["required", "preferred", "disabled"].includes(exec.isolation)
      ? exec.isolation
      : exec.mode === "worktree"
        ? "preferred"
        : "disabled",
  };
}

function packetMap(packets) {
  const map = new Map();
  for (const packet of packets || []) {
    if (packet && packet.id) map.set(packet.id, packet);
  }
  return map;
}

function isDone(packet, doneSet) {
  if (!packet) return false;
  if (doneSet.has(packet.id)) return true;
  return ["done", "completed", "verified"].includes(String(packet.status || "").toLowerCase());
}

function readyPackets(task, doneIds) {
  const doneSet = new Set(doneIds || []);
  const packets = Array.isArray(task.packets) ? task.packets : [];
  const map = packetMap(packets);
  const ready = [];
  const blocked = [];
  for (const packet of packets) {
    const deps = Array.isArray(packet.depends_on) ? packet.depends_on : [];
    const missing = deps.filter((id) => !isDone(map.get(id), doneSet) && !doneSet.has(id));
    if (missing.length) blocked.push({ id: packet.id, waiting_on: missing });
    else if (!isDone(packet, doneSet)) ready.push(packet);
  }
  return { ready, blocked, execution: normalizeExecution(task) };
}

function sameWaveConflict(ready) {
  // ready packets with overlapping allows cannot run in parallel even if deps clear
  const conflicts = [];
  for (let i = 0; i < ready.length; i++) {
    for (let j = i + 1; j < ready.length; j++) {
      const a = ready[i].allowed_files || [];
      const b = ready[j].allowed_files || [];
      for (const x of a) {
        for (const y of b) {
          if (x === y) conflicts.push([ready[i].id, ready[j].id, x]);
        }
      }
    }
  }
  return conflicts;
}

function planDispatch(task, doneIds) {
  const { ready, blocked, execution } = readyPackets(task, doneIds);
  const conflicts = sameWaveConflict(ready);
  const parallel = ready.filter((packet) => !conflicts.some((c) => c[0] === packet.id || c[1] === packet.id));
  const serial = ready.filter((packet) => !parallel.includes(packet));
  return {
    execution,
    ready: ready.map((p) => p.id),
    blocked,
    parallel: parallel.map((p) => p.id),
    serial: serial.map((p) => p.id),
    conflicts,
    useWorktree:
      execution.mode === "worktree" ||
      (execution.isolation === "required" && parallel.length > 1) ||
      (execution.isolation === "preferred" && parallel.length > 1),
  };
}

function main() {
  const file = process.argv[2];
  if (!file) {
    process.stderr.write("usage: node dispatch.js <TASK.json> [doneId,doneId]\n");
    process.exit(2);
  }
  const task = JSON.parse(fs.readFileSync(file, "utf8"));
  const done = (process.argv[3] || "").split(",").filter(Boolean);
  const plan = planDispatch(task, done);
  try {
    const { appendEvent } = require("../observability/events.js");
    appendEvent(process.cwd(), {
      type: "dispatch.plan",
      task_id: task.id || null,
      meta: {
        parallel: plan.parallel,
        serial: plan.serial,
        useWorktree: plan.useWorktree,
      },
    });
  } catch {
    // optional
  }
  process.stdout.write(JSON.stringify(plan, null, 2) + "\n");
}

if (require.main === module) main();

module.exports = { normalizeExecution, readyPackets, planDispatch, sameWaveConflict };
