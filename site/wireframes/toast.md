# Toast

`fandhe-frontend-wireframe-ui` の通知トースト風プレースホルダーです。
任意のアイコン + 短い本文 + 見た目だけの閉じる「×」で「一時的な通知
カード」の配置イメージだけを示す非インタラクティブな部品です。API は
`toast(message, icon, dismissible, size)` の 4 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

`toast` は blocks.pm に対応する部品を持たない、wireframe-ui 独自追加の
部品です。層全体の参照元は [blocks.pm](https://www.blocks.pm/) ですが、
本部品自体には blocks.pm 上の対応物がないためスクリーンショットは
掲載していません。

`role="status"`・`aria-live`・`<button>` のいずれも実装せず、画面隅への
固定配置（`position: fixed`/`absolute`）も行わない表示専用プレースホルダー
です。実際に操作可能な通知が必要な場合は
[Toast（Themes）](../themes/toast.md) /
[Toast（Primitives）](../primitives/toast.md) を検討してください。

## 原案差分メモ

- **独自追加部品**: `toast` は `docs/design/wireframe-ui-architecture.md`
  §8 が挙げる、blocks.pm に対応部品がない wireframe-ui 独自追加の 14 部品
  のひとつです。転写元がないため、引数構成は同文書 §6 の汎用変換規約
  から独立に設計しました。
- **引数名は `text` ではなく `message`**: イシュー本文が想定する引数名は
  `text` ですが、`fandhe_frontend_core::text`（テキストノードを作る関数）
  と同じ値名前空間で衝突するため `message` に改めました。意味は同じです。
- **アイコンは `Option<Node>` スロットで受ける**: イシュー本文が想定する
  見た目（アイコン + 短文 + close）に対し、info/success 専用のグリフは
  ありません。専用グリフを新設すると `icon` モジュールの件数表記等へ
  影響が波及するため、`link`（イシュー #2618）・`file-drop`（イシュー
  #2633）と同じ `Option<Node>` アイコンスロット規約（同文書 §11.4）を
  採用しました。呼び出し側は `icon::bell`・`icon::check` 等、任意の既存
  アイコンを渡せます。
- **閉じるグリフは `icon::x` 固定（instance swap にしない）**: `select`
  の末尾指示子・`ratings` の `icon::star` と同じく固定パートとして扱い、
  `dismissible: bool` の 1 引数だけで有無を切り替えます（`tag` の
  `remove: Option<Node>` のような差し替え可能スロットにはしていません）。
- **表示状態軸・重要度バリアントを持たない**: `Bold`/`Primary`/`Active`/
  `Disabled` はいずれも使わず、info/success/error 等の重要度表現も本部品
  の対象外です。最小構成にとどめ、重要度表現が必要な場合は上記の Themes /
  Primitives の Toast を案内します。`toast` ルート・子要素には `data-*`
  を一切付与しません（アイコンスロット・閉じるグリフが持つ `data-icon`
  は例外としてそのまま透過します）。
- **`<button>`・`role`・`aria-live` は出力しない**: 非対話制約（同文書
  §7）により、閉じるパートは `<button>` にせず、`role="status"`・
  `aria-live`・`<output>`・`tabindex`・`style`・`on*` は一切出力しません。
- **固定配置は付けない**: `position: fixed`/`absolute` による画面隅への
  固定配置は利用者のレイアウトの責務とし、docs のデモ枠の中に収まる
  in-flow のカードとして描きます。「浮いている感じ」はトークン参照の
  ハードシャドウ（`box-shadow: 0 0.25em 0 var(--fw-wire-line-subtle)`）で
  表現します。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本部品には blocks.pm 上の対応物自体がないため、上記の外部リンクのみで
  代替します。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted` トークンの
  みを使い、`ColorPalette` には依存しません。
