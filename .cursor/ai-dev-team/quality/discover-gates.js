#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const SCRIPT_MAP = [
  { id: "typecheck", when: "frontend", keys: ["typecheck", "type-check", "tsc"] },
  { id: "lint", when: "always", keys: ["lint", "eslint"] },
  { id: "test", when: "always", keys: ["test", "test:unit", "vitest", "jest"] },
  { id: "build", when: "always", keys: ["build"] },
];

function readPackage(dir) {
  const file = path.join(dir, "package.json");
  if (!fs.existsSync(file)) return null;
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function pickScript(scripts, keys) {
  if (!scripts) return null;
  for (const key of keys) {
    if (scripts[key]) return key;
  }
  return null;
}

function pmFor(dir) {
  if (fs.existsSync(path.join(dir, "pnpm-lock.yaml"))) return "pnpm";
  if (fs.existsSync(path.join(dir, "yarn.lock"))) return "yarn";
  if (fs.existsSync(path.join(dir, "bun.lockb"))) return "bun";
  return "npm";
}

function runCmd(pm, script) {
  if (pm === "pnpm") return `pnpm ${script}`;
  if (pm === "yarn") return `yarn ${script}`;
  if (pm === "bun") return `bun run ${script}`;
  return `npm run ${script}`;
}

function discoverFromPackage(dir, pkgName, root) {
  const pkg = readPackage(dir);
  if (!pkg) return { gates: [], discoveredFrom: [] };
  const pm = pmFor(dir);
  const gates = [];
  const discoveredFrom = [path.join(dir, "package.json")];
  const relCwd = path.relative(root, dir) || ".";
  for (const item of SCRIPT_MAP) {
    const script = pickScript(pkg.scripts || {}, item.keys);
    if (!script) {
      gates.push({
        id: item.id,
        command: null,
        required: false,
        when: item.when,
        kind: "command",
        timeout: 300000,
        cwd: relCwd.replace(/\\/g, "/"),
        package: pkgName || pkg.name || path.basename(dir),
        note: "no matching script in package.json",
      });
      continue;
    }
    gates.push({
      id: item.id,
      command: runCmd(pm, script),
      required: true,
      when: item.when,
      kind: "command",
      timeout: item.id === "test" || item.id === "build" ? 300000 : 120000,
      cwd: relCwd.replace(/\\/g, "/"),
      package: pkgName || pkg.name || path.basename(dir),
      note: `from scripts.${script}`,
    });
  }
  return { gates, discoveredFrom };
}

function discover(rootDir) {
  const root = path.resolve(rootDir || process.cwd());
  const rootDisc = discoverFromPackage(root, "root", root);
  const gates = [...rootDisc.gates];
  const discoveredFrom = [...rootDisc.discoveredFrom];

  // simple workspace packages/* if present
  for (const rel of ["packages", "apps"]) {
    const base = path.join(root, rel);
    if (!fs.existsSync(base) || !fs.statSync(base).isDirectory()) continue;
    for (const name of fs.readdirSync(base)) {
      const dir = path.join(base, name);
      if (!fs.statSync(dir).isDirectory()) continue;
      if (!fs.existsSync(path.join(dir, "package.json"))) continue;
      const disc = discoverFromPackage(dir, `${rel}/${name}`, root);
      for (const gate of disc.gates) {
        if (!gate.command) continue;
        gates.push({
          ...gate,
          id: `${gate.id}:${rel}/${name}`,
        });
      }
      discoveredFrom.push(...disc.discoveredFrom);
    }
  }

  return {
    version: 1,
    project: path.basename(root),
    source: "discovered",
    discoveredFrom: discoveredFrom.map((p) => path.relative(root, p) || p),
    gates,
  };
}

function writeProjectGates(rootDir, outFile) {
  const doc = discover(rootDir);
  const target = outFile || path.join(rootDir, ".cursor", "ai-dev-team", "quality", "gates.project.json");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, JSON.stringify(doc, null, 2) + "\n", "utf8");
  return { target, doc };
}

function main() {
  const root = process.argv[2] || process.cwd();
  const out = process.argv[3];
  const result = writeProjectGates(root, out);
  process.stdout.write(JSON.stringify({ ok: true, path: result.target, gates: result.doc.gates.length }, null, 2) + "\n");
}

if (require.main === module) main();

module.exports = { discover, writeProjectGates, SCRIPT_MAP };
