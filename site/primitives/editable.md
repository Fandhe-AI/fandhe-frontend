# Editable

クリック（または表示切替）でテキストを「表示専用」から「編集可能」へ切り替える inline 編集部品です。preview/edit の 2 モードを anatomy 全体で切り替え、確定・キャンセルの 2 操作をトリガーとして提供します。

`fandhe-frontend-headless-ui` の `editable` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・`data-*` のみを提供します。CSS は利用者が当てます。

`EditableInputFlags`（`disabled`/`readonly`/`required`/`invalid`）を Root/Label/Area/Preview/Input へ共通で渡します。反映先はパートごとに異なり、Root は `data-disabled`/`data-readonly` のみ、Label は `data-disabled`/`data-invalid`/`data-required`、Area は `data-disabled`、Preview は `data-disabled`/`data-readonly`/`data-invalid`・`aria-disabled="true"`（disabled 時のみ）・`aria-invalid="true"`（invalid 時のみ）・`tabindex="0"`（`!disabled && !readonly` のときのみ）、Input はネイティブ `disabled`/`readonly`/`required`・`data-disabled`/`data-readonly`/`data-required`/`data-invalid`・`aria-invalid="true"`（invalid 時のみ）を出力します。`data-focus`（実行時のフォーカス状態）・`data-autoresize`（レイアウト計測）・`aria-readonly`（role なし `span` への付与が ARIA in HTML 上不正）は意図的に出力しません。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter | Input（edit 中） | 確定して edit を抜ける（dispatch `"submit"`。`submit_mode` が `enter`/`both` のときのみ確定する想定） |
| Escape | Input（edit 中） | 取り消して edit を抜ける（dispatch `"cancel"`） |
| Tab | Preview（`activation_mode="focus"`） | `tabindex="0"`（`!disabled && !readonly` のときのみ）によりキーボードで到達できる |
| Space / Enter | EditTrigger / SubmitTrigger / CancelTrigger | ネイティブ `<button type="button">` の標準操作 |

ポインタ・キーボード操作の実際の DOM 配線（`fandhe-frontend-wasm-full`）は本イシューのスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

スタイル済みの表示例は [Editable](../themes/editable.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="editable"][data-part="..."]` セレクタでスタイルを当てます。以下は preview のテキストらしさ・フォーカスリング・placeholder 表現、input の invalid 状態、root の disabled 状態の最小例です。

```css
[data-scope="editable"][data-part="preview"] {
  cursor: text;
  border-bottom: 1px dashed transparent;
}

[data-scope="editable"][data-part="preview"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="editable"][data-part="preview"][data-placeholder-shown] {
  color: #9ca3af;
  font-style: italic;
}

[data-scope="editable"][data-part="input"][data-invalid] {
  border-color: #dc2626;
}

[data-scope="editable"][data-part="root"][data-disabled] {
  opacity: 0.5;
}
```
