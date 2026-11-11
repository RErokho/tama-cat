#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { readEvents, obsDir } = require("./events.js");

function bump(map, key) {
  map[key] = (map[key] || 0) + 1;
}

function loadTaskStats(repoRoot) {
  const dir = path.join(repoRoot, ".cursor", "ai-dev-team", "tasks");
  const stats = {
    tasks: 0,
    by_status: {},
    verified_or_completed: 0,
    failed: 0,
    history_transitions: 0,
    avg_history_len: 0,
  };
  if (!fs.existsSync(dir)) return stats;
  const files = fs.readdirSync(dir).filter((n) => /^TASK-.*\.json$/i.test(n));
  let histSum = 0;
  for (const name of files) {
    let task;
    try {
      task = JSON.parse(fs.readFileSync(path.join(dir, name), "utf8"));
    } catch {
      continue;
    }
    stats.tasks += 1;
    bump(stats.by_status, task.status || "unknown");
    if (task.status === "verified" || task.status === "completed") stats.verified_or_completed += 1;
    if (task.status === "failed") stats.failed += 1;
    const hist = Array.isArray(task.history) ? task.history.length : 0;
    histSum += hist;
    stats.history_transitions += hist;
  }
  stats.avg_history_len = stats.tasks ? Math.round((histSum / stats.tasks) * 10) / 10 : 0;
  return stats;
}

function summarize(events, repoRoot) {
  const byType = {};
  const bySkill = {};
  const byAgent = {};
  const byTask = {};
  const gateRuns = [];
  const gateDurations = [];
  let handoffs = 0;
  let ownershipDenies = 0;
  let gatePassed = 0;
  let gateFailed = 0;

  for (const e of events) {
    bump(byType, e.type || "unknown");
    if (e.skill) bump(bySkill, e.skill);
    if (e.agent) bump(byAgent, e.agent);
    if (e.task_id) bump(byTask, e.task_id);
    if (e.type === "agent.handoff") handoffs += 1;
    if (e.type === "ownership.deny") ownershipDenies += 1;
    if (e.type === "gates.run") {
      gateRuns.push({
        task_id: e.task_id || null,
        status: e.status || null,
        duration_ms: e.duration_ms || null,
        ts: e.ts,
      });
      if (e.status === "passed" || e.status === "skipped") gatePassed += 1;
      if (e.status === "failed" || e.status === "incomplete") gateFailed += 1;
      if (typeof e.duration_ms === "number") gateDurations.push(e.duration_ms);
    }
  }

  const top = (obj, n) =>
    Object.entries(obj)
      .sort((a, b) => b[1] - a[1])
      .slice(0, n)
      .map(([name, count]) => ({ name, count }));

  const avg = (arr) => (arr.length ? Math.round(arr.reduce((a, b) => a + b, 0) / arr.length) : null);
  const tasks = loadTaskStats(path.resolve(repoRoot || process.cwd()));

  const qos = {
    gate_pass_rate:
      gatePassed + gateFailed > 0 ? Math.round((gatePassed / (gatePassed + gateFailed)) * 1000) / 1000 : null,
    gate_runs: gatePassed + gateFailed,
    gate_avg_duration_ms: avg(gateDurations),
    handoffs,
    ownership_denies: ownershipDenies,
    tasks_open_ish: (tasks.by_status.in_progress || 0) + (tasks.by_status.review || 0) + (tasks.by_status.testing || 0),
    tasks_verified_or_completed: tasks.verified_or_completed,
    tasks_failed: tasks.failed,
    avg_task_history_len: tasks.avg_history_len,
    note: "No billed token meter from Cursor; these are proxy QoS signals.",
  };

  return {
    version: 2,
    generated_at: new Date().toISOString(),
    event_count: events.length,
    by_type: byType,
    top_skills: top(bySkill, 20),
    top_agents: top(byAgent, 20),
    tasks_touched: Object.keys(byTask).length,
    task_stats: tasks,
    qos,
    recent_gate_runs: gateRuns.slice(-10),
  };
}

function writeReport(repoRoot) {
  const root = path.resolve(repoRoot);
  const events = readEvents(root);
  const report = summarize(events, root);
  const dir = obsDir(root);
  fs.mkdirSync(dir, { recursive: true });
  const out = path.join(dir, "observability-report.json");
  fs.writeFileSync(out, JSON.stringify(report, null, 2) + "\n", "utf8");
  return { ok: true, path: out, report };
}

function main() {
  const root = process.argv.includes("--root")
    ? process.argv[process.argv.indexOf("--root") + 1]
    : process.argv[2] || process.cwd();
  const result = writeReport(root);
  process.stdout.write(
    JSON.stringify(
      {
        ok: true,
        path: result.path,
        event_count: result.report.event_count,
        qos: result.report.qos,
      },
      null,
      2
    ) + "\n"
  );
}

if (require.main === module) main();

module.exports = { summarize, writeReport, loadTaskStats };
