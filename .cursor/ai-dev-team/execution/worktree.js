#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

function run(cmd, args, cwd) {
  const result = spawnSync(cmd, args, { cwd, encoding: "utf8", shell: process.platform === "win32" });
  return {
    ok: result.status === 0,
    status: result.status,
    stdout: result.stdout || "",
    stderr: result.stderr || "",
  };
}

function defaultWorktreeRoot(repoRoot) {
  return path.join(repoRoot, ".cursor", "ai-dev-team", "worktrees");
}

function sanitize(id) {
  return String(id || "packet").replace(/[^\w.-]+/g, "-");
}

function ensureGitRepo(repoRoot) {
  const check = run("git", ["rev-parse", "--is-inside-work-tree"], repoRoot);
  return check.ok && /true/.test(check.stdout);
}

function createPacketWorktree({ repoRoot, packetId, branchBase }) {
  if (!ensureGitRepo(repoRoot)) {
    return { ok: false, error: "not a git repository" };
  }
  const base = branchBase || "HEAD";
  const root = defaultWorktreeRoot(repoRoot);
  fs.mkdirSync(root, { recursive: true });
  const dir = path.join(root, sanitize(packetId));
  const branch = `ai-dev-team/${sanitize(packetId)}`;
  if (fs.existsSync(dir)) {
    return { ok: false, error: `worktree already exists: ${dir}`, path: dir, branch };
  }
  const add = run("git", ["worktree", "add", "-b", branch, dir, base], repoRoot);
  if (!add.ok) {
    // branch may exist; try without -b
    const add2 = run("git", ["worktree", "add", dir, branch], repoRoot);
    if (!add2.ok) {
      return { ok: false, error: add.stderr || add2.stderr || "worktree add failed" };
    }
  }
  return { ok: true, path: dir, branch, base };
}

function removePacketWorktree({ repoRoot, packetId, force }) {
  const dir = path.join(defaultWorktreeRoot(repoRoot), sanitize(packetId));
  if (!fs.existsSync(dir)) return { ok: true, removed: false, path: dir };
  const args = ["worktree", "remove", dir];
  if (force) args.push("--force");
  const result = run("git", args, repoRoot);
  if (!result.ok) return { ok: false, error: result.stderr || "worktree remove failed", path: dir };
  return { ok: true, removed: true, path: dir };
}

function listPacketWorktrees(repoRoot) {
  const root = defaultWorktreeRoot(repoRoot);
  if (!fs.existsSync(root)) return [];
  return fs
    .readdirSync(root)
    .filter((name) => fs.statSync(path.join(root, name)).isDirectory())
    .map((name) => ({ id: name, path: path.join(root, name) }));
}

function main() {
  const [cmd, packetId] = process.argv.slice(2);
  const repoRoot = process.cwd();
  if (cmd === "add") {
    const result = createPacketWorktree({ repoRoot, packetId: packetId || "PKT" });
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    process.exit(result.ok ? 0 : 1);
  }
  if (cmd === "remove") {
    const result = removePacketWorktree({
      repoRoot,
      packetId: packetId || "PKT",
      force: process.argv.includes("--force"),
    });
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    process.exit(result.ok ? 0 : 1);
  }
  if (cmd === "list") {
    process.stdout.write(JSON.stringify({ ok: true, worktrees: listPacketWorktrees(repoRoot) }, null, 2) + "\n");
    return;
  }
  process.stderr.write("usage: node worktree.js add|remove|list [packetId] [--force]\n");
  process.exit(2);
}

if (require.main === module) main();

module.exports = {
  createPacketWorktree,
  removePacketWorktree,
  listPacketWorktrees,
  defaultWorktreeRoot,
  ensureGitRepo,
};
