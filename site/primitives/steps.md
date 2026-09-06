# Steps

段階ナビゲーション（ウィザード）です。`fandhe-frontend-headless-ui` の `steps` mod は Root / List / Item / Trigger / Indicator / Separator / Content / CompletedContent / PrevTrigger / NextTrigger / Progress の 11 anatomy パーツと、complete/current/incomplete の 3 状態を導出する決定的な状態機械を提供します。Themes 版と異なりインジケーターの見た目を持たず、構造・`aria-current="step"` 等の属性・境界（先頭/末尾）判定のみを担います。

スタイル済みの表示例は [Steps](../themes/steps.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab / Shift+Tab | Trigger / PrevTrigger / NextTrigger | ネイティブ `button` 要素のフォーカス順序に従います。境界（`step==0`/`step==count`）では `disabled` によりフォーカス順序から除外されます（実装済み）。 |
| Enter / Space | Trigger / PrevTrigger / NextTrigger | ネイティブ `button` 要素の暗黙アクティベーション（click イベント発火）のみ機能します。**`fandhe-frontend-wasm-full` の `headless::MAPPING_TABLE` に `"steps"` scope は登録されておらず、click から dispatch（`"goto"`/`"prev"`/`"next"`）への実配線は未実装です**（イシュー #1665 時点で確認済み、別 Issue の起票対象）。呼び出し側が独自に配線するまで、Trigger をクリックしても step は遷移しません。 |

**参考サイトとの差分**

イシュー #1665 で ark-ui/Zag.js（`steps` machine）・chakra-ui の Steps と突合しました。

- **`data-disabled` の欠落を是正**: `prev_trigger`/`next_trigger` の境界（`step==0`/`step==count`）で native `disabled` に加えて `data-disabled` を出力するようにしました（本リポジトリの disabled 語彙統一）。
- **`data-orientation` の欠落を是正**: Zag.js が trigger/content に出力する `data-orientation` を追加しました（`completed-content` は `content` と対称にするための加算）。
- **`progress` パーツを新設**: Zag.js anatomy 10 パーツ中、本実装に唯一欠けていたパートです。`role="progressbar"` + `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-valuetext` を出力し、`step==count`（全 step 完了）のときのみ `data-complete` を付与します。
- **予約キーなりすまし除去を追加**: 呼び出し側 `attrs` が `role`/`aria-*`/`data-*`/`type`/`hidden` 等の固定付与属性を偽装・重複出力できないよう `drop_reserved` を全パーツへ導入しました（`toolbar`/`splitter` 等と同型のパターン）。

一方で以下は意図的に参照サイトと合わせていません。

- **trigger の `data-state="open"\|"closed"`**: 非採用（既存の `complete`/`current`/`incomplete` を維持）。変更すると `fandhe-frontend-pre-styled-ui` の golden CSS を壊す破壊的変更になります。
- **`aria-current="step"` の item への付与**: 非採用（trigger のみに付与）。フォーカス可能な要素への付与が支援技術に読まれやすく、両方に付けると重複読み上げになります。
- **tabs 意味論**（`list` の `role="tablist"`/`aria-owns`/`aria-orientation`、`trigger` の `role="tab"`/`aria-selected`/`aria-controls`、`content` の `role="tabpanel"`/`aria-labelledby`/`tabindex="0"`）: 非採用。id 相互参照の配管が必要でシグネチャ変更＝破壊的変更になります（後続イシュー候補）。
- **indicator の `aria-hidden="true"`**: 非採用。trigger の子が indicator（数字）のみの構成が多く、無条件付与は trigger のアクセシブルネームを消してしまいます。
- **root の `style="--percent"`（装飾用 CSS 変数）・`dir`（RTL）**: 非採用（本リポジトリ横断で未採用の判断軸）。
- **`data-skippable`・`linear` 時の roving tabIndex 制御・`isStepValid`/`isStepSkippable`**: 非採用（アプリケーションロジック依存、UI 部品の責務境界外）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="steps"][data-part="..."]` セレクタと `data-state`/`data-complete`/`data-current`/`data-incomplete`/`data-orientation`/`data-disabled` で見た目を組み立てます。

```css
[data-scope="steps"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

[data-scope="steps"][data-part="root"][data-orientation="vertical"] {
  flex-direction: row;
}

[data-scope="steps"][data-part="list"] {
  display: flex;
  list-style: none;
  gap: 0.5rem;
  padding: 0;
  margin: 0;
}

[data-scope="steps"][data-part="trigger"][data-current] {
  font-weight: 600;
}

[data-scope="steps"][data-part="indicator"][data-complete] {
  background: #16a34a;
  color: #fff;
}

[data-scope="steps"][data-part="content"][data-state="closed"] {
  display: none;
}

[data-scope="steps"][data-part="prev-trigger"][data-disabled],
[data-scope="steps"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="progress"] {
  height: 4px;
  background: #e5e7eb;
}

[data-scope="steps"] :focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
