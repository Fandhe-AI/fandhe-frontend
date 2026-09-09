# Message

AI チャット UI の「会話 1 発言」を表現する shadcn/ui Message 相当の部品です（参照軸はイシュー #2001、shadcn/ui 追加はイシュー #2004）。`fandhe-frontend-headless-ui` の `message` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `avatar` / `header` / `content` / `footer` / `group` の 6 anatomy パーツを持ちます。状態機械を持たない静的部品であり、応答待ち・送信失敗の判定や再送、ストリーミング更新はアプリ側の責務です。

`root` は `role`（`user` / `assistant` / `system`）・`align`（`start` / `end`）・`loading`（`bool`）・`error`（`bool`）の props を受け取り、それぞれ `data-role` / `data-align` / `data-loading` / `data-error` として出力します。`data-align` は `data-role` から自動導出されない独立した軸で、既定は `start` です。`data-loading` / `data-error` は存在属性（値が真のときのみ出力）で、応答待ち・送信失敗の表示のみを担います。この 4 語彙は会話系部品（message / bubble / attachment / marker）が共有する共通語彙として `message` mod が最初に確定したものです。

`avatar` はスロットで、既存の `avatar` mod を内部で呼び出しません。呼び出し側が中に `avatar::root` / `avatar::image` / `avatar::fallback` を自由に組み込めます。`content` も同様にスロットで、Markdown レンダリング結果等の中身は利用者側で用意します。同じ発言者の連続発言は `group` でまとめられます（先頭以外の `avatar` を省略する見た目は CSS 側の責務です）。

**アクセシビリティ**

- `root` は `role="listitem"` を固定付与します。
- `group` は `role="list"` を固定付与します。WAI-ARIA の `listitem` は `list`（または同等のコンテナ）を required context として要求するため、会話全体は「発言者ターンごとの `group`（list）の並び」として読み上げられる想定です。`label` を渡すと `aria-label` を出力します（空文字列のときは省略します）。`group` を介さず `root` 単体で使う場合は、呼び出し側が `ul` / `role="list"` コンテナへ置いてください。
- `aria-live` / `aria-busy` は付与しません。ストリーミング応答の通知や応答待ちの読み上げはアプリケーション固有の UX 判断であり、本部品の責務外です。

Themes（`fandhe-frontend-pre-styled-ui`）に対応するスタイル済み部品は今後追加予定です（イシュー #2106）。

自前 CSS の最小例:

```css
[data-scope="message"][data-part="root"] {
  display: flex;
  flex-direction: column;
  max-width: 32rem;
}
[data-scope="message"][data-part="root"][data-align="end"] {
  margin-left: auto;
}
[data-scope="message"][data-part="content"] {
  padding: 0.5rem 0.75rem;
  border-radius: 0.75rem;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
