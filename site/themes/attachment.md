# Attachment

`fandhe-frontend-pre-styled-ui` の `attachment` mod が提供するスタイル済み
Attachment 部品です。「添付ファイル 1 件の表示」を表現する Root / Media /
Content / Name / Meta / Progress / Actions / Action の 8 パーツ構成で、
横並びの行カード（`file` 形態）・縦積みのサムネイルカード（`image` 形態）
の 2 意匠と、アップロード失敗時の枠色を見た目に反映します。

`variant`（`file` / `image`）・`state`（`idle` / `uploading` / `error`）・
`disabled` は headless 層が出力する `data-variant` / `data-state` /
`data-disabled` を CSS セレクタとして参照するだけで、class ベースの軸は
持ちません。`file` は `media`（角丸アイコン枠）・`content`（`name` /
`meta` の縦積み）・`actions`（末尾へ常時表示）の横並びカード、`image` は
`media` を正方形サムネイルとして敷き詰める縦積みカードです。`error` は
`root` の枠色と `meta` の文字色を強調色へ切り替えます。

`image` 形態の `actions` は既定で隠れ、`root` への hover / focus-within
（キーボード操作）で表示します。この隠蔽規則自体は hover 機構を持つ端末
向けの `@media (hover: hover)` 配下にのみ限定しており、タッチ端末等
hover 機構を持たない端末では規則が適用されず、CSS の初期値である
`opacity: 1`（常時表示）のまま残ります。「見えないボタンで操作不能になる」
到達不能を避けるための意図的な設計です。

`progress` は attachment scope の単純なスロットで、`Progress`
（`fandhe-frontend-headless-ui`）を委譲しません。アップロード進捗を表示
するときは、呼び出し側がスタイル済み `progress`（root / track / range）を
中身へ入れ子にします。`name` / `meta` は整形済み文字列を受け取るだけの
スロットで、byte → KB 変換等の数値整形はこの部品では行いません。

`action` はゴーストボタン（`type="button"` 固定・`label` が空文字列で
ないときのみ `aria-label`）です。`disabled` はネイティブ `disabled` と
`data-disabled` の両方に反映しますが、`root` の `disabled`（opacity 0.5 +
cursor: not-allowed）とは独立しており、二重減衰を避けるため `action`
自身の `disabled` は `cursor: not-allowed` のみを適用します。

アップロード進捗の判定・エラー分類・削除処理・ファイル名やサイズの整形は
アプリケーションロジックであり、この部品では実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
