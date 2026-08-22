/**
 * `svelte/compiler`（`generate: "server"`）でコンパイルした Svelte
 * コンポーネント + `svelte/server`（`render()`）による SSR renderer。
 *
 * Svelte は `.svelte` 単一ファイルコンポーネントを事前コンパイルする
 * 前提のフレームワークのため、ソースは実行時に文字列として保持し
 * `svelte.compiler.compile()` でサーバー向け JS へコンパイルする。
 * **コンパイルはモジュール読み込み時（top-level await）に 1 回だけ行い、
 * 計測ループには含めない**（PROTOCOL 指示どおり）。コンパイル結果は
 * ESM モジュールとして `import()` する必要があるため、`bench/ssr/.tmp/`
 * （`bench/.gitignore` で追跡除外済み）へ一時ファイルとして書き出す。
 * `bench/ssr/node_modules` を bare specifier 解決できるようにするため、
 * 一時ファイルは `bench/ssr` 配下（`.tmp/`）に置く（`os.tmpdir()` 配下に
 * 置くと `svelte/internal/server` 等の import が解決できない）。
 *
 * テキストは `{#each}`/`{expression}` のテンプレート補間として渡し、
 * Svelte の既定エスケープ経路のみを経由する（`{@html}` は使わない）。
 * ラベル生成（`rowLabel`）はコンポーネントの `<script>` ブロックから
 * `lib/label.mjs` を絶対パス（`file://` URL）で import し、他フレーム
 * ワークと同じく「render 呼び出し（コンポーネント実行）のたびに再計算
 * される」構成に揃える（コンパイル時に埋め込んで静的化しない）。
 */
import { compile } from "svelte/compiler";
import { render } from "svelte/server";
import { writeFileSync, mkdirSync, rmSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
import path from "node:path";
import { pkgVersion } from "../lib/version.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const labelModuleUrl = pathToFileURL(
  path.join(__dirname, "..", "lib", "label.mjs"),
).href;
const tmpDir = path.join(__dirname, "..", ".tmp");

const SOURCE = `<script>
  import { rowLabel } from ${JSON.stringify(labelModuleUrl)};
  let { rows } = $props();
  const items = Array.from({ length: rows }, (_, i) => ({ i, label: rowLabel(i) }));
</script>
<html><body>
<header><h1>Benchmark</h1></header>
<table id="bench-table"><tbody>
{#each items as row}
<tr><td>{row.i}</td><td>{row.label}</td></tr>
{/each}
</tbody></table>
<footer><p>generated {rows} rows</p></footer>
</body></html>
`;

async function compilePage() {
  mkdirSync(tmpDir, { recursive: true });
  const { js } = compile(SOURCE, {
    generate: "server",
    filename: "BenchPage.svelte",
  });
  const compiledPath = path.join(
    tmpDir,
    `bench-page-${process.pid}-${Date.now()}.mjs`,
  );
  writeFileSync(compiledPath, js.code, "utf8");
  try {
    const mod = await import(pathToFileURL(compiledPath).href);
    return mod.default;
  } finally {
    // コンパイル済みモジュールは import 完了後は不要なため即座に削除する
    // （`.tmp/` の蓄積防止）。
    rmSync(compiledPath, { force: true });
  }
}

// トップレベル await: コンパイルは計測外（本モジュール import 時に 1 回）。
const Component = await compilePage();

export const name = "svelte";

export function getVersion() {
  return pkgVersion("svelte");
}

export function renderRows(rows) {
  return render(Component, { props: { rows } }).body;
}
