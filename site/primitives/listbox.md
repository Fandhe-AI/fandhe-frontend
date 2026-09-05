# Listbox

単一/複数選択の一覧から選ばせる、常時展開のリスト部品です（ポップオーバー型の [Combobox](combobox.md)/Select と異なり、`content` パーツ自身がフォーカスを受けます）。`multiple` の有無で `aria-multiselectable`・単一/複数選択のセマンティクスを切り替えます。

`fandhe-frontend-headless-ui` の `listbox` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

`ListboxProps`（`disabled`/`orientation`）が Root/Label/Content/ItemGroup/Item/ValueText へ `data-disabled`（Root 由来）・`data-orientation`（Root/Content/ItemGroup/Item）を一律付与します。Root の `disabled` は Item の有効 disabled（`props.disabled || Item 個別の disabled`）へ伝播します（参考実装の zag.js Listbox の `getItemState` と同じ挙動）。Item は選択時のみ `data-selected` 存在属性を追加し、`data-state` は `"open"`/`"closed"` の既存語彙を維持します（Themes recipe がこの語彙に依存するため）。ItemText は Item の先頭 3 引数（`selected_state`/`disabled`/`highlighted`）と同型の `data-state`/`data-disabled`/`data-highlighted` を出力します。ItemGroupLabel は `role="presentation"` を、ItemIndicator は `aria-hidden="true"` を固定付与します。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| ArrowDown / ArrowUp | Content | 次/前の Item へ highlight を移動する（既定・非循環。`fandhe-frontend-wasm-full` の keynav で配線済み） |
| ArrowRight / ArrowLeft | Content（`data-orientation="horizontal"` のときのみ） | 次/前の Item へ highlight を移動する |
| Home / End | Content | 先頭/末尾の非 disabled Item へ highlight を移動する |
| 文字キー | Content | typeahead。連続入力したキーに前方一致する非 disabled Item へ highlight を移動する |
| Enter / Space | Content | typeahead バッファ非活性時、highlight 中の非 disabled Item へ click を合成する（選択状態を書き換える dispatch へは未接続。別イシューで対応予定） |
| Escape | Content | typeahead バッファのリセットのみを行う（選択解除は行わない。ダイアログ内 Listbox が親の Escape 閉鎖を奪わないための意図的な非対称） |

ポインタ・キーボード操作の実際の DOM 配線は `fandhe-frontend-wasm-full` の `keynav` モジュールが担います。Enter/Space による決定は highlight 中の項目への click 合成までで、選択状態を書き換える dispatch（`"select"`/`"toggle"`）への接続は本モジュールのスコープ外です。

スタイル済みの表示例は [Listbox](../themes/listbox.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="listbox"][data-part="..."]` セレクタでスタイルを当てます。以下は選択・highlight・disabled 状態、横並びレイアウト、root disabled の最小例です。

```css
[data-scope="listbox"][data-part="item"][data-selected] {
  background: #dbeafe;
}

[data-scope="listbox"][data-part="item"][data-highlighted] {
  outline: 2px solid #2563eb;
}

[data-scope="listbox"][data-part="item"][data-disabled] {
  opacity: 0.5;
}

[data-scope="listbox"][data-part="content"][data-orientation="horizontal"] {
  flex-direction: row;
}

[data-scope="listbox"][data-part="root"][data-disabled] {
  opacity: 0.5;
}
```
