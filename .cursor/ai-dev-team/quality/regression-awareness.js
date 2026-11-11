#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

function gitDiffFiles(repoRoot, base) {
  const args = base ? ["diff", "--name-only", base] : ["diff", "--name-only", "HEAD"];
  const staged = spawnSync("git", ["diff", "--name-only", "--cached"], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  const unstaged = spawnSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  const set = new Set();
  for (const text of [staged.stdout || "", unstaged.stdout || ""]) {
    for (const line of text.split(/\r?\n/)) {
      if (line.trim()) set.add(line.trim().replace(/\\/g, "/"));
    }
  }
  return [...set];
}

function packageForFile(file, repoRoot) {
  const parts = file.split("/");
  if ((parts[0] === "packages" || parts[0] === "apps") && parts[1]) {
    return `${parts[0]}/${parts[1]}`;
  }
  if (fs.existsSync(path.join(repoRoot, "package.json"))) return "root";
  return "unknown";
}

function isPublicSurface(file) {
  return /(^|\/)(index\.(ts|tsx|js|jsx)|types?\.ts|.*\.d\.ts|api\/.*|shared\/.*)$/i.test(file);
}

function loadProjectGates(repoRoot) {
  const file = path.join(repoRoot, ".cursor", "ai-dev-team", "quality", "gates.project.json");
  if (!fs.existsSync(file)) return null;
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function recommendGates(files, gatesDoc) {
  const packages = new Set(files.map((f) => packageForFile(f, process.cwd())));
  const publicHit = files.some(isPublicSurface);
  const ids = new Set();
  if (!gatesDoc) {
    if (publicHit) ["typecheck", "lint", "test", "build"].forEach((id) => ids.add(id));
    else ["typecheck", "lint"].forEach((id) => ids.add(id));
    return { packages: [...packages], gateIds: [...ids], publicHit };
  }
  for (const gate of gatesDoc.gates || []) {
    if (!gate.command) continue;
    const pkg = gate.package || "root";
    if (packages.has(pkg) || packages.has("root") && pkg === "root") {
      if (gate.id.startsWith("typecheck") || gate.id === "typecheck") ids.add(gate.id);
      if (gate.id.startsWith("lint") || gate.id === "lint") ids.add(gate.id);
      if (publicHit && (gate.id.startsWith("test") || gate.id.startsWith("build") || gate.id === "test" || gate.id === "build")) {
        ids.add(gate.id);
      }
    }
    // package-scoped ids like typecheck:packages/ui
    for (const p of packages) {
      if (gate.id.endsWith(`:${p}`)) ids.add(gate.id);
    }
  }
  return { packages: [...packages], gateIds: [...ids], publicHit };
}

function analyze(repoRoot, base) {
  const files = gitDiffFiles(repoRoot, base);
  const gatesDoc = loadProjectGates(repoRoot);
  const rec = recommendGates(
    files,
    gatesDoc && {
      ...gatesDoc,
      // packageForFile uses cwd; keep analysis rooted
    }
  );
  // recompute packages with correct root
  const packages = [...new Set(files.map((f) => packageForFile(f, repoRoot)))];
  const publicHit = files.some(isPublicSurface);
  const gateIds = recommendGates(files, gatesDoc).gateIds;
  return {
    version: 1,
    files,
    packages,
    public_surface_changed: publicHit,
    recommended_gates: gateIds,
    run_hint:
      gateIds.length > 0
        ? `node .cursor/ai-dev-team/quality/run-gates.js --task TASK-ID --only ${gateIds.join(",")}`
        : "no command gates matched; use quality-gates / browser as needed",
  };
}

function main() {
  const repoRoot = process.argv[2] || process.cwd();
  const base = process.argv.includes("--base")
    ? process.argv[process.argv.indexOf("--base") + 1]
    : null;
  const report = analyze(path.resolve(repoRoot), base);
  const out = path.join(path.resolve(repoRoot), ".cursor", "ai-dev-team", "quality", "regression-report.json");
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, JSON.stringify(report, null, 2) + "\n", "utf8");
  process.stdout.write(JSON.stringify({ ok: true, path: out, report }, null, 2) + "\n");
}

if (require.main === module) main();

module.exports = { analyze, gitDiffFiles, packageForFile, recommendGates, isPublicSurface };
