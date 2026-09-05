# Toast

一時的な通知の queue 表示です。`fandhe-frontend-headless-ui` の `toast`
mod は group（live region）/ root（通知 1 件）/ title / description /
action-trigger / close-trigger の 6 anatomy パーツと、複数通知を有界な
キューとして管理する状態機械 Toaster を提供します。`aria-live` は
`ToastStatus` から決定的に導出され（Error のみ `assertive`、他は
`polite`）、group は `role="region"` + 必須の `aria-label` + `tabindex="-1"`
を、root は `role="status"` + `data-state="open"`（固定値）+ `tabindex="0"`
を固定付与します。タイマーによる自動 dismiss の実配線は本部品のスコープ外
です。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | root → action-trigger → close-trigger | root の `tabindex="0"` とネイティブ `button` により、ブラウザ標準でこの順にフォーカス到達します。Radix の F8 / Zag の Alt+T によるリージョンフォーカスは未配線です。 |
| Enter / Space | action-trigger / close-trigger | ネイティブ button の click が発火します。click から `"dismiss"` dispatch への配線（`fandhe-frontend-wasm-full` の `MAPPING_TABLE`）は未実装で、Escape での閉鎖も同様に未配線です。 |

**参考サイトとの差分**

Zag.js（`toast.connect.ts`/`toast-group.connect.ts`、ark-ui の実体）・
Radix Primitives Toast・chakra-ui v3 Toast と突合しました（イシュー
#1643）。anatomy（6 パート）はいずれの参照軸とも増減なしで一致します。
是正した差分は次の 4 点です。

- **root に `data-state="open"` を追加**: Zag・Radix 双方が持ちます。固定値
  `"open"` のみを発行し、`"closed"`（exit 遷移用の中間状態）は発行しません
  （`Toaster` に閉鎖・退出中間状態が存在せず、描画された root は構造上つねに
  open のため）。
- **root に `tabindex="0"` を追加**: Zag・Radix 双方が持ちます。
- **group に `tabindex="-1"` を追加**: Zag group・Radix viewport 双方が
  持ちます。
- **group ラベルの既定値**: `Toaster::view` が `aria-label=""` を出していた
  不具合を `DEFAULT_GROUP_LABEL`（`"Notifications"`）で是正しました。

`action-trigger` に `data-disabled` を発行するかは Themes 側イシュー
#1544/#1545 が保留していた判断で、本イシューで**発行しないことを確定**
しました（Zag の `getActionTriggerProps` は `type="button"` のみを発行し、
ネイティブ `disabled` 属性を呼び出し側 attrs 経由で渡す運用を維持します）。

意図的に合わせていない点:

- Zag のスタック・遷移・一時停止系 `data-*`（`data-mounted`/`data-paused`/
  `data-first`/`data-sibling`/`data-stack`/`data-overlap`）と Radix の
  スワイプ系（`data-swipe`/`data-swipe-direction`）は装飾・アニメーション・
  ジェスチャ計測の関心であり headless 層へ持ち込みません。
- `data-placement`/`data-side`/`data-align` は group のみに付与し、Zag の
  ように root へは重複付与しません。
- `aria-live` は root に置く方針を維持します（Zag は group に置きますが、
  本クレートは Radix 準拠の既存方針を維持します）。
- root の `aria-labelledby`/`aria-describedby`、title/description の自動
  `id` は付与しません（`role="status"` + `aria-atomic="true"` で通知全体が
  読み上げられるため必須ではなく、Radix も付けません）。
- close-trigger の既定 `aria-label`（Zag の `"Dismiss notification"`）は
  付与しません（headless-ui の `Anatomy::part` は呼び出し側 attrs を後置
  追記するため、固定既定値は利用者指定と二重出力になります。アイコンのみの
  close-trigger には利用者が `attrs` で `aria-label` を渡してください）。
- `ToastStatus` への `loading` variant 追加（Zag/chakra）は公開 enum の
  破壊的変更のため見送りました。
- chakra の `Indicator`（アイコン）パートは装飾であり headless には置きません
  （Themes 候補として記録）。
- 要素型は Radix の `ol`/`li` ではなく ark-ui と同じ `div` を維持します。

上記の `data-state`/`tabindex` の追加は Themes（`fandhe-frontend-pre-styled-ui`）
側イシュー #1543/#1545 へコメントで共有しました。Themes 側の CSS 変更は
不要ですが、exit 遷移（`data-state="closed"`）は `fandhe-frontend-wasm-full`
の dismiss 配線待ちのまま保留です。

**スコープ外候補**（是正対象ではなく、別途起票を検討する余地がある事項）:

- `fandhe-frontend-wasm-full` 側の toast 配線（close/action-trigger →
  `"dismiss"` dispatch、Escape での閉鎖、ホットキー F8/Alt+T によるリージョン
  フォーカス、タイマーによる自動 dismiss と hover/focus による一時停止、
  exit 遷移用 `data-state="closed"` の追加）
- `ToastStatus::Loading` variant
- Themes 側 `Indicator` パート

スタイル済みの表示例は [Toast](../themes/toast.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

`[data-scope="toast"][data-part="..."]` セレクタでスタイルを当てます。
以下は root の枠・影、Error 状態の配色、open 状態のフェードイン、group の
固定配置、close-trigger のフォーカスリングの最小例です。

```css
[data-scope="toast"][data-part="root"] {
  border: 1px solid #888;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

[data-scope="toast"][data-part="root"][data-type="error"] {
  border-color: #dc2626;
  background: #fef2f2;
}

[data-scope="toast"][data-part="root"][data-state="open"] {
  animation: fd-toast-fade-in 150ms ease-out;
}

@keyframes fd-toast-fade-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

[data-scope="toast"][data-part="group"][data-placement="bottom-end"] {
  position: fixed;
  bottom: 16px;
  right: 16px;
}

[data-scope="toast"][data-part="close-trigger"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
