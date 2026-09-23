# Tooltip

`fandhe-frontend-wireframe-ui` の吹き出しのプレースホルダーです。本文 + 三角形の
指示子（矢印）からなる、画面設計図で「ここに補足の吹き出しが出る」という配置を
示すための部品です。API は `tooltip(label, side, size)` の 3 引数です（詳細は
下の引数表を参照）。

原案の参考: [blocks.pm](https://www.blocks.pm/) の Tooltip カタログ（ライセンス
上の理由でスクリーンショットは掲載しません）。

本部品は表示専用のため、実際にホバー・フォーカスで開閉する tooltip としては
出力しません。実際に操作できる tooltip が必要な場合は
[Tooltip（Themes）](../themes/tooltip.md)・[Tooltip（Primitives）](../primitives/tooltip.md)
を検討してください。

## 原案差分メモ

- **blocks.pm の Figma プロパティは書き写さない**: ライセンス保留
  （`docs/design/wireframe-ui-architecture.md` §2）により、blocks.pm 側の
  Size / Pointer / Text プロパティ構成は転写せず、§6 の汎用変換規約と本
  リポジトリの Themes/Primitives Tooltip が既に持つ `side` の語彙から独立に
  設計しました。
- **`side` は「吹き出しが対象のどちら側に出るか」**: `TooltipSide` は Themes の
  `side`（Floating UI 相当）と同じ意味で、矢印は反対側の辺から対象の方を
  向いて出ます（例: `Top` のとき吹き出しは対象の上に出て、矢印は吹き出しの
  下辺から下向きに出ます）。名前を「矢印がどの辺に付くか（Pointer）」にしな
  かったのは、blocks.pm の Figma プロパティ名をそのまま写すことになりかね
  ないのと、本リポジトリ既存の `side` と意味が逆になり混乱を招くためです。
- **対話セマンティクスは一切出力しない**: `role="tooltip"`・`aria-*`
  （`aria-describedby` 等も含む）・`title` 属性を出力しません。対象要素
  （トリガー）へのスロットや、対象への位置合わせ（Floating UI 相当の計算）も
  持ちません。位置計算は wasm-full の責務であり、本部品は「吹き出しの見た目」
  だけを表します。対象と組み合わせた配置イメージは Demo の最後の例のように
  `stack`/`button` 等との合成で示します。
- **方向は `data-*` ではなく修飾 class で表す**: 方向は表示状態ではないため、
  `fw-wire-tooltip-side-<top|right|bottom|left>` の修飾 class で表します
  （`frame` の `fw-wire-frame-bordered` と同じ扱い）。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-ink-muted` の地に
  `--fw-wire-paper` の文字（コントラスト比はおよそ 7:1）で表し、読みやすさを
  優先します。
- **責務外の軸**: `Bold`/`Primary`/`Disabled`/`Active`・アイコンスロットは
  持ちません。吹き出しは常に単一の反転配色で表し、強調のバリエーションや
  無効・選択といった表示状態を持たせる必要がないためです。
