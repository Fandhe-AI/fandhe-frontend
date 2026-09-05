# Navigation Menu

トリガーのクリックで開くナビゲーションパネル（高々 1 項目のみ開く）です。
`fandhe-frontend-headless-ui` の `navigation_menu` mod は Root / List / Item /
Trigger / ItemIndicator / Content / Link の 7 anatomy パーツを提供する
unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。
`nav_list`（ディスクロージャなしの静的リンク集）とは、開閉状態機械の有無で
使い分けます。

`NavigationMenuProps { orientation }`（既定 `Horizontal`）を root / list /
item / content へ渡すと `data-orientation` が出力され、`item` / `content` は
`value` を `data-value` として出力します。本部品は SSR での属性出力と状態
機械のみを担い、click → dispatch 配線・キーボード操作・Escape キーでの
閉鎖は `fandhe-frontend-wasm-full` が実 DOM 配線を担います。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter / Space | Trigger | claim しません（ネイティブ `<button>` の click → `"toggle"` dispatch）。 |
| ArrowRight / ArrowLeft（`orientation="horizontal"`）、ArrowDown / ArrowUp（`orientation="vertical"`） | Trigger | Trigger 間のフォーカスを移動します（disabled をスキップ、既定は循環なし。`data-loop-focus="true"` で循環）。 |
| Home / End | Trigger | 先頭 / 末尾の非 disabled Trigger へフォーカスします。 |
| ArrowDown / ArrowUp（`orientation="horizontal"`）、ArrowRight / ArrowLeft（`orientation="vertical"`） | Trigger | closed なら click を合成して開き先頭 / 末尾リンクへ、open なら合成なしで先頭 / 末尾リンクへフォーカスします。 |
| ArrowDown / ArrowUp / ArrowRight / ArrowLeft / Home / End | Content 内 Link | 同一 Content 内の非 disabled リンク間を非循環で移動します。 |
| Escape | open 中の Trigger / Content 内 Link | click を合成して閉じ、Trigger へフォーカスを戻します（closed の Trigger 上は no-op）。 |
| Tab / Shift+Tab | — | 配線はありません（roving tabindex 不採用、全ボタン・リンクがタブ順に残ります）。 |

**参考サイトとの差分**

ark-ui NavigationMenu・Radix Primitives Navigation Menu と突合し、
`ItemIndicator` パートを新設（6 → 7 anatomy パーツ）、`NavigationMenuProps`
による `data-orientation`（root/list/item/content）、`data-value`
（item/content）を追加し、各パーツ関数の呼び出し側 `attrs` から固定属性
キーの偽装を除去する `drop_reserved` を追加しました。一方、以下は意図的に
合わせていません。

- **Indicator（スライドバー）/ Viewport / ViewportPositioner / Arrow**: レイアウト計測を伴う装飾関心のため headless 層には持ち込みません（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- **Sub（入れ子ナビゲーション）**: 状態機械の入れ子は未実装です。
- **hover / delay による自動 open・open-follows-focus・typeahead**: クリック起点の開閉のみをサポートし、ホバーでの自動展開は実装しません。
- **`data-trigger-proxy-id`（ark-ui）**: 実行時 proxy 要素向けの内部属性のため不採用です。
- **Radix の `data-active`（link）**: ark-ui 語彙の `data-current` へ統一しています。
- **`data-motion`**: viewport 寸法測定と同様、装飾・アニメーション関心のため未実装です。

スタイル済みの表示例は [Navigation Menu](../themes/navigation-menu.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、
`[data-scope="navigation-menu"][data-part="..."]` セレクタでスタイルを当てます。
`content` には `[hidden]` ガードを必ず含めます（closed のとき `hidden`
存在属性が同一フレームで即時付与・除去されるため、CSS トランジションは
開閉どちらの向きにも発火しません）。

```css
[data-scope="navigation-menu"][data-part="content"][hidden] {
  display: none;
}

[data-scope="navigation-menu"][data-part="trigger"][data-state="open"] {
  font-weight: 600;
}

[data-scope="navigation-menu"][data-part="link"][data-current] {
  text-decoration: underline;
}

[data-scope="navigation-menu"][data-part="item"][data-disabled] {
  opacity: 0.5;
}
```
