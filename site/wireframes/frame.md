# Frame

`fandhe-frontend-wireframe-ui` の配置コンテナ部品です。padding と境界線だけを
持つ矩形のコンテナで、画面設計図上で領域をまとめる用途を想定しています。API は
`frame(children, padding, bordered)` の 3 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。

blocks.pm に同名の部品はなく、本部品は `fandhe-frontend-wireframe-ui` 独自追加の
1 つです（`docs/design/wireframe-ui-architecture.md` §8「49 部品一覧」参照）。
そのためスクリーンショットは掲載していません。

Themes には類似の [Card](../themes/card.md) がありますが、Card は
header/body/footer の anatomy を持つのに対し、Frame は境界線と padding のみの
単純な配置コンテナです。

## 原案差分メモ

- **`Vec<Node>` 採用**: イシュー本文の想定引数「子ノード群」は `&[Node]`
  （借用）も選択肢でしたが、core のノード木 API・pre-styled-ui 全部品・
  wireframe-ui `icon::glyph` がいずれも `Vec<Node>` 所有渡しであり、
  `&[Node]` だと子ツリー全体の `clone()` が毎回発生するため
  `.claude/rules/coding-rust.md`「不要な clone() を避け、借用を優先」との
  整合を優先し `Vec<Node>` を採用しました。
- **非 bordered でも透明な境界線幅を確保**: `bordered=false` のときも
  `border-color: transparent` として境界線幅自体は確保しています。
  `bordered` の切り替えでレイアウト幅がずれない決定的な表示にするためです。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本部品は blocks.pm 原案が存在しないためそもそも参照画像がありません。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
- **padding は Frame 専用 class で表現（コードレビュー指摘、PR #2679 で
  是正）**: 当初は `Size::class()` が返す共有 `fw-wire-size-*` class を
  ルートへ直接付与し、`--fw-wire-control-size` を `calc()` で参照して
  padding を決めていましたが、この共有 class は `--fw-wire-font-size` も
  同時に定義し子孫へ継承されるため、独自の size class を再宣言しない
  子部品の文字サイズまで Frame の padding 引数で意図せず変化させてしまう
  問題がありました。[Stack](../wireframes/stack.md) が同種の問題を
  `fw-wire-stack-gap-*` 専用 class で先に回避した設計に倣い、Frame も
  `fw-wire-frame-padding-<段階>`（`.fw-wire-frame.fw-wire-frame-padding-xs`
  〜`-xl`、`0.75rem`〜`1.5rem`）を `FRAME_CSS` 内に 5 段明示する専用 class
  へ変更し、共有変数を子孫へ漏らさないようにしています。`size.rs` 自体は
  変更していません。
