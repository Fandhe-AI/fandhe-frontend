# Link Overlay

`fandhe-frontend-pre-styled-ui` の `link_overlay` mod が提供するスタイル済み Link Overlay 部品です。`root`（位置決めコンテキスト）/ `overlay`（カード全面へ拡張されるリンク）の 2 anatomy パーツで構成し、chakra-ui の LinkBox/LinkOverlay パターンに倣ってカード全体をクリック可能にします。`overlay` 以外の子ノード（見出し・画像等）が `root` の高さを確立する契約です。キーボード操作時は `overlay` の `:focus-visible` にカード全面を囲むフォーカスリングが表示され（`border-radius: inherit` により `root` に角丸を与えた場合はリングもその角丸へ追従します）、マウス操作では表示されません。`overlay` には `cursor: pointer` を明示しています。枠線・余白・角丸を含むカード意匠自体は本部品の責務外で、`card` 等との合成で与えてください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
