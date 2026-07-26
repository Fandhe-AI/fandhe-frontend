# Accordion

高々 1 項目が開く single モードと、複数項目が同時に開く multiple モードの
2 状態機械を提供する開閉可能な項目リストです。`fandhe-frontend-headless-ui`
の `accordion` mod は Root / Item / ItemTrigger / ItemIndicator /
ItemContent の 5 anatomy パーツと、開閉状態を表す `data-state`・
`aria-expanded`・`aria-controls`・（ラベル付き時のみ）`role="region"` を
提供します。orientation・キーボードナビゲーションは SSR 静的マークアップに
寄与しない CSR 挙動層の責務としてスコープ外です。

スタイル済みの表示例は [Accordion](../themes/accordion.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
