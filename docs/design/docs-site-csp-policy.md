# docs サイトの CSP 導入方針

- ステータス: 提案（ユーザーの方針確認待ち）。本文書は設計記録であり、実装を含まない
- 起票元: #3661（親 #3656 / ルート #3588）
- 範囲: 公開 docs サイト（`crates/docs-site`）が生成する HTML への Content-Security-Policy の導入可否と方式。コード・CI・ruleset は変更しない

## 1. 結論

- 導入する方向を推奨する。方式は **案 E（meta 導入 + インライン script / style の外部化）** を優先し、実機検証で FOUC または View Transitions が劣化する場合に限り **案 H（ハッシュ許可）** を部分的に併用する
- GitHub Pages はレスポンスヘッダを設定できないため、`<meta http-equiv="Content-Security-Policy">` が唯一の手段である。`frame-ancestors` と違反報告は使えず、段階導入（Report-Only）もできない。CSP は二次防御であり、一次防御は既定エスケープ（REQ-1）と `site.js` の危険 API 禁止テストのまま変えない
- 実装 issue はユーザーの方針確認後に起票する。本文書の §9 は起票候補の列挙にとどまる

## 2. 現状の棚卸し

`crates/docs-site/src/` の現行コードで確認した事実である。行番号は変動するため関数名を主とする。

| 項目 | 現状 |
|------|------|
| CSP | 出していない（回帰レポート `docs/reports/docs-site-styled-blocks-redesign-report.md` §9 E） |
| インライン `<script>` | 1 個。`layout.rs` の `docs_page_with_layout` が `script::inline_theme_bootstrap()` を head に入れる。文字列は引用符・`<>&` を含まず、`is_escape_safe` が fail-closed で保証するため、ブラウザが見る本文はソース定数とバイト一致する |
| インライン `<style>` | layout の `@view-transition { navigation: auto; }` に加え、demo 側にも存在する（`component_specs_overlay.rs` の split-menu 用スコープ CSS ほか）。生成箇所は実装前に全件再列挙する |
| `style` 属性 | 広範に使用。demo が `("style", "...")` を直接渡し、pre-styled-ui の `angle_slider`・`questionnaire`・`recipe::stagger_index_style` は計算値を出す。静的に列挙できない |
| `site.js` | `innerHTML`・`eval`・`new Function`・`document.write`・`insertAdjacentHTML` を使わない（既存テストで固定）。ネットワークは検索インデックスの同一オリジン `fetch(url)` のみ |
| 画像・フォント | サイト CSS は `@import`・`@font-face`・`url(` を持たない（`site_theme.rs` のテストで固定）。favicon は同一オリジン。一方、`primitive_showcase`・`primitive_specs`・`showcase.rs`・一部の Blocks（ecommerce の order / checkout など）の demo は外部ダミー画像（`example.com`）を参照する（回帰レポート §9 D） |
| `<form>` | Blocks の demo に存在。送信しない静的デモのはずで、`form-action 'none'` は成立する見込み（実装時に確認） |
| `iframe` / `object` / `embed` | 生成コードに無い |
| 出力経路 | 3 経路: 通常ページ（`layout.rs`）、404（`not_found.rs`、layout 経由）、リダイレクト案内（`redirect.rs`、`meta refresh`・script 無し） |

## 3. meta 方式の制約

| 区分 | 内容 |
|------|------|
| 守れる | 注入された外部 script の実行、`<base>` 書き換え、外部へのデータ送出（`connect-src`）、プラグイン（`object-src`） |
| 守れない | clickjacking（`frame-ancestors` は meta で無効）、違反報告（`report-uri` / `report-to` も無効）、`sandbox` |
| 段階導入 | Report-Only 相当の meta は無く、観測だけして後で強制する運用はできない。検証はローカルと Playwright で完結させる必要がある |
| 配置 | meta より前の要素には効かない。head の `charset` / `viewport` の直後に置く |

## 4. 評価軸

