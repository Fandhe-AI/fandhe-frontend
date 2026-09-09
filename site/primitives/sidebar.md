# Sidebar

アプリシェル用サイドバーに相当する shadcn/ui Sidebar 部品です（参照軸はイシュー #2001、shadcn/ui 追加はイシュー #2004）。`fandhe-frontend-headless-ui` の `sidebar` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、`provider` / `root` / `header` / `content` / `footer` / `separator` / `input` / `group` / `group-label` / `group-content` / `group-action` / `menu` / `menu-item` / `menu-button` / `menu-action` / `menu-badge` / `menu-sub` / `menu-sub-item` / `menu-sub-button` / `rail` / `trigger` / `inset` の 22 anatomy パーツを持ちます。

`nav_list`（`menu`/`menu-item`/`menu-button` の構造・`aria-current` 語彙）・`collapsible`（`menu-sub` の開閉は呼び出し側が合成する）・`tooltip`（`menu-button` の `describedby` が `aria-describedby` のみで関連付けを表す）の既存語彙を再利用し、新規の意味論は持ち込みません。`Sidebar` 状態機械は `data-state="expanded"|"collapsed"` を管理します。

**アクセシビリティ**

- `root` は `nav` で `label`（`aria-label`）が必須引数です。`div` への `aria-label` は支援技術に露出しないため、ランドマークが必要な本部品では型でアクセシブルネームを強制します。
- `provider`/`root` は `data-state`/`data-collapsible`/`data-variant`/`data-side`（mobile 時のみ `data-mobile`）を固定出力します。`data-collapsible` は状態に関わらず常に出力します（SSR 決定性優先の意図的差分）。
- `menu-button`/`menu-sub-button` は `href` が `Some` のとき `a`、`None` のとき `button type="button"` を描画し、`active` のときのみ `data-active` + （`a` のときのみ）`aria-current="page"` を付与します。
- `group` は `role="group"` を固定出力し、`labelledby` が `Some` のときのみ `aria-labelledby` を併記します（`group-label` の `id` が対になります）。
- `menu-sub` は開閉状態を持たない静的な `ul` です。開閉が必要な場合は呼び出し側で `collapsible::root`/`trigger`/`content` を合成します（他 scope を内包しない設計）。
- `trigger` は `aria-expanded`/`aria-controls`（`controls` が `Some` のとき）を持つキーボード操作可能な開閉ボタン、`rail` は `tabindex="-1"` のマウス専用領域です。
- `inset` は `div` です（`main` ではありません。docs サイトのページ骨格が既に `main` を描画しているため、`role="main"` が必要な利用者は `attrs` で明示的に渡します）。
- `menu-skeleton`（ローディング装飾）は本層に含めません。`fandhe-frontend-pre-styled-ui` 側（後続イシュー #2073）の責務です。
- Cmd/Ctrl+B のショートカット・モバイル drawer 切替・`menu-button` の tooltip hover 配線の実 DOM 配線は本層の範囲外です（`fandhe-frontend-wasm-full` の後続責務、#2074）。

自前 CSS の最小例:

```css
[data-scope="sidebar"][data-part="provider"][data-state="collapsed"] {
  width: 3rem;
}
[data-scope="sidebar"][data-part="menu-button"][data-active] {
  background: #eef;
}
[data-scope="sidebar"][data-part="rail"] {
  cursor: col-resize;
}
```

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Sidebar](../themes/sidebar.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
