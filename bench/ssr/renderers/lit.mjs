/**
 * `lit`（`html` タグ付きテンプレート）+ `@lit-labs/ssr`（`render`）による
 * SSR renderer。
 *
 * `@lit-labs/ssr` の `render()` は文字列チャンクの同期 `Iterable` を返す
 * （`AsyncIterable` ではない）。計測対象は「render to string」までを含むため、
 * チャンクの集約（`for...of` による文字列連結）も計測に含める。テキストは
 * `html` タグ付きテンプレートの `${}` 差し込みとして渡し、Lit の既定
 * エスケープ経路のみを経由する（`unsafeHTML` は使わない）。
 */
import { html } from "lit";
import { render } from "@lit-labs/ssr";
import { rowLabel } from "../lib/label.mjs";
import { pkgVersion } from "../lib/version.mjs";

function page(rows) {
  const rowsTpl = [];
  for (let i = 0; i < rows; i += 1) {
    rowsTpl.push(html`<tr><td>${i}</td><td>${rowLabel(i)}</td></tr>`);
  }
  return html`<html><body><header><h1>Benchmark</h1></header><table id="bench-table"><tbody>${rowsTpl}</tbody></table><footer><p>generated ${rows} rows</p></footer></body></html>`;
}

export const name = "lit";

export function getVersion() {
  return pkgVersion("lit");
}

export function renderRows(rows) {
  let out = "";
  for (const chunk of render(page(rows))) {
    out += chunk;
  }
  return out;
}
