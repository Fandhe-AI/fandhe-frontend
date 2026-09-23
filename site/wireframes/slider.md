# Slider

`fandhe-frontend-wireframe-ui` のスライダー風プレースホルダーです。
トラック + 円形ハンドルで進捗（Progress %）の配置イメージだけを示す
非インタラクティブな部品です。API は `slider(value, orientation, size, active, disabled)`
の 5 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Slider 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Slider」という名前ですが、`<input type="range">` 要素・ドラッグ操作・
キーボード操作のいずれも実装しない表示専用プレースホルダーです。実際に
操作可能なスライダーが必要な場合は
[Slider（Themes）](../themes/slider.md) /
[Slider（Primitives）](../primitives/slider.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`style`/`on*`・
  `<input type="range">` は一切出力しません。
- **Progress(%) は固定 class への量子化で表現する**: `value: u8` は
  0〜100 へクランプしたうえで 5 刻みへ丸め（四捨五入相当、`42 → 40`・
  `43 → 45`）、対応する固定 class（`fw-wire-slider-value-<q>`、21 種）を
  付与します。`style` 属性・`format!` による動的な CSS 値の組み立ては
  行いません（`class_list` の `&'static str` 限定という型制約、
  `crates/wireframe-ui/src/grid.rs` の `columns_class` と同型のパターン）。
  進捗値を `data-value` のような表示状態用途外の `data-*` へ出力すること
  もしません（`docs/design/wireframe-ui-architecture.md` §5「表示状態を
  示す `data-*` まで」の責務境界）。
- **`Orientation` を採用**: `docs/design/wireframe-ui-architecture.md` の
  `props::Orientation` rustdoc が「stack / divider / tabs / slider 等が
  共通で使う」と slider を消費者として名指ししているため、水平/垂直の
  向きは既存の共通型 `Orientation` で表現します。専用の列挙型は追加
  しません。
- **State は共通型 `Active`/`Disabled` へ畳み込む**: 専用の `SliderState`
  列挙は追加せず、フォーカス風の強調は `Active`、無効状態は `Disabled`
  （いずれも既存の共通型）で表現します。`true` のときそれぞれ
  `data-active=""`/`data-disabled=""` を付与します（見た目のみで、
  実際のフォーカス管理・操作不能を実装するものではありません）。
- **値ラベルは発明しない**: Figma 相当プロパティが Progress(%) のみの
  ため、`aria-valuenow` に相当する表示テキスト（値ラベル）用の引数は
  追加していません。
- **`&str` 引数を持たないため XSS 回帰は構造的に充足**: 動的入力は
  `u8` 1 個のみで固定 class 集合へ量子化されるため、テキスト引数の既定
  エスケープ回帰テストは対象外です（`crates/wireframe-ui/tests/slider.rs`
  冒頭コメント参照）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-*` トークン
  （`ink`/`fill`/`paper`/`line-subtle`）で読みやすさを優先します。
