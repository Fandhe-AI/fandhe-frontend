# Select

`fandhe-frontend-headless-ui` の `select` mod が提供するリストボックス選択の unstyled 部品です。Root / Label / Control / Trigger / ValueText / ClearTrigger / Indicator / Positioner / Content / ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator / HiddenSelect の 15 anatomy パーツを持ち、listbox の開閉と選択値（高々 1 個）を合成した状態機械を備えます。`HiddenSelect` はフォーム統合専用のネイティブ `<select>` です。

`/themes/` 側の `Select`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`SelectProps`（`disabled`/`readonly`/`invalid`/`required`）は root/label/control/trigger/value-text/clear-trigger/indicator/item-group へ一律で `data-disabled`/`data-invalid`/`data-readonly` を付与します（`data-required` は label のみに付与）。`trigger` はさらに `props.disabled` のときネイティブ `disabled` 存在属性を、`clear-trigger` は `props.disabled` のときネイティブ `disabled` を持ちます。`hidden-select` は `props.disabled`/`props.required` をそれぞれネイティブ `disabled`/`required` 存在属性へ反映します（`readonly` は `<select readonly>` が無効な HTML のため反映しません）。

`trigger` は選択なしのとき `data-placeholder-shown` を持ちます（`value-text` と同じ判定）。`item` は root disabled が伝播した有効値（`props.disabled || 個別の disabled`）で `data-disabled`/`aria-disabled` を出力し、選択中のみ `data-selected` を追加で持ちます。`item-text` は `item` と同じ 3 状態（選択有無・disabled・highlighted）を `data-state`/`data-disabled`/`data-highlighted` として持ちます。`item-group-label` は `role="presentation"` を、`item-indicator` は `aria-hidden="true"` を常時持ちます。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Space / Enter（closed） | Trigger | listbox を開きます。選択済み項目（非 disabled）があれば初期 highlight をそこへ合わせ、無ければ先頭項目へ合わせます。 |
| ArrowDown（closed） | Trigger | listbox を開き、先頭の非 disabled 項目（選択済み項目があればそちらを優先）へ初期 highlight を合わせます。 |
| ArrowUp（closed） | Trigger | listbox を開き、末尾の非 disabled 項目（選択済み項目があればそちらを優先）へ初期 highlight を合わせます。 |
| ArrowDown / ArrowUp（open） | Content | highlight を次/前の非 disabled 項目へ移動します。`data-loop-focus="true"` オプトイン時のみ端で循環します。 |
| Space / Enter（open） | Content | highlight 中の項目を選択し、listbox を閉じます（`closeOnSelect` 既定）。 |
| Home / End（open） | Content | 先頭/末尾の非 disabled 項目へ highlight を移動します（WAI-ARIA APG の拡張）。 |
| A–Z 等（typeahead） | Trigger / Content | 先頭一致する項目へ highlight を移動します（350ms バッファ）。closed 時は開いてから移動します。 |
| Escape | Content | listbox を閉じます。 |

`data-readonly` が付いた trigger は上記のいずれのキーも no-op です（click も同様）。実 DOM 配線は `fandhe-frontend-wasm-full` の `keynav`（`handle_menu_or_select_trigger_keydown`/`initial_highlight_index`）が担いますが、JS なしでも `hidden-select`（ネイティブ `<select>`）がフォーム送信そのものは成立させます。

**参考サイトとの差分**

ark-ui 公式 Data Attributes / Keyboard Support 表（zag `select.connect.ts`）・Radix Primitives `select` と突合し、`SelectProps` の一律付与・`data-placeholder-shown`（trigger）・root disabled 伝播・`data-selected`・item-text の 3 状態属性・`item-group-label` の `role="presentation"`・`item-indicator` の `aria-hidden`・readonly ガードを是正しました。一方、以下は意図的に合わせていません。

- `data-state` の `checked`/`unchecked` 語彙: ark-ui は item に `checked`/`unchecked` を使いますが、本リポジトリでは `crate::state::OpenState` の `"open"`/`"closed"` に一元化しています（combobox/listbox とのクレート横断整合を優先）。
- Radix 固有の `Portal`/`Viewport`/`ScrollUpButton`/`ScrollDownButton`: レイアウト計測・DOM 配置の関心のため headless へ持ち込みません（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- `Arrow`: Select は arrow を持たない構成として確定済みです（positioning ADR §4.2）。
- `Icon`/`Value`（Radix）: `Indicator`/`ValueText` と同義のため個別追加しません。
- `data-focus`（DOM ローカル focus）: SSR 静的出力に持たせません（`combobox`/`listbox` と同じ契約）。
- trigger の combobox 化（`role="combobox"` + `aria-activedescendant`）: 現行 anatomy（素の `button`）を維持し、select-only combobox パターンは別部品（`combobox`）の責務とします。
- multiple 選択: 高々 1 個の選択のみを扱います。

スタイル済みの表示例は [Select](../themes/select.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="select"][data-part="..."]` セレクタでスタイルを当てます。以下は trigger の開閉・invalid・readonly・プレースホルダー表示、highlight 中の item の見た目の最小例です。

```css
[data-scope="select"][data-part="trigger"] {
  border: 1px solid #ccc;
  border-radius: 6px;
  padding: 0.5rem 0.75rem;
}

[data-scope="select"][data-part="trigger"][data-state="open"] {
  border-color: #6366f1;
}

[data-scope="select"][data-part="trigger"][data-placeholder-shown] {
  color: #6b7280;
}

[data-scope="select"][data-part="trigger"][data-invalid] {
  border-color: #dc2626;
}

[data-scope="select"][data-part="trigger"][data-readonly] {
  background: #f3f4f6;
  cursor: default;
}

[data-scope="select"][data-part="item"][data-highlighted] {
  background: #eef2ff;
}
```
