# Link

汎用のインラインリンク（`a` 要素 1 パーツ）です。`fandhe-frontend-headless-ui` の `link` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`root`（`a`）1 パーツのみで、時間変化する内部状態を持たないため `data-current` 以外の状態語彙は一切出力しません。`external` を true にすると `target="_blank"` と `rel="noopener noreferrer"` を不可分に付与し、reverse tabnabbing を防ぎます。

スタイル済みの表示例は [Link](../themes/link.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Root | ネイティブ `a[href]` へのフォーカス移動です（ブラウザ既定、headless-ui 側の配線なし）。 |
| Enter | Root | ネイティブ `a[href]` の起動（遷移）です。**Space はリンクを起動しません**（`<a>` 要素は Space キーでは発火しないブラウザ標準挙動です）。 |

**参考サイトとの差分**

chakra-ui の Link（`variant`/`colorPalette`/`asChild` のみを持つ styled `a`）・Radix Themes の Link（`size`/`weight`/`underline`/`color`/`highContrast`/`asChild` 等のスタイル prop のみを持つ styled `a`）はいずれも Anatomy 節・Keyboard Interactions 節・`data-*` 語彙を持ちません（ark-ui・Radix Primitives には Link 相当のコンポーネント自体が存在しません）。本実装は以下の点で意図的に差分を残しています。

- **`data-scope`/`data-part`/`data-current`**: 参考サイトには anatomy の概念自体が無く、本実装のこれらの属性はこちら側の superset です。
- **`external` の `target`+`rel` 不可分付与**: 参考実装は生の `target`/`rel` 属性を利用者が渡す設計ですが、本実装は `external` 引数の true/false のみで両属性をまとめて出力/省略し、片方だけを付与できる API を公開しません（reverse tabnabbing 対策を API 側で保証する意図的差分）。
- **非対応の prop**: `asChild`（Slot 相当）・`variant`・`colorPalette`・`size`・`underline`・`highContrast`・`truncate`・`wrap`・`trim` はいずれも非対応です。`asChild` は headless-ui 全体での再導入検討事項（保留）であり、装飾軸（`variant` 以下）は Themes 層 `pre-styled-ui::link`（`/themes/link/`）の責務です。
- **`disabled` の非提供**: `a` 要素にはネイティブの disabled 意味論が無く、参考サイトも `disabled` を提供しません。無効状態が必要な場合は、呼び出し側で `link::root` の呼び出し自体を止め、非操作要素へ描画を差し替えてください。

一方で「`role`/`aria-*` を独自付与しない（暗黙の `link` ロールに委ねる）」点は参考サイトと一致しています。危険な URL スキーム（`javascript:` 等）は `href` 属性ごと拒否され、`href` を失った `a` はフォーカス不能になり暗黙の `link` ロールも失います（fail-closed の意味論的帰結）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="link"][data-part="root"]` セレクタと `:hover`・`:focus-visible`・`[aria-current="page"]` 擬似クラス／属性セレクタでスタイルを当てます。

```css
[data-scope="link"][data-part="root"] {
  color: #2563eb;
  text-decoration: none;
}

[data-scope="link"][data-part="root"]:hover {
  text-decoration: underline;
}

[data-scope="link"][data-part="root"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="link"][data-part="root"][aria-current="page"] {
  color: #111827;
  font-weight: 600;
  text-decoration: none;
}

[data-scope="link"][data-part="root"][target="_blank"]::after {
  content: " ↗";
}
```
