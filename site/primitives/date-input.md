# Date Input

年・月・日を独立したセグメントとして編集する、WAI-ARIA の spinbutton パターンに従った unstyled 部品です。3 セグメントすべてが揃った場合のみ実在する日付か検証し、存在しない日付（例: 2 月 30 日）はセグメント値を保持したまま invalid 表示に倒します（fail-closed）。

ark-ui（zag.js の `date-input` machine）の Data Attributes 表・キーボード操作と突合済みです（イシュー #1626）。`data-type`（year/month/day）・`data-value`（入力済み）・`data-editable`（常時）・`data-placeholder-shown`（未入力）を各セグメントへ、`data-focus` を control/segment-group へ、`data-readonly` を全パーツへ付与します。状態機械の dispatch 語彙として `increment`/`decrement`（境界で wrap-around）・`page-increment`/`page-decrement`（PageUp/PageDown 相当、境界で clamp）・`home`/`end`・`prev`/`next`（矢印キーでのセグメント間移動相当）・`backspace` を提供しますが、実 DOM キーイベントへの配線（`fandhe-frontend-wasm-full`）は本部品の責務外です。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Date Input](../themes/date-input.md) を参照してください。

headless-ui はスタイルレスです。`data-scope`/`data-part`/`data-*` をセレクタに使い、自前の CSS を当てることができます（例: `[data-scope="date-input"][data-part="segment"][data-type="year"]`）。

実在性検証以外の入力値検証・ロケール依存の日付整形・フォーム送信処理は本部品の責務外であり、利用側が担います。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
