# Grid

`fandhe-frontend-wireframe-ui` の列数固定グリッド配置部品です。子ノード列を
指定した列数のグリッドへ配置し、各セルは破線境界のプレースホルダーとして
表示します。API は `grid(children, columns, gap)` の 3 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

blocks.pm に対応する部品はありません（本部品は `fandhe-frontend-wireframe-ui`
独自の追加部品です）。参照元となる外部サイトはないため、[blocks.pm](https://www.blocks.pm/)
そのものへのリンクのみ掲載します（詳細は `docs/design/wireframe-ui-architecture.md`
§2）。

Primitives・Themes には同名の "Grid" 部品はありません（本部品は
`fandhe-frontend-wireframe-ui` 固有です）。実際に操作可能なレイアウトが必要な
場合は、CSS Grid を直接使用するか、上位のアプリケーションコードで配置を
組み立ててください。

## 原案差分メモ

- **blocks.pm 対応なしの独自追加部品**: イシュー本文の想定（子ノード列・
  列数・間隔の 3 引数プレースホルダー）をそのまま設計へ反映しました。
  Figma プロパティの転写元がないため、原案差分という形の記述はなく
  「独自設計そのもの」です。
- **`children` は `Vec<Node>`**: イシューの想定表記は `&[Node]` でしたが、
  `fandhe_frontend_core::div`/`el_owned` のシグネチャ（`Vec<Node>`）に揃え、
  深いノード木の呼び出し元へ不要な `clone()` を強制しないようにしました
  （`.claude/rules/coding-rust.md` 「不要な `clone()` を避け」）。
- **`columns` は 1〜12 へ丸める**: `0` は `1` へ、`MAX_COLUMNS`（12）を
  超える値は `MAX_COLUMNS` へ丸めます。極端な入力でも出力サイズが有界に
  なるようにするための設計判断です。
- **`gap` と `Size` の継承**: `gap.class()`（`fw-wire-size-<段階>`）を root へ
  付与するため、`--fw-wire-font-size`/`--fw-wire-control-size` も
  子孫へ継承されます。各 wireframe 部品は自分の size class で上書きするため
  実害はありませんが、`Grid` 自体のセル間隔（`gap`）専用の CSS 宣言は
  共有の `Size` スケール表ではなく `grid` モジュール内の CSS 定数へ独自に
  持たせています（他部品と共有するファイルを Phase 1 の並列部品と同時に
  変更する衝突を避けるため）。
- **セルは破線境界**: 実線境界の `Annotation` と区別するため、
  `fw-wire-grid-item` は破線（`dashed`）境界にしています。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
