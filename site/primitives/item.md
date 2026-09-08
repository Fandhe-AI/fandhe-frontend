# Item

media（アイコン・画像・アバター）+ title/description + actions からなる汎用リスト行を表現する shadcn/ui Item 相当の部品です（参照軸はイシュー #2001、shadcn/ui 追加はイシュー #2004）。`fandhe-frontend-headless-ui` の `item` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `media` / `content` / `title` / `description` / `actions` / `header` / `footer` / `group` / `separator` の 10 anatomy パーツを持ちます。

`root` は `href` を指定すると `div` ではなく `a` として描画されます。`external=true` を渡すと `target="_blank"` と `rel="noopener noreferrer"` を不可分に付与します（reverse tabnabbing 対策）。`variant`（`default` / `outline` / `muted`）・`size`（`default` / `sm`）を `data-variant` / `data-size` として固定出力します。

**アクセシビリティ**

- `group` は `role="group"` を固定付与します。shadcn/ui の `ItemGroup` は `role="list"` ですが、`a[href]` は WAI-ARIA 上 `listitem` ロールを持てず `list`/`listitem` 対を成立させられないため、意図的に `role="group"` へ差分化しています。`label` を渡すと `aria-label` を出力します（空文字列のときは省略します）。
- `separator` は `role="separator"` + `aria-orientation="horizontal"` を固定付与します。`group` は常に縦並びのコンテナのため、水平固定です。
- `root` が `a` として描画されるときのみ、キーボード操作はネイティブ `a[href]` の `Tab`/`Shift+Tab`/`Enter` に依存します。`div` のときはキー操作を提供しません。`role` は付与せず、`a` の暗黙の `link` ロールに委ねます。

現時点では `fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品はありません（今後追加予定）。

自前 CSS の最小例:

```css
[data-scope="item"][data-part="root"] {
  display: flex;
  gap: 0.75rem;
  padding: 0.75rem;
  border-radius: 0.5rem;
}
[data-scope="item"][data-part="root"][data-variant="outline"] {
  border: 1px solid currentColor;
}
[data-scope="item"][data-part="separator"] {
  border-top: 1px solid currentColor;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
