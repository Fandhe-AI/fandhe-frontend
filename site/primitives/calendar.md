# Calendar

月表示と単一日付の選択を、WAI-ARIA の grid パターンに従った 11 個の anatomy パーツ（Root/Heading/PrevTrigger/NextTrigger/Table/…/DayTrigger）として提供する unstyled 部品です。「今日」は呼び出し側が明示的に渡す引数であり、内部で時刻取得 API を使わない決定的な設計です。矢印キー等でのキーボード操作は `fandhe-frontend-wasm-full` が配線します。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Calendar](../themes/calendar.md) を参照してください。

日付の実在性検証以外の入力値検証・ロケール依存の日付整形・フォーム送信処理は本部品の責務外であり、利用側が担います。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
