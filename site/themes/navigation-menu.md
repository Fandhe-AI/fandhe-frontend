# Navigation Menu

`fandhe-frontend-pre-styled-ui` の `navigation_menu` mod が提供するスタイル済み Navigation Menu 部品です。

トリガー起点で開閉するナビゲーションパネルです。Root / List / Item / Trigger / ItemIndicator / Content / Link の 7 anatomy パーツを持ち（イシュー #1654 で ItemIndicator を新設し 6 → 7 パーツへ拡張、イシュー #2035 で Themes 層の CSS 付与を完了）、高々 1 個の Trigger だけが開く状態機械（`NavigationMenu`）を提供します。ItemIndicator はトリガー横の開閉シェブロン（装飾専用、`aria-hidden="true"` 固定）で、開いているトリガーでは 180° 回転します。素の `nav`/`ul`/`li`/`button`/`div`/`a` の暗黙 ARIA role に依拠し、`role` は一切付与しません（`role="menu"`/`role="menuitem"` を付与すると文書ナビを操作メニューと誤伝達するため）。アクティブリンクは `aria-current="page"` で表します。状態機械を持たない静的なリンク集が必要な場合は [Nav List](nav-list.md) を使ってください。ルートレベルの viewport スライドポインタ（`NavigationMenuIndicator`）・`data-motion`（アニメーション方向の露出）は装飾・アニメーション・レイアウト計測の関心のため実装していません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
