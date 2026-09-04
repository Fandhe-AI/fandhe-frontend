# Separator

`fandhe-frontend-pre-styled-ui` の `separator` mod が提供するスタイル済み Separator 部品です。

コンテンツ間を区切る罫線部品です。orientation が role="separator"（固定）+ aria-orientation + data-orientation + variant クラスの 3 箇所へ連動します。呼び出し側が role/aria-orientation を偽装しても常にフレームワーク値へ一本化されます。variant は solid/dashed/dotted の 3 種を提供し、罫線の太さは `--fandhe-separator-thickness`（既定 1px）の上書きで変更できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
