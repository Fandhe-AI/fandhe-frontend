// Preact 10 による CSR ベンチアプリ。React 版と対称的な構成（preact/hooks の
// useState + preact の render）にし、フレームワークオーバーヘッドの差分のみが
// 計測結果へ現れるようにする。
import { render } from "preact";
import { useState } from "preact/hooks";
import { act } from "preact/test-utils";
import { generateRows, updateRows } from "../../common/rowData.mjs";

// React 版と同様、useEffect を経由せず render 本体で setter を直接退避する
// （effect コミット後実行を待つ非同期な隙間をなくすため）。
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
  // React 版と異なり、preact の render() は初回マウントも含めて常に同期的
  // にコンポーネント関数を実行してコミットする（React の createRoot の
  // ような並行スケジューラを持たない）。そのため setRowsExternal はこの
  // 呼び出しが返った時点で必ず代入済みであり、PR #1370 レビュー指摘の
  // React 側の初期化順序問題（flushSync で初回マウントを包む必要が
  // あった件）は preact には存在しない（確認済み・追加対応不要）。
  render(<Rows />, tbody);

  // Preact の setState はデフォルトで rAF/マイクロタスクにデバウンスされ
  // 非同期に描画されるため、__bench 呼び出し完了時点で DOM 反映済みを
  // 保証するには同期フラッシュが要る。preact/test-utils の act() は
  // options.debounceRendering を一時的に同期化して pending rerender を
  // 即時ドレインする（内部的には preact 自身が提供する唯一の公式な
  // 同期フラッシュ手段）。hooks + useState の構成をそのまま維持でき、
  // React 版と対称的な実装を保てるため、component ツリーを直接
  // render() し直す構成（フックを使わない案）よりこちらを採用した
  // （計測境界の変更、bench/PROTOCOL.md §2.2 参照）。
  const create = () => act(() => setRowsExternal(generateRows(1000)));
  const update = () => act(() => setRowsExternal((prev) => updateRows(prev)));
  const clear = () => act(() => setRowsExternal([]));

  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