防御効果の実益 / 保守コスト / FOUC・View Transitions・無 JS 契約への影響 / 依存ゼロ制約（docs-site の依存閉包を変えない、`docs-site-styled-blocks-redesign.md` §10 A06）/ 機械検証可能性。

## 5. 選択肢の比較

### 案 N: 導入しない

- 根拠: 公開静的サイトで利用者入力を描画しない（検索語は `textContent` 経由、URL 反射なし、フォーム送信なし）。一次防御が既にあり、meta では `frame-ancestors` と報告が使えず効果が限定的
- 短所: 注入や将来の混入に対する深さの防御が無い。`style` 属性が残るため完全な CSP にもならない
- 再評価トリガー（案 N を採る場合）: 利用者入力を扱う機能の追加、第三者スクリプトの導入、ヘッダ設定可能な配信基盤への移行

### 案 H: meta 導入 + ハッシュ許可

`script-src 'self' 'sha256-<theme bootstrap>'`、`style-src 'self' 'sha256-<view-transition>'` と demo `<style>` ごとのハッシュ。

- 長所: インラインのままなので FOUC 抑止が変わらない
- 短所: std のみで sha256 と base64 を自前実装する必要がある（NIST テストベクタ必須）。スクリプト・スタイル変更のたびに再計算が要る。demo `<style>` はページごとにハッシュが変わり、ページ単位の収集をビルドに組み込む必要がある。再計算漏れはビルドが通ってもブラウザで拒否されるため、「出力 HTML 中の全インライン script / style のハッシュは meta のハッシュ集合に含まれる」契約テストが必須

### 案 E: meta 導入 + 外部化（推奨）

- `INLINE_THEME_BOOTSTRAP` を `<head>` 先頭の同期（`defer` なし）外部 `<script src>` へ移す。同期外部 script はパーサをブロックするため、インラインと同様に stylesheet より前に `data-theme` を確定できる想定（要実機検証）
- `@view-transition { navigation: auto; }` は `site.css` へ移す。render-blocking なので `pagereveal` 前に効く想定（要実機検証）
- demo の split-menu `<style>` は、スコープ付き規則を既存の showcase 用 CSS アセットへ移すか、`style` 属性で表現できる範囲に直す
- 長所: ハッシュ機構・自前 sha256 が不要。「インライン script ゼロ」で最も強く、`no_js_contract` の唯一の例外が消えて契約が単純になる
- 短所: 同期スクリプトの往復が 1 回増える。GitHub Pages のキャッシュ（max-age 10 分）の影響を受ける。ビルド出力アセットが増える

### 案 E→H フォールバック

案 E の FOUC・View Transitions 検証が不合格の項目だけ、案 H のハッシュ許可でインラインを残す。

## 6. 推奨と理由

導入する。案 E を優先し案 H を代替とする。理由は、依存ゼロ制約を保てること、ハッシュ再計算という保守負債を作らないこと、「インライン script ゼロ」が機械検証しやすいこと。保護範囲は §3 のとおり限定的で、防御の深さを足す位置づけである。

## 7. ディレクティブ案（案 E 前提）

```
default-src 'none';
script-src 'self';
style-src 'self';
style-src-attr 'unsafe-inline';
img-src 'self' data:;
font-src 'none';
connect-src 'self';
base-uri 'none';
form-action 'none';
object-src 'none';
```

