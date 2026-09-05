# Dialog

モーダルダイアログです。`fandhe-frontend-headless-ui` の `dialog` mod は
Root / Trigger / Backdrop / Positioner / Content / Title / Description /
CloseTrigger の 8 anatomy パーツと、`DialogRole`（Dialog/Alertdialog）で
切り替えられる `role`・`aria-modal`・`aria-haspopup="dialog"` を提供します。
本部品は SSR での属性出力と状態機械のみを担い、Escape キーでの閉鎖・外側
クリックでの閉鎖・フォーカストラップ・閉鎖時の trigger へのフォーカス
復帰・click → dispatch 配線は `fandhe-frontend-wasm-full` が実 DOM 配線を
担います。`content` は `tabindex="-1"` を固定付与します（プログラム的
フォーカスのみを許可する WAI-ARIA dialog パターンの前提）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter / Space | Trigger | ダイアログを開きます（`fandhe-frontend-wasm-full` の headless part → action 対応表が `"toggle"` を dispatch）。 |
| Enter / Space | CloseTrigger | ダイアログを閉じます（同対応表が `"close"` を dispatch）。 |
| Escape | Content | ダイアログを閉じます。`role="alertdialog"` でも Escape は閉じます（外側クリックのみ alertdialog は既定で無効）。`data-close-on-escape="false"` で無効化できます。 |
| Tab / Shift+Tab | Content | `aria-modal="true"` のとき content 内でフォーカスを循環させます（フォーカストラップ）。`data-autofocus` で初期フォーカス先を指定でき、tabbable な子が無い場合は content 自身（`tabindex="-1"`）へフォーカスします。 |
| （閉鎖時） | — | フォーカスはフォーカストラップ開始時点でフォーカスされていた要素（取得不能なら trigger）へ復帰します。 |

**参考サイトとの差分**

ark-ui（zag `dialog.connect.ts`）・Radix Primitives・chakra-ui と突合し、
`content` の `tabindex="-1"` 固定付与を追加しました。anatomy（8 パート）・
`data-state` 語彙（`open`/`closed`）は一致しており、パート・`data-*` の
増減はありません。一方、以下は意図的に合わせていません。

- **DOM 上の `root` パート**: zag の `Dialog.Root` は context のみで DOM を持ちませんが、本リポジトリの全部品が root を DOM 要素として持つ規約（`data-state` の付与先）のため維持しています。
- **`positioner` の `data-state` + `hidden`**: zag は `pointer-events` のインラインスタイルで代替しますが、headless-ui はスタイルを出力しないため JS なしの SSR での閉状態表現として維持しています。
- **trigger の `data-ownedby` / `data-value` / `data-current`**（zag の複数トリガー識別）: `aria-controls` による id 関連付けが同等の役割を担うため不採用です。
- **content の `data-nested` / `data-has-nested`**（ark-ui のネストダイアログ実行時計測）: レイアウト計測・実行時関心は headless へ持ち込まない方針（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）のため不採用です。
- **Radix `Portal`**: DOM 配置の関心のため不採用です。
- **Radix AlertDialog の `Cancel` / `Action` パート**: `DialogRole::Alertdialog` + `CloseTrigger` + 素の `button` で構成でき、ark-ui にも該当パートは無いため不採用です。

スタイル済みの表示例は [Dialog](../themes/dialog.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="dialog"][data-part="..."]`
セレクタでスタイルを当てます。以下は backdrop・positioner（中央寄せ）・
content（枠・余白）の最小例です。author スタイルの `display` が UA の
`[hidden] { display: none }` を上書きして closed 状態が見えてしまわないよう、
`[hidden]` ガードを必ず含めます。

```css
[data-scope="dialog"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 0.4);
}

[data-scope="dialog"][data-part="backdrop"][hidden] {
  display: none;
}

[data-scope="dialog"][data-part="positioner"] {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

[data-scope="dialog"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="dialog"][data-part="content"] {
  background: #fff;
  border-radius: 8px;
  padding: 1.5rem;
  max-width: 28rem;
}

[data-scope="dialog"][data-part="content"][hidden] {
  display: none;
}
```
