# Spinner

`fandhe-frontend-pre-styled-ui` の `spinner` mod が提供するスタイル済み Spinner 部品です。

読み込み中を示す回転インジケータです。role="status" + aria-label（既定 "Loading"）でスクリーンリーダーへ状態を伝えます。ボタン内部等、既に aria-busy で状態が伝わる文脈では装飾用途の spinner_decorative（role/aria-label なし）を使う設計です。トラックは既定で透明（上・右 2 辺のみ弧を描画、chakra-ui 基準）で、--fandhe-spinner-track-color / --fandhe-spinner-thickness / --fandhe-spinner-duration の custom property で線色・線幅・回転速度を上書きできます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
