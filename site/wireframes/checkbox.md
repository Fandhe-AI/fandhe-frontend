# Checkbox

`fandhe-frontend-wireframe-ui` のチェックボックス風プレースホルダーです。
正方形のボックス + 任意のラベルを持つ非インタラクティブな部品です。API は
`checkbox(label, size, active, disabled)` の 4 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Checkbox 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

`<input type="checkbox">` を出力しない表示専用プレースホルダーです。実際に
操作可能なチェックボックスが必要な場合は
[Checkbox（Themes）](../themes/checkbox.md) や
[Checkbox（Primitives）](../primitives/checkbox.md) を検討してください。

## 原案差分メモ

- **`div`/`span` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-checked`/`tabindex`/`on*` は
  一切出力しません（チェックグリフが持つ `aria-hidden="true"` は装飾用途の
  既定属性で対話的 ARIA ではありません）。
- **ラベル有無はラベルスロットの有無に畳み込む**: `docs/design/
  wireframe-ui-architecture.md` §6 の汎用変換規約に従い、「ラベル有無 +
  ラベル文言」の 2 軸を `label: Option<&str>` 1 引数へ畳み込みます
  （`annotation` の `description` と同型）。`None` のときはラベルの
  パート要素自体を出力しません。
- **チェック済み状態は共通型 `Active` を再利用し、専用 `Checked` 型は見送り**:
  選択済み（チェック済み）という表示状態を表す一般的な UI パターンとして、
  既に用意済みの共通型 `Active`（`data-active`）をそのまま使います。
  `docs/design/wireframe-ui-architecture.md` §10.1 が現時点の表示状態属性
  として `data-active`/`data-disabled` を規定していること、radio/switch
  等の並行実装との型衝突を避けることを理由に、専用の `Checked`/
  `data-checked` 型の新設は本イシューでは見送りました。`true` のとき
  `data-active=""` を付与し、ボックス内へチェックグリフを描画します
  （`false` のときはグリフ自体を出力せず、CSS での非表示にはしません）。
- **`Disabled` は既存の表示状態軸をそのまま使用**: Forms 部品向けに
  用意済みの共通型 `Disabled` をそのまま使い、`true` のとき
  `data-disabled=""` を付与します（見た目のみで、操作不能を実装する
  ものではありません）。`Primary`/`Bold`/`Orientation` は付与しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール反転**: チェック済み状態はボックスを
  `--fw-wire-ink` 背景・`--fw-wire-paper` 文字色へ反転し、黒塗り二値では
  なく `--fw-wire-*` トークンを使います。
