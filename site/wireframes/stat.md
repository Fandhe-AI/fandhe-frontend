# Stat

`fandhe-frontend-wireframe-ui` の数値指標カードプレースホルダーです。ラベル・
大きな数値・任意の増減インジケータからなる、非インタラクティブな表示専用
部品です。API は `stat(label, value, delta, size)` の 4 引数（詳細は下の
引数表を参照）で、props 構造体は導入していません。

blocks.pm には対応する部品がなく、wireframe-ui 独自の追加部品です（詳細は
`docs/design/wireframe-ui-architecture.md` §8 の「独自追加部品」区分を
参照してください）。

本部品は表示専用のため、数値の桁区切り・単位・時系列比較等の整形は一切
行いません（`label`/`value`/`delta.value` はいずれも不透明な文字列として
扱います）。実際にアクセシブルな統計表示が必要な場合は
[Stat（Themes）](../themes/stat.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §4・§6 の
  Figma プロパティ変換規約から独立設計しました。
- **独自追加部品**: blocks.pm に対応する部品はありません。
- **増減インジケータは `Option<&str>` ではなく `StatDelta`**: 増減値の
  文字列だけでは向き（上昇/下降/変化なし）を表現できず、先頭の `+`/`-`
  から向きを推測する暗黙の文字列解析も採りませんでした。代わりに
  `menu::MenuItem` と同型の公開構造体 `StatDelta { value, trend }` を導入し、
  delta が存在するのに向きだけ存在しないという矛盾した状態を型で作れない
  ようにしています。
- **既存の caret アイコンを再利用**: 上昇/下降のグリフには新規アイコンを
  追加せず、既存の `icon::caret_up`/`icon::caret_down` を再利用します。
  変化なし（`Flat`、既定）はグリフを出力しません。
- **`size` は最後の引数**: `alert`/`toast`/`menu` 等の既存部品の引数順に
  揃え、`Size` を最後の引数にしています。
- **数値の整形はしない**: 桁区切り・単位・符号の付与はいずれも UI
  コンポーネント層の責務外という既存方針
  （`docs/policy/intentional-non-adoption.md` §3.23）と同じ判断です。
- **非対話**: `role`/`aria-*`/`tabindex`/`style`/`on*`/`<button>`/`<a>` は
  一切出力しません（同文書 §7 の非対話制約）。
- **配色はグレースケールのみ**: 向きの区別に色を使わず、上昇のみ太字、
  下降・変化なしは `--fw-wire-ink-muted` で表します。
