# Tree View

`fandhe-frontend-pre-styled-ui` の `tree_view` mod が提供するスタイル済み Tree View 部品です。

ファイル階層等の入れ子構造を表示する部品です。WAI-ARIA Tree View パターン（role="tree"/role="treeitem"）に従い、展開・折りたたみは [hidden] 属性と CSS の詳細度制御で表現します。JS 有効時は `fandhe-frontend-wasm-full` が実測した branch-content の高さを CSS 変数（`--fandhe-content-height`）へ書き込み、`@starting-style` + `transition-behavior: allow-discrete` によって展開・折りたたみを高さトランジションとして表現します（[Collapsible](collapsible.md) と同型。`hidden` 契約自体は維持したままの CSS ネイティブな遷移）。JSON データ専用の配色付き表示には [JSON Tree View](json-tree-view.md) を検討してください。

`root` は `size`（xs/sm/md/lg/xl、既定 md）で行密度・文字サイズを切り替えます。`color-palette` 軸は提供しません。行はキーボード操作時のフォーカスリングと hover 面を持ち、選択中の行の背景は hover で洗い流されません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
