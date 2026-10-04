# docs サイトの CSP 導入方針

- ステータス: 確定・実装済み。ユーザー判断（2026-10-04、ルート #3667 に記録）は 4 点で、①導入する、②案 E（meta + インライン外部化）、③`style-src-attr 'unsafe-inline'` を許容する、④リダイレクト案内ページには付けない。実装は #3676（theme bootstrap の外部化）・#3677（`@view-transition` と split-menu の外部化）・#3678（meta CSP と契約テスト）・#3679（Playwright 実機検証）。§1〜§6 は当初の設計記録、実装結果は §7 末尾の「実装済み」を参照
- 起票元: #3661（親 #3656 / ルート #3588）
- 範囲: 公開 docs サイト（`crates/docs-site`）が生成する HTML への Content-Security-Policy の導入可否と方式。コード・CI・ruleset は変更しない

## 1. 結論

- 当初の推奨は「導入する」「案 E を優先し、FOUC または View Transitions が劣化する場合に限り案 H を部分的に併用する」だった。結果は §11 のとおり導入・案 E で確定し、#3679 の実機検証で FOUC・View Transitions とも合格したため、案 H へのフォールバックは使っていない
- GitHub Pages はレスポンスヘッダを設定できないため、`<meta http-equiv="Content-Security-Policy">` が唯一の手段である。`frame-ancestors` と違反報告は使えず、段階導入（Report-Only）もできない。CSP は二次防御であり、一次防御は既定エスケープ（REQ-1）と `site.js` の危険 API 禁止テストのまま変えない
- 実装 issue は #3676 / #3677 / #3678 / #3679 として起票・実施済み（§9）

## 2. 現状の棚卸し

当初の棚卸し時点（#3661）の事実を主とし、現状（#3678 / #3679 後）を併記する。行番号は変動するため関数名を主とする。

| 項目 | 現状 |
|------|------|
| CSP | 当初は出していなかった（回帰レポート `docs/reports/docs-site-styled-blocks-redesign-report.md` §9 E）。現在は `src/csp.rs` の `CONTENT_SECURITY_POLICY` を meta として全ページへ出す（§7 末尾、イシュー #3678） |
| インライン `<script>` | 0 個（イシュー #3676 で外部化済み）。外部 script は 2 本で、`<head>` 先頭の同期読み込み `assets/theme-init.js`（`script::theme_init_js`、`data-theme` を stylesheet より前に確定して FOUC を抑止）と、`defer` の `assets/site.js` である。`layout.rs` の `docs_page_with_layout` はインラインのブートストラップを head に入れない |
| インライン `<style>` | 当初は layout の `@view-transition { navigation: auto; }` と、demo 側の split-menu 用スコープ CSS（`component_specs_overlay.rs`）の 2 系統だった。現在は 0 個（イシュー #3677）。`@view-transition` は `site_theme::assemble()` の末尾へ移し `site.css` と `site-primitives.css` の両方に含め、split-menu は `showcase::stylesheet()`（`assets/pre-styled-ui.css`）へ移した。回帰は `no_js_contract.rs` の `no_generated_page_emits_inline_style_elements` |
| `style` 属性 | 広範に使用。demo が `("style", "...")` を直接渡し、pre-styled-ui の `angle_slider`・`questionnaire`・`recipe::stagger_index_style` は計算値を出す。静的に列挙できない |
| `site.js` | `innerHTML`・`eval`・`new Function`・`document.write`・`insertAdjacentHTML` を使わない（既存テストで固定）。ネットワークは検索インデックスの同一オリジン `fetch(url)` のみ |
| 画像・フォント | サイト CSS は `@import`・`@font-face`・`url(` を持たない（`site_theme.rs` のテストで固定）。favicon は同一オリジン。当初の棚卸しでは、`primitive_showcase`・`primitive_specs`・`showcase.rs`・一部の Blocks の demo が外部ダミー画像（`example.com`）を参照するとされていた（回帰レポート §9 D）。現在の `crates/docs-site/src` には外部 URL の `img` の `src` が無く、Blocks のダミー素材は `blocks/dummy_assets.rs` の `role="img"` 付きインライン SVG である。`#3679` の実機検証でも `img-src` の違反は 0 件だった |
| `<form>` | 当初は Blocks の demo に存在する見込みとされた。現在の `crates/docs-site/src` に `<form>` 要素の生成は無く（`settings_webhook_form.rs` の `"form"` はラジオ項目の値の文字列）、`form-action 'none'` で困る箇所は無い |
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
  - 実装済み（イシュー #3676）: `assets/theme-init.js`（`src/script.rs` の `THEME_INIT_JS`）。Playwright（CPU 4x・キャッシュ無効）で初回 rAF 時点の `data-theme="dark"` を確認
