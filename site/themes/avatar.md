# Avatar

`fandhe-frontend-pre-styled-ui` の `avatar` mod が提供するスタイル済み Avatar 部品です。

ユーザーやチームの識別に使うアイコン画像・イニシャル表示部品です。ImageStatus（Loading/Loaded/Error）に応じて image パーツと fallback パーツの表示・非表示を CSS の data-state セレクタで切り替えます。

size（Xs〜Xl）・shape（Circle/Rounded/Square）・variant（Subtle/Solid/Outline）・colorPalette（6 値、既定 Neutral）の 4 軸を持ちます（イシュー #1554 で参照サイト〔chakra-ui/Radix Themes〕基準へ調整）。

`Avatar` の Demo は `ImageStatus` を固定し、画像読み込み成功時の表示と
フォールバック（イニシャル等）表示の両方、および variant・colorPalette の
組み合わせを掲示しています。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
