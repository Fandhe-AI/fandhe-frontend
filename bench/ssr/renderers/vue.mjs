/**
 * `vue`（`createSSRApp` + `h()`）+ `vue/server-renderer`（`renderToString`）
 * による SSR renderer。
 *
 * テンプレートコンパイル（`.vue` SFC / テンプレート文字列コンパイル）は
 * 不使用で、`h()` を直接呼ぶ Render Function API のみを使う。テキストは
 * `h()` の子要素として渡し、Vue の既定エスケープ経路のみを経由する
 * （`v-html` は使わない）。`renderToString` は非同期（`Promise<string>`）。
 */
import { createSSRApp, h } from "vue";
import { renderToString } from "vue/server-renderer";
import { rowLabel } from "../lib/label.mjs";
import { pkgVersion } from "../lib/version.mjs";

function buildApp(rows) {
  return createSSRApp({
    render() {
      const trs = [];
      for (let i = 0; i < rows; i += 1) {
        trs.push(h("tr", [h("td", String(i)), h("td", rowLabel(i))]));
      }
      return h("html", [
        h("body", [
          h("header", [h("h1", "Benchmark")]),
          h("table", { id: "bench-table" }, [h("tbody", trs)]),
          h("footer", [h("p", `generated ${rows} rows`)]),
        ]),
      ]);
    },
  });
}

export const name = "vue";

export function getVersion() {
  return pkgVersion("vue");
}

export function renderRows(rows) {
  return renderToString(buildApp(rows));
}