- `script-src` は緩めない。`unsafe-inline` / `unsafe-eval` を script に使わない
- `style-src-attr 'unsafe-inline'` が唯一緩めた点。demo と pre-styled-ui 部品が計算値入りの `style` 属性を出し、`unsafe-hashes` での列挙が非現実的なため。残余リスクはスタイル注入による UI 偽装と属性セレクタ経由の情報漏えいで、`connect-src` / `img-src` / `default-src 'none'` で送出先を制限して緩和する
- `connect-src 'self'` で検索インデックスの同一オリジン `fetch` は維持される
- `img-src 'self' data:` は外部ダミー画像（§2）を明示的にブロックする。`data:` 画像または同一オリジン SVG への置換（回帰レポート §9 D）が CSP 実装の前提または同時対応になる
- `manifest-src` / `worker-src` / `media-src` などは `default-src 'none'` に委ねる（未使用）
- 配置: 共通 head ヘルパ 1 箇所に集約し、通常ページ・404・リダイレクトの 3 経路すべてで同一ヘルパから出す。meta は `charset` と `viewport` の直後、`title` / `link` / `script` より前
- 案 H を採る場合は `script-src 'self' 'sha256-…'` の形とし、ハッシュはビルド時に生成して契約テストで固定する
- リダイレクト案内ページは過去に `script-src 'none'` 配信下の `meta refresh` 撮影がハングした実績がある。適用するなら `default-src 'none'` のみの最小 CSP とし、headless chromium で実機確認する

## 8. 既存契約・テストとの整合（案 E）

| 区分 | 対象 |
|------|------|
| 書き換え | `no_js_contract.rs` の `site_js_is_loaded_as_single_deferred_external_script`（インライン script ゼロ + 同期外部ブートストラップ 1 本）、`layout_render.rs` のブートストラップ逐語一致アサーション、`site_build.rs` の dist sanity（新アセット）、新アセットを追加する場合は `docs-site.yml` の dist チェック |
| 新設 | 全出力 HTML に meta CSP が先頭近傍で 1 個、インライン `<script>` / `<style>` が 0 個、`on*=` 属性 0 個（既存）、ディレクティブが許可リストと一致 |
| 維持（弱めない） | `site_css_contract` / `site_typography_contract` / XSS 回帰 |
| 防壁 | 外部ファイルへ書き出す場合も、`is_escape_safe` による fail-closed 検証を書き出し前に維持する |

## 9. 実装 issue の起票候補（候補のみ、起票はしない）

1. 外部化（theme bootstrap・view-transition・demo `<style>`）と FOUC / VT 検証
2. 外部ダミー画像の置換（D）
3. meta CSP の生成ヘルパと契約テスト（1・2 に依存）
4. Playwright による全ページの CSP 違反ゼロ確認（3 に依存）

検証手順の骨子: ローカル配信で全ページの console の CSP violation が 0 件であること。localStorage に `dark` を設定し、CPU スロットリング・キャッシュ無効で再読み込みして初回ペイント時点の `data-theme` を比較すること。同一オリジン遷移で `@view-transition` が有効であること。リダイレクト案内と 404 の動作（ハング再現の有無）。

## 10. セキュリティ考慮

- A03: 既定エスケープを弱めない。`raw_html()` の新規使用なし、HTML 文字列の直接組み立てなし
- A05: meta 方式の限界（§3）と `style-src-attr 'unsafe-inline'` の根拠・残余リスク（§7）を隠さず記載する
- A06: 案 H の自前 sha256 は暗号実装リスクがあり、NIST ベクタ必須。案 E はこの論点を回避できる
- A08: 案 H の再計算漏れは契約テストで防ぐ
- ruleset・branch protection の変更は本件の範囲外。実装時に必須チェックへ影響する場合は実行せず報告する

## 11. 要ユーザー判断

1. 導入する / しない
2. 案 E / 案 H の選択
3. `style-src-attr 'unsafe-inline'` の許容
4. リダイレクト案内ページへの適用要否
5. 外部ダミー画像の置換（D）を先行させるか

## 12. 再評価トリガー

利用者入力を扱う機能の追加 / 第三者スクリプトの導入 / ヘッダ設定可能な配信基盤への移行 / 外部化で FOUC を許容できない結果が出た場合。

## 13. 関連文書

`docs-site-search-design.md` §8 / `docs-site-styled-blocks-redesign.md` §10 / `opt-in-thin-js-glue.md` / `docs/policy/intentional-non-adoption.md` / `docs/reports/docs-site-styled-blocks-redesign-report.md` §9。
