# Progress

`fandhe-frontend-wireframe-ui` の進捗表示のプレースホルダーです。バー形・円形の
2 通りで「ここに進捗が表示される」という配置を示すための部品です。API は
`progress(value, shape, size)` の 3 引数です（詳細は下の引数表を参照）。

原案の参考: [blocks.pm](https://www.blocks.pm/) の Progress カタログ（ライセンス
上の理由でスクリーンショットは掲載しません）。

本部品は表示専用のため、実際に値が変化する進捗表示としては出力しません。実際に
操作・更新できる進捗表示が必要な場合は
[Progress（Themes）](../themes/progress.md)・[Progress（Primitives）](../primitives/progress.md)
を検討してください。

## 原案差分メモ

- **blocks.pm の Figma プロパティは書き写さない**: ライセンス保留
  （`docs/design/wireframe-ui-architecture.md` §2）により、blocks.pm 側の
  Progress プロパティ構成は転写せず、§6 の汎用変換規約と一般的な UI キットの
  線形/円形進捗表示パターンから独立に設計しました。
- **形状は部品ローカルの列挙型で表す**: `ProgressShape`（`Bar`/`Circle`）は
  bool 引数にしていません。Rust API Guidelines が bool 引数を推奨していない
  ことと、Figma の boolean プロパティをそのまま写すことになりかねないためです。
- **進捗値は 5 刻みの固定 class 集合へ量子化**: `value: u8` は `slider`
  （イシュー #2628）と同じ規則で 0〜100 へクランプしたうえで 5 刻みへ丸め、
  対応する固定 class（`fw-wire-progress-value-<q>`）を付与します。
  `style="--…: 42%"` のような属性値の動的組み立てや `data-value` の出力は
  行いません（`class_list` の型制約〔`&'static str` 限定〕と、
  `docs/design/wireframe-ui-architecture.md` §5「表示状態を示す `data-*` まで」
  の責務境界に従います）。丸め・量子化ロジックは `slider.rs` を直接再利用せず、
  本部品側に独立に持ちます（並行実装との衝突を避けるための判断、`grid`/
  `slider` の先例に合わせます）。
- **Circle は `conic-gradient` + くり抜き要素で描く**: リング状の進捗を
  `mask` ではなく `conic-gradient` の背景と、中央に重ねた `fw-wire-progress-hole`
  要素（内側を紙面色でくり抜く）の組み合わせで表現します。`mask` を避けたのは
  互換性の懸念と、div だけの anatomy に揃えるためです。
- **対話セマンティクスは一切出力しない**: `<progress>` 要素・`role="progressbar"`・
  `aria-valuenow`/`aria-valuemin`/`aria-valuemax` は出力しません。値の変化・
  アニメーションも実装しません（不確定進捗〔indeterminate〕・スピナーは別部品
  `spinner`〔#2649〕の対象です）。
- **`Active`/`Disabled` を持たない**: 進捗表示は表示専用で、フォーカスや無効
  状態に意味を持たせる必要がないためです（`tooltip` と同じ判断）。ラベルや
  パーセント値のテキスト表示も発明しません（`slider` の「値ラベルは発明しない」
  方針に合わせます）。この結果、動的な入力は `u8` 1 個のみで固定 class 集合へ
  量子化されるため、text 引数の XSS 回帰テストは構造的に充足されます
  （`crates/wireframe-ui/tests/progress.rs` 冒頭コメントも参照）。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-ink`/`--fw-wire-fill`
  によるコントラストで進捗を表します。
