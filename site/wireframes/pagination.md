# Pagination

`fandhe-frontend-wireframe-ui` のページネーション風プレースホルダーです。
「前へ/次へ・ページ番号の並び・省略記号（…）・現在ページの強調」という配置
イメージだけを示す非インタラクティブな部品です。API は
`pagination(pages, active, prev_next, first_last, size)` の 5 引数（詳細は
下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Pagination 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Pagination」という名前ですが、`<nav>`/`<a>`/`href`/`<button>` のいずれも
出力しない表示専用プレースホルダーです。実際にページ送りできる部品が必要な
場合は
[Pagination（Themes）](../themes/pagination.md) /
[Pagination（Primitives）](../primitives/pagination.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`（アイコン基盤の装飾用
  `aria-hidden` を除く）/`tabindex`/`on*` は一切出力しません。
- **ページ項目は `&[Option<&str>]`**: `Some(label)` がページ番号セル、
  `None` が省略記号（…）のギャップセルを表します（`calendar` の
  `Option<u32>`（`None` = 空きマス）と同じ表現）。利用者に `"…"` という
  文字列そのものを渡させると通常のページセル（枠線付き）として描画されて
  しまうため、ギャップは構造として区別しました。ページ番号の妥当性検証・
  現在ページ前後の自動省略計算はアプリケーションロジックとして責務外と
  しています（`docs/policy/intentional-non-adoption.md` §3.25 と同じ
  判断軸）。
- **選択状態は `active: Option<usize>` 1 引数で表す**: `tabs` と同型の
  判断です。専用の `Selected`/`Checked` 型は新設せず、既存の共通型
  `Active` は項目ごとの `data-active` 出力にのみ用います（`radio`/
  `tabs` と同じ再利用判断）。`None`・範囲外・ギャップを指す添字は、
  どのセルにも選択インジケータを付けない決定的な挙動へ倒し、`unwrap`/
  `expect`/`panic` は使いません。
- **先頭/前/次/末尾コントロールは `prev_next`/`first_last` の 2 bool に
  畳む**: blocks.pm の Figma プロパティが持つ 4 つの独立 bool（先頭/前/
  次/末尾）は実用上「前後」と「先頭末尾」の対で使われるため、2 引数へ
  畳みました（§6 の boolean 爆発の畳み込み）。新しい props 型・構造体は
  導入していません。片側だけの表示（例: 次のみ）が必要なケースは本部品の
  対象外です。
- **`Orientation` を持たない**: ページ送りは水平のみのため、
  `props.rs` の doc が消費者として明記していない `Orientation` は
  必須引数に取っていません。`Disabled`/`Bold`/`Primary`、アイコン
  差し替えスロット、件数表示・ページサイズ選択も持ちません。
- **黒塗り二値ではなくグレースケール**: 現在ページは `--fw-wire-ink`・
  非選択ページは `--fw-wire-ink-muted`・現在ページの背景は
  `--fw-wire-fill-subtle` を使い、読みやすさを優先します（他の
  wireframe-ui 部品と同じ配色方針）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
