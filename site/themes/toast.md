# Toast

`fandhe-frontend-pre-styled-ui` の `toast` mod が提供するスタイル済み Toast 部品です。
一時的な通知を有界なキューとして管理する状態機械 `Toaster` を持ち、`aria-live`
は通知の状態（`ToastStatus`）から決定的に導出します（`Error` のみ `assertive`、
他は `polite`）。`aria-atomic="true"` を併用し通知全体を単位として読み上げさせ
ます。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品と同じく、画面端固定の通知を「表示中の状態」で
> 固定掲示しています。本来の配置（画面端固定・複数通知の積み上げ）ではページ内の
> 他セクションと重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の
> `.pre-styled-showcase` スコープ）でページの流れの中へ収めています。実アプリケーション
> での overlay 配置は pre-styled-ui の recipe CSS がそのまま担います。

`close_trigger` はアイコン専用の契約です。固定正方（右上絶対配置）+
`overflow: hidden` のため、テキストを渡しても切り詰められます。
`close_trigger(vec![("aria-label", "Close")], vec![text("×")])` のように、
1〜2 文字のグリフと `aria-label` の組み合わせで呼び出してください。

`action_trigger` は outline 小ボタンとして表示され、hover / キーボード操作時の
フォーカスリング / disabled 状態のスタイルを備えます:
`action_trigger(vec![], vec![text("Update")])`。ネイティブ `<button disabled>`
のみが disabled 状態を反映します（headless 層が `data-disabled` を発行しない
ため）。

`title` は省略可能で、`description` のみで構成する通知も既存 anatomy の
まま実現できます（Examples 節を参照）。イシュー #2040 で shadcn/ui と突合
した結果、状態配色（淡色面 tint）と `action_trigger` の縦積み配置は既存の
まま維持することを確定しています。理由の詳細は
`crates/pre-styled-ui/src/toast.rs` のモジュール doc「イシュー #2040」節を
参照してください。

## 積層表示（stack、motion feature opt-in）

`motion` feature（既定 off）を有効化すると、`crate::toast_motion` モジュールが
通知の積層表示（後ろの通知ほど縮小・オフセット）を opt-in で提供します。
group（`toast::group`）の `attrs` へ `toast_motion::STACK_ATTR`（値は空文字列）
を渡すだけで有効になり、`:hover` / `:focus-within` で通常の縦並び表示へ展開
します。前面 3 枚のみ可視で、4 枚目以降は `opacity: 0` になりますが DOM・
`aria-live` の読み上げ対象からは外れません。

DOM 順は**新しい通知が先頭（前面）**である必要があります（CSS は兄弟の総数を
知れないため）。`Toaster::push` は末尾追加のため、呼び出し側は
`entries.iter().rev()` で渡してください。

動的な追加・削除・並べ替えを行う場合は `toast_motion::stack_group_keyed` を
使うと、`fandhe-frontend-wasm-full` の stagger 書き戻し・layout FLIP が自動で
適用されます（keyed list の自動配線。静的な SSR 専用ページでは
`toast::group` へ `STACK_ATTR` を渡すだけで足ります）。

展開はレイアウト（`display`）の切替を伴うため、積層状態と展開状態の間の
位置補間はありません（`translate`/`scale`/`opacity` の戻り遷移のみが滑らか
に動きます）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
