#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const TYPES = new Set([
  "session.start",
  "skill.used",
  "agent.handoff",
  "task.transition",
  "gates.run",
  "dispatch.plan",
  "ownership.deny",
  "note",
]);

function obsDir(repoRoot) {
  return path.join(path.resolve(repoRoot), ".cursor", "ai-dev-team", "observability");
}

function eventsPath(repoRoot) {
  return path.join(obsDir(repoRoot), "events.jsonl");
}

function normalizeEvent(input) {
  const event = input && typeof input === "object" ? { ...input } : {};
  const type = String(event.type || "note");
  if (!TYPES.has(type)) {
    return { ok: false, error: `unknown type ${type}; allowed: ${[...TYPES].join(", ")}` };
  }
  const out = {
    ts: event.ts || new Date().toISOString(),
    type,
  };
  if (event.skill) out.skill = String(event.skill);
  if (event.agent) out.agent = String(event.agent);
  if (event.task_id) out.task_id = String(event.task_id);
  if (event.level) out.level = String(event.level);
  if (event.status) out.status = String(event.status);
  if (event.duration_ms != null && Number.isFinite(Number(event.duration_ms))) {
    out.duration_ms = Number(event.duration_ms);
  }
  if (event.meta && typeof event.meta === "object" && !Array.isArray(event.meta)) {
    out.meta = event.meta;
  }
  if (event.note) out.note = String(event.note).slice(0, 500);
  return { ok: true, event: out };
}

function appendEvent(repoRoot, input) {
  const normalized = normalizeEvent(input);
  if (!normalized.ok) return normalized;
  const dir = obsDir(repoRoot);
  fs.mkdirSync(dir, { recursive: true });
  const file = eventsPath(repoRoot);
  fs.appendFileSync(file, JSON.stringify(normalized.event) + "\n", "utf8");
  return { ok: true, path: file, event: normalized.event };
}

function readEvents(repoRoot, limit) {
  const file = eventsPath(repoRoot);
  if (!fs.existsSync(file)) return [];
  const lines = fs.readFileSync(file, "utf8").split(/\r?\n/).filter(Boolean);
  const slice = limit && limit > 0 ? lines.slice(-limit) : lines;
  const events = [];
  for (const line of slice) {
    try {
      events.push(JSON.parse(line));
    } catch {
      // skip corrupt lines
    }
  }
  return events;
}

function main() {
  const args = process.argv.slice(2);
  const root = args.includes("--root") ? args[args.indexOf("--root") + 1] : process.cwd();
  if (args[0] === "append") {
    let payload = {};
    const jsonIdx = args.indexOf("--json");
    if (jsonIdx >= 0) {
      payload = JSON.parse(args[jsonIdx + 1]);
    } else {
      const get = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : null);
      payload = {
        type: get("--type") || "note",
        skill: get("--skill"),
        agent: get("--agent"),
        task_id: get("--task"),
        level: get("--level"),
        status: get("--status"),
        note: get("--note"),
      };
    }
    const result = appendEvent(root, payload);
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    process.exit(result.ok ? 0 : 1);
  }
  if (args[0] === "tail") {
    const n = args.includes("--n") ? Number(args[args.indexOf("--n") + 1]) : 20;
    process.stdout.write(JSON.stringify({ ok: true, events: readEvents(root, n) }, null, 2) + "\n");
    return;
  }
  process.stderr.write(
    "usage: node events.js append --type skill.used --skill NAME [--task ID] [--json '{...}']\n" +
      "       node events.js tail [--n 20] [--root .]\n"
  );
  process.exit(2);
}

if (require.main === module) main();

module.exports = { TYPES, appendEvent, readEvents, normalizeEvent, eventsPath, obsDir };
