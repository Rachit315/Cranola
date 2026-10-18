#!/usr/bin/env node
// Quality gate: every step must pass with zero errors and zero warnings.
// Usage: npm run verify [-- --skip-build]
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const desktop = resolve(root, 'apps/desktop');
const skipBuild = process.argv.includes('--skip-build');

// RUSTFLAGS=-D warnings turns every rustc warning into an error for build and test.
const rustEnv = { ...process.env, RUSTFLAGS: '-D warnings' };

const steps = [
  { name: 'typescript', cmd: 'npx tsc --noEmit -p tsconfig.json', cwd: desktop },
  { name: 'eslint', cmd: 'npx eslint . --max-warnings 0', cwd: desktop },
  { name: 'prettier', cmd: 'npx prettier --check .', cwd: root },
  { name: 'vitest', cmd: 'npx vitest run', cwd: desktop },
  { name: 'rustfmt', cmd: 'cargo fmt --all --check', cwd: root },
  {
    name: 'clippy',
    cmd: 'cargo clippy --workspace --all-targets --locked -- -D warnings',
    cwd: root,
    env: rustEnv,
  },
  { name: 'cargo test', cmd: 'cargo test --workspace --locked', cwd: root, env: rustEnv },
  ...(skipBuild
    ? []
    : [{ name: 'tauri build', cmd: 'npx tauri build --no-bundle', cwd: desktop, env: rustEnv }]),
];

const results = [];
for (const step of steps) {
  console.log(`\n▶ ${step.name}: ${step.cmd}`);
  const started = Date.now();
  const run = spawnSync(step.cmd, {
    cwd: step.cwd,
    env: step.env ?? process.env,
    stdio: 'inherit',
    shell: true,
  });
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  const ok = run.status === 0;
  results.push({ name: step.name, ok, seconds });
  if (!ok) break;
}

console.log('\nQuality gate summary');
for (const r of results) console.log(`  ${r.ok ? 'PASS' : 'FAIL'}  ${r.name} (${r.seconds}s)`);
const skipped = steps.length - results.length;
if (skipped > 0) console.log(`  ${skipped} step(s) not run because an earlier step failed`);

process.exit(results.every((r) => r.ok) && skipped === 0 ? 0 : 1);