- `@view-transition { navigation: auto; }` は `site.css` へ移す。render-blocking なので `pagereveal` 前に効く想定（要実機検証）
  - 実装済み（イシュー #3677）: `site_theme::assemble()` の末尾に置き、`site.css` と `site-primitives.css` の両方へ含めた。View Transitions の維持は #3679 の実機検証で確認した
- demo の split-menu `<style>` は、スコープ付き規則を既存の showcase 用 CSS アセットへ移すか、`style` 属性で表現できる範囲に直す
  - 実装済み（イシュー #3677）: `showcase::stylesheet()`（`assets/pre-styled-ui.css`）へ移した
- 長所: ハッシュ機構・自前 sha256 が不要。「インライン script ゼロ」で最も強く、`no_js_contract` の唯一の例外が消えて契約が単純になる
- 短所: 同期スクリプトの往復が 1 回増える。GitHub Pages のキャッシュ（max-age 10 分）の影響を受ける。ビルド出力アセットが増える

### 案 E→H フォールバック

案 E の FOUC・View Transitions 検証が不合格の項目だけ、案 H のハッシュ許可でインラインを残す。

## 6. 推奨と理由

導入する。案 E を優先し案 H を代替とする。理由は、依存ゼロ制約を保てること、ハッシュ再計算という保守負債を作らないこと、「インライン script ゼロ」が機械検証しやすいこと。保護範囲は §3 のとおり限定的で、防御の深さを足す位置づけである。

## 7. ディレクティブ案（案 E 前提・当初案）

以下は当初案である。最終形は本節末尾の「実装済み」と `src/csp.rs` を正とする（`img-src` から `data:` を外し、`object-src` を明示しない点が差分）。

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

### 実装済み（イシュー #3678）

- 定数の所在: `crates/docs-site/src/csp.rs` の `CONTENT_SECURITY_POLICY`（単一情報源）。`layout::docs_page_with_layout` が charset・viewport の直後、`title` と全 `script`/`link` より前に meta として出す（通常ページ・トップ・404 が同経路）
- 最終形: `default-src 'none'; script-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; font-src 'none'; connect-src 'self'; base-uri 'none'; form-action 'none'`
- `img-src` は `'self'` のみに絞った。core の `is_safe_url` が `data:` の `src` を落とすため、出力に `data:` 画像は残らない（実測 0 件、回帰は `no_js_contract.rs` の `generated_output_contains_no_data_uri_images`）。上記案の `data:` は不要になった
- `object-src 'none'` は明示しない。`default-src 'none'` のフォールバックで同じ効果になる
- リダイレクト案内ページには付けない（`redirect.rs` の rustdoc 参照）
- 残余リスク: `style-src-attr` 非対応のブラウザは `style-src 'self'` にフォールバックし `style` 属性を拒否する。
- 実機検証済み（イシュー #3679）: 本体 614 ページで CSP 違反 0 件・FOUC なし・View Transitions 維持・リダイレクト案内 125 件の遷移維持を Chromium で確認した（検証は Chromium のみ）。詳細は `docs/reports/docs-site-csp-report.md`。

## 8. 既存契約・テストとの整合（案 E）

