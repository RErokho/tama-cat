#!/usr/bin/env node
"use strict";

const denyPatterns = [
  { name: "git-push-force", re: /\bgit(?:\.exe)?\s+push\b[\s\S]*?(--force|--force-with-lease|(^|\s)-f(\s|$))/i },
  { name: "git-reset-hard", re: /\bgit(?:\.exe)?\s+reset\b[\s\S]*--hard\b/i },
  { name: "git-clean-force", re: /\bgit(?:\.exe)?\s+clean\b[\s\S]*-f/i },
  { name: "rm-recursive", re: /\brm\s+-[^\n]*r[^\n]*\s+\S+/i },
  { name: "powershell-recursive-delete", re: /\bRemove-Item\b[\s\S]*-Recurse\b[\s\S]*-Force\b|\bRemove-Item\b[\s\S]*-Force\b[\s\S]*-Recurse\b/i },
  { name: "format-volume", re: /\bformat(?:\.com)?\s+[a-zA-Z]:/i },
];

const askPatterns = [
  { name: "git-push", re: /\bgit(?:\.exe)?\s+push\b/i },
  { name: "npm-publish", re: /\b(npm|pnpm|yarn)\s+publish\b/i },
  { name: "docker-push", re: /\bdocker\s+push\b/i },
  { name: "network", re: /\b(curl|wget|Invoke-WebRequest|iwr)\b/i },
];

function decide(command) {
  const text = String(command || "");
  const denied = denyPatterns.find((item) => item.re.test(text));
  if (denied) {
    return {
      permission: "deny",
      user_message: `AI Dev Team blocked a dangerous command (${denied.name}).`,
      agent_message: `Command blocked by a safety rule: ${denied.name}. Do not retry it and do not look for a bypass. If the action is truly needed, stop and ask the user.`,
    };
  }
  const asked = askPatterns.find((item) => item.re.test(text));
  if (asked) {
    return {
      permission: "ask",
      user_message: `AI Dev Team asks to confirm the command (${asked.name}).`,
      agent_message: `Command requires user confirmation: ${asked.name}.`,
    };
  }
  return { permission: "allow" };
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
  let command = "";
  try {
    command = JSON.parse(raw || "{}").command || "";
  } catch {
    process.stdout.write(JSON.stringify({ permission: "allow" }));
    return;
  }
  process.stdout.write(JSON.stringify(decide(command)));
}

if (require.main === module) {
  main().catch(() => {
    process.stdout.write(JSON.stringify({ permission: "allow" }));
  });
}

module.exports = { decide };
