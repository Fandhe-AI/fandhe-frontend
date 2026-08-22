/**
 * react-dom/server の `renderToString` による SSR renderer。
 *
 * JSX は不使用（`React.createElement` を直接呼ぶ）。テキストは全て
 * `createElement` の子要素として渡し、React の既定エスケープ経路のみを
 * 経由する（`dangerouslySetInnerHTML` は使わない）。
 */
import { createElement as h } from "react";
import { renderToString } from "react-dom/server";
import { rowLabel } from "../lib/label.mjs";
import { pkgVersion } from "../lib/version.mjs";

function Page({ rows }) {
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

export const name = "react";

export function getVersion() {
  return pkgVersion("react");
}

export function renderRows(rows) {
  return renderToString(h(Page, { rows }));
}
