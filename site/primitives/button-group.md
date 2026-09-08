# Button Group

関連ボタンを角丸・境界線で連結してひとつのグループに見せる shadcn/ui Button Group 相当の部品です（参照軸はイシュー #2001、shadcn/ui 追加はイシュー #2004）。`fandhe-frontend-headless-ui` の `button_group` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root`（`role="group"`）/ `separator` / `text` の 3 anatomy パーツを持ちます。

`toolbar` mod の roving tabindex（矢印キーでフォーカスが移動する複合ウィジェット）とは異なり、`button_group` は **静的なグループ化**です。子ボタンのフォーカス順序はネイティブの `Tab` 順序に委ね、状態機械を持ちません。ネスト（グループの中に別のグループ）も許容します。

**アクセシビリティ**

- `root` は `role="group"` を固定付与します。WAI-ARIA は `group` ロールへの `aria-orientation` を許可していないため、`aria-orientation` は付与しません。向きは `data-orientation`（`"horizontal"` / `"vertical"`）のみで表現します。
- `label` を渡すと `aria-label` を出力します（空文字列のときは省略します）。
- `separator` は `role="separator"` を固定付与し、グループ自身の向きと**直交**する `aria-orientation`/`data-orientation` を出力します（横並びグループの区切り線は縦線になるため `vertical`）。
- キーボード操作はネイティブ `button` 要素の `Tab`/`Shift+Tab` によるフォーカス移動のみに依存します。独自キーハンドラは持ちません。
- 先頭/末尾ボタンの角丸連結は本 mod の責務外です。CSS の `:first-child`/`:last-child` セレクタで表現します（装飾を headless-ui へ持ち込まない責務境界の判断）。

自前 CSS の最小例:

```css
[data-scope="button-group"][data-part="root"] {
  display: inline-flex;
}
[data-scope="button-group"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}
[data-scope="button-group"][data-part="root"] > button:first-child {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}
[data-scope="button-group"][data-part="root"] > button:last-child {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}
```

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Button Group](../themes/button-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
