# Floating Panel

ドラッグ移動・リサイズ可能な浮遊パネルです。`fandhe-frontend-headless-ui`
の `floating_panel` mod は Root / Trigger / Positioner / Content / Header /
Title / Control / StageTrigger / CloseTrigger / Body の 10 anatomy パーツ
と、開閉・`Stage`（Default/Minimized/Maximized）・座標を持つ状態機械を
提供します。`content` は `role="dialog"` を固定付与しますが、非モーダル
overlay のため `aria-modal` は出力しません（ユーザーは他の要素を操作し
続けられます）。`header`/`control` は現在の `Stage` を `data-stage` へ
反映し、`body` は `Stage::Minimized` のとき `hidden` 存在属性を付与します。

**キーボード操作（現時点の配線状況）**

`fandhe-frontend-wasm-full` の headless 部品 → dispatch 対応表
（`headless.rs`）に `floating-panel` scope の行が無く、trigger /
close-trigger / stage-trigger の click・Escape キーでの閉鎖・矢印キーでの
移動はいずれも実際には配線されていません（本文書時点の事実）。headless
層は型付きアクション（`"open"`/`"close"`/`"toggle"`/`"minimize"`/
`"maximize"`/`"restore"`/`"set_position"`）を提供済みであり、配線自体は
`fandhe-frontend-wasm-full` 側の将来スコープです。

**参考サイトとの差分**

ark-ui（zag `floating-panel.connect.ts`/`floating-panel.anatomy.ts`。
chakra-ui は ark-ui のラッパのため実質同一）と突合し、`header`/`control`
への `data-stage` 付与、`body` への `Stage::Minimized` 時 `hidden` 付与を
追加しました。一方、以下は意図的に合わせていません。

- **`dragTrigger`/`resizeTrigger`（`data-axis` 付き）anatomy パーツ**: ポインタイベント配線が `fandhe-frontend-wasm-full` に無い状態で anatomy だけ追加すると、利用者へ「操作できる」という誤った安心を与えるため不採用です（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- **`trigger`/`content` の `data-dragging`、`content` の `data-topmost`/`data-behind`、CSS 変数 `--width`/`--height`/`--z-index`**: いずれもドラッグ・重なり順という実行時計測の関心のため headless 層へ持ち込みません。
- **`content` の `tabIndex: 0`**: zag は矢印キー移動の配線とセットでフォーカス可能にしていますが、本実装は矢印移動が未配線のため、`tabindex="0"` だけを先に付けると機能しないタブストップになるため不採用です。
- **`stage-trigger` の現在 stage に応じた `hidden`**: 現在 stage を引数に要する破壊的変更がさらに増えるため見送りました。代わりに `control` が `data-stage` を持つため、`control[data-stage="minimized"] [data-part="stage-trigger"][data-stage="minimized"]` のような子孫セレクタで同等の表示切替を実装できます。
- **`header`/`control`/`body` の真偽 3 属性（`data-minimized`/`data-maximized`/`data-staged`）**: 本実装は最初から `data-stage` 列挙 1 属性で同じ情報を表しており、二重化しません。

スタイル済みの表示例は [Floating Panel](../themes/floating-panel.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="floating-panel"][data-part="..."]`
セレクタでスタイルを当てます。以下は positioner（座標反映）・content
（枠・余白）・body（minimized 時の折り畳み）・control 経由の
stage-trigger 表示切替の最小例です。author スタイルの `display` が UA の
`[hidden] { display: none }` を上書きして隠れるべき状態が見えてしまわない
よう、`[hidden]` ガードを必ず含めます。

```css
[data-scope="floating-panel"][data-part="positioner"] {
  position: fixed;
  top: var(--fandhe-y);
  left: var(--fandhe-x);
}

[data-scope="floating-panel"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="floating-panel"][data-part="content"] {
  border: 1px solid #d0d0d0;
  background: #fff;
  border-radius: 8px;
}

[data-scope="floating-panel"][data-part="body"] {
  padding: 1rem;
}

[data-scope="floating-panel"][data-part="body"][hidden] {
  display: none;
}

/* control の data-stage に応じて、現在の stage と同じ遷移先を示す
   stage-trigger を隠す（例: minimized 中は「最小化」ボタン自体を隠す）。*/
[data-scope="floating-panel"][data-part="control"][data-stage="minimized"]
  [data-part="stage-trigger"][data-stage="minimized"] {
  display: none;
}
```
