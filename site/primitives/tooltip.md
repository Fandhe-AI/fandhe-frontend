# Tooltip

吹き出しヒントです。`fandhe-frontend-headless-ui` の `tooltip` mod は
Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy
パーツを提供します。WAI-ARIA tooltip パターンに従い、trigger は
`aria-describedby` で content（`role="tooltip"`）と関連付けます。
`aria-expanded`/`aria-controls` は使用しません（Collapsible 等の
disclosure 系との違い）。headless-ui 自体はタイマー・キー・ポインタを
解釈しません。`openDelay`/`closeDelay`/`interactive` は
`fandhe-frontend-wasm-full` の `tooltip::wiring::TooltipDelayController`
（`data-open-delay`/`data-close-delay`/`data-interactive` 属性で設定）、
Escape 閉鎖は同 `overlay` が実装済みです。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Trigger | trigger はネイティブ `button` 要素のため、ブラウザ標準でフォーカス到達します（`disabled` 時は除外）。`fandhe-frontend-wasm-full` の `tooltip::wiring::TooltipDelayController::register_tooltip` が `focusin` で遅延なしに open を、`focusout` で close を要求します（ark-ui / Radix の「Tab で開閉」と同義）。 |
| Escape | — | `overlay::OverlayKind::Tooltip` は `close_on_escape` が既定 `true` です（`data-close-on-escape="false"` で無効化可能）。 |
| Enter / Space | Trigger | ネイティブ `button` の click が発火し、`headless::MAPPING_TABLE` が `"toggle"` を dispatch します。参照（ark `closeOnClick: true` / Radix の Space・Enter）は「閉じるのみ」であり、本実装の `toggle`（開閉反転）との差は `fandhe-frontend-wasm-full` 側の挙動差です（下記スコープ外候補）。 |

**参考サイトとの差分**

ark-ui・chakra-ui・Radix Primitives と突合した結果、**是正すべき具体的な
欠落は見つかりませんでした**（イシュー #1645）。anatomy（6 パート、Radix
の `Provider > Root > Trigger > Portal > Content > Arrow` を含む）・
`data-state`/`data-disabled` 語彙・`role="tooltip"`/`aria-describedby` は
いずれも一致しています。一方、以下は意図的に合わせていません。

- **Zag/chakra の `data-expanded`**: `data-state` と重複するため不採用です。
- **Zag/chakra の `data-placement`、Radix の `[data-side]`/`[data-align]`**:
  `positioner` の `data-side`/`data-align`（`positioning`、#590）が同役割を
  担うため置き換えていません。
- **Radix の `[data-state]` 語彙 `delayed-open`/`instant-open`**:
  本実装は `OpenState`（`"open"`/`"closed"`）語彙統一を優先しています。
- **`aria-describedby` の常時出力**: zag/Radix は open 時のみ出力しますが、
  本実装は `describedby` が `Some` のとき状態に関係なく常時出力します
  （SSR 静的出力の性質上。`hidden` な参照先も accessible description の
  算出に含まれるため害はなく、むしろ SSR/no-JS で説明が結び付く利点が
  あります）。
- **`Provider`/`Portal`**: 遅延設定共有・DOM 配置の関心のため headless
  anatomy には持ち込みません。

上記の差分に伴う `data-*`・パートの増減は無いため、Themes 側イシュー
#1548（closed）への追加コメントは行っていません。

スコープ外候補（是正対象ではなく、別途起票を検討する余地がある事項）:

- `fandhe-frontend-wasm-full` の tooltip trigger click →
  `"toggle"`（`headless::MAPPING_TABLE`）を参照準拠の「閉じるのみ」
  （ark `closeOnClick` / Radix Space・Enter）へ寄せるかの検討
- `crates/headless-ui/src/tooltip.rs` rustdoc（モジュール doc §スコープ外・
  `content` の doc）の「openDelay/closeDelay/interactive/closeOnEscape は
  スコープ外」記述の陳腐化是正（現在は `fandhe-frontend-wasm-full` 側に
  実装済みのため。`src/` 変更のため semver バンプまたは
  `version-bump-exempt` 宣言を伴う別 PR）
- `aria-describedby` を open 時のみ出力する参照挙動への追随要否
  （`fandhe-frontend-wasm-full` 側での動的付け外しを含む設計判断）

スタイル済みの表示例は [Tooltip](../themes/tooltip.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、
`[data-scope="tooltip"][data-part="..."]` セレクタでスタイルを当てます。
author スタイルの `display` が UA の `[hidden] { display: none }` を
上書きして closed 状態が見えてしまわないよう、`positioner`/`content`
には `[hidden]` ガードを必ず含めます。

```css
[data-scope="tooltip"][data-part="root"] {
  position: relative;
  display: inline-block;
}

[data-scope="tooltip"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
}

[data-scope="tooltip"][data-part="positioner"][hidden],
[data-scope="tooltip"][data-part="content"][hidden] {
  display: none;
}

[data-scope="tooltip"][data-part="content"] {
  background: #1a1a1a;
  color: #fff;
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
}

[data-scope="tooltip"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tooltip"][data-part="arrow"] {
  position: absolute;
  width: 0.5rem;
  height: 0.5rem;
}
```
