# Message Scroller

AI チャット UI の会話ログを収めるスクロールコンテナを表現する shadcn/ui Message Scroller 相当の部品です（参照軸はイシュー #2001）。`fandhe-frontend-headless-ui` の `message_scroller` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `viewport` / `content` / `anchor` / `jump_to_latest` / `load_more` の 6 anatomy パーツを持ちます。[Message](./message.md) と同じく状態機械を持たない静的部品です。

**責務境界**

最下部追従・新着検知・スクロール位置の計測や復元は一切内包しません。SSR は常に「最下部に居る」初期状態（`data-stuck="bottom"`）を決定的に描画し、実行時のスクロール位置計測・自動追従・新着到着の検知は `fandhe-frontend-wasm-full` 側の配線が担います（本部品自体の責務ではありません）。

`root` は `stuck`（`bottom` / `free`）・`has_new`（`bool`）の props を受け取り、それぞれ `data-stuck` / `data-has-new` として出力します。`data-has-new` は存在属性（値が真のときのみ出力）です。`jump_to_latest` の実際の可視判定（`stuck=free` かつ `has_new` 等の組み合わせ）は呼び出し側または配線層が行い、本部品は `visible: bool` を受け取って `data-visible`（可視時）/ `hidden`（非可視時、JS 無効時に「押しても何も起きないボタン」を見せないため）を切り替えるだけです。

`viewport` は [Scroll Area](./scroll-area.md) の `viewport` と同じ `tabindex="0"` 固定契約を持ちますが、`message-scroller` scope 自身のパーツとして独立に実装しています（`scroll_area` へは委譲しません）。`label` を渡すと `role="region"` + `aria-label` を出力します（空文字列のときは省略し、名前のない region を作りません）。カスタムスクロールバーが必要な場合は `scroll_area::scrollbar` / `scroll_area::thumb` を `viewport` の中へ入れ子にできます（両 scope は独立して共存できます）。

`content` はスロットで、`role="log"` を固定付与しません（ストリーミング通知の読み上げタイミングはアプリケーション固有の UX 判断であり、[Message](./message.md) と同じ判断軸で責務外としています）。`anchor` は `viewport` 末尾に置く `aria-hidden="true"` の計測用センチネルで、配線層が最下部到達の観測対象として使います。

`load_more` は `loading` / `disabled` の 2 つの `bool` を受け取り、それぞれ独立に `data-loading` 存在属性 / ネイティブ `disabled` + `data-disabled` を出力します（自動連動しません）。`aria-busy` は付与しません。

**アクセシビリティ**

- `viewport` は `tabindex="0"` を固定付与します（WAI 慣行に従い矢印キー・Page キーでフォーカス済み要素をスクロールできるため）。`label` が非空のときのみ `role="region"` + `aria-label` を出力します。
- `anchor` は `aria-hidden="true"` を固定付与します。
- `jump_to_latest` / `load_more` はネイティブ `button`（`type="button"`）で、Enter・Space が既定で作動します。
- `aria-live` / `aria-busy` / `aria-posinset` / `aria-setsize` はいずれも付与しません。通知タイミング・総数（無限履歴では SSR 時点で確定できません）はアプリケーション固有の判断であり、本部品の責務外です。

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Message Scroller](../themes/message-scroller.md) を参照してください。

自前 CSS の最小例:

```css
[data-scope="message-scroller"][data-part="viewport"] {
  overflow-y: auto;
  height: 12rem;
}
[data-scope="message-scroller"][data-part="jump-to-latest"] {
  position: sticky;
  bottom: 0.5rem;
}
[data-scope="message-scroller"][data-part="jump-to-latest"]:not([data-visible]) {
  display: none;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
