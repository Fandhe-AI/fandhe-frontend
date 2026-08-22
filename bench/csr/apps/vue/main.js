// Vue 3 による CSR ベンチアプリ。SFC（*.vue）は使わず、createApp + h()（render
// 関数）のみで構成する（ビルドにテンプレートコンパイラを要さない構成）。
import { createApp, h, nextTick } from "vue";
import { generateRows, updateRows } from "../../common/rowData.mjs";

function main() {
  const tbody = document.querySelector("#bench-table tbody");

  const app = createApp({
    data() {
      return { rows: [] };
    },
    render() {
      return this.rows.map((row) => h("tr", { key: row.id }, [h("td", String(row.id)), h("td", row.label)]));
    },
  });

  const vm = app.mount(tbody);

  // Vue のリアクティブ更新は次の DOM 更新サイクルまで非同期に遅延される
  // ため、__bench 呼び出し完了時点で DOM 反映済みを保証するには
  // nextTick() の解決を待つ必要がある（計測境界の変更、
  // bench/PROTOCOL.md §2.2 参照）。
  const create = () => {
    vm.rows = generateRows(1000);
    return nextTick();
  };
  const update = () => {
    vm.rows = updateRows(vm.rows);
    return nextTick();
  };
  const clear = () => {
    vm.rows = [];
    return nextTick();
  };

  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
