# Avatar

プロフィール画像とフォールバック表示を切り替える部品です。`fandhe-frontend-headless-ui` の `avatar` mod は画像読み込みステータス（loading/loaded/error）を管理する状態機械を提供し、Root / Image / Fallback の 3 anatomy パーツで構成されます。Themes 版と異なりスタイル（外形・サイズ）は一切持たず、`data-state` による表示切り替えのみを担います。

スタイル済みの表示例は [Avatar](../themes/avatar.md) を参照してください。

**アクセシビリティ・参考サイトとの対応**

- `root`/`image`/`fallback` はいずれも `role`/`aria-*` を付与しません。`image` の `alt` を必須引数にすることが実質的なアクセシビリティ担保です。
- キーボード操作はありません（参照サイト 4 件〔ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui〕もキーボード操作表を持たない非インタラクティブな表示系コンポーネントです）。
- `data-state`（`visible`/`hidden`）は `image`/`fallback` のみが持ちます。`root` へは付与しません（ark-ui 準拠）。
- Radix Primitives の `Fallback` が持つ `delayMs`（表示遅延によるフラッシュ回避）は意図的に採用していません。JS なしの SSR では `hidden` 存在属性 + `data-state` で表示制御が成立しており、遅延が必要な場合は呼び出し側（`fandhe-frontend-wasm-full`/`fandhe-frontend-pre-styled-ui`）で実装してください。自前 CSS で組み立てる最小例は [API Reference](../../docs/api/headless-ui-api.md) の Avatar 節を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
