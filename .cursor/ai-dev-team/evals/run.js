#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { validateTask } = require("../tasks/validate.js");

const root = path.resolve(__dirname, "..", "..", "..");
const cursor = path.join(root, ".cursor");
let failed = 0;

function ok(name, cond, detail) {
  if (cond) {
    process.stdout.write(`PASS ${name}\n`);
  } else {
    failed += 1;
    process.stdout.write(`FAIL ${name}${detail ? ` — ${detail}` : ""}\n`);
  }
}

function sampleContract(overrides) {
  return Object.assign(
    {
      goal: "g",
      context: "c",
      inputs: ["i"],
      constraints: [],
      allowed_files: ["src/a.ts"],
      forbidden_files: [],
      acceptance: ["done"],
      report_format: "writer-brief",
      max_reply: "short",
      risks: [],
      unfinished: [],
      evidence: [],
    },
    overrides || {}
  );
}

function evalAdaptSurface() {
  const skill = fs.readFileSync(path.join(cursor, "skills", "adapt-team-to-project", "SKILL.md"), "utf8");
  const steps = fs.readFileSync(path.join(cursor, "skills", "adapt-team-to-project", "references", "steps.md"), "utf8");
  const core = fs.readFileSync(path.join(cursor, "rules", "team-core.mdc"), "utf8");
  ok(
    "adapt-surface",
    /adapt-team-to-project/.test(skill + core) && /Manifest scan|package\.json/i.test(skill + steps),
    "adapt/manifest missing"
  );
}

