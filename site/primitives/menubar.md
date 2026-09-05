# Menubar

複数の Menu を水平（または垂直）に並べたアプリケーションメニューバーです。
`fandhe-frontend-headless-ui` の `menubar` mod は Root / Menu / Trigger /
Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator /
ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent /
CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを提供する
unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません
（イシュー #1652 で Radix Primitives Menubar と参照突合し、11 → 18 パーツへ
拡充しました）。開いている Menu を跨いだ左右移動の状態遷移 API
（`MenubarAction`）を持ちます。矢印キー・Home/End・typeahead の実 DOM 配線
は `fandhe-frontend-wasm-full` が担います。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| ArrowRight / ArrowLeft（垂直配置は ArrowDown / ArrowUp） | Trigger | 隣の Trigger へフォーカスを移動します。ある Menu が開いていれば、開く Menu もフォーカス移動に追随します（open-follows-focus）。 |
| ArrowDown / Enter / Space | Trigger（closed） | Menu を開き、先頭の非 disabled 項目を highlight します。 |
| ArrowUp | Trigger（closed） | Menu を開き、末尾の非 disabled 項目を highlight します。 |
| ArrowDown / ArrowUp / Home / End | Content（open） | highlight 中の項目を次/前/先頭/末尾の非 disabled 項目へ移動します。 |
| 印字可能文字 | Content（open） | typeahead。ItemText 子があればそのテキストを優先してマッチします。 |
| ArrowRight | SubTrigger（highlight 中、水平配置） | サブメニューを展開します。 |
| ArrowLeft | サブメニュー内（水平配置） | 親 SubTrigger へ復帰しサブメニューを閉じます。 |
| Escape | — | 開いている Menu を閉じます。フォーカスは元々 Trigger から離れないため、フォーカス復帰は構造的に成立します。 |
| Tab / Shift+Tab | — | 配線はありません（ブラウザ既定）。roving tabindex により `tabindex="0"` の Trigger のみが Tab 順序に含まれます。 |

**参考サイトとの差分**

参照基準は Radix Primitives Menubar のみです（ark-ui / chakra-ui / Radix
Themes には Menubar 相当が存在しません）。Arrow/ArrowTip/ItemText/
ItemIndicator/CheckboxItem/RadioItemGroup/RadioItem の 7 パーツを新設し
（11 → 18 anatomy パーツ）、各パーツ関数の呼び出し側 `attrs` から固定属性
キーの偽装を除去する `drop_reserved` を全パーツへ適用しました。一方、
以下は意図的に合わせていません。

- **Radix `Portal`**: DOM 配置の関心のため不採用です（`positioner` が配置コンテナとして表現）。
- **content/sub-content の `data-side`/`data-align`**: `positioner` へ `attrs` 経由で渡す既存設計のため置き換えていません。
- **sub-content の `data-orientation`**: 不採用です。
- **RTL `dir`**: 未対応です。
- **Root の `value`/`defaultValue`**: 制御値は `Menubar` 状態機械の `open: Option<usize>` が担います。
- **Trigger の Space/Enter による実 DOM フォーカス移動**: Trigger にフォーカスを留め `data-highlighted`（+ 実行時の `aria-activedescendant`）で仮想フォーカスを表現する設計のため未採用です。

**既知のギャップ**（`fandhe-frontend-wasm-full` 側の後続対応）: CheckboxItem/
RadioItem は highlight 移動・typeahead・Enter/Space による checked トグルの
配線が未実装です（`menu` の CheckboxItem/RadioItem と同じ既知ギャップ）。

スタイル済みの表示例は [Menubar](../themes/menubar.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、
`[data-scope="menubar"][data-part="..."]` セレクタでスタイルを当てます。
`positioner`/`content`/`sub-content`/`item-indicator` には `[hidden]`
ガードを必ず含めます。

```css
[data-scope="menubar"][data-part="positioner"][hidden],
[data-scope="menubar"][data-part="content"][hidden],
[data-scope="menubar"][data-part="sub-content"][hidden],
[data-scope="menubar"][data-part="item-indicator"][hidden] {
  display: none;
}

[data-scope="menubar"][data-part="trigger"][data-state="open"] {
  background: #eff6ff;
}

[data-scope="menubar"][data-part="item"][data-highlighted] {
  background: #eff6ff;
}

[data-scope="menubar"][data-part="item"][data-disabled] {
  color: #9ca3af;
}

[data-scope="menubar"][data-part="checkbox-item"][data-state="checked"] {
  font-weight: 600;
}
```
