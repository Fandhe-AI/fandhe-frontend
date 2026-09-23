# Tag

分類・キーワードラベルを示す、ピル形状の非インタラクティブなローファイ・
プレースホルダーです。`fandhe_frontend_wireframe_ui::tag` の Rust API は
`label`（必須のタグ文言）・`size`（`Size` 5 段）・`primary`（強調・反転色
バリアント）・`remove`（削除「×」アイコンスロット、`Option<Node>`。見た目
のみで対話操作は行わない）を受け取ります。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Tag 部品です。本
ページは blocks.pm のスクリーンショットや外観・プロパティ構成をそのまま
取り込みません（詳細は `docs/design/wireframe-ui-architecture.md` §2）。
参照したい場合は外部リンク先を直接確認してください。

実際に操作可能なタグ入力・削除機能が必要な場合は、[Themes](../themes.md)
の Tag / Tags Input、または [Primitives](../primitives.md) の Tags Input を
使用してください。

## 原案差分メモ

- **API は §6 から独立設計**: `docs/design/wireframe-ui-architecture.md`
  §6 の汎用変換規約（`Size` 軸 + 強調 bool + テキスト + 表示状態 bool）から
  独立設計しており、blocks.pm の Tag 部品の Figma プロパティ構成をそのまま
  転写したものではありません。
- **スクリーンショット非掲載**: イシュー #2602 で参照スクリーンショット
  （`docs/design/reference-screenshots/wireframe-*.png`）の取り込みは
  不採用が確定しているため、blocks.pm への外部リンクで代替しています。
- **削除「×」は §11.4 の `Option<Node>` スロット規約に従う**: 設計文書
  §11.4 は `tag` をアイコン差し替え（instance swap）スロット規約の標準
  適用先として明示的に列挙しています。Link 部品（`link.rs`）の
  `trailing` と同型の `remove: Option<Node>` として受け取り、呼び出し側が
  `Some(icon::x(size))` を渡したときのみ削除パートを出力します。
- **ピル形状・グレースケール反転**: `border-radius: 999px` の固定ピル
  形状とし（`--fw-wire-radius`〔角丸カード等向けの小さい値〕は使わない）、
  `primary` の反転時は削除アイコンの色を `--fw-wire-ink-muted` ではなく
  `--fw-wire-fill`（明るいトークン）へ切り替えています。暗地上で
  `ink-muted` を使うとコントラスト不足になるためで、Annotation 部品
  （イシュー #2617）と同じ判断軸です（設計文書 §3「黒塗り二値を強制せず
  読みやすさ優先」）。
- **非インタラクティブ**: 削除「×」は `<button>` にせず、見た目だけの
  表示専用パートです。`role`/`aria-*`/`tabindex`/実際の削除操作は一切
  実装しません。
