/**
 * `solid-js/web` の低水準 SSR プリミティブ（`ssrElement`/`renderToString`）
 * による SSR renderer。
 *
 * Solid は通常 `babel-preset-solid` による JSX コンパイルを前提とし、
 * コンパイラがテキスト補間へ自動的に `escape()` 呼び出しを挿入する。本
 * renderer は JSX 不使用のため、コンパイラが生成するのと同じ呼び出し形
 * （`ssrElement(tag, props, children)` + テキスト子を `escape()` へ通す）を
 * 手で組み立てることで、Solid の「既定エスケープ経路」を JSX なしで再現
 * する。`solid-js/web` の `resolveSSRNode` は文字列をそのまま返す
 * （エスケープしない）ため、`escape()` を明示的に呼ばないとエスケープされ
 * ない点に注意（Solid コンパイラが行う変換を手動で肩代わりしている）。
 *
 * `solid-js/web` は `exports` の `node` 条件で SSR 版実装
 * （`web/dist/server.js`）へ解決される（Node.js は import 時に `node`
 * 条件を自動で有効化するため、追加設定は不要）。
 */
import { renderToString, ssrElement, escape } from "solid-js/web";
import { rowLabel } from "../lib/label.mjs";
import { pkgVersion } from "../lib/version.mjs";

function buildPage(rows) {
  const trs = [];
  for (let i = 0; i < rows; i += 1) {
    trs.push(
      ssrElement("tr", {}, [
        ssrElement("td", {}, escape(String(i))),
        ssrElement("td", {}, escape(rowLabel(i))),
      ]),
    );
  }
  return ssrElement("html", {}, [
    ssrElement("body", {}, [
      ssrElement("header", {}, ssrElement("h1", {}, escape("Benchmark"))),
      ssrElement("table", { id: "bench-table" }, ssrElement("tbody", {}, trs)),
      ssrElement("footer", {}, ssrElement("p", {}, escape(`generated ${rows} rows`))),
    ]),
  ]);
}

export const name = "solid";

export function getVersion() {
  return pkgVersion("solid-js");
}

export function renderRows(rows) {
  return renderToString(() => buildPage(rows));
}
