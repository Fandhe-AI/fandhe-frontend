# Alert

`fandhe-frontend-wireframe-ui` の横長の警告・通知バナーのプレースホルダーです。
アイコン + タイトル + 任意の説明文からなる、画面設計図で「ここに警告・通知が
出る」という配置を示すための部品です。API は
`alert(severity, title, description, icon, size)` の 5 引数です（詳細は下の
引数表を参照）。props 構造体はありません。

原案の参考: [blocks.pm](https://www.blocks.pm/) のカタログ（ライセンス上の
理由でスクリーンショットは掲載しません）が、Alert は blocks.pm に対応部品を
持たない wireframe-ui 独自追加の部品です。

本部品は表示専用のため、`role="alert"`・`aria-live` は出力せず、閉じるボタン
も持ちません。実際にアクセシブルな alert が必要な場合は
[Alert（Themes）](../themes/alert.md) を検討してください。

## 原案差分メモ

- **独自追加部品**: blocks.pm に対応部品がないため、Figma プロパティの転写
  対象がなく、`docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約
  から独立設計しました。
- **重要度はモノクロの 3 段差で表す**: `Severity`（`info`/`warning`/`error`）
  は色ではなく、地色・枠の太さ・反転配色の組み合わせで区別します。Error は
  背景・枠・文字を反転させ、Warning は開始辺の枠を太くします。
- **`Severity` は部品ローカルの型**: `crate::props` へは昇格しません。
  Phase 6 の兄弟イシューが並行して触る共有ファイルであるため、他部品での
  需要が見えるまでは `alert.rs` 内に留めています（`tooltip::TooltipSide` と
  同じ判断）。
- **アイコンは `Option<Node>` スロット**: 重要度から自動でアイコンを選ばず、
  呼び出し側が任意の既存アイコン（例: `icon::bell`）を渡します。`link`・
  `file_drop` と同じ規約です。
- **`size` 引数を追加**: イシュー本文の想定引数は重要度・タイトル・説明文の
  3 つでしたが、既存の全部品が `Size` 引数を受け取る規約に合わせて追加しま
  した。
- **対話セマンティクスは一切出力しない**: `role="alert"`・`aria-live`・
  `aria-*`・`tabindex`・閉じるボタン（`<button>`）を出力しません。
- **スクリーンショットは載せていません**（イシュー #2602）。
- **配色はグレースケールのトークンのみ**で、`ColorPalette` には依存しません。
