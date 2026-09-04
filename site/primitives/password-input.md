# Password Input

`fandhe-frontend-headless-ui` の `password_input` mod が提供する表示切替トリガー付きパスワード入力の unstyled 部品です。Root / Label / Control / Input / VisibilityTrigger / Indicator の 6 anatomy パーツで構成され、表示切替（`visible`）状態機械のみを持ちます。パスワード値そのものは一切保持・出力しません（`input` は `value` 引数自体を持たない設計です）。

`/themes/` 側の `PasswordInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

`PasswordInputProps`（`disabled`/`readonly`/`invalid`/`required`）は次のように各パーツへ反映されます。`root`/`control`/`indicator` は `data-disabled`/`data-invalid`/`data-readonly` を、`label` はこれら 3 つに加えて `data-required` を、`input` はこれら 4 つ全てを付与します（`root`/`control`/`input` の `data-state`（`"visible"`/`"hidden"`）と `input` の `data-required` は ark-ui/zag には無い superset ですが、後方互換のため維持しています）。`readonly: true` のとき `input` にはネイティブ `readonly` 存在属性も付きますが、`visibility_trigger` にはネイティブ `disabled` を合成しません（表示切替は値を変更しない操作のため、読み取り専用でもキーボード操作での表示確認を封じない安全側の判断です）。`input` にはあわせて `autocapitalize="off"`・`spellcheck="false"` を固定付与し、表示中のパスワード文字列がブラウザのリモートスペルチェック・自動大文字化サービスへ送られる経路を減らします。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter / Space | VisibilityTrigger | ネイティブ `<button type="button">` のため、フォーカス時の Enter/Space によるクリック相当の発火はブラウザ標準操作として成立します。クリックから表示切替への dispatch（`"toggle"`）配線自体は `fandhe-frontend-wasm-full` 側の責務です。 |
| Tab | Input → VisibilityTrigger | この順にフォーカスが移動します。VisibilityTrigger には `tabindex` を付与しないため、tab 順序から除外されません（下記「参考サイトとの差分」参照）。 |

**参考サイトとの差分**

ark-ui/zag は VisibilityTrigger へ `aria-expanded`（表示中で `true`）+ `tabIndex: -1`（tab 順序から除外しポインタ操作専用にする）を採用しますが、本コンポーネントは意図的にこれへ追随していません。表示切替はトグルボタンパターン（WAI-ARIA APG）であり `input` は開閉する別領域ではないため `aria-pressed` を維持し、キーボード操作のみの利用者が表示切替できるよう `tabindex` を付与せず tab 順序に残しています（Radix Primitives の Password Toggle Field も「キーボード操作時はトリガーへフォーカス維持」としており、この判断と整合します）。

ポインタ・キーボード操作の実際の DOM 配線（`fandhe-frontend-wasm-full`）は本部品のスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

スタイル済みの表示例は [Password Input](../themes/password-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="password-input"][data-part="..."]` セレクタでスタイルを当てます。以下は control の枠線、表示中トリガーの強調、invalid 時の input 枠線、disabled/readonly 時の減光の最小例です。

```css
[data-scope="password-input"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  border: 1px solid #ccc;
}

[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: #0a7;
}

[data-scope="password-input"][data-part="input"][data-invalid] {
  border-color: #d33;
}

[data-scope="password-input"][data-part="root"][data-disabled],
[data-scope="password-input"] [data-readonly] {
  opacity: 0.6;
}
```
