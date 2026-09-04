# Scroll Area

`fandhe-frontend-pre-styled-ui` の `scroll_area` mod が提供するスタイル済み Scroll Area 部品です。

CSS overflow を主体としたスクロール領域を提供する部品です。headless 層は anatomy と tabindex="0" のみを提供し、::-webkit-scrollbar 系規則でカスタムスクロールバーの見た目を表現します。JS によるスクロール位置追従は対象外です。

thumb 色は custom property `--fandhe-scroll-area-thumb-bg` で一元化されており、`root` へ再定義するだけで見た目を調整できます（hover 時は `--fandhe-scroll-area-thumb-hover-bg` へ自動的に強調されます）。`transparent` を指定すれば、既定は非表示で hover 時のみ出現する見た目も再現できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
