/**
 * ベースライン renderer（フレームワーク非依存）。
 *
 * 手書きの最小エスケープ関数 + 文字列連結のみで SSR HTML を組み立てる。
 * 依存パッケージを持たないため `getVersion()` は Node 自身のバージョンを
 * 返す（`bench/ssr/run_ssr.mjs` の `--framework vanilla` で単独実行可能）。
 * 各フレームワークの render 呼び出しコストを相対評価する基準点として使う。
 */
import { rowLabel } from "../lib/label.mjs";

const ESCAPE_MAP = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

function escapeHtml(s) {
  return s.replace(/[&<>"']/g, (c) => ESCAPE_MAP[c]);
}

function renderRow(i) {
  return `<tr><td>${escapeHtml(String(i))}</td><td>${escapeHtml(rowLabel(i))}</td></tr>`;
}

export const name = "vanilla";

export function getVersion() {
  return process.version;
}

export function renderRows(rows) {
  let rowsHtml = "";
  for (let i = 0; i < rows; i += 1) {
    rowsHtml += renderRow(i);
  }
  return (
    "<html><body><header><h1>Benchmark</h1></header>" +
    `<table id="bench-table"><tbody>${rowsHtml}</tbody></table>` +
    `<footer><p>generated ${escapeHtml(String(rows))} rows</p></footer></body></html>`
  );
}
