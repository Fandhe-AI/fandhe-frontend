# Action Bar

複数選択（チェックボックス等）に対する一括操作を提示する画面下部固定の操作バーです。`fandhe-frontend-headless-ui` の `action_bar` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。開閉状態は Disclosure を埋め込んだ状態機械が管理しますが、「選択操作から開閉状態を決定する」判断自体は呼び出し側アプリケーションの責務です。`content` は `role="dialog"`（非モーダル）で表され、開状態のときのみ `data-expanded` が付与されます。

スタイル済みの表示例は [Action Bar](../themes/action-bar.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Escape | Content | バーを閉じます（`fandhe-frontend-wasm-full` の `OverlayKind::ActionBar` が配線。`data-close-on-escape="false"` で opt-out できます）。 |
| Tab / Shift+Tab | Root 配下のボタン | ネイティブ `button` 要素間のフォーカス移動です。開いた瞬間にフォーカスが自動で移動することはありません（`content` は `tabindex="-1"` を固定で持ち、chakra-ui の `autoFocus: false` と同じ挙動です）。 |

**参考サイトとの差分（イシュー #1647）**

ark-ui・Radix Primitives のいずれにも ActionBar 相当のコンポーネントは存在しません。参照できるのは chakra-ui の ActionBar のみで、その実体は独自の状態機械を持たず Ark Popover（zag.js `popover.connect`）をそのまま再利用しています。このため本部品の属性仕様は zag.js popover の content/close-trigger の出力を基準にしています。

- **是正した点**: `content` の `role` を `"toolbar"`（矢印キーでの roving tabindex を伴わない不完全な適用でした）から `"dialog"`（非モーダル、`aria-modal` は付与しません）へ変更しました。あわせて開状態のときのみ `data-expanded` を付与し、`tabindex="-1"` を固定で付与するようにしました（呼び出し側が `tabindex` を指定していれば出力しません）。`close-trigger` は呼び出し側が `aria-label` を指定しなければ既定値 `"close"`（zag.js popover の `translations.closeTrigger` 既定値と同じ）を出力します。
- **意図的に合わせていない点**: `root`（hydration のルート要素として DOM を持つ必要があるため、DOM を描画しない参考サイトの `Root` とは異なります）・`positioner`（`hidden` 存在属性を pre-styled-ui が利用するため付与しており、参考サイトは素の `div` です）・`separator`（`role="separator"` + `aria-orientation="vertical"` をアクセシビリティ上の superset として維持しており、参考サイトは素の `div` です）。`data-placement`/`data-side`（配置バリエーション）はスタイル層（`fandhe-frontend-pre-styled-ui`）の責務としてスコープ外のままです（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。外側クリックでの閉鎖（`closeOnInteractOutside`）も既定で無効のまま opt-in 属性を持ちません — 選択操作用のチェックボックス等は ActionBar の外側（一覧側）に存在するため、外側クリックでの誤閉鎖を防ぐ安全側の判断です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`data-scope`/`data-part` 属性セレクタと `data-state`/`data-expanded` の有無で見た目を組み立てます。

```css
[data-scope="action-bar"][data-part="positioner"] {
  position: fixed;
  bottom: 1.5rem;
  left: 50%;
  transform: translateX(-50%);
}

[data-scope="action-bar"][data-part="content"][data-state="open"] {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

[data-scope="action-bar"][data-part="close-trigger"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
