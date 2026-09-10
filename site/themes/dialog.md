# Dialog

`fandhe-frontend-pre-styled-ui` の `dialog` mod が提供するスタイル済み Dialog 部品です。
画面全体を覆うモーダルダイアログで、通常の確認ダイアログ（`role="dialog"`）と
警告用ダイアログ（`role="alertdialog"`）を `DialogRole` で切り替えられます。
フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖といったクライアント
挙動は JS ランタイム側の責務とし、本レイヤーは決定的な SSR 属性出力のみを担います。

alert-dialog（確認ダイアログ、Radix Primitives / Radix Themes の Alert Dialog に相当）は
独立部品や新しい variant 軸としては追加せず、`DialogRole::Alertdialog`（`role="alertdialog"`）と
`footer`（pre-styled-only のアクション列レイアウトパート）に既存の `button` の
variant / colorPalette 軸（Solid + Danger と Outline の組み合わせ等）を組み合わせて構成します。
`role="alertdialog"` の dialog は wasm-full 層が外側クリックでの閉鎖を既定で無効化します。
構成例は下記 Demo の Examples 節（Alert dialog）を参照してください。

スクロール可能コンテンツ（shadcn/ui〔Base UI スタイル〕突合、イシュー #2030）には
pre-styled-only の `body` パートを使います。見出し（`title`/`description`）と
`footer`（アクション列）を `content` の子として `body` の外側の兄弟に配置すると、
`body` にだけ `overflow-y: auto` / `max-height: 50vh` が効き、見出し・フッターを
固定したまま本文だけを縦スクロールさせられます（`position: sticky` は使いません）。
`content`/`positioner` 自体は変更していないため、`body` を使わない既存の構成には
影響しません。close ボタンを content 右上のアイコンではなく `footer` 内の通常の
ボタンとして掲示したい場合（shadcn の「Custom Close Button」相当）は、
`close_trigger_with_variant(CloseTriggerVariant::Text, ...)` を `footer` 内に置くと、
アイコン専用契約のまま既存の閉じる配線（`(dialog, close-trigger) -> "close"`）を
共有した平文ボタンとして機能します（構成例は下記 Demo の Examples 節「Share link
(custom close button)」を参照してください）。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
