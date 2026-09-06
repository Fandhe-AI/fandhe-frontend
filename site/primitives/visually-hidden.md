# Visually Hidden

視覚的には隠すが支援技術（スクリーンリーダー）には読ませ続けるテキストコンテナです。`fandhe-frontend-headless-ui` の `visually_hidden` mod は Root（`span`）の 1 パーツのみで構成され、装飾要素の `aria-hidden="true"` 固定付与パターンとは逆に `aria-hidden` を意図的に付与しません。Themes 版と異なり clip 手法の CSS を持たず、構造の出力のみを担います。

スタイル済みの表示例は [Visually Hidden](../themes/visually-hidden.md) を参照してください。

**アクセシビリティ・参考サイトとの対応**

- anatomy は Radix Primitives の `VisuallyHidden.Root`（`span`）・chakra-ui の `VisuallyHidden`（`span`）と一致します（Ark UI には該当ページがありません）。
- `role`/`aria-*`/`data-*` はいずれも、参照サイト（Radix Primitives・Radix Themes・chakra-ui）・本実装ともに固有のものを出力しません。本実装が出力する `data-scope="visually-hidden"`/`data-part="root"` は role/aria-* の代替ではない独自フックです。`aria-hidden` を自ら付与しない不変条件も参照サイトと一致します。
- キーボード操作はありません（非対話要素）。ボタン内で使う場合はボタン側がフォーカス・操作を担います。
- Radix の `asChild`・chakra-ui の `as`/`asChild`（要素差し替え API）は本フレームワークのノード木 API とは前提が異なるため採用していません。視覚的に隠した入力（chakra-ui の `asChild + <input>` 用法）は別パートを新設せず、`checkbox`/`switch`/`radio_group`/`select` の hidden input 系パーツが同用途を担います。
- 自前 CSS で組み立てる最小例はこのページ下部の Examples 節（「Custom CSS」）を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
