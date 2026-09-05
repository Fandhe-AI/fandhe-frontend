# Pagination

総件数からページ番号列（省略記号を含む）を決定的に導出するページ送りです。
`fandhe-frontend-headless-ui` の `pagination` mod は Root / Item / Ellipsis /
PrevTrigger / NextTrigger / FirstTrigger / LastTrigger の 7 anatomy パーツを
提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。
クリックで dispatch する Button モードと、`href` 遷移の Link モードの両方に
対応します。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Item / 各トリガー | ネイティブのフォーカス移動です。各パーツはネイティブ `<button>`/`<a>` のため roving tabindex 等の独自実装は持ちません（ark-ui と同じ契約）。 |
| Enter / Space | Item / 各トリガー（Button モード） | ネイティブ `<button>` の既定動作としてクリックと同じ効果で発火します。 |
| Enter | Item / 各トリガー（Link モード） | ネイティブ `<a>` の既定動作として `href` へ遷移します。 |

**参考サイトとの差分**

ark-ui Pagination（chakra-ui Pagination も同型。Radix Primitives / Radix
Themes に Pagination は存在しません）と突合し、以下を是正しました。

- **FirstTrigger / LastTrigger パーツを新設**（5 → 7 anatomy パーツ）。ark-ui
  の Anatomy・Data Attributes 表に準拠し、`prev-trigger`/`next-trigger` と
  同じ `disabled`/`aria-disabled`/`data-disabled` 契約を持ちます。
- **`item` へ `data-index`（ページ番号の 10 進数文字列）を追加**。ark-ui の
  Item Data Attributes 表に準拠します。
- **`PaginationAction::First`/`Last` を追加**し、`"first"`/`"last"` dispatch
  で `page` を `1`/`total_pages` へ直接遷移できるようにしました
  （ark-ui の `goToFirstPage`/`goToLastPage` 相当）。
- **呼び出し側 `attrs` からの固定属性キー偽装を除去する `drop_reserved`
  を追加**しました（`data-selected`/`data-index`/`aria-current`/`href`/
  `type`/`disabled`/`aria-disabled`/`data-disabled`/`aria-hidden`）。

一方、以下は意図的に合わせていません。

- **キーボードの独自実装**: ark-ui も roving tabindex 等の独自キーボード
  操作を持たず、ネイティブ `<button>`/`<a>` の既定操作に委ねる契約のため、
  本部品も新規実装していません（上表参照）。
- **`aria-label`（"page N" 相当のローカライズ文字列）**: ark の
  `translations.itemLabel` はローカライズ機構前提のため、`item`/トリガー系
  パーツでは `aria-label` を予約キーに含めません。呼び出し側が `attrs`
  経由で明示的に供給する契約のまま維持しています。
- **chakra-ui の `PageText`/`Items`（"1–10 of 50" 相当の件数レンジ文字列）**:
  数値整形はアプリケーションロジックであり、UI コンポーネント層の責務外
  として不採用です。
- **`fandhe-frontend-wasm-full` のクリック配線**: `"goto"`/`"next"`/`"prev"`/
  `"first"`/`"last"` を DOM イベントへ接続する処理は別クレート責務のため
  未実装です。

スタイル済みの表示例は [Pagination](../themes/pagination.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="pagination"][data-part="..."]`
セレクタでスタイルを当てます（`positioner` のような非表示パーツを持たないため
`[hidden]` ガードは不要です）。

```css
[data-scope="pagination"][data-part="item"][data-selected] {
  font-weight: 600;
  background: #eff6ff;
}

[data-scope="pagination"][data-part="prev-trigger"][data-disabled],
[data-scope="pagination"][data-part="next-trigger"][data-disabled],
[data-scope="pagination"][data-part="first-trigger"][data-disabled],
[data-scope="pagination"][data-part="last-trigger"][data-disabled] {
  color: #9ca3af;
  pointer-events: none;
}

[data-scope="pagination"][data-part="ellipsis"] {
  color: #9ca3af;
}
```
