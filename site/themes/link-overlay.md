# Link Overlay

`fandhe-frontend-pre-styled-ui` の `link_overlay` mod が提供するスタイル済み Link Overlay 部品です。`root`（位置決めコンテキスト）/ `overlay`（カード全面へ拡張されるリンク）の 2 anatomy パーツで構成し、chakra-ui の LinkBox/LinkOverlay パターンに倣ってカード全体をクリック可能にします。`overlay` 以外の子ノード（見出し・画像等）が `root` の高さを確立する契約です。キーボード操作時は `overlay` に共通のフォーカスリング（`:focus-visible`）が表示されます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
