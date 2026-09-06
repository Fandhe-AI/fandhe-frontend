# Tour

オンボーディング向けステップガイドです。`fandhe-frontend-headless-ui` の `tour` mod は Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description / ProgressText / Control / CloseTrigger / ActionTrigger の 13 anatomy パーツと、Idle/Active/Skipped/Completed の決定的な状態機械を提供します。Themes 版と異なり対象要素の実座標追従・スポットライトの視覚表現を持たず、`role="dialog"` 等の構造・ARIA 属性と静的な配置情報の出力のみを担います。

スタイル済みの表示例は [Tour](../themes/tour.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Escape | Content | zag.js は `closeOnEscape` 既定で dismiss（本状態機械では `"skip"` dispatch 相当）へ写像します。状態機械は提供済みですが、**DOM keydown 配線は `fandhe-frontend-wasm-full` に存在せず未実装です**（イシュー #1666 時点で確認済み、別 Issue の起票対象）。 |
| ArrowRight / ArrowLeft | Content | zag.js は `keyboardNavigation` 既定で `"next"`/`"prev"` dispatch へ写像します。DOM keydown 配線は同様に未実装です。 |
| Space / Enter | ActionTrigger / CloseTrigger | ネイティブ `<button type="button">` のため、ブラウザ既定動作としてすでに機能します。 |
| Tab / Shift+Tab | Content / 対象要素 | zag.js は `trapFocus([content, target])` で Content と対象要素の間を巡回させます。フォーカストラップは配線されておらず未対応です（`aria-modal="true"` を付与しない理由も同じ、下記「参考サイトとの差分」参照）。 |

**参考サイトとの差分**

イシュー #1666 で ark-ui docs・実装（`tour.anatomy.ts`・`tour-actions.tsx`・`tour-control.tsx`）・zag.js（`tour.anatomy.ts`・`tour.connect.ts`・`tour.machine.ts`）と突合しました（Radix Primitives に Tour 相当は存在しません）。

- **`control` パーツの欠落を是正**: ark-ui 実装の実 DOM 値 `data-part="control"`（docs の Anatomy 図が示す「Actions」ラベルに対応する DOM パーツ。`Tour.Actions` 自体は DOM を描画しない render-prop コンポーネント）を追加しました。`ActionTrigger`/`CloseTrigger` を並べるコンテナとして使う想定です。
- **`content` の `tabindex="-1"`/`data-step` 欠落を是正**: `crate::dialog`/`crate::popover` の content と同型のフォーカス移動の受け皿として `tabindex="-1"` を固定付与し、`Active` 時のみ現在ステップの `id` を `data-step` として出力するようにしました（zag `content.connect.ts` 準拠）。
- **`action_trigger` の `data-type`/`disabled` 欠落を是正**: `TourTriggerKind`（Next/Prev/Skip/Complete/Custom）引数を追加し `data-type` を出力するようにしました（破壊的シグネチャ変更）。`Prev` は dispatch が no-op になる境界（先頭ステップ・非 Active）でのみ `disabled`/`data-disabled` を付与します。
- **予約キーなりすまし除去を追加**: 呼び出し側 `attrs` が `role`/`aria-*`/`tabindex`/`data-*`/`disabled`/`id` を偽装・重複出力できないよう `drop_reserved` を導入しました（`toast`/`splitter` 等と同型のパターン）。`aria-label` は素通しします。

一方で以下は意図的に参考サイトと合わせていません。

- **`role="alertdialog"`**: WAI-ARIA `alertdialog` は即時応答を要する警告向けであり、オンボーディング案内は該当しません（支援技術の割り込み読み上げを避けるため）。`role="dialog"` を維持します。
- **`aria-modal="true"`**: `fandhe-frontend-wasm-full` にツアー用フォーカストラップの配線がまだ無く、SSR でトラップされていない状態を偽って主張しません。トラップ配線が入った時点で再評価します。
- **content 自体の `aria-live`**: `progress_text` のみに `aria-live="polite"` を付与する既存方針を維持します（content と progress-text の二重ライブリージョンによる重複読み上げを避けるため）。
- **ステップ種別 `data-type`（tooltip/dialog/floating/wait）**: `type`/`effect`/`actions` の宣言的定義は初版スコープ外です（`action-trigger` の `data-type` は「アクション種別」の意味で採用し、ステップ種別の意味では出しません）。
- **`data-placement`/`data-side` on content/title/description**: `positioning::placement_attrs` は positioner のみへ出力する既存設計を維持します。
- **`data-nested`/`data-has-nested`**: ネスト popover 機構は未採用です。
- **`dismissed` status（`skip` との区別）**: `data-status` 語彙・hydration 語彙の変更は Themes 側 CSS へ波及するため変更しません。Escape・close-trigger はいずれも `"skip"` dispatch へ写像します。
- **`arrow` の `hidden`**: tooltip 配置計測結果に依存する DOM ローカル状態のため非採用です。
- **`close-trigger` の既定 `aria-label`**: 呼び出し側が `attrs` 経由で明示的に渡す既存規約を維持します。
- **`next` の最終 step での `disabled`**: 本状態機械では最終 step の `"next"` が `Completed` へ遷移する有効な操作であるため disabled にしません（zag の `!hasNextStep` 判定とは意図的に非同値）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="tour"][data-part="..."]` セレクタと `data-state`/`data-status`/`data-disabled`/`[hidden]` で見た目を組み立てます。

```css
[data-scope="tour"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
}

[data-scope="tour"][data-part="spotlight"] {
  position: fixed;
}

[data-scope="tour"][data-part="positioner"] {
  position: fixed;
}

[data-scope="tour"][data-part="content"] {
  max-width: 320px;
}

[data-scope="tour"][data-part="control"] {
  display: flex;
  gap: 0.5rem;
}

[data-scope="tour"][data-part="action-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tour"] [hidden] {
  display: none;
}

[data-scope="tour"] :focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
