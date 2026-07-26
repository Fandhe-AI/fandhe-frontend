# Link Overlay

カード全体をクリック可能にする（カード全面クリック化）ための部品です。`fandhe-frontend-headless-ui` の `link_overlay` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`overlay` パーツを `root` 全面へ展開するのは呼び出し側の CSS（`position: absolute; inset: 0;`）の責務です。

スタイル済みの表示例は [Link Overlay](../themes/link-overlay.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
