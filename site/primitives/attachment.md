# Attachment

添付ファイル 1 件の表示を表現する shadcn/ui Attachment 相当の部品です（参照軸はイシュー #2001）。`fandhe-frontend-headless-ui` の `attachment` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `media` / `content` / `name` / `meta` / `progress` / `actions` / `action` の 8 anatomy パーツを持ちます。既存の File Upload（選択・ドロップの入力側部品）とは異なり、本部品は「表示側」を担います。状態機械を持たない静的部品です。

`root` は `variant`（`file` / `image`）・`state`（`idle` / `uploading` / `error`）・`disabled` の props を受け取り、それぞれ `data-variant` / `data-state` / `data-disabled`（存在属性）として出力します。会話系部品（message / bubble / attachment / marker）が共有する `data-role` / `data-align` は意図的に持ちません。添付ファイルは常に message / bubble の `content` スロット内に置かれ、整列はその親から継承する設計です。

`media` は画像プレビューまたは種別アイコンを children として受けるスロットで、独自の `data-variant` は持ちません（`[data-variant="image"] [data-part="media"]` の子孫セレクタで分岐します）。`content` は `name` / `meta` を束ねるスロットです。`name` / `meta` は整形済み文字列を children で受け取るだけのスロットであり、byte → KB 変換等の数値・単位整形は行いません（数値・日時整形は UI コンポーネント層の責務外という既定判断の系です）。

`progress` は attachment scope の単純なスロットです。`fandhe-frontend-headless-ui` の `Progress`（`data-scope="progress"`）のパーツ（`root` / `track` / `range` 等）をそのまま入れ子にする契約で、両 scope は独立して残ります。`actions` はアクションボタン群を束ねるコンテナで、`action` は削除等の個別アクション（ネイティブ `<button type="button">`）です。`label` を渡すと `aria-label` を出力し（空文字列のときは省略します）、`disabled` はネイティブ `disabled` と `data-disabled` の両方に反映します。削除処理そのものはアプリケーション側の責務です。

**アクセシビリティ**

- `action` は `type="button"` を固定付与し、フォーム内配置時の意図しない submit を防ぎます。
- `action` の `label` が空文字列でないときのみ `aria-label` を出力します。アイコンのみのボタンでは必ず `label` を渡してください。
- `action` の `disabled` はネイティブ `disabled`（ブラウザ標準の Space/Enter 抑止）と `data-disabled` の両方に反映します。

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品は現時点でまだありません（Themes 版は後続イシューで追加予定です）。

自前 CSS の最小例:

```css
[data-scope="attachment"][data-part="root"] {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
}
[data-scope="attachment"][data-part="root"][data-disabled] {
  opacity: 0.5;
}
[data-scope="attachment"][data-part="root"][data-state="error"] {
  border-color: #dc2626;
}
[data-scope="attachment"][data-part="content"] {
  display: flex;
  flex-direction: column;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
