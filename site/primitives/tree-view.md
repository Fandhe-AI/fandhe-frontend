# Tree View

階層構造の展開・折りたたみ・選択を扱う部品です。`fandhe-frontend-headless-ui` の `tree_view` mod は 12 anatomy パーツと、展開集合 + 選択値を合成した状態機械を提供し、WAI-ARIA APG の Tree パターンに従う `role="tree"`/`role="treeitem"`/`role="group"` と `aria-expanded`/`aria-selected`/`aria-level` 等を出力します。Themes 版と異なりインデントガイドの見た目を持たず、構造とアクセシビリティ属性のみを担います。

**キーボード操作**

矢印キー・Home/End・Enter/Space・typeahead の DOM 配線は `fandhe-frontend-wasm-full` の `keynav.rs`（イシュー #1072）が担います（headless-ui 自体は SSR 静的マークアップのみ提供します）。

| キー | 効果 |
|---|---|
| ArrowDown / ArrowUp | 可視かつ disabled でない treeitem 間で 1 件ずつフォーカス移動します（折りたたみ中の子孫はスキップ、非循環）。 |
| ArrowRight | 閉じたブランチは展開します。開いたブランチは最初の子へフォーカス移動します。葉ノードでは no-op です。 |
| ArrowLeft | 開いたブランチは折りたたみます。それ以外（葉ノード・閉じたブランチ）は親ブランチへフォーカス移動します（ルート直下では no-op）。 |
| Home / End | 可視かつ disabled でない最初/最後の treeitem へフォーカス移動します。 |
| Enter / Space | 葉ノードは選択（select）、ブランチは開閉（toggle）を発火します。ブランチ自体は選択できません。 |
| 印字可能文字 | typeahead: 直近の入力から一致する treeitem へフォーカス移動します。 |
| Escape | typeahead バッファをリセットします。 |

**参考サイトとの差分（イシュー #1667）**

ark-ui docs の Data Attributes / Keyboard 表と zag.js
`packages/machines/tree-view/src/tree-view.connect.ts` と突合し、以下を
是正しました。Radix Primitives に Tree View 相当は存在しないため突合対象に
含めていません。

- **`branch` へ `data-branch`（= ノード値）を追加**しました。既存の
  `data-value` と同値ですが、`[data-part="branch"][data-branch]` セレクタで
  ブランチ限定スタイルを書ける ark docs 準拠の別名です。
- **`branch-control` へ `data-value`/`data-depth` を追加**しました。
- **`branch-indicator` へ `data-disabled`/`data-selected`/`aria-hidden="true"`
  を追加**しました（装飾アイコンを支援技術から隠す ARIA APG の一般的な
  慣例）。
- **`branch-text` へ `data-state`/`data-disabled` を追加**しました。
- **`branch-content` へ `data-depth`/`data-value` を追加**しました。
- **`branch-indent-guide` へ `data-depth` を追加**しました。
- **`item-text` へ `data-selected`/`data-disabled` を追加**しました。
- **`item-indicator` へ `data-disabled`/`aria-hidden="true"`/非選択時
  `hidden` を追加**しました（選択されていない葉では選択マークを DOM 上
  非表示にします）。
- **呼び出し側 `attrs` からの固定属性キー偽装を除去する `drop_reserved`
  を全パーツへ追加**しました。

一方、以下は意図的に合わせていません。

- **`data-focus`/`data-renaming`/`data-checked`/`data-indeterminate`/
  `data-loading`/`aria-busy`**: focus・rename・checkbox・lazy loading という
  実行時ローカル状態であり、SSR に実データがないため非採用です。
- **`data-path`/`data-ownedby`/`id`/`dir`/`tabindex`/`--depth` style**:
  `fandhe-frontend-wasm-full` は `data-value` の文字列比較と document 順
  走査で親子解決するため不要です。`tabindex` は wasm-full の roving
  tabindex が実行時に付与します。
- **`aria-multiselectable`**: 本部品は単一選択（`SingleSelect`）のみを
  提供するため対象外です。
- **`aria-current="true"`（zag の item）**: WAI-ARIA APG Tree では
  `aria-selected` が選択の正であり重複させません。
- **`role="button"` on branch-control**: APG ではフォーカス可能要素は
  treeitem（`branch`）が担うため、入れ子の widget ロールを避けています。
- **branch-trigger / node-checkbox / node-rename-input**: ark Anatomy 図に
  載らないパーツです。checkbox モード・inline rename は本部品のスコープ外
  です。
- **`*`（兄弟一括展開）・Shift+Arrow / Ctrl+A（複数選択前提）・F2（rename）**:
  いずれも zag.js のみが持つ拡張操作、または複数選択・rename 前提の操作の
  ため非採用です。

**APG superset として維持**しているもの:

- `aria-posinset`/`aria-setsize`（zag は出力しません）。
- disabled 時も明示する `aria-selected="false"`（zag は disabled 時に
  省略します）。

**`data-depth` の起点**: zag.js は `depth = indexPath.length`（トップレベル
= 1 起点）ですが、本部品は 0 起点を意図的に維持しています。`aria-level` が
1 起点の深さを既に担っており、`fandhe-frontend-wasm-full` の `keynav.rs` が
`depth == 0` をルート判定に使う実装との整合を優先しました。

スタイル済みの表示例は [Tree View](../themes/tree-view.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="tree-view"][data-part="..."]`
セレクタでスタイルを当てます。`branch-content`/`item-indicator` は base
規則で `display` を宣言する構成にすると UA 既定の `[hidden] { display:
none }` を上書きしてしまうため、`[hidden]` 属性セレクタで明示的に上書きが
必要です。ただし上書き先は両者で異なります。`branch-content` は子ノード列
そのものを畳むコンテナのため `display: none` で問題ありませんが、
`item-indicator` は選択マーク幅の列を揃える整列用スペーサでもあるため
`display: none` にすると非選択葉の `item-text` が選択済み葉・
`branch-text` に対して列幅分だけ左へ寄る視覚崩れが生じます。
`visibility: hidden` でレイアウトボックスを残したまま非表示にします
（支援技術からの除外は headless 層が固定出力する `aria-hidden="true"` が
既に担います）。

```css
[data-scope="tree-view"][data-part="branch-content"][hidden] {
  display: none;
}

[data-scope="tree-view"][data-part="item-indicator"][hidden] {
  visibility: hidden;
}

[data-scope="tree-view"][data-part="branch-indent-guide"] {
  width: 1rem;
}

[data-scope="tree-view"][data-part="branch-control"][data-selected],
[data-scope="tree-view"][data-part="item"][data-selected] {
  background: #eff6ff;
}

[data-scope="tree-view"][data-part="branch-control"][data-disabled],
[data-scope="tree-view"][data-part="item"][data-disabled] {
  opacity: 0.5;
}

[data-scope="tree-view"][data-part="branch-control"]:focus-visible,
[data-scope="tree-view"][data-part="item"]:focus-visible {
  outline: 2px solid #2563eb;
}
```
