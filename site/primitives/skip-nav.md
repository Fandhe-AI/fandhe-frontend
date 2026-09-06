# Skip Nav

キーボード操作時のみ視覚的に現れる「本文へスキップ」リンクです（WCAG 2.1 SC 2.4.1 Bypass Blocks）。`fandhe-frontend-headless-ui` の `skip_nav` mod は link / content の 2 anatomy パーツを提供し、`href` に任意スキームを受け付けず常に `#<id>` フラグメントのみを組み立てます。Themes 版と異なり focus 時のみ表示する CSS を持たず、構造とフォーカス移動属性のみを担います。

スタイル済みの表示例は [Skip Nav](../themes/skip-nav.md) を参照してください。

**アクセシビリティ・参考サイトとの対応**

- anatomy は chakra-ui の `SkipNavLink`/`SkipNavContent` と一致します（Ark UI・Radix Primitives・Radix Themes には該当部品がありません）。
- `data-*`/`role`/`aria-*` はいずれも、参照サイト（chakra-ui）・本実装ともに出力しません。
- chakra-ui の `SkipNavContent` が出力する inline `outline: 0` は装飾のため採用していません（Themes 版が CSS で提供します）。
- `href`/`id`/`tabindex` は呼び出し側 attrs に同名キーが含まれていても fail-closed に除去します。chakra-ui は `id`/`tabIndex` を呼び出し側から上書きできますが、本実装は意図的により厳格な挙動を採っています。
- キーボード操作は独自のキーリスナを持たず、ネイティブ `<a>` の挙動（Tab でフォーカス → Enter でフラグメント遷移 → content へのフォーカス移動）に委ねます。link はページのできるだけ先頭に置いてください。
- 自前 CSS で組み立てる最小例はこのページ下部の Examples 節（「Custom CSS」）を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