| 区分 | 対象 |
|------|------|
| 書き換え | `no_js_contract.rs` の `page_scripts_are_two_external_files_with_no_inline_script`（インライン script ゼロ + 同期外部ブートストラップ `theme-init.js` と `defer` の `site.js` の 2 本。旧名 `site_js_is_loaded_as_single_deferred_external_script` から置換）、`layout_render.rs` のブートストラップ逐語一致アサーション、`site_build.rs` の dist sanity（新アセット）、新アセットを追加する場合は `docs-site.yml` の dist チェック。いずれも #3676 で実施済み |
| 新設（#3678 実装済み: `no_js_contract.rs::body_pages_carry_exactly_one_csp_meta_before_any_script_or_link`・`layout_render.rs::csp_meta_is_emitted_once_right_after_viewport_for_docs_and_landing`・`csp.rs` 単体テスト） | 全出力 HTML に meta CSP が先頭近傍で 1 個、インライン `<script>` / `<style>` が 0 個、`on*=` 属性 0 個（既存）、ディレクティブが許可リストと一致 |
| 維持（弱めない） | `site_css_contract` / `site_typography_contract` / XSS 回帰 |
| 防壁 | 外部ファイルへ書き出す場合も、`is_escape_safe` による fail-closed 検証を書き出し前に維持する |

## 9. 実装 issue の起票候補（候補のみ、起票はしない）

以下は当初の起票候補と、その実績である。

1. 外部化（theme bootstrap・view-transition・demo `<style>`）と FOUC / VT 検証: #3676（theme bootstrap）/ #3677（view-transition・split-menu）として実施済み
2. 外部ダミー画像の置換（D）: 独立した issue は立てていない。現状の `crates/docs-site/src` には外部 URL の画像 `src` が無い（§2）
3. meta CSP の生成ヘルパと契約テスト: #3678 として実施済み
4. Playwright による全ページの CSP 違反ゼロ確認: #3679 として実施済み、結果は `docs/reports/docs-site-csp-report.md`

検証手順の骨子: ローカル配信で全ページの console の CSP violation が 0 件であること。localStorage に `dark` を設定し、CPU スロットリング・キャッシュ無効で再読み込みして初回ペイント時点の `data-theme` を比較すること。同一オリジン遷移で `@view-transition` が有効であること。リダイレクト案内と 404 の動作（ハング再現の有無）。

## 10. セキュリティ考慮

- A03: 既定エスケープを弱めない。`raw_html()` の新規使用なし、HTML 文字列の直接組み立てなし
- A05: meta 方式の限界（§3）と `style-src-attr 'unsafe-inline'` の根拠・残余リスク（§7）を隠さず記載する。`style-src-attr` 非対応ブラウザの残余リスクは Chromium だけで検証しており、他ブラウザでの確認は未実施
- A06: 案 H の自前 sha256 は暗号実装リスクがあり、NIST ベクタ必須。案 E はこの論点を回避できる
- A08: 案 H の再計算漏れは契約テストで防ぐ
- ruleset・branch protection の変更は本件の範囲外。実装時に必須チェックへ影響する場合は実行せず報告する

## 11. 判断結果

ユーザー判断（2026-10-04、ルート #3667 に記録）は次のとおり。

1. 導入する / しない: 導入する
2. 案 E / 案 H の選択: 案 E
3. `style-src-attr 'unsafe-inline'` の許容: 許容する
4. リダイレクト案内ページへの適用要否: 付けない
5. 外部ダミー画像の置換（D）を先行させるか: 判断の記録は無い。事実として、現状の `crates/docs-site/src` に外部 URL の画像 `src` は残っていない（§2）

## 12. 再評価トリガー

利用者入力を扱う機能の追加 / 第三者スクリプトの導入 / ヘッダ設定可能な配信基盤への移行 / 外部化で FOUC を許容できない結果が出た場合。

## 13. 関連文書

`docs-site-search-design.md` §8 / `docs-site-styled-blocks-redesign.md` §10 / `opt-in-thin-js-glue.md` / `docs/policy/intentional-non-adoption.md` / `docs/reports/docs-site-styled-blocks-redesign-report.md` §9。
