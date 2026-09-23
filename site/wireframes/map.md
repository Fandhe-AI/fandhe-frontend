# Map

`fandhe-frontend-wireframe-ui` の地図タイル配置イメージプレースホルダー
です。街路・区画・幹線道路の線画とズーム段階、任意のマーカーだけを
CSS で組み立てる、非インタラクティブな表示専用部品です。API は
`map(zoom, marker, size)` の 3 引数（詳細は下の引数表を参照）です。

実際の地図表示が必要な場合は、外部の地図サービス（タイルプロバイダ等）
または独自実装を検討してください。参照元（blocks.pm）のスクリーンショッ
トは掲載しません（ライセンス保留、
`docs/design/wireframe-ui-architecture.md` §2/§7 参照。視覚的な参照は
<https://www.blocks.pm/> への外部リンクに限ります）。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §2/§7 に
  より、blocks.pm の Figma プロパティ（Zoom の段階値等）は書き写していま
  せん。§4/§6 の汎用変換規約と、クレート内の既存パターン（`alert::Severity`・
  `tooltip::TooltipSide`・`progress::ProgressShape` と同型の部品ローカル
  列挙型、`link`/`file_drop`/`alert` と同型の `Option<Node>` アイコン
  スロット規約 §11.4）から独自に設計しました。
- **`MapZoom` は部品ローカル列挙型**: 遠景（Far）・標準（Medium、既定）・
  近景（Near）の 3 段です。修飾 class を 1 つだけ付与し、CSS 側で街路
  グリッドのピッチ（`--fw-wire-map-cell`）だけを切り替えます。
  `props.rs` へは昇格していません。
- **`marker` は `Option<Node>` スロット**: `crate::icon` にピン専用の
  グリフがないため、`link`/`file_drop`/`alert` と同じ `Option<Node>`
  アイコンスロット規約を採用しました。**新しいピンアイコンは追加して
  いません**（デモでは `icon::house` を代用します）。`None` のときは
  マーカーのパート要素自体を出力しません。
- **街路・区画・道路は CSS の固定ルールで描く**: `repeating-linear-gradient`
  による街路グリッド、固定位置の区画矩形、固定位置の幹線道路 2 本
  （横・縦 1 本ずつ）はすべて CSS 側の静的ルールで表現し、Rust 側で乱数・
  ハッシュは使いません（golden テストと docs デモが純関数であるための
  決定性）。
- **`&str` 引数を持たない**: 地名ラベル・凡例・帰属表示・検索欄は発明して
  いません。動的入力は `MapZoom`・`Option<Node>`・`Size` のみのため、
  既定エスケープ（REQ-1）の対象となる動的文字列は本部品に構造的に存在
  しません（`crates/wireframe-ui/tests/map.rs` 冒頭コメント参照）。
- **グレースケール配色**: `ColorPalette` には依存せず、`--fw-wire-line`・
  `--fw-wire-line-subtle`・`--fw-wire-fill-subtle` 系の単色で描きます。
- **スコープ外**: ＋/−ズームボタン、地名ラベル、凡例、帰属表示、検索欄、
  実タイル画像や外部 URL の読み込み、複数マーカーは対象外です。
- **非対話**: `role`/`aria-*`/`tabindex`/`style`/`on*`/`<img>`/`<iframe>`/
  `<a>`/`href`/`<svg>`/`<canvas>`/`data-*` は一切出力しません（同文書 §7
  の非対話制約。ただし `marker` スロット由来の
  `<svg aria-hidden data-icon>` は許容します）。
