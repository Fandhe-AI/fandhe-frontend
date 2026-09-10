# Blocks

Blocks は shadcn/ui の Blocks に相当する、既存の [Primitives](./primitives.md)
（`fandhe-frontend-headless-ui`）・[Themes](./themes.md)
（`fandhe-frontend-pre-styled-ui`）部品を組み合わせた実例集です。**新規の
UI コンポーネントを追加するものではありません**。ログインフォームや
ダッシュボードのような「よくある画面」を、既存部品だけでどう組み立てるかを
示す完成例として提供します。

各 block ページは静的な表示例であり、`<form>` 要素を持たず、実際の送信処理・
認証処理・データ取得を行いません（無 JS 制約により、開閉状態を持つ block は
初期状態を固定して掲示します）。実装に組み込む際は、送信処理・バリデーション・
データ取得を利用者自身の Rust コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25: UI コンポーネント層は
アプリケーションロジックを内包しません）。

## 掲載済み

- [login-01](./blocks/login-01.md)
- [dashboard-01](./blocks/dashboard-01.md)
- [sidebar-07](./blocks/sidebar-07.md)
- [sidebar-03](./blocks/sidebar-03.md)
- [signup-01](./blocks/signup-01.md)

## 掲載予定

以下は今後追加予定の block です（未掲載のためリンクはありません）。

- login-04
- signup-05
