# Tabs

WAI-ARIA APG の Tabs パターンに準拠したタブ切り替え UI です。`fandhe-frontend-headless-ui` の `tabs` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。`ActivationMode`（Automatic/Manual）や `Orientation`（Horizontal/Vertical）を切り替えられます。

スタイル済みの表示例は [Tabs](../themes/tabs.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Root | roving tabindex により active（無ければ最初の非 disabled）trigger へフォーカス移動します。trigger から Tab で active content（`tabindex="0"`、inactive は `hidden`）へ移動します（配線なし、ブラウザ既定）。 |
| ArrowRight / ArrowLeft（horizontal）・ArrowDown / ArrowUp（vertical） | Trigger | 次/前の非 disabled trigger へフォーカスを移動します（`fandhe-frontend-wasm-full` の `keynav.rs::tabs_next_index`）。`data-loop-focus="false"` のときのみ端で停止し、既定（`true`）では反対端へ循環します。`ActivationMode::Automatic` では同時に活性化します。 |
| Home / End | Trigger | 最初/最後の非 disabled trigger へフォーカスを移動します。Automatic では同時に活性化します。 |
| Enter / Space | Trigger | ネイティブ button の click 委譲（`handle_trigger_click`）で活性化します。`ActivationMode::Manual` での主な活性化手段で、Automatic でもクリックと同じ経路で動作します。disabled trigger は no-op です。 |

**参考サイトとの差分**

イシュー #1656 で ark-ui（Zag.js）/ Radix Primitives / Radix Themes / chakra-ui と突合しました。本実装は Radix Primitives の `data-*` 表（root: `data-orientation` / list: `data-orientation` / trigger: `data-state="active"|"inactive"`, `data-disabled` / content: `data-state`, `data-orientation`）と属性単位で一致し、ark-ui の 5 パーツ（root/list/trigger/content/indicator）とも一致するため、**是正は行っていません**（パート・`data-*` の増減なし）。

- **ark-ui の `data-selected`（trigger/content）**: 本実装は Radix 語彙 `data-state="active"|"inactive"` を #528 で採用済みで、同義のため非採用です。
- **ark-ui の `data-focus`（root/list/trigger）・`data-ssr`（trigger）**: 実行時フォーカス状態・ハイドレーション判定マーカーであり、SSR 静的出力の責務外です。`:focus-visible` で代替できます。
- **ark-ui の `lazyMount`/`unmountOnExit`/`hideMode`/`composite`/`navigate`/`onValueChange`/`onFocusChange`/制御 `value`、Radix Content の `forceMount`**: 実行時マウント制御・イベント関心のため非採用です。本実装は全 content を常時マウントし inactive を `hidden` で隠します（`forceMount` 相当の固定挙動）。
- **ark-ui の `deselectable`**: SSR では `TabsProps::selected` がどの value にも一致しない場合に全 inactive で描画できるため、静的表現としては充足しています。実行時の解除トグルは wasm 層の将来課題です。
- **Radix Root の `dir`**: RTL は本リポジトリ横断で未採用です。
- **chakra-ui の `Tabs.ContentGroup`（アニメーション用ラッパー）と `variant`/`size`/`fitted`/`justify`/`colorPalette`**: 装飾・アニメーション関心は headless 層へ持ち込まず、Themes 層（`pre-styled-ui::tabs`）の責務です。
- **Radix Themes の `TabNav`**: 別部品（Themes 層 `tab_nav`）が担うため対象外です。

一方で、本実装は list の `data-activation-mode`/`data-loop-focus`（`fandhe-frontend-wasm-full` との契約）・indicator の `data-state`/`aria-hidden="true"`/`hidden`（active なし時の fail-safe）を参照サイトに無い superset として持ちますが、これらは削除していません。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="tabs"][data-part="..."]` セレクタと `data-state`/`data-disabled`/`data-orientation`/`[hidden]` で見た目を組み立てます。

```css
[data-scope="tabs"][data-part="list"] {
  display: flex;
  gap: 0.5rem;
}

[data-scope="tabs"][data-part="root"][data-orientation="vertical"] [data-part="list"] {
  flex-direction: column;
}

[data-scope="tabs"][data-part="trigger"][data-state="active"] {
  border-bottom: 2px solid #2563eb;
  font-weight: 600;
}

[data-scope="tabs"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
}

[data-scope="tabs"][data-part="trigger"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

[data-scope="tabs"][data-part="content"][hidden] {
  display: none;
}

[data-scope="tabs"][data-part="indicator"] {
  position: absolute;
  left: var(--left);
  width: var(--width);
}
```
