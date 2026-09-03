# Table

`fandhe-frontend-pre-styled-ui` の `table` mod が提供するスタイル済み Table 部品です。

表形式データを表示する部品です。TableVariant（Line/Outline）で外枠・区切り線を、striped（bool）で縞模様表示を、sticky_header（bool）で column-header（th）の position: sticky 表示を切り替えます。Outline は内側の行罫線・muted なヘッダー背景・最終行罫線なし・tfoot 上罫線を持ちます。column_header は scope="col" を関数側で固定し呼び出し側の偽装を除去します。scroll_area（div）でスクロール可能な枠を作り、sticky_header と組み合わせることで有界のスクロール枠内で見出しを固定できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
