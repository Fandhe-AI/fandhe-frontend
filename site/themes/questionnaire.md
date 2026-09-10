# Questionnaire

`fandhe-frontend-pre-styled-ui` の `questionnaire` mod が提供するスタイル済み
Questionnaire 部品です。shadcn/ui Questionnaire 相当の多段質問 UI を
Root / Progress / Question / Prompt / Description / Options / Freeform /
Actions / Back / Next / Skip の 11 パーツ構成で表現します。

`headless-ui` の `Questionnaire` 状態機械（`count` = 全質問数、`step` =
現在位置）へ薄く委譲する層であり、`size` / `colorPalette` のような見た目
クラス軸は持ちません。全パーツ関数が `state: &Questionnaire` を第 1
引数に取り、内部で headless 層の同名メソッドへ委譲します。見た目は
headless が出力する `data-state`（`active` / `completed` / `upcoming`）・
`data-answered` / `data-skipped` / `data-required` / `data-invalid` /
`data-disabled` / `data-complete` を CSS セレクタとして参照するだけで
切り替わります。

質問（`question`）は非 active（`completed` / `upcoming`）のとき常に
`hidden` 属性を伴います。そのため `data-state="completed"` / `"upcoming"`
の枠色・破線表現が実際に可視化されるのは、アプリケーション側が独自 CSS
で `[hidden]` を打ち消して一覧表示する場合（回答レビュー画面等）に限られ、
既定表示は active な質問 1 件のみです。

`progress` は中身空の `role="progressbar"` な `div` です。スタイル済み
Progress 部品を入れ子にはせず、`step` / `count` から計算した百分率を
`--fandhe-questionnaire-percent` custom property として `style` へ設定し、
CSS の `linear-gradient` で塗り幅を表現します。ステップ遷移後にこの塗りを
更新するのはアプリケーション（クライアントランタイム）側の責務であり、
`fandhe-frontend-wasm-full` の back/next/skip 配線はこの custom property
自体を更新しません。

`options` スロットは純スロットで、呼び出し側がスタイル済み RadioGroup /
CheckboxGroup の item を入れ子にする契約です。本部品の CSS は `options`
配下の RadioGroup / CheckboxGroup item をカード状（枠線 + パディング）に
整形する子孫セレクタを含みますが、item 自体の基本規則は含まないため、
呼び出し側は RadioGroup / CheckboxGroup 自身の CSS も併せて読み込む必要が
あります。`freeform` スロットも同様の純スロットで、Field の label や
Textarea を入れ子にする用途を想定します。

回答値の保持・検証（必須判定）・分岐（次にどの質問へ進むか）・送信は
アプリケーションロジックであり、この部品では実装しません。`next` /
`skip` の無効化は、必須判定の結果（真偽）だけを呼び出し側が引数として
渡す契約です。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
