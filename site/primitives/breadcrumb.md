# Breadcrumb

現在地までの階層（サイト内の位置）をリンク列として示すパンくずナビゲーションです。`fandhe-frontend-headless-ui` の `breadcrumb` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。末尾項目のみ非対話の `current-link`（`aria-current="page"`）として描画し、中間項目は `ellipsis` パーツで折り畳めます。

スタイル済みの表示例は [Breadcrumb](../themes/breadcrumb.md) を参照してください。

**アクセシビリティ**

- `root` の `aria-label` は省略時 `"breadcrumb"` が既定値になります（WAI-ARIA APG の Breadcrumb パターン準拠）。
- 末尾項目のみ `current-link`（非対話の `span`）として描画し、`aria-current="page"` + `data-current` を固定付与します（中間項目は `link`、遷移可能な `a`）。
- `separator`/`ellipsis` はいずれも `role="presentation"` + `aria-hidden="true"` で装飾扱いとし、スクリーンリーダーの読み上げから除外します。
- キーボード操作はネイティブ `<a href>`（`link` パーツ）のみに依存します（`Tab`/`Shift+Tab` でのフォーカス移動・`Enter` での遷移はブラウザ既定動作）。独自キーハンドラは持ちません。
- 参照実体は chakra-ui の Breadcrumb のみです（ark-ui には対応部品がなく、Radix Primitives / Radix Themes にも Breadcrumb は存在しません）。突合の結果、anatomy 7 パーツ・WAI-ARIA とも差分なしと確認しています（イシュー #1648）。WAI-ARIA APG は末尾項目も `<a aria-current="page">` として残しますが、本実装は chakra-ui に倣い非対話 `span` とする意図的な差分です。`data-current` は `link`/`nav_list`/`pagination` と共有する本リポジトリ独自の `data-*` 語彙です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
