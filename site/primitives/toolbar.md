# Toolbar

ボタン・リンク・ToggleGroup を横方向（または縦方向）にグループ化する操作バーです。`fandhe-frontend-headless-ui` の `toolbar` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。roving tabindex により disabled 項目もフォーカス順序から除外しない WAI-ARIA APG 準拠の設計です。

スタイル済みの表示例は [Toolbar](../themes/toolbar.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Root | roving tabindex により focused 項目（`tabindex="0"`）のみが Tab 順序に含まれます（ネイティブ `button`/`a` 要素の暗黙挙動、実装済み）。 |
| Space / Enter | Button / ToggleItem | ネイティブ `button` 要素の暗黙アクティベーション（実装済み）。 |
| ArrowRight / ArrowLeft（horizontal）・ArrowDown / ArrowUp（vertical）、Home、End | 項目間 | `Toolbar`（`ToolbarAction::Next`/`Prev`/`First`/`Last`/`Focus`）として状態遷移 API のみを提供します。**`fandhe-frontend-wasm-full` に本モジュールの keydown 配線は現状存在せず未実装です**（イシュー #1657 時点で確認済み、別 Issue の起票対象）。呼び出し側が独自に配線するまで実 DOM 上では矢印キー等は機能しません。 |

**参考サイトとの差分**

イシュー #1657 で Radix Primitives Toolbar（ark-ui・chakra-ui には Toolbar 相当が存在しません）と突合しました。

- **`data-orientation` の欠落を是正**: Radix の `data-*` 表（root/button/toggle-group/toggle-item に `data-orientation`）と一致するよう、`button`/`link`/`toggle_group`/`toggle_item` の自由関数へ先頭引数 `orientation: Orientation` を追加し `data-orientation` を出力するようにしました（`separator` は既存の直交 `aria-orientation` と同値の `data-orientation` を追加）。`link` への追加は Radix の `data-*` 表には載りませんが、実 DOM（RovingFocusGroup.Item 経由）では出力される値であり、`button`/`toggle_item` との対称性を優先した superset です。
- **予約キーなりすまし除去を追加**: 呼び出し側 `attrs` が `role`/`aria-*`/`data-*`/`type`/`tabindex` 等の固定付与属性を偽装・重複出力できないよう `drop_reserved` を導入しました（`tabs`/`nav-list` 等と同型のパターン）。

一方で以下は意図的に Radix と合わせていません。

- **Radix Button の native `disabled` 透過**: RovingFocus が `focusable={!disabled}` でフォーカス順序から除外しますが、本実装は WAI-ARIA APG 推奨（disabled もフォーカス可能）に従い `aria-disabled="true"` + `data-disabled` + `tabindex` を維持します。
- **`dir`（RTL）**: 本リポジトリ横断で未採用です。
- **`asChild`・`loop` 既定値 `true`**: 本実装は `loop_focus` を呼び出し側が明示する設計です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="toolbar"][data-part="..."]` セレクタと `data-state`/`data-disabled`/`data-orientation` で見た目を組み立てます。

```css
[data-scope="toolbar"][data-part="root"] {
  display: flex;
  gap: 0.25rem;
}

[data-scope="toolbar"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="toolbar"][data-part="separator"][data-orientation="vertical"] {
  width: 1px;
  align-self: stretch;
}

[data-scope="toolbar"][data-part="toggle-item"][data-state="on"] {
  background: #dbeafe;
}

[data-scope="toolbar"] [data-disabled] {
  opacity: 0.5;
}

[data-scope="toolbar"] :focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
