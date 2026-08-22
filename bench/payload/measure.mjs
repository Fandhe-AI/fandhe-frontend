#!/usr/bin/env node
// フレームワーク横断 CSR ビルド成果物の payload サイズ計測。
//
// bench/csr/dist/<name>/ 配下の JS（+ fandhe のみ .wasm も対象）について
// raw バイト数と gzip（zlib, level 9）バイト数を計測する。index.html は
// 全フレームワーク共通の骨格であり比較対象として無意味なため除外する。
import { gzipSync } from "node:zlib";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(fileURLToPath(import.meta.url));
const DIST = join(ROOT, "..", "csr", "dist");

const TARGET_EXTENSIONS = new Set([".js", ".mjs", ".wasm"]);
const EXCLUDE_BASENAMES = new Set(["index.html", "meta.json"]);

// dir 配下を再帰的に走査し、計測対象ファイル（.js/.mjs + .wasm）の
// 絶対パス一覧を返す。node_modules 等の巨大ディレクトリは dist 配下に
// 存在しない前提のため特別な除外は行わない。
function collectTargetFiles(dir) {
  const results = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...collectTargetFiles(fullPath));
      continue;
    }
    if (EXCLUDE_BASENAMES.has(entry.name)) continue;
    if (extname(entry.name) === ".map") continue;
    if (!TARGET_EXTENSIONS.has(extname(entry.name))) continue;
    results.push(fullPath);
  }
  return results;
}

function measureFramework(name, dir) {
  const files = collectTargetFiles(dir).sort();
  const fileStats = files.map((filePath) => {
    const buf = readFileSync(filePath);
    const gz = gzipSync(buf, { level: 9 });
    return {
      file: relative(dir, filePath),
      raw: buf.length,
      gzip: gz.length,
    };
  });

  const totalRaw = fileStats.reduce((sum, f) => sum + f.raw, 0);
  const totalGzip = fileStats.reduce((sum, f) => sum + f.gzip, 0);

  return {
    framework: name,
    mode: "payload",
    files: fileStats,
    total_raw: totalRaw,
    total_gzip: totalGzip,
  };
}

function main() {
  const argv = process.argv.slice(2);
  const frameworkFlagIdx = argv.indexOf("--framework");
  const only = frameworkFlagIdx >= 0 ? argv[frameworkFlagIdx + 1] : null;

  if (!existsSync(DIST)) {
    console.error(`[payload] dist directory not found: ${DIST} (run bench/csr/build.mjs first)`);
    process.exitCode = 1;
    return;
  }

  const names = readdirSync(DIST, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .filter((name) => !only || name === only)
    .sort();

  if (names.length === 0) {
    console.error(`[payload] no dist frameworks found under ${DIST}`);
    process.exitCode = 1;
    return;
  }

  for (const name of names) {
    const dir = join(DIST, name);
    if (!statSync(dir).isDirectory()) continue;
    const result = measureFramework(name, dir);
    console.log(JSON.stringify(result));
  }
}

main();
