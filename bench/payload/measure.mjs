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
// 比較対象一覧は run_csr.mjs と共有する正本（bench/csr/frameworks.mjs）を
// import して使う。dist に現存するディレクトリを動的列挙する方式は、
// 7 種中 1 件でもビルド済みなら成功扱いになる fail-open だった
// （PR #1370 codex 再レビュー指摘 P1）。
import { ALL_FRAMEWORKS } from "../csr/frameworks.mjs";

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
  const targets = only ? [only] : ALL_FRAMEWORKS;

  if (!existsSync(DIST)) {
    console.error(`[payload] dist directory not found: ${DIST} (run bench/csr/build.mjs first)`);
    process.exitCode = 1;
    return;
  }

  // dist に存在する任意のディレクトリではなく、ALL_FRAMEWORKS（run_csr.mjs
  // と共有する正本）に載っている名前だけを対象にする。dist 配下に紛れ込んだ
  // 一覧外ディレクトリ（例: 手動デバッグ用の一時出力）は対象外として無視し、
  // stderr へ警告のみ出す（測定結果には含めない）。
  const knownDirNames = new Set(readdirSync(DIST, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name));
  for (const dirName of knownDirNames) {
    if (!ALL_FRAMEWORKS.includes(dirName)) {
      console.error(`[payload] warning: ignoring unlisted directory under dist: ${dirName}`);
    }
  }

  const measured = [];
  const missing = [];
  for (const name of targets) {
    const dir = join(DIST, name);
    if (!existsSync(dir) || !statSync(dir).isDirectory()) {
      missing.push(name);
      continue;
    }
    const files = collectTargetFiles(dir);
    if (files.length === 0) {
      // ディレクトリはあるが計測対象ファイルが 1 件もない（ビルド途中の
      // 空ディレクトリ等）も欠落扱いにする。
      missing.push(name);
      continue;
    }
    measured.push(name);
  }

  // 既定実行（--framework 未指定）では ALL_FRAMEWORKS 全 7 種の dist と
  // 各 1 件以上の計測対象ファイルが揃うことを必須とする。1 件でも欠落が
  // あれば fail-closed で終了コード 1 にする（run_csr.mjs と同じ契約、
  // bench/PROTOCOL.md §2.2 参照）。--framework <name> による明示的な
  // 部分実行のときだけ、その 1 件のみの成功を許可する。
  if (!only && missing.length > 0) {
    console.error(
      `[payload] missing framework(s) under default full run (${measured.length}/${ALL_FRAMEWORKS.length} built): ` +
        `${missing.join(", ")} — build them first (bench/csr/build.mjs, bench/csr/fandhe/build.sh), ` +
        `or pass --framework <name> for an explicit partial run`,
    );
    process.exitCode = 1;
    return;
  }
  if (measured.length === 0) {
    console.error(`[payload] no framework was measured (target: ${targets.join(", ")})`);
    process.exitCode = 1;
    return;
  }

  for (const name of measured.sort()) {
    const dir = join(DIST, name);
    const result = measureFramework(name, dir);
    console.log(JSON.stringify(result));
  }
}

main();
