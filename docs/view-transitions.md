# View Transitions 機構（TASK-8.1／TASK-8.2）

本ドキュメントは REQ-8（`docs/spec/04-requirements.md` REQ-8 節「View Transitions API の
ネイティブ活用」）のうち、クロスドキュメントナビゲーション側（TASK-8.1・#59）の
製品仕様を固定するものです。同一文書内（SPA 的）遷移側（TASK-8.2、`static/view-transitions.js`
の `withViewTransition()`）は既にマージ済みであり、本書では両者の役割分担も整理します。

## 1. 機構の全体像

REQ-8 が求める「JS 0 行での宣言的な遷移有効化」は、遷移の種類によって異なる
標準 API で実現します。

| 遷移の種類 | 有効化する機構 | 実装場所 | JS 行数 |
|---|---|---|---|
| クロスドキュメントナビゲーション（SSR/SSG のページ遷移） | `@view-transition { navigation: auto; }`（CSS at-rule） | `rws_app::page_shell()` / `templates/embed/embed.html` | 0 行 |
| 同一文書内（SPA 的）更新 | `document.startViewTransition()` | `static/view-transitions.js` の `withViewTransition()` | 呼び出し側が明示的に利用 |

本書は前者（TASK-8.1）を主題とします。

## 2. `<meta name="view-transition">` から `@view-transition` at-rule への置換

`docs/spec/04-requirements.md` REQ-8・`docs/spec/05-tasks.md` TASK-8.1 は、成果物として
`<meta name="view-transition" content="same-origin">` を記載しています。しかしこれは
View Transitions API の実験段階（Level 1 初期）で提案された構文であり、標準化過程で
廃止されました。現行の標準（View Transitions Level 2）はクロスドキュメント遷移の
有効化を CSS の `@view-transition` at-rule で行います。

```css
@view-transition {
  navigation: auto;
}
```

そのため本実装は、仕様書の文言どおりの meta タグではなく、標準化された at-rule を
採用しています。これは「宣言 1 行・JS 0 行でクロスドキュメント遷移を有効化する」
という仕様の**意図**を、廃止された旧構文ではなく現行標準で満たす判断です。

> 仕様書（`docs/spec/`）自体はサブモジュールであり本リポジトリからは編集できません
> （`.claude/rules/delegation-impl.md`）。文言の乖離は frontend-framework-spec
> リポジトリ側の Issue として起票を提案します（`.claude/rules/out-of-scope-tracking.md`、
> ユーザー承認後に起票）。

## 3. 標準テンプレートへの既定同梱

「標準テンプレートへの既定同梱」という受け入れ基準は、本フレームワークが持つ
2 種類の標準構成それぞれで満たしています。

1. **フルスタック標準（SSR/SSG）**: `rws_app::page_shell()` が `<head>` 内に
   `<style>@view-transition { navigation: auto; }</style>` を出力します。
   `page_shell()` は SSR（`rws_server::ssr::respond`）・SSG（`rws_server::ssg::generate`）
   の両方から分岐なく呼ばれる共通関数（REQ-6）であるため、全ルートに既定同梱されます。
   回帰は `server/tests/view_transitions.rs`（トップページ・全アイテム詳細ページ・
   404 ページ・SSG 全出力ファイルを対象）と `app/src/lib.rs` の単体テストで固定して
   います。
2. **最小埋め込み標準**: `templates/embed/embed.html`（TASK-7.1a・#52）の `<head>` にも
   同一の at-rule を明示的に配置しています。このファイルはフレームワーク管理下の
   マウントポイント（`<div id="app-list">`）を除き利用者が自由に書き換える前提の
   雛形であるため、この `<style>` 行は利用者がコピー後に削除しても構いません
   （責務境界は `templates/embed/embed.html` 冒頭のコメント参照）。回帰は
   `xtask/tests/template_embed_html.rs` で固定しています。

`templates/default/*.html` という成果物パスは作成していません。標準テンプレートの
HTML 骨格は `page_shell()`（Rust 関数）が生成する設計であり、実際には使われない
静的 HTML を別途置くことは構成管理上有害と判断しました（`templates/default/` の
本格的なテンプレート骨格整備自体は `docs/app-api.md` 設計判断 3 に記録された別スコープの
設計余地です）。

## 4. 非対応ブラウザへの配慮

`@view-transition` at-rule は non-supporting ブラウザでは単に無視され、通常の
ナビゲーション（アニメーションなしの即時遷移）にフォールバックします。JS 分岐や
feature detection を必要としない graceful degradation であり、これは
`static/view-transitions.js` の `withViewTransition()` が `document.startViewTransition`
の存在チェックを行っているのと対照的です（クロスドキュメント側は CSS の性質上、
その種のチェック自体が不要）。

## 5. セキュリティ不変条件

at-rule の内容はユーザー入力を一切含まない固定リテラルであり、`page_shell()` は
`el`/`text`（既定エスケープ経路）経由でこれを `<style>` 子ノードとして出力します
（REQ-1 非弱体化）。`raw_html()` 等のエスケープ迂回 API・HTML 文字列の直接組み立ては
使用していません。`server/tests/view_transitions.rs` と
`xtask/tests/template_embed_html.rs` は、廃止済み `<meta name="view-transition">`
構文が再導入されていないことも併せて回帰固定しています。
