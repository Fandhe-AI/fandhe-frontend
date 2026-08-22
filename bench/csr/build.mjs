#!/usr/bin/env node
// フレームワーク横断 CSR ベンチアプリのビルドスクリプト。
//
// bench/csr/apps/<name>/ の各エントリポイントを esbuild でバンドルし、
// bench/csr/dist/<name>/bundle.js + index.html + meta.json を生成する。
// production 相当の最適化フラグ（NODE_ENV 定義・minify）を各フレームワークへ
// 適用し、payload/実行時間の双方で「実運用に近い」計測条件を揃える。
//
// fandhe（bench/csr/fandhe/）はここではビルドしない。Rust/wasm ツール
// チェーンを要する別工程（bench/csr/fandhe/build.sh）が bench/csr/dist/
// fandhe/ を生成する。run_csr.mjs / payload/measure.mjs の既定実行は
// frameworks.mjs の全 7 種（fandhe 含む）の dist が揃うことを必須とし、
// 欠落は fail-closed でエラー終了する（bench/PROTOCOL.md §2.2）。
import { build } from "esbuild";
import { compile } from "svelte/compiler";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(fileURLToPath(import.meta.url));
const DIST = join(ROOT, "dist");
const TEMPLATE_HTML = readFileSync(join(ROOT, "common", "index.template.html"), "utf8");

function pkgVersion(name) {
  const pkgPath = join(ROOT, "node_modules", name, "package.json");
  return JSON.parse(readFileSync(pkgPath, "utf8")).version;
}

// App.svelte を JS へ事前コンパイルする esbuild プラグイン。
// generate: "client" で svelte 5 のクライアント向け出力を得て、esbuild が
// 通常の JS モジュールとしてバンドルする（内部の 'svelte/internal/client' 等の
// 裸 import は resolveDir 起点の node 解決で node_modules から見つかる）。
const sveltePlugin = {
  name: "svelte-compile",
  setup(pluginBuild) {
    pluginBuild.onLoad({ filter: /\.svelte$/ }, (args) => {
      const source = readFileSync(args.path, "utf8");
      const compiled = compile(source, {
        filename: args.path,
        generate: "client",
      });
      return { contents: compiled.js.code, loader: "js", resolveDir: dirname(args.path) };
    });
  },
};

// Vue の esm-bundler ビルドが要求する feature flag 定義（本番向け最小化）。
const VUE_DEFINES = {
  "__VUE_OPTIONS_API__": "true",
  "__VUE_PROD_DEVTOOLS__": "false",
  "__VUE_PROD_HYDRATION_MISMATCH_DETAILS__": "false",
};

const FRAMEWORKS = [
  {
    name: "vanilla",
    entry: "apps/vanilla/main.js",
    version: "n/a",
  },
  {
    name: "react",
    entry: "apps/react/main.jsx",
    version: pkgVersion("react"),
    jsx: "automatic",
    jsxImportSource: "react",
  },
  {
    name: "preact",
    entry: "apps/preact/main.jsx",
    version: pkgVersion("preact"),
    jsx: "automatic",
    jsxImportSource: "preact",
  },
  {
    name: "vue",
    entry: "apps/vue/main.js",
    version: pkgVersion("vue"),
    define: VUE_DEFINES,
  },
  {
    name: "svelte",
    entry: "apps/svelte/main.js",
    version: pkgVersion("svelte"),
    plugins: [sveltePlugin],
  },
  {
    name: "lit",
    entry: "apps/lit/main.js",
    version: pkgVersion("lit"),
  },
];

async function buildFramework(fw) {
  // ビルド前に当該フレームワークの出力ディレクトリのみを清掃する。
  // 旧成果物（例: 過去に生成された別名ファイル）を残したまま上書きすると、
  // payload/measure.mjs が stale なファイルを計測に混入させてしまうため
  // （PR #1370 codex 第 4 巡レビュー指摘 P1）。削除対象は path.resolve で
  // dist ルート配下であることを検証してからに限定する（フレームワーク名は
  // 正本 frameworks.mjs 由来の固定リストだが、防御的にパストラバーサルを
  // 遮断する。.claude/rules/security.md A01）。
  const distRoot = resolve(DIST);
  const outDir = resolve(DIST, fw.name);
  if (!outDir.startsWith(distRoot + sep)) {
    throw new Error(`refusing to clean output directory outside dist root: ${outDir}`);
  }
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  const define = {
    "process.env.NODE_ENV": '"production"',
    ...(fw.define ?? {}),
  };

  await build({
    entryPoints: [join(ROOT, fw.entry)],
    outfile: join(outDir, "bundle.js"),
    bundle: true,
    minify: true,
    format: "iife",
    platform: "browser",
    target: ["es2020"],
    define,
    jsx: fw.jsx,
    jsxImportSource: fw.jsxImportSource,
    plugins: fw.plugins,
    logLevel: "warning",
  });

  writeFileSync(join(outDir, "index.html"), TEMPLATE_HTML.replace("__FRAMEWORK_NAME__", fw.name));
  writeFileSync(join(outDir, "meta.json"), `${JSON.stringify({ framework: fw.name, version: fw.version }, null, 2)}\n`);

  console.error(`[build] ${fw.name} -> dist/${fw.name}/bundle.js (version=${fw.version})`);
}

async function main() {
  mkdirSync(DIST, { recursive: true });
  const only = process.argv.includes("--framework") ? process.argv[process.argv.indexOf("--framework") + 1] : null;

  let builtCount = 0;
  for (const fw of FRAMEWORKS) {
    if (only && fw.name !== only) continue;
    await buildFramework(fw);
    builtCount += 1;
  }

  // --framework の値が一覧に無い（typo 等）とき、0 件ビルドのまま exit 0 に
  // なる fail-open を防ぐ（run_csr.mjs / measure.mjs の「対象 0 件は
  // エラー」契約と同型）。fandhe はこのスクリプトの対象外（build.sh 担当）
  // のため、known の列挙にはその旨を添える。
  if (only && builtCount === 0) {
    console.error(
      `[build] no framework was built (target: ${only}; known: ${FRAMEWORKS.map((fw) => fw.name).join(", ")}; ` +
        "fandhe is built by bench/csr/fandhe/build.sh, not this script)",
    );
    process.exitCode = 1;
    return;
  }

  if (!existsSync(join(DIST, "fandhe"))) {
    console.error("[build] note: dist/fandhe not present yet — build it with bench/csr/fandhe/build.sh before the default full run of run_csr.mjs / measure.mjs");
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
