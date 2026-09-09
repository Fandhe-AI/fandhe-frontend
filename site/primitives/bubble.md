# Bubble

チャット吹き出し 1 個を表現する shadcn/ui Bubble 相当の部品です（参照軸はイシュー #2001）。`fandhe-frontend-headless-ui` の `bubble` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`root` / `content` / `reactions` / `reaction` / `collapse-trigger` / `collapse-content` の 6 anatomy パーツを持ちます。状態機械を持たない静的部品です。

`root` は `variant`（`solid` / `outline` / `plain`）・`align`（`start` / `end`）・`group_position`（`single` / `first` / `middle` / `last`）の props を受け取り、それぞれ `data-variant` / `data-align` / `data-group-position` として出力します。`data-align` は会話系部品（message / bubble / attachment / marker）が共有する共通語彙（正は `message` mod）をそのまま再利用します。`data-variant` は shadcn/ui の 7 色調バリアントを「塗り・枠線・無装飾」の 3 形態へ縮約したもので、実際の色調選択（アクセント・破壊的操作色等）は `fandhe-frontend-pre-styled-ui` の ColorPalette 軸の責務です。`data-group-position` は連続発言の角丸連結用の表示状態のみを持ち、「何番目か」の算出は利用者側の責務です（部品はリスト全体を受け取りません）。

`content` はスロットで、Markdown レンダリング結果等の中身は利用者側で用意します。`reactions` は `role="group"` + 任意 `aria-label` を固定付与し、内部の `reaction` は `data-selected`（存在属性）のみを持つ非インタラクティブな表示用パーツです。押下・集計・トグルはアプリケーション側の責務です。

`collapse-trigger` / `collapse-content` は開閉パネル（`collapsible` mod）と同じ属性契約（`aria-expanded` / `aria-controls` / `data-state`）を `bubble` scope のまま再利用します。`collapse-trigger` はネイティブ `<button type="button">` として描画され、ブラウザ標準の Space/Enter → click 発火に従います。`fandhe-frontend-wasm-full` の click → `"toggle"` dispatch 配線は現時点で未整備のため、CSR での実際の開閉には呼び出し側が状態（`OpenState`）を保持して各パーツへ注入する配線が必要です。

**アクセシビリティ**

- `reactions` は `role="group"` を固定付与します。`label` を渡すと `aria-label` を出力します（空文字列のときは省略します）。
- `collapse-trigger` は `aria-expanded` と `data-state` を開閉状態に同期させます。`controls` を渡すと `aria-controls` で `collapse-content` と関連付けます。
- `collapse-content` は closed のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現します。

Themes（`fandhe-frontend-pre-styled-ui`）に対応するスタイル済み部品は今後追加予定です（イシュー #2109）。

自前 CSS の最小例:

```css
[data-scope="bubble"][data-part="root"] {
  display: inline-block;
  max-width: 32rem;
  padding: 0.5rem 0.75rem;
  border-radius: 0.75rem;
}
[data-scope="bubble"][data-part="root"][data-variant="solid"] {
  background: #e2e8f0;
}
[data-scope="bubble"][data-part="root"][data-variant="outline"] {
  border: 1px solid #cbd5e1;
}
[data-scope="bubble"][data-part="root"][data-group-position="first"] {
  border-radius: 0.75rem 0.75rem 0.75rem 0.25rem;
}
[data-scope="bubble"][data-part="root"][data-group-position="last"] {
  border-radius: 0.75rem 0.75rem 0.25rem 0.75rem;
}
```

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
