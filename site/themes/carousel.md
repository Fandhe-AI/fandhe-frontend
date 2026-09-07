# Carousel

`fandhe-frontend-pre-styled-ui` の `carousel` mod が提供するスタイル済み Carousel 部品です。

複数のコンテンツを 1 件ずつ切り替えて表示するナビゲーション部品です。orientation（Horizontal/Vertical）に応じて item-group の移動方向を切り替えます。選択・チェック状態を示す部品ではないため colorPalette 軸は提供しません。ページ送りには [Pagination](pagination.md) も参照してください。

`--fandhe-carousel-item-basis`（既定 `100%`）を root へ設定すると、複数スライドを同時表示できます（shadcn/ui との突合、イシュー #2028）。Demo では垂直方向の実演と、この CSS カスタムプロパティで 3 件同時表示にした実演もあわせて確認できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
