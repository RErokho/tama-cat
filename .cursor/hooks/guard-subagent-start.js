#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const BUILTIN = new Set([
  "explore",
  "browser",
  "generalPurpose",
  "shell",
  "ci-investigator",
  "bugbot",
  "security-review",
  "cursor-guide",
  "best-of-n-runner",
]);

const BANNED_MODEL = /fast|xhigh|extra-high|extra high|context=500k|grok-4\.6/i;

function agentsDir() {
  return path.resolve(__dirname, "..", "agents");
}

function knownTeamAgents() {
  const dir = agentsDir();
  if (!fs.existsSync(dir)) return new Set();
  return new Set(
    fs
      .readdirSync(dir)
      .filter((name) => name.endsWith(".md"))
      .map((name) => name.slice(0, -3))
  );
}

function decide(payload) {
  const type = String((payload && payload.subagent_type) || "");
  const model = String((payload && payload.subagent_model) || "");
  const team = knownTeamAgents();

  if (model && BANNED_MODEL.test(model)) {
    return {
      permission: "deny",
      user_message: `AI Dev Team blocked subagent model (${model}). Use Grok 4.7 low/medium/high pins only.`,
    };
  }

  if (BUILTIN.has(type) || team.has(type)) {
    return { permission: "allow" };
  }

  return {
    permission: "deny",
    user_message: `AI Dev Team blocked unknown subagent type (${type || "empty"}). Use a card from .cursor/agents/ or an allowed built-in.`,
  };
}

function readStdin() {
  return new Promise((resolve, reject) => {
    const chunks = [];
    process.stdin.on("data", (chunk) => chunks.push(chunk));
    process.stdin.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    process.stdin.on("error", reject);
  });
}

async function main() {
  const raw = await readStdin();
  let payload = {};
  try {
    payload = JSON.parse(raw || "{}");
  } catch {
    process.stdout.write(JSON.stringify({ permission: "allow" }));
    return;
  }
  process.stdout.write(JSON.stringify(decide(payload)));
}

if (require.main === module) {
  main().catch(() => {
    process.stdout.write(JSON.stringify({ permission: "allow" }));
  });
}

module.exports = { decide, BUILTIN };
