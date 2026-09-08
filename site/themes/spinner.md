# Spinner

`fandhe-frontend-pre-styled-ui` の `spinner` mod が提供するスタイル済み Spinner 部品です。

読み込み中を示す回転インジケータです。role="status" + aria-label（既定 "Loading"）でスクリーンリーダーへ状態を伝えます。ボタン内部等、既に aria-busy で状態が伝わる文脈では装飾用途の spinner_decorative（role/aria-label なし）を使う設計です。トラックは既定で透明（上・右 2 辺のみ弧を描画、chakra-ui 基準）で、--fandhe-spinner-track-color / --fandhe-spinner-thickness / --fandhe-spinner-duration の custom property で線色・線幅・回転速度を上書きできます。

`spinner_decorative` は公開 API（イシュー #2051 で `pub(crate)` から公開化）で、Button 末尾配置・Badge・Empty state 等、周囲テキストが既に読み込み状態を伝える合成に使えます。また `style="--fandhe-palette: currentColor"` を指定すると、shadcn/ui Spinner 相当の「親の文字色へ追随する」色付けを既存 API のまま再現できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
