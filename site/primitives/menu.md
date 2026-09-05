# Menu

トリガーのクリックで開くアクション項目リスト（ドロップダウンメニュー）です。
`fandhe-frontend-headless-ui` の `menu` mod は Root / Trigger / Indicator /
Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator /
ItemGroup / ItemGroupLabel / Separator / TriggerItem / ContextTrigger /
CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを提供する
unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。
CheckboxItem/RadioItemGroup による checked 状態やサブメニューの入れ子構成
にも対応します。本部品は SSR での属性出力と状態機械のみを担い、
click → dispatch 配線・キーボード操作・Escape キーでの閉鎖は
`fandhe-frontend-wasm-full` が実 DOM 配線を担います。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| ArrowDown / ArrowUp / Enter / Space | Trigger（closed） | Menu を開き、先頭（ArrowUp は末尾）の非 disabled 項目を highlight します。 |
| ArrowDown / ArrowUp / Home / End | Content（open） | highlight 中の項目を次/前/先頭/末尾の非 disabled 項目へ移動します。既定は循環なし、`data-loop-focus="true"` で有効化できます。 |
| Enter / Space | Content（open、highlight 中の項目） | highlight 中の項目へ click を合成します。Item は利用者の click ハンドラへ、TriggerItem は `"toggle"` を dispatch します。CheckboxItem/RadioItem は click 合成による checked トグルの配線が未実装です。 |
| 印字可能文字 | Content（open） | typeahead（350ms バッファ）。ItemText 子があればそのテキストを優先してマッチします。 |
| ArrowRight | TriggerItem（highlight 中） | 非 disabled でサブメニューが解決できるときのみサブメニューを展開します。 |
| ArrowLeft | サブメニュー内 | 親 TriggerItem へ復帰しサブメニューを閉じます。 |
| Escape | — | 最上位のメニューを閉じます。`data-close-on-escape="false"` で無効化できます。 |
| Tab / Shift+Tab | — | 配線はありません（ブラウザ既定）。 |

**参考サイトとの差分**

ark-ui Menu・Radix Primitives Dropdown Menu・chakra-ui Menu と突合し、
`ItemText`/`ItemIndicator` の 2 パーツを新設し（16 → 18 anatomy パーツ）、
各パーツ関数の呼び出し側 `attrs` から固定属性キーの偽装を除去する
`drop_reserved` を追加しました。一方、以下は意図的に合わせていません。

- **Radix `Portal`**: DOM 配置の関心のため不採用です。
- **zag の `data-placement`・Radix content の `data-side`/`data-align`**: `positioner` へ `attrs` 経由で渡す既存設計のため置き換えていません。
- **Radix の `data-orientation`（content/item）**: 不採用です。
- **zag content の `tabindex="0"` + item への実 DOM フォーカス移動**: trigger にフォーカスを留め `aria-activedescendant` + `data-highlighted` で仮想フォーカスを表現する設計のため未採用です。
- **chakra-ui の `ItemCommand`**（ショートカット表示）: Themes 層（`fandhe-frontend-pre-styled-ui`）の装飾関心のため headless anatomy には持ち込みません。
- **`asChild`**: 本リポジトリ全体で保留継続中の意図的非採用方針です。
- **ark-ui の `closeOnSelect`/`lazyMount`/portal**: クライアント配置・実行時関心のため未採用です。
- **Escape 後の「trigger へのフォーカス復帰」**: 本実装がそもそも trigger からフォーカスを離さない設計のため、構造的に同等の結果になります（追加実装は不要）。
- **checkbox-item/radio-item への Enter/Space（click 合成による checked トグル）**: `fandhe-frontend-wasm-full` の `MAPPING_TABLE` に対応行が無く未実装です。

スタイル済みの表示例は [Menu](../themes/menu.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="menu"][data-part="..."]`
セレクタでスタイルを当てます。`positioner`/`content`/`item-indicator` には
`[hidden]` ガードを必ず含めます。

```css
[data-scope="menu"][data-part="positioner"][hidden],
[data-scope="menu"][data-part="content"][hidden],
[data-scope="menu"][data-part="item-indicator"][hidden] {
  display: none;
}

[data-scope="menu"][data-part="item"][data-highlighted] {
  background: #eff6ff;
}

[data-scope="menu"][data-part="item"][data-disabled] {
  color: #9ca3af;
}

[data-scope="menu"][data-part="checkbox-item"][data-state="checked"] {
  font-weight: 600;
}
```
