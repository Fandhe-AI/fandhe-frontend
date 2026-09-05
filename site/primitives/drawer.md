# Drawer

画面端からスライドインするパネルです。ark-ui・chakra-ui の Drawer は
WAI-ARIA 上 Dialog パターンの変種であり、`fandhe-frontend-headless-ui` の
`drawer` mod は新規状態機械を作らず `dialog` mod の開閉状態機械をそのまま
再利用します。追加する要素は専用 anatomy（`data-scope="drawer"`）と、
画面のどの端から出現するかを表す `DrawerPlacement`（Start/End/Top/Bottom、
既定 End）のみです。`content` は `tabindex="-1"` を固定付与します
（プログラム的フォーカスのみを許可する WAI-ARIA dialog パターンの前提）。

**キーボード操作**

`fandhe-frontend-wasm-full` は drawer scope を一切配線していません
（trigger/close-trigger の click 配線・Escape/外側クリックでの閉鎖・
フォーカストラップのいずれも未対応）。このためハイドレーション後も
キーボード操作・クリック操作に応答せず、キーボード操作の表は現時点では
提供しません（fail-closed のため未対応でも安全側です。別イシューで
追跡します）。

**参考サイトとの差分**

ark-ui（zag `drawer.connect.ts`）・chakra-ui と突合し、`content` の
`tabindex="-1"` 固定付与を追加しました。anatomy（8 パート）・`data-*`
語彙の増減はありません。一方、以下は意図的に合わせていません。

- **grabber / grabber-indicator / swipe-area / indent / indent-background パーツ**（zag のドラッグ・スタック積層 UI）: ドラッグ操作・実行時計測に紐づく装飾関心のため不採用です（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
- **`data-swipe-direction` / `data-swiping` / `data-dragging` / `data-expanded` / `data-nested-drawer-*`**（zag のドラッグ・ネスト計測状態）: 同様にドラッグ操作の実行時計測関心のため不採用です。`data-placement`（論理方向、RTL 対応）はこれらの物理方向語彙へ置き換えていません。
- **trigger の `data-ownedby` / `data-value` / `data-current`**（zag の複数トリガー識別）: `aria-controls` による id 関連付けが同等の役割を担うため不採用です（`dialog` と同判断）。
- **DOM 上の `root` パート**: 全部品共通の規約（`data-state` の付与先）のため維持しています。
- **`backdrop` の `aria-hidden="true"`**: zag は付けませんが、装飾層として読み上げ対象外にする既存方針のため維持しています。
- **`content` の `role="dialog"` 固定**（zag は `alertdialog` も選択可能）: Drawer は確認・警告用途ではなく常設ナビ・フィルタ等の補助パネル用途に限定するため維持しています。
- **chakra-ui の Header / Body / Footer / ActionTrigger**: `fandhe-frontend-pre-styled-ui`（Themes 層）の関心のため headless anatomy には持ち込みません。

スタイル済みの表示例は [Drawer](../themes/drawer.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="drawer"][data-part="..."]`
セレクタでスタイルを当てます。author スタイルの `display` が UA の
`[hidden] { display: none }` を上書きして closed 状態が見えてしまわない
よう、`backdrop`/`positioner`/`content` には `[hidden]` ガードを必ず含めます。
`[data-placement="..."]` で 4 方向（`start`/`end`/`top`/`bottom`）ごとの
配置・サイズを切り替えます。`start`/`end` は書字方向依存のため、物理方向
（`left`/`right`）ではなく論理プロパティ（`inset-inline-start` 等）で
書きます。

```css
[data-scope="drawer"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 0.4);
}

[data-scope="drawer"][data-part="backdrop"][hidden],
[data-scope="drawer"][data-part="positioner"][hidden],
[data-scope="drawer"][data-part="content"][hidden] {
  display: none;
}

[data-scope="drawer"][data-part="positioner"] {
  position: fixed;
  inset: 0;
}

[data-scope="drawer"][data-part="content"] {
  position: fixed;
  background: #fff;
  box-shadow: 0 0 1rem rgb(0 0 0 / 0.2);
}

[data-scope="drawer"][data-part="content"][data-placement="start"] {
  inset-block: 0;
  inset-inline-start: 0;
  inline-size: 20rem;
  block-size: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="end"] {
  inset-block: 0;
  inset-inline-end: 0;
  inline-size: 20rem;
  block-size: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="top"] {
  inset-inline: 0;
  inset-block-start: 0;
  block-size: 20rem;
  inline-size: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="bottom"] {
  inset-inline: 0;
  inset-block-end: 0;
  block-size: 20rem;
  inline-size: 100%;
}
```