function evalHandoffOwnership() {
  const handoff = fs.readFileSync(path.join(cursor, "skills", "handoff-management", "SKILL.md"), "utf8");
  const tasks = fs.readFileSync(path.join(cursor, "skills", "tasks", "SKILL.md"), "utf8");
  const contract = fs.readFileSync(path.join(cursor, "skills", "task-contract", "SKILL.md"), "utf8");
  ok(
    "handoff-ownership",
    /allowed_files/.test(handoff) && /contract/.test(tasks) && /report_format/.test(contract),
    "contract/handoff missing"
  );
  ok(
    "brief-report-presets",
    fs.existsSync(path.join(cursor, "skills", "task-contract", "references", "report-formats.md")) &&
      /writer-brief/.test(contract) &&
      /no preamble/i.test(handoff),
    "report presets missing"
  );
  ok("compact-handoff-levels", /Mandatory/i.test(handoff) && /Conditional/i.test(handoff) && /Completion/i.test(handoff));
  const fe = fs.readFileSync(path.join(cursor, "agents", "frontend-developer.md"), "utf8");
  const cr = fs.readFileSync(path.join(cursor, "agents", "code-reviewer.md"), "utf8");
  ok("agent-writer-brief", /Changed:/.test(fe) && /## 1\. Purpose/.test(fe) && !/Next step:/.test(fe));
  ok("agent-reviewer-brief", /Verdict:/.test(cr) && /Diff-first/i.test(cr) && /Blockers:/.test(cr));
  ok("agent-card-six-blocks", /## 6\. Result/.test(fe) && /## 3\. Forbid/.test(fe));
}

function evalCommitGateReadonly() {
  const card = fs.readFileSync(path.join(cursor, "agents", "commit-gate.md"), "utf8");
  ok(
    "commit-gate-readonly",
    /readonly:\s*true/.test(card) && (/Running git/i.test(card) || /Do not run git/i.test(card)),
    "commit-gate weak"
  );
}

function evalNoExtraAgents() {
  const orch = fs.readFileSync(path.join(cursor, "skills", "team-orchestration", "SKILL.md"), "utf8");
  ok("no-extra-agents", /Smallest roster|smallest roster/i.test(orch), "orchestration too broad");
}

function evalOwnershipValidate() {
  const base = {
    id: "TASK-EVAL-1",
    status: "planned",
    priority: "medium",
    title: "t",
    request: "r",
    interpretation: "i",
    acceptance: ["a"],
    assignees: ["frontend-developer", "backend-developer"],
    history: [],
    budget: { effortCap: "medium", maxAttempts: 2 },
    contract: sampleContract({ allowed_files: ["src/ui/Button.tsx", "src/api/user.ts"] }),
    packets: [
      { id: "PKT-1", allowed_files: ["src/ui/Button.tsx"], assignee: "frontend-developer" },
      { id: "PKT-2", allowed_files: ["src/ui/Button.tsx"], assignee: "backend-developer" },
    ],
  };
  const overlap = validateTask(base);
  ok("ownership-overlap", overlap.some((e) => /overlap/i.test(e)), overlap.join("; ") || "expected overlap error");

  base.packets[1].allowed_files = ["src/api/user.ts"];
  const clean = validateTask(base);
  ok("ownership-clean", clean.length === 0, clean.join("; "));
}

function evalContractRequired() {
  const missing = validateTask({
    id: "TASK-EVAL-2",
    status: "planned",
    priority: "medium",
    title: "t",
    request: "r",
    interpretation: "i",
    acceptance: ["a"],
    assignees: ["repository-analyst"],
    history: [],
    budget: { effortCap: "low", maxAttempts: 2 },
  });
  ok("contract-required", missing.some((e) => /contract/i.test(e)), missing.join("; "));

  const good = validateTask({
    id: "TASK-EVAL-3",
    status: "planned",
    priority: "medium",
    title: "t",
    request: "r",
    interpretation: "i",
    acceptance: ["a"],
    assignees: ["repository-analyst"],
    history: [],
    budget: { effortCap: "low", maxAttempts: 2 },
    contract: sampleContract({ allowed_files: [] }),
  });
  ok("contract-valid", good.length === 0, good.join("; "));

  const legacy = validateTask({
    id: "TASK-EVAL-4",
    status: "planned",
    priority: "medium",
    title: "t",
    request: "r",
    interpretation: "i",
    acceptance: ["a"],
    assignees: ["repository-analyst"],
    history: [],
    budget: { effortCap: "low", maxAttempts: 2 },
    contract: sampleContract({ report_format: "Goal/Result", allowed_files: [] }),
  });
  ok("contract-rejects-legacy-format", legacy.some((e) => /report_format|brief preset/i.test(e)), legacy.join("; "));
}

function evalMemoryAndSafety() {
  ok("memory-decisions", fs.existsSync(path.join(cursor, "ai-dev-team", "memory", "DECISIONS.md")), "DECISIONS missing");
  ok("skill-project-memory", fs.existsSync(path.join(cursor, "skills", "project-memory", "SKILL.md")), "project-memory missing");
  ok("skill-packet-rollback", fs.existsSync(path.join(cursor, "skills", "packet-rollback", "SKILL.md")), "packet-rollback missing");
  const safety = fs.readFileSync(path.join(cursor, "rules", "team-safety.mdc"), "utf8");
  ok("safety-fix-loop", /two|2/.test(safety) && /escalat/i.test(safety), "fix-loop rule missing");
  const roster = fs.readFileSync(path.join(cursor, "skills", "team-orchestration", "references", "roster.md"), "utf8");
  ok("contract-first", /Contract-first|contract-first/i.test(roster), "contract-first missing");
}

function evalHooksAndMcp() {
  const hooks = JSON.parse(fs.readFileSync(path.join(cursor, "hooks.json"), "utf8"));
  const mcp = JSON.parse(fs.readFileSync(path.join(cursor, "mcp.json"), "utf8"));
  ok("hooks-precompact", Boolean(hooks.hooks && hooks.hooks.preCompact), "preCompact missing");
  ok("hooks-stop", Boolean(hooks.hooks && hooks.hooks.stop), "stop missing");
  ok("hooks-session", Boolean(hooks.hooks && hooks.hooks.sessionStart), "sessionStart missing");
  ok(
    "hooks-pretool-ownership",
    Boolean(hooks.hooks && hooks.hooks.preToolUse && hooks.hooks.preToolUse.some((h) => /guard-allowed-files/.test(h.command))),
    "preToolUse ownership guard missing"
  );
  ok("mcp-context7", Boolean(mcp.mcpServers && mcp.mcpServers.context7), "context7 missing");
  ok("mcp-no-github", !mcp.mcpServers.github, "github mcp should be absent");
  ok("agents-md", fs.existsSync(path.join(root, "AGENTS.md")), "AGENTS.md missing");
  ok(
    "always-on-thin",
    (() => {
      const rules = fs.readdirSync(path.join(cursor, "rules")).filter((n) => n.endsWith(".mdc"));
      let always = 0;
      for (const name of rules) {
        const text = fs.readFileSync(path.join(cursor, "rules", name), "utf8");
        if (/alwaysApply:\s*true/.test(text)) always += 1;
      }
      return always <= 3;
    })(),
    "too many alwaysApply rules"
  );
  const session = require(path.join(cursor, "hooks", "session-start-brief.js"));
  const brief = session.decide(root);
  ok("session-brief", /AI Dev Team/.test(brief.additional_context || ""), "session brief weak");
}

function evalCoreVersion() {
  const core = JSON.parse(fs.readFileSync(path.join(cursor, "ai-dev-team", "core.json"), "utf8"));
  ok("core-0-8-0", core.version === "0.8.0", `version=${core.version}`);
  const skillDirs = fs
    .readdirSync(path.join(cursor, "skills"))
    .filter((n) => fs.statSync(path.join(cursor, "skills", n)).isDirectory());
  ok("skills-lean", skillDirs.length <= 28, `skill count=${skillDirs.length}`);
  ok("no-role-mirror-skills", !skillDirs.includes("frontend-development") && !skillDirs.includes("team-help"));
  ok("merged-verify-menu", !skillDirs.includes("verify-menu"));
  ok("merged-token-budgeting", !skillDirs.includes("token-budgeting"));
  ok("graphify-kept", skillDirs.includes("graphify"));
  ok("no-weak-drift", !fs.existsSync(path.join(cursor, "ai-dev-team", "quality", "drift-detector.js")));
  ok("no-weak-freshness", !fs.existsSync(path.join(cursor, "ai-dev-team", "memory", "freshness.js")));
  ok("no-empty-projects-registry", !fs.existsSync(path.join(cursor, "ai-dev-team", "projects")));
  ok("no-routing-policy-dup", !fs.existsSync(path.join(cursor, "ai-dev-team", "config", "routing-policy.json")));
  const constitution = fs.readFileSync(path.join(cursor, "ai-dev-team", "constitution", "TEAM_CONSTITUTION.md"), "utf8");
  ok("constitution-slim", constitution.split(/\n/).length <= 60, `lines=${constitution.split(/\n/).length}`);
  ok("constitution-context-thrift", /does not affect its decision/i.test(constitution));
  const coreRoster = JSON.parse(fs.readFileSync(path.join(cursor, "ai-dev-team", "employees", "core-roster.json"), "utf8"));
  ok("core-roster-size", Array.isArray(coreRoster.core) && coreRoster.core.length >= 8 && coreRoster.core.length <= 12);
  const registry = JSON.parse(fs.readFileSync(path.join(cursor, "ai-dev-team", "employees", "registry.json"), "utf8"));
  ok("registry-tiers", (registry.employees || []).some((e) => e.tier === "core") && (registry.employees || []).some((e) => e.tier === "optional"));
  const rosterMd = fs.readFileSync(path.join(cursor, "skills", "team-orchestration", "references", "roster.md"), "utf8");
  ok("roster-core-first", /Default = core/i.test(rosterMd));
  ok("roster-min-table", /Minimum by task type/i.test(rosterMd));
  const mem = fs.readFileSync(path.join(cursor, "skills", "project-memory", "SKILL.md"), "utf8");
  ok("memory-topical", /Topical load/i.test(mem));
  const teamCore = fs.readFileSync(path.join(cursor, "rules", "team-core.mdc"), "utf8");
  ok("always-on-core-thrift", /no agent gets context/i.test(teamCore));
  ok(
    "always-on-sizes",
    teamCore.length < 2200 &&
      fs.readFileSync(path.join(cursor, "rules", "team-models.mdc"), "utf8").length < 900 &&
      fs.readFileSync(path.join(cursor, "rules", "team-safety.mdc"), "utf8").length < 1600
  );
}

function evalTrustPlane() {
  const quality = path.join(cursor, "ai-dev-team", "quality");
  ok("gates-schema", fs.existsSync(path.join(quality, "gates.schema.json")));
  ok("gates-core", fs.existsSync(path.join(quality, "gates.core.json")));
  ok("discover-gates", fs.existsSync(path.join(quality, "discover-gates.js")));
  ok("run-gates", fs.existsSync(path.join(quality, "run-gates.js")));
  ok("lifecycle", fs.existsSync(path.join(cursor, "ai-dev-team", "tasks", "lifecycle.js")));
  ok("fixture-vite", fs.existsSync(path.join(cursor, "ai-dev-team", "fixtures", "vite-ts", "package.json")));
  const commit = fs.readFileSync(path.join(cursor, "agents", "commit-gate.md"), "utf8");
  ok("commit-gate-evidence", /verification-report/.test(commit), "commit-gate missing evidence rule");
  const { canTransition } = require("../tasks/lifecycle.js");
  ok("lifecycle-verified-edge", canTransition("review", "verified") && canTransition("verified", "completed"));
  ok("lifecycle-no-skip", !canTransition("review", "completed"));
  const { summarizeOutput } = require("../quality/run-gates.js");
  const sumFail = summarizeOutput("error TS2304: x\nfail\n", 1);
  ok("gate-summary-errors", sumFail && sumFail.ok === false && Array.isArray(sumFail.first_errors));
  const sumOk = summarizeOutput("typecheck ok\n", 0);
  ok("gate-summary-ok", sumOk && sumOk.ok === true);
}

function evalIsolationPlane() {
  const exec = path.join(cursor, "ai-dev-team", "execution");
  const quality = path.join(cursor, "ai-dev-team", "quality");
  ok("worktree-js", fs.existsSync(path.join(exec, "worktree.js")));
  ok("dispatch-js", fs.existsSync(path.join(exec, "dispatch.js")));
  ok("regression-js", fs.existsSync(path.join(quality, "regression-awareness.js")));
  ok("skill-worktree", fs.existsSync(path.join(cursor, "skills", "worktree-isolation", "SKILL.md")));
  ok("skill-regression", fs.existsSync(path.join(cursor, "skills", "regression-awareness", "SKILL.md")));

  const { planDispatch } = require("../execution/dispatch.js");
  const plan = planDispatch(
    {
      execution: { mode: "shared", isolation: "preferred" },
      packets: [
        { id: "PKT-1", depends_on: [], allowed_files: ["a.ts"], status: "open" },
        { id: "PKT-2", depends_on: ["PKT-1"], allowed_files: ["b.ts"], status: "open" },
        { id: "PKT-3", depends_on: [], allowed_files: ["c.ts"], status: "open" },
      ],
    },
    []
  );
  ok("dispatch-ready", plan.ready.includes("PKT-1") && plan.ready.includes("PKT-3") && !plan.ready.includes("PKT-2"));
  ok("dispatch-blocked", plan.blocked.some((b) => b.id === "PKT-2"));
  ok("dispatch-worktree-preferred", plan.useWorktree === true, JSON.stringify(plan));

  const sharedOnly = planDispatch(
    {
      execution: { mode: "shared", isolation: "disabled" },
      packets: [
        { id: "A", depends_on: [], allowed_files: ["x.ts"] },
        { id: "B", depends_on: [], allowed_files: ["y.ts"] },
      ],
    },
    []
  );
  ok("dispatch-shared-default", sharedOnly.useWorktree === false);

  const { recommendGates, isPublicSurface, packageForFile } = require("../quality/regression-awareness.js");
  ok("regression-public", isPublicSurface("packages/ui/src/index.ts"));
  ok("regression-pkg", packageForFile("packages/ui/src/Button.tsx", root) === "packages/ui");
  const rec = recommendGates(["packages/ui/src/index.ts"], {
    gates: [
      { id: "typecheck", command: "npm run typecheck", package: "root" },
      { id: "lint", command: "npm run lint", package: "root" },
      { id: "test", command: "npm test", package: "root" },
      { id: "typecheck:packages/ui", command: "npm run typecheck -w ui" },
    ],
  });
  ok("regression-gates", rec.gateIds.includes("typecheck") || rec.gateIds.includes("typecheck:packages/ui"), rec.gateIds.join(","));

  const when = fs.readFileSync(path.join(cursor, "skills", "team-orchestration", "references", "when.md"), "utf8");
  ok("when-isolation-skills", /worktree-isolation/.test(when) && /regression-awareness/.test(when));
  ok("when-no-drift-skill", !/architecture-drift/.test(when));
}

function evalObservability() {
  const obs = path.join(cursor, "ai-dev-team", "observability");
  ok("obs-events-js", fs.existsSync(path.join(obs, "events.js")));
  ok("obs-report-js", fs.existsSync(path.join(obs, "report.js")));
  const obsSkill = fs.readFileSync(path.join(cursor, "skills", "kit-observability", "SKILL.md"), "utf8");
  ok("skill-kit-observability", /disable-model-invocation:\s*true/.test(obsSkill), "observability must be manual-only");
  const when = fs.readFileSync(path.join(cursor, "skills", "team-orchestration", "references", "when.md"), "utf8");
  ok("when-observability", /kit-observability/.test(when));

  const { appendEvent, readEvents, normalizeEvent } = require("../observability/events.js");
  const { summarize, writeReport } = require("../observability/report.js");
  ok("obs-reject-bad-type", normalizeEvent({ type: "nope" }).ok === false);
  const tmpRoot = path.join(cursor, "ai-dev-team", "observability", ".eval-tmp");
  fs.mkdirSync(tmpRoot, { recursive: true });
  // write into tmpRoot as if it were a repo with nested observability — use a nested fake project
  const fake = path.join(root, ".cursor", "ai-dev-team", "observability", ".fake-root");
  fs.mkdirSync(path.join(fake, ".cursor", "ai-dev-team"), { recursive: true });
  const a1 = appendEvent(fake, { type: "skill.used", skill: "work-packet", task_id: "TASK-OBS" });
  const a2 = appendEvent(fake, { type: "gates.run", task_id: "TASK-OBS", status: "passed", duration_ms: 12 });
  ok("obs-append", a1.ok && a2.ok);
  const events = readEvents(fake);
  ok("obs-read", events.length >= 2);
  const sum = summarize(events, fake);
  ok("obs-summarize", sum.event_count >= 2 && sum.top_skills.some((s) => s.name === "work-packet"));
  ok("obs-qos", sum.qos && typeof sum.qos.ownership_denies === "number");
  const written = writeReport(fake);
  ok("obs-write-report", written.ok && fs.existsSync(written.path));
  try {
    fs.rmSync(path.join(fake, ".cursor"), { recursive: true, force: true });
    fs.rmSync(fake, { recursive: true, force: true });
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  } catch {
    /* ignore */
  }

  const mono = path.join(cursor, "ai-dev-team", "fixtures", "next-mono");
  ok("fixture-next-mono", fs.existsSync(path.join(mono, "package.json")));
  const { discover } = require("../quality/discover-gates.js");
  const disc = discover(mono);
  ok(
    "mono-discover-packages",
    disc.gates.some((g) => g.id === "typecheck:packages/ui" && g.command) &&
      disc.gates.some((g) => g.id === "lint:apps/web" && g.command),
    disc.gates.map((g) => g.id).join(",")
  );
  ok("harness-e2e-file", fs.existsSync(path.join(cursor, "ai-dev-team", "evals", "harness-e2e.js")));
  ok("smoke-project-file", fs.existsSync(path.join(cursor, "ai-dev-team", "evals", "smoke-project.js")));

  const { pathAllowed } = require("../../hooks/guard-allowed-files.js");
  ok("ownership-glob", pathAllowed("src/ui/Button.tsx", ["src/ui/**"]));
}

evalAdaptSurface();
evalHandoffOwnership();
evalCommitGateReadonly();
evalNoExtraAgents();
evalOwnershipValidate();
evalContractRequired();
evalMemoryAndSafety();
evalHooksAndMcp();
evalCoreVersion();
evalTrustPlane();
evalIsolationPlane();
evalObservability();


if (failed) {
  process.stdout.write(`\n${failed} failed\n`);
  process.exit(1);
}
process.stdout.write("\nall evals passed\n");
