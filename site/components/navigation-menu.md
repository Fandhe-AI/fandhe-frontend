# Navigation Menu

`fandhe-frontend-pre-styled-ui` の `navigation_menu` mod が提供するスタイル済み Navigation Menu 部品です。

トリガー起点で開閉するナビゲーションパネルです。Root / List / Item / Trigger / Content / Link の 6 anatomy パーツを持ち、高々 1 個の Trigger だけが開く状態機械（`NavigationMenu`）を提供します。素の `nav`/`ul`/`li`/`button`/`div`/`a` の暗黙 ARIA role に依拠し、`role` は一切付与しません（`role="menu"`/`role="menuitem"` を付与すると文書ナビを操作メニューと誤伝達するため）。アクティブリンクは `aria-current="page"` で表します。状態機械を持たない静的なリンク集が必要な場合は [Nav List](nav-list.md) を使ってください。Radix の viewport 寸法測定・`data-motion`（アニメーション方向の露出）は装飾・アニメーション関心のため実装していません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
