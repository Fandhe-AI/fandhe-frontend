#!/usr/bin/env node
// フレームワーク横断 CSR ベンチアプリのビルドスクリプト。
//
// bench/csr/apps/<name>/ の各エントリポイントを esbuild でバンドルし、
// bench/csr/dist/<name>/bundle.js + index.html + meta.json を生成する。
// production 相当の最適化フラグ（NODE_ENV 定義・minify）を各フレームワークへ
// 適用し、payload/実行時間の双方で「実運用に近い」計測条件を揃える。
//
// fandhe（bench/csr/fandhe/）はここではビルドしない。別エージェントの
// build.sh が bench/csr/dist/fandhe/ を独立に生成する前提であり、
// run_csr.mjs / measure.mjs 側で「存在すれば使う・なければ skip」の
// fail-soft 処理を行う。
import { build } from "esbuild";
import { compile } from "svelte/compiler";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
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
  const outDir = join(DIST, fw.name);
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

  for (const fw of FRAMEWORKS) {
    if (only && fw.name !== only) continue;
    await buildFramework(fw);
  }

  if (!existsSync(join(DIST, "fandhe"))) {
    console.error("[build] fandhe: skip (dist/fandhe not present; built independently by another agent's build.sh)");
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
