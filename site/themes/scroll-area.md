# Scroll Area

`fandhe-frontend-pre-styled-ui` の `scroll_area` mod が提供するスタイル済み Scroll Area 部品です。

CSS overflow を主体としたスクロール領域を提供する部品です。headless 層は anatomy と tabindex="0" のみを提供し、::-webkit-scrollbar 系規則でカスタムスクロールバーの見た目を表現します。JS によるスクロール位置追従は対象外です。

thumb 色は custom property `--fandhe-scroll-area-thumb-bg` で一元化されており、`root` へ再定義するだけで見た目を調整できます（hover 時・キーボードフォーカス時は `--fandhe-scroll-area-thumb-hover-bg` へ自動的に強調されます）。`transparent` を指定すれば、既定は非表示で hover 時のみ出現する見た目も再現できます。

`viewport` へ `data-orientation="horizontal"` を付与すると直下の `content` が横並びになり（shadcn/ui `ScrollBar orientation="horizontal"` 相当、`gap`/`padding` は呼び出し側が `content` へ指定）、`data-fade` を付与すると端が `mask-image` でフェードします（shadcn/ui `utils/scroll-fade` 相当。`animation-timeline: scroll()` 対応ブラウザではスクロール量に応じてフェード端が動的に切り替わり、非対応ブラウザは両端固定フェードになります。`data-orientation="horizontal"` 併用時は横方向グラデーション、`dir="rtl"` では方向が反転します）。いずれも既定の見た目を変えない opt-in で、フェード幅は custom property（`--fandhe-scroll-area-fade-size`・`--fandhe-scroll-area-fade-reveal`・`--fandhe-scroll-area-fade-start`/`-end`）で調整できます。詳細は下記 Features / Examples 節を参照してください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
