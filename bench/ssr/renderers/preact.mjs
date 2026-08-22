/**
 * preact-render-to-string による SSR renderer。
 *
 * JSX は不使用（`h()` を直接呼ぶ）。テキストは `h()` の子要素として渡し、
 * preact の既定エスケープ経路のみを経由する（`dangerouslySetInnerHTML` は
 * 使わない）。
 */
import { h } from "preact";
import render from "preact-render-to-string";
import { rowLabel } from "../lib/label.mjs";
import { pkgVersion } from "../lib/version.mjs";

function Page(rows) {
  const trs = [];
  for (let i = 0; i < rows; i += 1) {
    trs.push(h("tr", { key: i }, h("td", null, i), h("td", null, rowLabel(i))));
  }
  return h(
    "html",
    null,
    h(
      "body",
      null,
      h("header", null, h("h1", null, "Benchmark")),
      h("table", { id: "bench-table" }, h("tbody", null, trs)),
      h("footer", null, h("p", null, `generated ${rows} rows`)),
    ),
  );
}

export const name = "preact";

export function getVersion() {
  return pkgVersion("preact");
}

export function renderRows(rows) {
  return render(Page(rows));
}
