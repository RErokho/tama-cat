#!/usr/bin/env node
"use strict";

const path = require("path");

const DENY_NAME = [
  /^\.env(\..+)?$/i,
  /\.pem$/i,
  /\.p12$/i,
  /\.pfx$/i,
  /\.key$/i,
  /^id_rsa(\.pub)?$/i,
  /^id_ed25519(\.pub)?$/i,
  /credentials\.json$/i,
  /service-account.*\.json$/i,
  /secrets?\.ya?ml$/i,
  /^\.npmrc$/i,
  /^\.netrc$/i,
  /^\.pgpass$/i,
];

const DENY_CONTENT = [
  /BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY/,
  /AWS_SECRET_ACCESS_KEY\s*=/,
  /-----BEGIN PGP PRIVATE KEY BLOCK-----/,
  /"private_key"\s*:\s*"-----BEGIN/,
];

function basename(filePath) {
  return path.basename(String(filePath || "").replace(/\\/g, "/"));
}

function decide(payload) {
  const filePath = payload && payload.file_path;
  const content = String((payload && payload.content) || "");
  const name = basename(filePath);

  if (DENY_NAME.some((re) => re.test(name))) {
    return {
      permission: "deny",
      user_message: `AI Dev Team blocked reading a secret-like file (${name}).`,
    };
  }

  if (content && DENY_CONTENT.some((re) => re.test(content))) {
    return {
      permission: "deny",
      user_message: `AI Dev Team blocked reading file content that looks like a secret (${name || "unknown"}).`,
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

module.exports = { decide };
