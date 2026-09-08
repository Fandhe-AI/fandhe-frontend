# Empty State

`fandhe-frontend-pre-styled-ui` の `empty_state` mod が提供するスタイル済み Empty State 部品です。

検索結果 0 件・初期状態など「表示すべきコンテンツがない」ことを伝える部品です。content/indicator/title/description/actions の 5 パーツで構成し、size variant で padding・gap・indicator/title/description の文字サイズが連動します。読み込み中の占位表示には [Skeleton](skeleton.md) を検討してください。

shadcn/ui の `Empty` と突合し、root の variant（Plain/Outline/Subtle、既定 Plain）と indicator の variant（`indicator_with`、Plain/Boxed、既定 Plain）を追加しました。Outline は破線枠、Subtle は淡色単色背景（shadcn の半透明グラデーションはトークン体系で表現できないため単色へ置換）、Boxed は media 部分を bg-muted の角丸タイルにします。いずれも独自語彙の追加バリアントです。**破壊的変更**: `variant` は `EmptyStateProps` の必須フィールドとして追加したため、`EmptyStateProps { size }` のようなフィールド全列挙の構造体リテラルは本変更以降コンパイルできなくなります。既存の呼び出しは `EmptyStateProps { size, ..Default::default() }` へ移行してください。不変なのは「既定 `EmptyStateProps::default()` を渡したときの CSS 出力・HTML 出力」と既存の CSS ブロックであり、各 variant 用の CSS ブロックは新規追加されています。shadcn の `EmptyHeader`（indicator/title/description をまとめる専用グループ）に相当する slot は追加していません。`content` の gap と indicator/actions の区画間余白で同等の視覚リズムを既に実現しているためです。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
