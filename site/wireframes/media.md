# Media

動画やメディア埋め込み領域の配置イメージを示す、16:9 固定の非
インタラクティブなプレースホルダーです。`fandhe_frontend_wireframe_ui::media`
の Rust API は `content`（省略可能なコンテンツスロット、`Option<Node>`）・
`size`（`Size` 5 段）の 2 引数だけを受け取ります。`content` を省略すると
既定の再生グリフが表示され、実際の動画再生は提供しません
（`<video>`・`<iframe>` はいずれも出力しません）。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Placeholder 部品
です。本ページは blocks.pm のスクリーンショットや外観・プロパティ構成を
そのまま取り込みません（詳細は `docs/design/wireframe-ui-architecture.md`
§2）。参照したい場合は外部リンク先を直接確認してください。

## 原案差分メモ

- **API は §4・§6 から独立設計**: `docs/design/wireframe-ui-architecture.md`
  §4・§6 の汎用変換規約から独立設計しており、blocks.pm の Placeholder 部品
  の Figma プロパティ構成（動画フラグの bool・アイコンサイズ・再生アイコン
  の差し替え）をそのまま転写したものではありません。
- **`None` は再生グリフへのフォールバック（§11.4 からの意図的な逸脱）**:
  `crate::icon` の `Option<Node>` アイコンスロット規約は「`None` なら
  スロット要素を出力しない」を原則としますが、本部品は `content` が
  `None` のとき既定の再生グリフ（`icon::play`）を差し込みます。中身の
  ないメディア枠は [`crate::frame`] と見分けがつかず、ワイヤーフレーム
  として「ここに動画/メディアが入る」という意図を伝えられないためです
  （[`crate::avatar`]〔イシュー #2651〕を先例とする逸脱。§11.4 本文への
  例外の明文化は #2712 から続く未反映の残課題です）。
- **動画か静止画かは bool ではなくスロット差し替えで表す**: 原案が持つ
  動画フラグの bool は導入せず、`Some(icon::image(size))` を渡すことで
  静止画のメディア枠を、`None`（既定）で動画の既定表示を表します。画像
  プレースホルダー自体は別部品（Image、イシュー #2660）の担当です。
- **サイズは共通の `Size` 5 段**: 中央のディスク・グリフの大きさにのみ
  効き、寸法は [`crate::size::css`] が定義する `--fw-wire-control-size`
  を `var()` で参照するのみで、値そのものは書き写していません。枠自体は
  16:9 固定で親幅いっぱいに広がります。
- **メディア URL 引数を持たない**: `src`・`poster` のような外部リソース
  を読み込む引数は設けていません。`controls` も出力しないため、見た目
  だけ操作できそうで実際には操作できない UI にはなりません。
- **配色は読みやすさ優先のグレースケール**: 黒塗り二値を強制せず、淡い
  塗り（`--fw-wire-fill-subtle`）と控えめな文字色（`--fw-wire-ink-muted`）
  で表現します。
- **表示専用**: `data-*`・`role`・`aria-*`・`tabindex` は一切出力しません。
  表示状態（`Active`/`Disabled`）も持ちません（`spinner`/`progress` と
  同じ判断）。
- **スクリーンショット非掲載**: イシュー #2602 で参照スクリーンショット
  （`docs/design/reference-screenshots/wireframe-*.png`）の取り込みは
  不採用が確定しているため、blocks.pm への外部リンクで代替しています。
