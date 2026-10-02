# Separator

`fandhe-frontend-pre-styled-ui` の `separator` mod が提供するスタイル済み Separator 部品です。

コンテンツ間を区切る罫線部品です。orientation が role="separator"（固定）+ aria-orientation + data-orientation + variant クラスの 3 箇所へ連動します。呼び出し側が role/aria-orientation を偽装しても常にフレームワーク値へ一本化されます。variant は solid/dashed/dotted の 3 種を提供し、罫線の太さは `--fandhe-separator-thickness`（既定 1px）の上書きで変更できます。

イシュー #2053 で shadcn/ui と突合し、線 – テキスト – 線のラベル付き区切り線（`group`/`label`、chakra-ui の HStack + Text 合成相当）を pre-styled-only パートとして追加しました。

イシュー #3137 で `group_with`/`SeparatorLabelPosition` を追加し、ラベル位置を `center`（既定、`group` と同一出力）/`start` から選べるようにしました。`start` は「左ラベル – 線」「左タイトル – 線 – ボタン」のような先頭寄せレイアウトを 1 class で表現します。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
