<script>
  // Svelte 5 の runes ($state) で行データを保持し、create/update/clear を
  // export function として公開する。mount() が返すインスタンスからこれらの
  // 関数を呼べる（Svelte 5 のコンポーネントエクスポート機構）。
  import { generateRows, updateRows } from "../../common/rowData.mjs";

  let rows = $state([]);

  export function create() {
    rows = generateRows(1000);
  }

  export function update() {
    rows = updateRows(rows);
  }

  export function clear() {
    rows = [];
  }
</script>

{#each rows as row (row.id)}
  <tr>
    <td>{row.id}</td>
    <td>{row.label}</td>
  </tr>
{/each}
