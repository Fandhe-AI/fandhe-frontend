# Pin Input

`fandhe-frontend-headless-ui` の `pin_input` mod が提供する PIN/OTP 桁入力の unstyled 部品です。Root / Label / Control / Input（桁ごと）/ HiddenInput の 5 anatomy パーツと、桁ごとの値・フォーカス位置・complete 判定を担う独自の値状態機械を持ちます。`data-complete` は全桁充足時のみの存在属性として一元管理されます。

`/themes/` 側の `PinInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`PinInputProps`（`disabled`/`readonly`/`invalid`/`required`）は次のように各パーツへ反映されます。`root`/`label`/`input` は `data-disabled`/`data-invalid`/`data-readonly` を、`label` はこれら 3 つに加えて `data-required` を付与します。`input` にはさらに `data-index`（0-origin の桁インデックス）と `data-filled`（値が非空のときのみ）が付きます。`invalid: true` のとき `input` へ `aria-invalid="true"` を（valid のときは属性自体を省略）、`readonly: true` のとき `input` へネイティブ `readonly` 存在属性を付与します（`type="hidden"` の `hidden_input` には `readonly`/`required` いずれも効果がないため付与しません）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| 0-9 / a-z / A-Z（`kind` 適合文字） | Input | 現在のフォーカス桁（未設定なら先頭の空き桁）へ入力し、次桁が存在すればそこへ前進します（dispatch: `"input"`）。種別不適合文字は no-op です。 |
| ArrowLeft | Input | 前の桁へフォーカスを移します（dispatch: `"prev"`、先頭桁なら留まります）。 |
| ArrowRight | Input | 次の桁へフォーカスを移します（dispatch: `"next"`、最終桁なら留まります）。 |
| Backspace | Input | 現在桁を消去し、前の桁へフォーカスを移します（dispatch: `"backspace"`、先頭桁なら消去のみで留まります）。 |
| Delete | Input | 現在桁のみを消去します（dispatch: `"delete"`、フォーカスは移動しません）。 |
| Control+V / Cmd+V | Input | クリップボードの文字列を先頭桁から一括入力します（dispatch: `"paste"`）。全文字が `kind` に適合する場合のみ受理し、1 文字でも不適合なら部分適用せず一切拒否します。 |

ポインタ・キーボード操作の実際の DOM 配線（`keydown`/`paste` イベントハンドラ、`fandhe-frontend-wasm-full`）は本部品のスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

**参考サイトとの差分**

ark-ui 公式 Data Attributes / Keyboard Support 表・Radix `one-time-password-field` と突合し、上記のキーボード操作・`data-*`/`aria-*` 語彙へ揃えました。一方、以下は意図的に合わせていません。

- Radix の `data-orientation`（root、既定 `vertical`）はレイアウト関心のため追加しません（headless-ui の責務境界、`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- Radix の `role="group"`（root）は ark-ui を主参照としているため付与しません。必要なら呼び出し側が `attrs` 経由で追加できます。
- zag `connect` のみが持つ Home/End・同一キー再入力での前進（`ADVANCE`）・`Enter` による自動送信（`autoSubmit`）・`blurOnComplete`/`selectOnFocus`/`sanitizeValue`/`pattern` は、クライアント DOM 配線・アプリロジックの関心のため対象外です。
- `inputmode`（`otp || numeric` のとき常に `numeric` を強制する zag の挙動）・`autocomplete="off"` の明示・`enterkeyhint`/`autocapitalize` は native ヒントの最小主義判断を維持し、追随していません。

スタイル済みの表示例は [Pin Input](../themes/pin-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="pin-input"][data-part="..."]` セレクタでスタイルを当てます。以下は input の枠線・寸法、値入力済み時の強調、invalid 時の枠線、全桁充足時の枠色、フォーカスリングの最小例です。

```css
[data-scope="pin-input"][data-part="input"] {
  width: 2.5rem;
  height: 2.5rem;
  text-align: center;
  border: 1px solid #ccc;
}

[data-scope="pin-input"][data-part="input"][data-filled] {
  border-color: #0a7;
}

[data-scope="pin-input"][data-part="input"][data-invalid] {
  border-color: #d33;
}

[data-scope="pin-input"][data-part="root"][data-complete] [data-part="input"] {
  border-color: #06c;
}

[data-scope="pin-input"][data-part="input"]:focus-visible {
  outline: 2px solid #06c;
  outline-offset: 2px;
}
```
