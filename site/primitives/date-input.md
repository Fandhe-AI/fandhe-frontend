# Date Input

年・月・日を独立したセグメントとして編集する、WAI-ARIA の spinbutton パターンに従った unstyled 部品です。3 セグメントすべてが揃った場合のみ実在する日付か検証し、存在しない日付（例: 2 月 30 日）はセグメント値を保持したまま invalid 表示に倒します（fail-closed）。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Date Input](../themes/date-input.md) を参照してください。

実在性検証以外の入力値検証・ロケール依存の日付整形・フォーム送信処理は本部品の責務外であり、利用側が担います。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
