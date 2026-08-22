// React 19 による CSR ベンチアプリ。
// createRoot + useState のみで構成し、外部 store は使わない。
// tbody 要素へ直接マウントし、Rows コンポーネントは <tr> の配列（フラグメント）を
// 返す（React はキー付き配列を root children として直接サポートする）。
import { createRoot } from "react-dom/client";
import { flushSync } from "react-dom";
import { useState } from "react";
import { generateRows, updateRows } from "../../common/rowData.mjs";

// benchmark harness（window.__bench）から状態更新を呼べるように、
// 唯一のコンポーネントインスタンスの setState を外側へ退避する。
// useState の setter はレンダー中も同期的に取得できる安定参照であるため、
// useEffect を経由せず render 本体で直接代入する（effect のコミット後
// 実行を待つ非同期な隙間をなくし、mount 直後の __bench 呼び出しが
// 確実に setter を掴めるようにするための意図的な設計）。
let setRowsExternal;

function Rows() {
  const [rows, setRows] = useState([]);
  setRowsExternal = setRows;
  return rows.map((row) => (
    <tr key={row.id}>
      <td>{row.id}</td>
      <td>{row.label}</td>
    </tr>
  ));
}

function main() {
  const tbody = document.querySelector("#bench-table tbody");
  const root = createRoot(tbody);
  // 初回マウントも flushSync で包む。createRoot(...).render() は同期コミットを
  // 保証しないため、これを省くとハーネスが window.__bench の存在を検知した
  // 直後に呼ぶ最初の clear() の時点で Rows がまだ一度も実行されておらず、
  // setRowsExternal が undefined のままになり得る（flushSync(() =>
  // setRowsExternal(...)) が TypeError になる不具合、PR #1370 レビュー指摘）。
  flushSync(() => root.render(<Rows />));

  // React 19 は createRoot 配下の setState を自動バッチングするため、
  // __bench 呼び出し完了時点で DOM 反映済みであることを保証するには
  // flushSync で同期コミットさせる必要がある（計測境界の変更、
  // bench/PROTOCOL.md §2.2 参照）。
  const create = () => flushSync(() => setRowsExternal(generateRows(1000)));
  const update = () => flushSync(() => setRowsExternal((prev) => updateRows(prev)));
  const clear = () => flushSync(() => setRowsExternal([]));

  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
