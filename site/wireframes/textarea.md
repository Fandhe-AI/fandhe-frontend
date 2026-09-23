# Textarea

`fandhe-frontend-wireframe-ui` の複数行テキスト入力欄部品です。画面設計図で
「ここに複数行の自由記述欄がある」という配置イメージを伝えるための
非インタラクティブなローファイ・プレースホルダーで、blocks.pm に対応する
部品はありません（wireframe-ui 独自追加部品）。API は
`textarea(text, rows, size, active, disabled)` の 5 引数（詳細は下の
引数表を参照）です。

`rows` はネイティブ `<textarea>` の属性や `style` 属性ではなく、行の高さを
表す行プレースホルダー要素を `rows` 個並べる構造表現です。実際に操作可能な
複数行入力欄が必要な場合は [Textarea（Themes）](../themes/textarea.md) を
検討してください。

## 原案差分メモ

- **独自設計（blocks.pm 該当なし）**: blocks.pm に Textarea 相当の部品は
  存在しません。イシュー本文の想定引数（テキスト・行数・サイズ・表示状態）
  から独立設計しました。
- **「State」軸を `Active`＋`Disabled` へ分解**: イシュー本文が挙げる
  「State」軸のための共有型は新設せず、既存の `props::Active`
  （フォーカス中の見た目）と `props::Disabled`（無効）の 2 型へ分解しました。
  `props.rs` は兄弟部品イシューが並行して触る共有ファイルであり、
  `docs/design/wireframe-ui-architecture.md` §5「表示状態は `data-*` まで」の
  既存型で表現できるため新設は不要と判断しています。
- **`rows` を行プレースホルダーで表現し `style` を使わない理由**: 本クレートは
  非インタラクティブ部品に `style` 属性・ネイティブ対話要素を一切出力しない
  方針です（設計文書 §5/§7）。そのため `rows` は行プレースホルダー要素
  （`div.fw-wire-textarea-line`）を `rows` 個生成する構造表現にしています。
  `0` は `1` へ丸め、上限 20（`MAX_ROWS`）で飽和させます。利用者値 1 個から
  無制限にノードを増やせないようにする資源有界化です。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、[blocks.pm](https://www.blocks.pm/) への外部リンクで
  代替します。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  ネイティブ `<textarea>` 要素も出力しません。リサイズグリップは CSS の
  `::after` 擬似要素のみで表現しています。
- **配色はグレースケール**: `data-active`/`data-disabled` の見た目はいずれも
  `--fw-wire-*` モノクロトークンのみで表現し、`ColorPalette` には依存しません
  （設計文書 §3「黒塗り二値を強制せず読みやすさ優先」の判断軸）。
