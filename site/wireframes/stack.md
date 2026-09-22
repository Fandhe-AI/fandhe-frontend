# Stack

`fandhe-frontend-wireframe-ui` の純粋なレイアウトコンテナ部品です。子要素を
縦または横に等間隔で並べる用途に使います。API は `stack(children,
orientation, gap)` の 3 引数（詳細は下の引数表を参照）で、props 構造体は
導入していません。

blocks.pm には対応する部品がなく、本部品は `fandhe-frontend-wireframe-ui`
独自追加の部品です（スクリーンショットの掲載元がないため、参照画像は
掲載していません）。

Primitives・Themes には同名の "Stack" 部品はありません（本部品は
`fandhe-frontend-wireframe-ui` 固有です）。

## 原案差分メモ

- **方向の型は再利用**: 新型を導入せず `Orientation`（`Horizontal` 既定 /
  `Vertical`）を再利用しました。`docs/design/wireframe-ui-architecture.md`
  §10.1 が `fw-wire-horizontal`/`fw-wire-vertical` を割り当て済みで、
  他部品（divider 等）からも横断利用される型です。
- **children は所有渡し**: `Vec<Node>` を受け取ります。core の `div`/
  `el_owned` と同じ設計で、`&[Node]` のような借用渡しにはしていません。
- **gap は Size 5 段を明示**: `Size` のスコープ付きカスタムプロパティ
  （`--fw-wire-font-size`/`--fw-wire-control-size`）には gap 用の値がない
  ため、`.fw-wire-stack.fw-wire-size-<段階>` の 5 セレクタへ `gap` を
  直接明示しています（`0.25rem`〜`2rem`）。
- **size class 継承の注記**: ルートへ `fw-wire-size-*` を付与すると
  `--fw-wire-font-size` も配下へ継承されます。子要素が自身のルートで
  size class を再宣言する部品（annotation 等）であれば実害はありません。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex`/`style`/`data-*` は
  一切付与せず、対話要素も出力しません（設計文書 §5/§7）。
