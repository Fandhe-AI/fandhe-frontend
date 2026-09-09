# Command

検索入力 + 絞り込み済みリスト + グループ + ショートカット表示 + dialog 内表示を組み合わせたコマンドパレット相当の shadcn/ui Command（cmdk 由来）部品です（参照軸はイシュー #2001、shadcn/ui 追加はイシュー #2004）。`fandhe-frontend-headless-ui` の `command` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `input` / `list` / `empty` / `group` / `group-heading` / `item` / `shortcut` / `separator` / `dialog` の 10 anatomy パーツを持ちます。

既存の `combobox`（ARIA 1.2 combobox パターン）・`listbox`（`role="listbox"`/`role="option"`）の ARIA 実装を再利用し、新規の意味論は持ち込みません。`Command` 状態機械（`Disclosure` + `TextInput` + `SingleSelect` を合成）は cmdk のコマンドパレット意味論に合わせ、入力（`Input`）は dialog を開閉せず、選択（`Select`）も dialog を閉じません（実行フックはアプリケーションロジックとして UI コンポーネント層の範囲外です）。

**アクセシビリティ**

- `input` は `role="combobox"` + `aria-controls`（必須引数）+ `aria-autocomplete="list"` を固定付与します。`aria-controls` を必須引数にすることで、`aria-controls`/`aria-expanded` の関連付けを型で構造的に満たします。
- `list` は `role="listbox"` を固定付与し、`id`/`aria-label` を必須引数にすることでアクセシブルネームを型で強制します。
- `item` は `role="option"` + `aria-selected` + `data-selected`（presence）で選択有無を表します。`data-highlighted`/`data-state` は出力しません（`combobox`/`listbox` との意図的な差分。`aria-activedescendant` が指すのは「確定選択中の item」であり「キーボードでハイライト中の行」ではないため）。
- `dialog` は `role="dialog"` + `aria-modal="true"` + `tabindex="-1"` を固定付与し、`label` が空文字列でないときのみ `aria-label` を付与します。closed のとき `hidden` 存在属性を付与します。
- `group` は `labelledby` が `Some` のときのみ `role="group"` + `aria-labelledby` を出力します。`group-heading` の `id` が参照先になります。
- `root`/`list`/`empty` はいずれも `empty: bool` 引数を取り、絞り込み結果 0 件のとき `data-empty` を出力します。表示切替（`hidden` 等）は呼び出し側または pre-styled-ui の CSS の責務です。
- キーボード操作（矢印キー・Enter・Escape・Cmd/Ctrl+K）の実 DOM 配線は `fandhe-frontend-wasm-full` の `command` モジュールが実装済みです（入力絞り込み・行選択・実行フック・dialog 開閉、イシュー #2069）。

自前 CSS の最小例:

```css
[data-scope="command"][data-part="root"][data-empty] {
  min-height: 4rem;
}
[data-scope="command"][data-part="item"][data-selected] {
  background: #eef;
}
[data-scope="command"][data-part="item"][data-disabled] {
  opacity: 0.5;
}
[data-scope="command"][data-part="dialog"][hidden] {
  display: none;
}
```

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Command](../themes/command.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
