# Clipboard

値のコピーとコピー済み表示を扱う unstyled 部品です。コピー済み/未コピーの 2 値状態を `data-copied`（存在属性）で表現します。参考実装（ark-ui/chakra-ui）と突合し、Label は `input` の `id` を指す `for`、Input は `data-readonly`、Trigger は既定 `aria-label`（コピー済み状態に応じて反転）を持ちます。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Clipboard](../themes/clipboard.md) を参照してください。

実際の `navigator.clipboard.writeText` への書き込み・コピー完了後の自動リセットはクライアントランタイム側の責務であり、本部品は状態遷移の SSR マークアップのみを提供します。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
