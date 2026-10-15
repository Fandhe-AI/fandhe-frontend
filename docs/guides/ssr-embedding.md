# SSR 組み込みガイド

既存のサーバー（fandhe-backend など、`fandhe-frontend-server` を使わないもの）のリクエストハンドラから、fandhe-frontend の描画で HTML を生成して返す方法を説明します。対象は `fandhe-frontend-app` 0.2.8 以降です（`Document` は 0.2.7、CSP と `ViewTransitions` は 0.2.8 で追加）。

`page_shell` はこの用途の入口ではありません。`/static/hydrate.js` とインラインの `<style>` が固定されているためで、代わりに `Document` を使います。

## 責務分担

| 担当 | 責務 |
|---|---|
| ハンドラ側 | ルーティング、Cookie とセッション、POST の受信とフォーム解析、入力検証、CSRF 判定、リダイレクト、レスポンスヘッダ一式（`Content-Type`・`Cache-Control`・`Vary`・CSP・`frame-ancestors`） |
| fandhe-frontend 側 | `Node` 木の構築と、`Document::render()` による DOCTYPE 付き HTML 文字列の生成だけ |

`Document` は HTTP ヘッダを持たない純関数層です。既定エスケープ（REQ-1）が一次防御で、CSP は二次防御です。

## 最小の組み込み

```rust
use fandhe_frontend_app::Document;
use fandhe_frontend_core::{el, text};

let form = el("form", vec![("method", "post")], vec![text("Sign in")]);
let html = Document::new("Sign in", form)
    .lang("en")
    .stylesheet("/assets/auth.css")
    .script_module("/assets/auth.js")
    .render();
assert!(html.starts_with("<!DOCTYPE html>\n"));
assert!(html.contains("<title>Sign in</title>"));
assert!(html.contains("/assets/auth.css"));
assert!(html.contains("type=\"module\""));
```

ハンドラの処理の流れは次のとおりです。

```text
GET:  リクエスト受信 -> 必要ならセッション確認 -> Node 構築
      -> Document::render() -> ヘッダを付けて 200 を返す
POST: Origin / Sec-Fetch-Site の判定 -> フォーム解析 -> 検証
      -> 成功なら 303（リダイレクト先は固定値）
      -> 失敗ならエラー文言をテキストノードとして渡して再描画
```

検証エラーなど利用者入力由来の文字列は必ず `text` 経由で渡します。`raw_html` に入れたり、HTML を `format!` で組み立てたりしません。

## フォーム

フォームは素の `<form method="post">` を `el("form", ...)` で組みます。構造は既存の `field` / `fieldset` 部品が担います。`Form` 部品は提供しません（バリデーションや送信処理を内包する部品は実装しない方針で、`docs/policy/intentional-non-adoption.md` §3.25 に決定記録があります）。検証は利用者の Rust コードで行い、その結果を渡します。

`action` には固定の同一オリジン相対パスのリテラル（例 `/login`）だけを渡し、リクエスト由来の値から組み立てません。core は `javascript:` や `data:` などの危険なスキームを落としますが、`//他ホスト` や `https://他オリジン` は通すため、送信先のオリジンは検証されないからです。多層防御として、CSP に `form-action 'self'` を入れます。

POST 後のリダイレクトも同様に、`next` などのクエリから組み立てず、固定の宛先にします（オープンリダイレクトの防止）。

## レスポンスヘッダとキャッシュ

利用者ごとに変わるページ（認証画面、エラーを伴う再描画、QR コードを含む設定画面など）には、次のヘッダを付けます。いずれもハンドラの責務で、`Document` は出力しません。

```http
Content-Type: text/html; charset=utf-8
Cache-Control: no-store
Vary: Cookie
```

## CSS の配り方

推奨の経路は次のとおりです。

