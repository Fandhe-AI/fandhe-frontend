# Timer

カウントダウン/カウントアップを表示する unstyled 部品です。時計 API を一切使わず、時間の前進は tick（デルタミリ秒）の明示的な注入のみで進む決定的な状態機械です。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Timer](../themes/timer.md) を参照してください。

実際の計時駆動（`setInterval` 相当）はクライアントランタイム側の責務であり、本部品は経過値の表示 anatomy と現在フェーズの表示状態のみを提供します。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | ActionTrigger | ネイティブ `<button type="button">` へのフォーカス移動です。`hidden` な ActionTrigger（下記「参考サイトとの差分」参照）はフォーカス対象から外れます。 |
| Enter / Space | ActionTrigger | ネイティブ `<button>` の起動です。参考サイト（ark-ui / Zag.js）に専用の Keyboard Interactions 表はなく、本実装もブラウザ標準動作に委ねます。 |

**参考サイトとの差分**

一次ソース（zag.js `packages/machines/timer/src/timer.connect.ts`。chakra-ui v3 / Radix Primitives / Radix Themes に Timer は存在しない）と突合し、以下を是正しました。

- **`area` の ARIA**: `role="timer"` と `aria-atomic="true"` を無条件付与するよう是正しました（旧版は付与していませんでした）。`aria-label`（既定書式 `"{days} days {hh}:{mm}:{ss}"`）は `Timer::area` 利便メソッド経由で注入されます（`area` 自由関数自体は `aria-label` を持ちません）。
- **`separator` の ARIA**: `aria-hidden="true"` を無条件付与するよう是正しました（装飾用の区切り文字を支援技術に読み上げさせないため）。
- **`action_trigger` の `hidden` 導出**: 第 2 引数に現在の `TimerPhase` を受け取り、zag.js と同じ真偽式（`running`/`paused` の 2 述語）で `hidden` 属性の要否を導出するよう破壊的変更しました。

一方、以下は意図的に参考サイトへ合わせていません。

- **`TimerControl::Restart`（5 値目）**: zag.js は `start`/`pause`/`resume`/`reset` の 4 値ですが、ark-ui docs のデモ構成に合わせ「常に可視」な `restart`（任意フェーズ → running、経過ゼロ）を追加しています。
- **`Completed` 状態**: zag.js は完了到達時に `idle` へ戻りますが、本実装は `data-state="completed"` で完了を表現し続ける拡張を維持します。可視性は `running`/`paused` のいずれでもないため `idle` と同じです（Start/Restart のみ表示）。
- **`Start` の任意フェーズ受理**: zag.js の `START` は idle 限定ですが、本実装は任意フェーズから running へ遷移できます。`hidden` により running/paused では UI 上到達不能なため実害はありません。
- **`item` の `style="--value: N"`**: zag.js は CSS カウンタ用の `style` 属性を付与しますが、装飾・レイアウト計測の関心を headless 層へ持ち込まない方針（`docs/policy/intentional-non-adoption.md` §3.25）により非採用です。必要な場合は呼び出し側 CSS か Themes 層で対応してください。
- **`root` の `data-*`**: `data-state`/`data-countdown`/`data-start-ms`/`data-target-ms`/`data-interval`/`data-elapsed` は zag.js の `root` プロパティ（`id` のみ）に対する本実装の superset です。クライアント配線層（wasm-full）と Themes 側 CSS がこれらへ依存します。
- **`id` / `dir` / `translations` / `asChild`**: 非対応です（`translations.areaLabel` 相当の差し替えは呼び出し側 `attrs` の `aria-label` で代替できます）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`action-trigger` に `[hidden] { display: none; }` を必ず併記してください（`display` を宣言する規則が先に一致すると、ブラウザ既定の `[hidden] { display: none }` を上書きしてしまいます）。

```css
[data-scope="timer"][data-part="area"] {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

[data-scope="timer"][data-part="item-value"] {
  font-variant-numeric: tabular-nums;
  font-size: 1.5rem;
}

[data-scope="timer"][data-part="root"][data-state="completed"] [data-part="item-value"] {
  color: #2563eb;
}

[data-scope="timer"][data-part="action-trigger"] {
  display: inline-flex;
  padding: 0.5rem 0.875rem;
  border: 1px solid #888;
  border-radius: 6px;
}

/* [hidden] を上書きしないよう、display 宣言を持つ規則には必ず併記する。 */
[data-scope="timer"][data-part="action-trigger"][hidden] {
  display: none;
}
```
