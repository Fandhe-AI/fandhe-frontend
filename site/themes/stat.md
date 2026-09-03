# Stat

`fandhe-frontend-pre-styled-ui` の `stat` mod が提供するスタイル済み Stat 部品です。

数値指標を表示する部品です。label/value-text/value-unit/help-text/up-indicator/down-indicator の 6 パーツで構成し、増減インジケータは装飾用途のため aria-hidden="true" を固定付与します。ラベルと値の一覧表示には [Data List](data-list.md) を検討してください。

size（xs〜xl、既定 md。chakra-ui の sm/md/lg は本実装の Sm/Md/Lg に対応）で value-text のフォントサイズが変化します。増減インジケータは固定セマンティック色（up-indicator は success、down-indicator は danger）を使用し、colorPalette 軸には連動しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