1. `StyleSheet::new()` に `push_theme(&Theme::default())`、`push_recipe`、使う部品の `stylesheet()` を `push_css` で積む（`Theme::to_css` と `StyleSheet` は `fandhe-frontend-pre-styled-ui` の API）。
2. View Transitions を使うなら `VIEW_TRANSITION_CSS` も同じ CSS に積む。
3. `write_css_file` でビルド時に書き出すか、ハンドラの静的ルートで配信する。
4. `Document::stylesheet("/assets/app.css")` で参照する。

`StyleSheet::style_element()` によるインライン `<style>` の埋め込みは、CSP でインライン style を許す構成でだけ使えます。View Transitions は `ViewTransitions::Stylesheet(..)` を使うか、配信 CSS に `VIEW_TRANSITION_CSS` を含めます。

`Document` は、安全な URL として通らない stylesheet / script の URL を、要素ごと黙って省略します（fail-closed）。URL の打ち間違いは「スタイルの無いページ」として現れるので、出力に `<link rel="stylesheet"` が含まれることをテストで確認してください。

## CSP

inline の `<style>` と `<script>` 本文を一切出さない構成のレシピです。

```rust
use fandhe_frontend_app::{Document, ViewTransitions};
use fandhe_frontend_core::{el, text};

let html = Document::new("Sign in", el("p", vec![], vec![text("hi")]))
    .content_security_policy("default-src 'none'; style-src 'self'; script-src 'self'")
    .view_transitions(ViewTransitions::Stylesheet("/assets/vt.css".into()))
    .stylesheet("/assets/app.css")
    .script_module("/assets/app.js")
    .render();
assert!(html.contains("http-equiv=\"Content-Security-Policy\""));
assert!(!html.contains("<style"));
assert!(html.contains("<script type=\"module\" src=\"/assets/app.js\"></script>"));
```

meta の CSP では `frame-ancestors`・`report-uri`・`report-to`・`sandbox` が無効で、clickjacking 対策になりません。HTTP ヘッダでの送出を推奨します。ヘッダ側で足すポリシーの例です（上のレシピとは別物です）。

```http
Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

meta とヘッダの両方を送ると両方が適用され、より厳しい側が効きます。片方に寄せるか、内容を揃えてください。

注意点が 2 つあります。

- `head_node` や body に入れるノードに、インラインの style / script を入れない責任は利用者にあります。`Document` は渡されたノードを走査・除外しません。
- pre-styled-ui の一部の部品（progress・slider・avatar・splitter など）は `style` 属性（CSS 変数など）を出すため、`style-src 'self'` だけでは表示が崩れます。使う部品を選ぶか、`style-src-attr 'unsafe-inline'` を追加します。docs サイトも同じ判断をしています（`docs/design/docs-site-csp-policy.md`）。

## CSRF 対策

状態を変える POST では、ハンドラ側で次を判定します。

- `Sec-Fetch-Site` が `same-origin` 以外なら拒否する。
- `Origin` が自オリジンと一致しなければ拒否する。ヘッダが無い場合の扱いは安全側（拒否）に倒す。
- Cookie の `SameSite` 属性を設定する。

判定方針の正は `Fandhe-AI/fandhe-backend-spec#15` です。fandhe-frontend は CSRF トークンの発行も検証もしません。

## 秘密情報の扱い

qr_code の `aria_label` に、`otpauth://` URI やシークレットを入れません。「認証アプリ設定用 QR コード」のような汎用のラベルにします。`title`・`data-*` 属性・ログにも秘密を出しません。QR コード自体に秘密が含まれるため、そのページには `Cache-Control: no-store` が必須です。

## example の置き場所

fandhe-backend に依存する example は、fandhe-frontend から fandhe-backend への依存を作らないため、本リポジトリの `examples/` には置きません。fandhe-backend 側または fandhe-auth-backend 側に置く想定です。

## 関連文書

- [最小埋め込みガイド](./embedding-guide.md)
- [fandhe-frontend-app API](../api/app-api.md)
- [fandhe-frontend-server API](../api/server-api.md)
- [デプロイガイド](./deployment.md)
