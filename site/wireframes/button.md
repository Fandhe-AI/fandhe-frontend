# Button

`fandhe-frontend-wireframe-ui` のボタン風プレースホルダーです。テキスト
ラベル + 任意の先頭アイコン + サイズ/強調/無効状態を持つ非インタラクティブな
部品です。API は `button(label, icon, size, primary, disabled)` の 5 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Button 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Button」という名前ですが、`<button>` 要素・クリック操作・フォーム送信の
いずれも実装しない表示専用プレースホルダーです。実際に操作可能なボタンが
必要な場合は [Button（Themes）](../themes/button.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切
  出力しません（アイコンが持つ `aria-hidden="true"` は装飾用途の既定属性で
  対話的 ARIA ではありません）。
- **Type はアイコンスロットの有無に畳み込む**: blocks.pm の Type（アイコン
  + テキスト / テキストのみ）プロパティは独立した列挙型を持たず、
  `icon: Option<Node>` の `Some`/`None` へ畳み込みました。
  アイコンのみ（テキストなし）の variant は本イシューでは実装していません
  （意図的な絞り込み）。
- **`Disabled` は独立追加した表示状態軸**: blocks.pm の列挙外ですが、
  Forms 部品向けに用意済みの共通型 `Disabled` をそのまま使い、`true` の
  とき `data-disabled=""` を付与します（見た目のみで、クリック不能を
  実装するものではありません）。`Bold`/`Active` は付与しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール反転**: `Primary` バリアントは `--fw-wire-ink`
  背景・`--fw-wire-paper` 文字色へ反転し、黒塗り二値ではなく
  `--fw-wire-*` トークンを使います。
