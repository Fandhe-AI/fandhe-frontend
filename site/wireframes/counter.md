# Counter

数字を収めた小さな丸（ピル）バッジで、件数表示のプレースホルダーです。
`fandhe_frontend_wireframe_ui::counter` の Rust API は `count`（件数文言、
`&str`）・`size`（`Size` 5 段）・`primary`（強調・反転色バリアント）を
受け取ります。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Counter 部品です。
本ページは blocks.pm のスクリーンショットや外観・プロパティ構成をそのまま
取り込みません（詳細は `docs/design/wireframe-ui-architecture.md` §2）。
参照したい場合は外部リンク先を直接確認してください。

実際に数値を動的に更新したり、未読数の読み上げのような支援技術向けの
セマンティクスが必要な場合は、[Themes](../themes.md) の
[Badge](../themes/badge.md) を使用してください。

## 原案差分メモ

- **API は §4・§6 から独立設計**: `docs/design/wireframe-ui-architecture.md`
  §4・§6 の汎用変換規約から独立設計しており、blocks.pm の Counter 部品の
  Figma プロパティ構成をそのまま転写したものではありません。
- **件数は `&str` で受ける**: `u32` にせず `&str` としました。
  `"99+"`・`"1.2k"` のような省略表記を呼び出し側が自由に選べるように
  するためで、数値の整形（桁丸め等）は UI コンポーネント層の責務外という
  判断軸（`docs/policy/intentional-non-adoption.md` §3.25）をそのまま
  踏襲しています。空文字列を渡すと中身のない丸（ドット状のバッジ）として
  描画されます。
- **配色は読みやすさ優先のグレースケール**: 原案の黒塗り二値を強制せず、
  既定は淡い塗り（`--fw-wire-fill-subtle`）と濃い文字（`--fw-wire-ink`）に
  し、黒塗り（反転配色）は共通型 [`crate::props::Primary`] の opt-in へ
  回しました（Annotation 部品〔イシュー #2617〕と同じ判断軸、設計文書
  §3「黒塗り二値を強制せず読みやすさ優先」）。
- **強調配色は新型を新設しない**: `docs/design/wireframe-ui-architecture.md`
  §6 の変換規約に沿って、部品ローカルの新しい修飾型は新設せず、既存の
  共通型 `Primary`（`Bold`/`Primary` と同系統の視覚修飾型）を再利用して
  います。
- **サイズは共通の `Size` 5 段**: フォントサイズは
  [`crate::size::css`] が定義する `--fw-wire-font-size` を `var()` で
  参照するのみで、値そのものは書き写していません。1 桁の件数は真円、
  複数桁はピル形状に伸びます。
- **表示専用**: `data-*`・`role`・`aria-*`・`tabindex` は一切出力しません。
  対話的・支援技術向けのセマンティクスが必要な場合は上記の Themes の
  Badge を案内します。
- **`nav_item` の内部カウンターパートとは独立**: `crate::nav_item` が
  持つ内部パート（`fw-wire-nav-item-counter`）は「行レイアウトと一体で
  あるため内部パートとした」というモジュール doc 上の根拠を持ち、本部品
  への置き換え・リファクタは行っていません（スコープ外事項として PR
  本文に記録します）。
- **スクリーンショット非掲載**: イシュー #2602 で参照スクリーンショット
  （`docs/design/reference-screenshots/wireframe-*.png`）の取り込みは
  不採用が確定しているため、blocks.pm への外部リンクで代替しています。
