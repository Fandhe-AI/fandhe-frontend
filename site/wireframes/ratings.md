# Ratings

`fandhe-frontend-wireframe-ui` の星評価風プレースホルダーです。
星 5 個のうち塗った個数だけを視覚的に示す非インタラクティブな部品です。
API は `ratings(rating, size)` の 2 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Ratings 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

星は塗り/線画の 2 状態のみを示す表示専用プレースホルダーで、
`<input type="radio">` 群・`role="radiogroup"`・ポインタ操作・キーボード操作の
いずれも実装しません。実際に操作できる評価入力が必要な場合は
[Rating group（Themes）](../themes/rating-group.md) /
[Rating group（Primitives）](../primitives/rating-group.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §2 の保留
  （イシュー #2602）に従い、blocks.pm の Figma プロパティ構成（Size・
  Rating・星ごと bool 等）を参照・書き写さず、同文書 §6 の汎用変換規約
  （「星評価は `rating: u8` の 1 引数へ畳む」）から独立に設計しました。
- **星の総数は 5 固定でクランプする**: `rating: u8` は星の総数
  （`STAR_COUNT = 5`）を超える値を 5 へクランプします。星の総数を
  変える引数（`max`）は追加していません。
- **`icon::star` を再利用する**: 星のジオメトリは新規に描かず、既存の
  SVG アイコン基盤（`crate::icon::star`）をそのまま再利用します
  （`docs/design/wireframe-ui-architecture.md` §11.4 のスロット規約・
  §2「ジオメトリの出自」の方針）。
- **塗り状態は共通型 `Active` へ畳み込む**: 専用の状態型は追加せず、
  先頭から `rating`（クランプ後）個の星へ `Active(true)` を渡し
  `data-active=""` を付与します（`Select`/`Radio`/`Switch`/`Checkbox` に
  続く「チェック済み」相当の意味での再利用）。
- **評価値は `data-*` へ出力しない**: `data-rating`/`data-value` のような
  評価値そのものを表す `data-*` 属性は一切出力しません（`slider` の
  「値は `data-*` へ出さない」判断と同じ、
  `docs/design/wireframe-ui-architecture.md` §5「表示状態を示す `data-*`
  まで」の責務境界）。
- **`role="img"`/`aria-label` は付けない**: ルートへ意味論を追加する
  属性は付与しません（`icon` 部品（イシュー #2652）との継ぎ目を侵さない
  ための判断）。
- **`&str` 引数を持たないため XSS 回帰は構造的に充足**: 動的入力は
  `u8` と `Size` のみで文字列が HTML へ流れる経路が存在しないため、
  テキスト引数の既定エスケープ回帰テストは対象外です
  （`crates/wireframe-ui/tests/ratings.rs` 冒頭コメント参照）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 塗り済みの星は `--fw-wire-ink`、未塗りの星は
  `--fw-wire-line` トークンで表現し、`ColorPalette` には依存しません。
