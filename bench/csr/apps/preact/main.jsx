// Preact 10 による CSR ベンチアプリ。状態（rows 配列）をモジュール変数に
// 置き、各操作で props 駆動のトップレベル render() を呼び直す構成にする。
// preact のトップレベル render() は同期的に diff・コミットするため、
// 呼び出しが返った時点で DOM 反映済みであり、計測境界（PROTOCOL §2.2 の
// 「__bench[op]() 完了時点で DOM 反映済み」）を本番 API のみで満たせる。
//
// React 版（useState + flushSync）と実装形が非対称になるが、flushSync は
// react-dom の本番 API である一方、preact で hooks の setState を同期
// フラッシュする唯一の公式手段 act() は preact/test-utils（テスト専用
// モジュール）であり、production bundle へテストランタイムが混入して
// CSR 実行時間・payload サイズ比較の双方で Preact だけが不要なコードを
// 負担する（PROTOCOL §4 の production 相当・同一条件契約に反する。
// PR #1370 codex レビュー P1 指摘）。production bundle 純度を優先し、
// hooks / test-utils を使わない本構成を採用した。
import { render } from "preact";
import { generateRows, updateRows } from "../../common/rowData.mjs";

// 現在の表示状態。各操作がこれを更新してから rerender() を呼ぶ。
let rows = [];

// props 駆動の関数コンポーネント。keyed 描画（key={row.id}）は維持する
// （キー付きリスト描画を持つフレームワークは id をキーに使う、PROTOCOL §2.2）。
function Rows({ rows }) {
  return rows.map((row) => (
    <tr key={row.id}>
      <td>{row.id}</td>
      <td>{row.label}</td>
    </tr>
  ));
}

function main() {
  const tbody = document.querySelector("#bench-table tbody");
  // トップレベル render() は前回の vnode ツリーとの diff を同期実行して
  // コミットする（初回マウント含む）。setState のような rAF/マイクロ
  // タスクへのデバウンスを経由しないため、追加の同期化手段は不要。
  const rerender = () => render(<Rows rows={rows} />, tbody);
  rerender();

  const create = () => {
    rows = generateRows(1000);
    rerender();
  };
  const update = () => {
    rows = updateRows(rows);
    rerender();
  };
  const clear = () => {
    rows = [];
    rerender();
  };

  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
