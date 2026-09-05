# Nav List

見出し + リンクリストのみで構成する文書ナビ向けの静的なリンク集です。`fandhe-frontend-headless-ui` の `nav_list` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`menu` ロールへの誤読を避けるため `role` 属性を一切付与せず、素の `nav`/`h2`/`ul`/`li`/`a` の暗黙 ARIA ロールに依拠します。

スタイル済みの表示例は [Nav List](../themes/nav-list.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Link | ネイティブ `a[href]` へのフォーカス移動です（ブラウザ既定、headless-ui 側の配線なし）。矢印キーでの項目間移動（roving）は WAI-ARIA の文書ナビパターンに存在しないため提供しません。 |
| Enter | Link | ネイティブ `a[href]` の起動（遷移）です。**Space はリンクを起動しません**（`<a>` 要素は Space キーでは発火しないブラウザ標準挙動です）。 |

**参考サイトとの差分**

本部品は ark-ui / Radix Primitives / Radix Themes に 1:1 対応物を持たない fandhe 独自部品です（`docs/design/component-coverage-map.md`）。イシューが指す chakra-ui の `List`（`variant`/`align`/`colorPalette`/`unstyled`/`as`/`asChild`/`List.Indicator` のみを持つ汎用の marker 付きリスト）は Anatomy 節・Keyboard Interactions 節・`data-*` 語彙・独自 ARIA のいずれも持ちません。chakra `List` の本リポジトリでの真の対応物は Themes 層 [`fandhe-frontend-pre-styled-ui::list`](../themes/list.md) であり、本部品は「`nav` ランドマーク + 見出し + リンクリスト」という文書ナビの意味論を持つ別部品として区別しています。以下は突合の結論です。

- **anatomy**: 参照側に 1:1 の Anatomy 図はありません。chakra `List.Root`/`List.Item` は本部品の `list`/`item` に相当し、`root`（`nav`）/`heading`（`h2`）/`link`（`a`、`aria-current`/`data-current` 語彙）は文書ナビ固有の superset です。増減はありません。
- **`data-*`**: 参照側は状態 `data-*` を一切持ちません。`data-current` は `link`/`breadcrumb` と共有する本リポジトリ独自語彙であり、削除は Themes 側の CSS セレクタへ波及する破壊的変更のため意図的に維持しています。
- **非対応の prop**: `List.Indicator`（装飾マーカー）・`as="ol"`（本部品は文書ナビとして常に `ul` 固定）・`variant`/`align`/`colorPalette`/`unstyled`（装飾軸、Themes 層の責務）・`asChild`（headless-ui 全体での再導入検討事項、保留）はいずれも非対応です。
- **`heading` の見出しレベル固定**: `h2` 固定であり可変化には対応していません（API 拡張のため今回はスコープ外）。

一方で「`role`/`aria-*` を独自付与しない（暗黙の `navigation`/`heading`/`list`/`listitem`/`link` ロールに委ねる）」点は参考サイトと一致しています。危険な URL スキーム（`javascript:` 等）は `href` 属性ごと拒否され、`href` を失った `a` は暗黙の `link` ロールも失います（fail-closed の意味論的帰結）。呼び出し側 `attrs` による `aria-label`/`href`/`aria-current`/`data-current` のなりすましは除去されます（予約キーなりすまし除去）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="nav-list"][data-part="..."]` セレクタと `:hover`・`:focus-visible`・`[aria-current="page"]` 擬似クラス／属性セレクタでスタイルを当てます。

```css
[data-scope="nav-list"][data-part="list"] {
  list-style: none;
  margin: 0;
  padding: 0;
}

[data-scope="nav-list"][data-part="link"] {
  color: #2563eb;
  text-decoration: none;
}

[data-scope="nav-list"][data-part="link"]:hover {
  text-decoration: underline;
}

[data-scope="nav-list"][data-part="link"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="nav-list"][data-part="link"][aria-current="page"] {
  font-weight: 700;
}
```
