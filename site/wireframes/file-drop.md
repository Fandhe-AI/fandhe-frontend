# File drop

`fandhe-frontend-wireframe-ui` のファイルドロップ領域風プレースホルダーです。
点線枠 + 任意のアイコン + 説明文 + 任意のヒントで「ファイルをドラッグ＆
ドロップする領域」の配置イメージだけを示す非インタラクティブな部品です。
API は `file_drop(label, hint, icon, size)` の 4 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

`file-drop` は blocks.pm に対応する部品を持たない、wireframe-ui 独自追加の
部品です。層全体の参照元は [blocks.pm](https://www.blocks.pm/) ですが、
本部品自体には blocks.pm 上の対応物がないためスクリーンショットは
掲載していません。

`<input type="file">`・`<label>`・ドラッグイベントのいずれも実装しない
表示専用プレースホルダーです。実際に操作可能なファイルアップロードが
必要な場合は
[File Upload（Themes）](../themes/file-upload.md) /
[File Upload（Primitives）](../primitives/file-upload.md) を検討してください。

## 原案差分メモ

- **独自追加部品**: `file-drop` は `docs/design/wireframe-ui-architecture.md`
  §8 が挙げる、blocks.pm に対応部品がない wireframe-ui 独自追加の 14 部品
  のひとつです。転写元がないため、引数構成は同文書 §6 の汎用変換規約
  （`Size` 軸 + テキスト + 省略可能テキスト + スロット）から独立に設計
  しました。
- **アイコンは `Option<Node>` スロットで受ける**: 専用のアップロード
  アイコンは追加せず、`link`（イシュー #2618）と同じ `Option<Node>`
  アイコンスロット規約（同文書 §11.4）を採用しました。呼び出し側は
  `icon::image`・`icon::plus` 等、任意の既存アイコンを渡せます。
- **表示状態軸を持たない**: `Bold`/`Primary`/`Active`/`Disabled` はいずれも
  使いません。「ドラッグ中」はインタラクションの状態であり、非対話層
  （同文書 §7）の責務外と判断しました。`file_drop` ルート・子要素には
  `data-*` を一切付与しません（アイコンスロットに渡した `Node` 自身が
  持つ `data-icon` は例外としてそのまま透過します）。
- **`<input type="file">`・`<label>`・ドラッグイベントは出力しない**:
  非対話制約（同文書 §7）により、`<input>`/`<label>`/`<button>`/`<form>`/
  `<a href>`・`draggable`/`ondrop`/`ondragover` 等のドラッグイベント・
  `accept`/`multiple` 等のファイル入力属性は一切出力しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本部品には blocks.pm 上の対応物自体がないため、上記の外部リンクのみで
  代替します。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted` トークンの
  みを使い、`ColorPalette` には依存しません。
