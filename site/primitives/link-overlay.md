# Link Overlay

カード全体をクリック可能にする（カード全面クリック化）ための部品です。`fandhe-frontend-headless-ui` の `link_overlay` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`root`（`div`、位置決めコンテキスト）/ `overlay`（`a`、カード全面へ拡張されるリンク）の 2 パーツ構成で、`overlay` パーツを `root` 全面へ展開するのは呼び出し側の CSS（`position: absolute; inset: 0;`）の責務です。

スタイル済みの表示例は [Link Overlay](../themes/link-overlay.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Overlay | ネイティブ `a[href]` へのフォーカス移動です（ブラウザ既定、headless-ui 側の配線なし）。`root` 内に内側リンクを併置する場合、フォーカス順は DOM 順に従います。 |
| Enter | Overlay | ネイティブ `a[href]` の起動（遷移）です。**Space はリンクを起動しません**（`<a>` 要素は Space キーでは発火しないブラウザ標準挙動です）。 |

**参考サイトとの差分**

参照実体は chakra-ui の `LinkBox`/`LinkOverlay` のみです。ark-ui の `link-overlay` ページは 404 で実在せず、Radix Primitives・Radix Themes にも対応部品がありません。chakra-ui の `LinkBox`/`LinkOverlay` は Anatomy 節・Keyboard Interactions 節・`data-*` 語彙・独自 ARIA 付与のいずれも持たない styled 部品です。本実装は以下の点で意図的に差分を残しています。

- **`data-scope`/`data-part`**: 参考サイトには anatomy の概念自体が無く、本実装のこれらの属性はこちら側の superset です。
- **状態 `data-*` の非出力**: `data-state`/`data-disabled`/`data-motion` 等の状態を表す `data-*` は一切出力しません。参考サイトも同様に状態 `data-*` を持ちません。
- **非対応の prop**: `external`（旧 `isExternal`。現行 chakra-ui v3 docs では既に削除済み）・`asChild`（Slot 相当）は非対応です。`external` は位置引数追加が破壊的変更になり呼び出し元全体へ波及するため追加せず、`target`/`rel` を付与する場合は呼び出し側が `attrs` で両方を同時に渡す運用とします（`link::root` の不可分保証は本部品には及びません）。`asChild` は headless-ui 全体での再導入検討事項（保留）です。
- **内側リンクの前面化**: chakra-ui の `LinkBox` は CSS の子孫セレクタで内側リンクを `overlay` より前面化しますが、headless 層は CSS を持たないため利用者側 CSS の責務です（下記の自前 CSS 例を参照）。
- **`href` の予約キー除去**: `overlay` の呼び出し側 `attrs` に `href` を渡しても、固定付与された正規 `href` のみが出力されます（同名属性のなりすまし・二重出力の防止）。

一方で「`role`/`aria-*` を独自付与しない（暗黙の `link` ロールに委ねる）」点は参考サイトと一致しています。危険な URL スキーム（`javascript:` 等）は `href` 属性ごと拒否され、`href` を失った `a` はフォーカス不能になり暗黙の `link` ロールも失います（fail-closed の意味論的帰結）。`overlay` が `root` 全面へ `absolute` 展開されるため、`root` 内のテキストをポインタで選択しにくくなる点も参考サイトと同様です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="link-overlay"][data-part="root"|"overlay"]` セレクタで `root` の位置決め・`overlay` の全面展開・内側リンクの前面化を組み立てます。

```css
[data-scope="link-overlay"][data-part="root"] {
  position: relative;
}

[data-scope="link-overlay"][data-part="overlay"] {
  position: absolute;
  inset: 0;
  z-index: 0;
}

[data-scope="link-overlay"][data-part="overlay"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="link-overlay"][data-part="root"] a[href]:not([data-part="overlay"]) {
  position: relative;
  z-index: 1;
}
```
