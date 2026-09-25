// Enforce the dependency release-age cooldown in CI.
//
// pnpm's `minimumReleaseAge` only applies while resolving dependencies. Both
// the documented dev path and the release build use `--frozen-lockfile`, which
// installs the recorded versions without resolving, so the cooldown never runs
// there. This checks the versions this change adds to the lockfile against the
// npm registry publish time and fails if any is younger than MIN_AGE_DAYS.
//
// Env: BASE_REF (default origin/main), LOCKFILE, MIN_AGE_DAYS (default 7).

import { execFileSync } from "node:child_process";

const MIN_AGE_DAYS = Number(process.env.MIN_AGE_DAYS ?? "7");
const BASE_REF = process.env.BASE_REF ?? "origin/main";
const LOCKFILE = process.env.LOCKFILE ?? "crates/gui/frontend/pnpm-lock.yaml";
const REGISTRY = "https://registry.npmjs.org";
const CONCURRENCY = 8;

function addedSpecs() {
  const diff = execFileSync(
    "git",
    ["diff", `${BASE_REF}...HEAD`, "--", LOCKFILE],
    { encoding: "utf8" },
  );
  // Lockfile v9 keys are `name@version` (packages) or `name@version(peers)`
  // (snapshots). Match added lines, capture name + a semver-shaped version.
  const re = /^\+\s+'?((?:@[^@/'()]+\/)?[^@/'()]+)@(\d[^'()\s]*)'?:/;
  const specs = new Map();
  for (const line of diff.split("\n")) {
    const m = line.match(re);
    if (m) specs.set(`${m[1]}@${m[2]}`, { name: m[1], version: m[2] });
  }
  return [...specs.values()];
}

async function publishedAt({ name, version }) {
  const url = `${REGISTRY}/${name.replace("/", "%2F")}`;
  let lastErr;
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      const res = await fetch(url);
      if (res.status === 429 || res.status >= 500) {
        lastErr = new Error(`registry ${res.status}`);
      } else if (!res.ok) {
        throw new Error(`registry ${res.status}`);
      } else {
        const time = (await res.json()).time?.[version];
        if (!time) throw new Error("no publish time recorded");
        return new Date(time).getTime();
      }
    } catch (err) {
      lastErr = err;
    }
    await new Promise((r) => setTimeout(r, 500 * (attempt + 1)));
  }
  throw lastErr;
}

const specs = addedSpecs();
if (specs.length === 0) {
  console.log("No new lockfile package versions to check.");
  process.exit(0);
}

const minMs = MIN_AGE_DAYS * 86_400_000;
const now = Date.now();
const violations = [];
const errors = [];
const queue = [...specs];

await Promise.all(
  Array.from({ length: CONCURRENCY }, async () => {
    for (let spec = queue.pop(); spec; spec = queue.pop()) {
      try {
        const ageMs = now - (await publishedAt(spec));
        if (ageMs < minMs) {
          const days = (ageMs / 86_400_000).toFixed(1);
          violations.push(`${spec.name}@${spec.version} published ${days}d ago`);
        }
      } catch (err) {
        errors.push(`${spec.name}@${spec.version}: ${err.message}`);
      }
    }
  }),
);

if (errors.length) {
  console.error("Could not verify:\n  " + errors.join("\n  "));
}
if (violations.length) {
  console.error(
    `\nDependency cooldown violation (minimum ${MIN_AGE_DAYS} days):\n  ` +
      violations.join("\n  ") +
      "\n\nWait until these versions age past the cooldown, or pick older ones.",
  );
  process.exit(1);
}
if (errors.length) process.exit(1);
console.log(
  `Checked ${specs.length} new package version(s); all older than ${MIN_AGE_DAYS} days.`,
);
