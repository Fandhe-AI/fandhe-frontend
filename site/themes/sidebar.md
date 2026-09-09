# Sidebar

`fandhe-frontend-pre-styled-ui` の `sidebar` mod が提供するスタイル済み
Sidebar 部品です。shadcn/ui `Sidebar` 相当の意匠を、headless 層
（`fandhe-frontend-headless-ui` の `sidebar`）が出力する 22 パーツ
（`provider`/`root`/`header`/`content`/`footer`/`separator`/`input`/
`group`/`group-label`/`group-content`/`group-action`/`menu`/`menu-item`/
`menu-button`/`menu-action`/`menu-badge`/`menu-sub`/`menu-sub-item`/
`menu-sub-button`/`rail`/`trigger`/`inset`）へ重ねます。

`variant`（`sidebar` / `floating` / `inset`）・`collapsible`（`offcanvas` /
`icon` / `none`）・`side`（`left` / `right`）はいずれも class ベースの
バリアント軸を持たず、headless が出力する `data-variant` / `data-collapsible`
/ `data-side` を CSS 属性セレクタとして参照するだけで見た目を切り替えます。
`--fandhe-sidebar-width` / `--fandhe-sidebar-width-icon` /
`--fandhe-sidebar-width-mobile` が展開幅・icon 折りたたみ幅・モバイル drawer
幅を制御し、`--fandhe-color-sidebar-*`（7 ロール: 背景 / 前景 / 枠線 /
accent 背景・前景 / ring / border）が配色を担います。icon 折りたたみ時は
`menu-button` のラベルテキストを clip 手法で視覚的に非表示化しつつ
アクセシブルネームは維持します（WCAG 4.1.2）。モバイル表示時は `root` が
`transform` のみで drawer として開閉します。`menu_skeleton` はローディング
装飾用のヘルパーで、ランダム幅を持たない決定的な固定幅の skeleton を返します。

Cmd/Ctrl+B のグローバルショートカット・モバイル判定（メディアクエリ）・
`menu-button` の tooltip hover 配線はこの部品では実装しません
（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）。これらは
`fandhe-frontend-wasm-full` 側の配線が担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
