#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname);

function loadJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function validateShape(doc, label) {
  const errors = [];
  if (!doc || typeof doc !== "object") return [`${label}: must be object`];
  if (doc.version !== 1) errors.push(`${label}: version must be 1`);
  if (!Array.isArray(doc.gates)) errors.push(`${label}: gates must be array`);
  const ids = new Set();
  for (const gate of doc.gates || []) {
    if (!gate || typeof gate !== "object") {
      errors.push(`${label}: gate must be object`);
      continue;
    }
    if (!gate.id || typeof gate.id !== "string") errors.push(`${label}: gate.id required`);
    if (ids.has(gate.id)) errors.push(`${label}: duplicate gate id ${gate.id}`);
    ids.add(gate.id);
    if (typeof gate.required !== "boolean") errors.push(`${label}:${gate.id} required must be boolean`);
    if (!gate.when) errors.push(`${label}:${gate.id} when required`);
    if (gate.kind === "command" || (gate.command !== undefined && gate.command !== null)) {
      if (typeof gate.command !== "string" || !gate.command.trim()) {
        errors.push(`${label}:${gate.id} command gates need non-empty command`);
      }
    }
    if (gate.command === null && gate.required && gate.kind === "command") {
      errors.push(`${label}:${gate.id} required command gate cannot have null command`);
    }
    if (gate.timeout !== undefined && (!Number.isInteger(gate.timeout) || gate.timeout < 1000)) {
      errors.push(`${label}:${gate.id} timeout must be integer >= 1000`);
    }
  }
  return errors;
}

function validateProjectAgainstInvented(doc) {
  const errors = [];
  for (const gate of doc.gates || []) {
    if (gate.kind === "command" && gate.required && !gate.command) {
      errors.push(`${gate.id}: required command missing`);
    }
    if (gate.note && /invent|guess|assume/i.test(gate.note) && gate.required) {
      errors.push(`${gate.id}: required gate must not be invented`);
    }
  }
  return errors;
}

function main() {
  const corePath = path.join(ROOT, "gates.core.json");
  const projectPath = path.join(ROOT, "gates.project.json");
  const errors = [];
  errors.push(...validateShape(loadJson(corePath), "gates.core.json"));
  if (fs.existsSync(projectPath)) {
    const project = loadJson(projectPath);
    errors.push(...validateShape(project, "gates.project.json"));
    errors.push(...validateProjectAgainstInvented(project));
  }
  if (errors.length) {
    process.stderr.write(JSON.stringify({ ok: false, errors }, null, 2) + "\n");
    process.exit(1);
  }
  process.stdout.write(JSON.stringify({ ok: true, core: corePath, project: projectPath }, null, 2) + "\n");
}

if (require.main === module) main();

module.exports = { validateShape, validateProjectAgainstInvented };
