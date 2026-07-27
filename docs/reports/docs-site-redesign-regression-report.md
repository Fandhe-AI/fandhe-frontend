# 3 カラム刷新の回帰検証レポート（イシュー #912）

> **本レポートは #912（ルート #899「3 カラム刷新」）時点の記録に、#961 で追補
> §8〜§11 を加えた統合記録である。** §1〜§7 は #912 実施時点の記録として保存し
> 書き換えていない。#924 ツリー（Radix 参照・docs サイト情報設計刷新、
> Phase 1〜8）完了後の状態は §8（4 観点 + 見出しアンカーの再確認、§6 保留事項の
> 解消状況）・§9（実測課題 A〜G の before / after）・§10（未解消・意図的な
> 見送りと追跡先）を参照。

## 1. 目的とトレーサビリティ

- イシュー #912「test(docs-site): ダークモード・View Transitions・SkipNav・
  レスポンシブの回帰検証」（親 #903「Phase 4」/ ルート #899）は、`crates/docs-site/`
  の 3 カラム刷新（#905〜#911, #916, #918〜#920）がサイト全体で退行していないこと
  を検証するタスクである。設計文書 `docs/design/docs-site-three-column-redesign.md`
  §6「回帰検証の観点（→ #912）」が示す 5 観点（ダークモード・View Transitions・
  SkipNav・レスポンシブ・見出しアンカー）に対応する。
- 検証・是正は origin/main の最新コミット `daedf7c`（#919「ヘッダー刷新」・
  #920「右カラム目次」まで反映済み）を起点とした専用 worktree（ブランチ
  `test/912-docs-site-regression`）で実施した。

| イシュー受け入れ条件 | 設計文書 §6 の観点 | 対応節 |
|---|---|---|
| ライト / ダーク両モードで全ページ種別（index / ガイド / ショーケース / API リファレンス）の表示が確認されている | ダークモード | §4.1（機械）・§4.2（実機、環境制約あり） |
| SkipNav・見出しアンカー・View Transitions が退行していない | SkipNav・見出しアンカー・View Transitions | §4.1（機械）・§4.2（実機、環境制約あり） |
| `cargo test -p fandhe-frontend-docs-site` を含む関連テストが通る | — | §4.1 |
| （レスポンシブは受け入れ条件本文に明記はないが設計文書 §6 の観点） | レスポンシブ | §4.1（機械）・§4.2（実機、環境制約あり） |

## 2. 判定ステータス

**制約付き Pass**。機械検証（`cargo test`/`cargo clippy`/`cargo fmt`）はすべて
Pass し、指摘に対する是正（ハードコード色 1 件）も実施済み。一方で実ブラウザに
よる視覚確認（Step 6、設計文書 §6 の「実機のみで確認可能」な項目）は、本実行
環境（後述 §5）で Chromium を起動できないため **実施不能** だった。この制約は
下記 §6 で明示し、機械検証で担保できる範囲を明確に切り分ける。

## 3. 判定基準

### 3.1 機械検証（本イシューで新設・強化したテスト）

| # | テスト | 対応観点 |
|---|---|---|
| 1 | `site_theme::tests::every_referenced_fandhe_token_is_defined` | ダークモード（トークン網羅） |
| 2 | `site_theme::tests::color_and_shadow_tokens_are_defined_in_all_three_mode_blocks` | ダークモード（3 ブロック網羅） |
| 3 | `site_theme::tests::stylesheet_declares_color_scheme_light_dark` | ダークモード（OS 追従） |
| 4 | `site_theme::tests::structural_and_typography_css_contain_no_hardcoded_colors` | ダークモード（ハードコード色ガード） |
| 5 | `site_theme::tests::stylesheet_base_breakpoint_matches_responsive_contract` | レスポンシブ（基底帯域） |
| 6 | `site_theme::tests::stylesheet_media_queries_are_ordered_mobile_first` | レスポンシブ（カスケード順） |
| 7 | `layout_render::docs_page_emits_view_transition_opt_in_style_in_head` | View Transitions |
| 8 | `layout_render::docs_page_with_assets_emits_view_transition_opt_in_style_in_head` | View Transitions |
| 9 | `layout_render::docs_page_skip_nav_link_href_matches_content_target_id` | SkipNav |
| 10 | `layout_render::docs_page_skip_nav_link_is_first_focusable_element_in_body` | SkipNav |
| 11 | `site_build::real_site_build_covers_all_page_kinds_with_shared_layout_contract` | 全観点（4 ページ種別横断） |

既存テスト（234 件、`stylesheet_defines_docs_accent_bg_token_in_light_dark_and_theme_attr_blocks`
等）は 1 件も削除・弱体化していない。

### 3.2 §3.3 の「狭幅で右目次が非表示になること」の許容可否

設計文書 §3.3 は「`<1200px` で右目次カラムが `display: none` になりページ内
目次への到達手段が失われる点は #912 で許容可否を確認する」としている。

**判定: 許容する。** 見出し自体は本文中に残るため情報は失われず、失われるのは
ページ内ナビゲーションの affordance のみである。加えて `<768px` では左ナビ
折りたたみトグルがセクション間移動手段を提供する。追加実装（狭幅での目次
`<details>` 化等）は本イシューでは行わない。

## 4. 実測結果

### 4.1 機械検証

#### 4.1.1 ベースライン確認（変更前）

```
$ cargo test -p fandhe-frontend-docs-site
test result: ok. 79 passed; ...
test result: ok. 5 passed; ...
test result: ok. 20 passed; ...
test result: ok. 95 passed; ...
test result: ok. 9 passed; ...
test result: ok. 8 passed; ...
test result: ok. 7 passed; ...
test result: ok. 2 passed; ...
test result: ok. 8 passed; ...
test result: ok. 1 passed; ...
```

合計 234 件、全 green（既存不良ゼロ）を確認してから着手した。

#### 4.1.2 追加後の全テスト

```
$ cargo test -p fandhe-frontend-docs-site
test result: ok. 85 passed; ...   # site_theme lib（+6）
test result: ok. 5 passed; ...
test result: ok. 24 passed; ...   # layout_render（+4）
test result: ok. 95 passed; ...
test result: ok. 10 passed; ...   # site_build（+1）
test result: ok. 8 passed; ...
test result: ok. 7 passed; ...
test result: ok. 2 passed; ...
test result: ok. 8 passed; ...
test result: ok. 1 passed; ...
```

合計 245 件（234 + 11）、全 green。既存テストの削除・`#[ignore]` 化はゼロ。

```
$ cargo test -p fandhe-frontend-docs-site -p fandhe-frontend-pre-styled-ui
（docs-site 全件 + pre-styled-ui 1286+α 件、すべて green）
```

#### 4.1.3 lint

```
$ cargo fmt --all --check   # 差分なし
$ cargo clippy -p fandhe-frontend-docs-site --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.66s   # 警告ゼロ
```

#### 4.1.4 実サイトビルド（linkcheck 内蔵・fail-closed）

```
$ cargo run --locked -p fandhe-frontend-docs-site -- --out target/tmp/docs-site-pages/fandhe-frontend
fandhe-frontend-docs-site: wrote 22 page(s) and 4 asset(s) to target/tmp/docs-site-pages/fandhe-frontend
```

22 ページ・4 アセットがエラーなく生成された（内蔵 linkcheck が全内部リンクを
検証済み、`build_site` の fail-closed 契約により 1 件でも壊れたリンクがあれば
非ゼロ終了・出力なしとなる）。

### 4.2 実ブラウザ検証（環境制約により実施不能）

Playwright MCP（`browser_navigate`）で Chromium を起動しようとしたところ、
`browser_install` が以下のエラーで失敗した。

```
Error: ERROR: Playwright does not support chromium on ubuntu26.04-x64
```

本実行環境（後述 §5）は Playwright が対応する OS 一覧に含まれておらず、
ブラウザのインストール自体が不可能だった。したがって以下は **本レポートでは
実施不能** であり、機械検証（§4.1）が担保する範囲を超える確認事項として
未検証のまま記録する（§6 参照）。

- 4 ページ種別 × 3 ブレークポイント × light/dark の視覚確認
- 左ナビ折りたたみ（キーボード操作）の実機確認
- ヘッダードロップダウンの `:focus-within`/`:hover` 実機確認・右端はみ出し確認
- 見出しアンカーの sticky ヘッダー回避の実機確認
- View Transitions のページ遷移・コンソールエラー確認

## 5. 実行環境

- OS: Linux 7.0.0-27-generic（`ubuntu26.04-x64` として Playwright に検出された
  ディストリビューション。詳細なディストリ名は本レポートの機微情報記載方針
  により省略）
- Rust: リポジトリの `rust-toolchain` / `Cargo.lock` に準拠（`cargo test`/`clippy`/
  `fmt` はすべて成功）
- ブラウザ: 未インストール（Playwright 未対応、§4.2 参照）
- 配信手順（実施できた範囲）: `python3 -m http.server 8765 --bind 127.0.0.1 --directory target/tmp/docs-site-pages`
  → `http://127.0.0.1:8765/fandhe-frontend/`（`base_path = "/fandhe-frontend"` の
  罠を避けるため、生成物の**親ディレクトリ**を配信ルートにした。`curl` で
  `index.html` が HTTP 200 で取得できることのみ確認済み）

## 6. 保留・環境制約事項（必須）

- **実ブラウザ確認が本実行環境で不可能だった**（§4.2）。Playwright が
  `ubuntu26.04-x64` 上の Chromium をサポートしないため、`browser_install` が
  fail-closed でエラー終了した。ダークモード・レスポンシブ・SkipNav・見出し
  アンカー・View Transitions の **視覚的・対話的な実機確認は本 PR の範囲では
  行えていない**。機械検証（§4.1・§3.1 の 11 テスト）が担保する範囲（CSS
  トークン定義・DOM 構造・href/id 対応・opt-in 宣言の存在）に限定して Pass と
  判定する。実機確認が可能な環境（Playwright 対応 OS・Chromium 導入済み
  runner 等）での追試を推奨する（下記 out-of-scope 提案は行わない。本 PR の
  受け入れ判定はこの制約を明示した上での機械検証ベースの Pass とする）。
- `@media (prefers-color-scheme: dark)` は Playwright MCP のツール集合に
  `browser_emulate_media` が存在しないため、いずれにせよエミュレートによる
  視覚確認はできない設計上の制約だった（実機確認ができていれば
  `:root[data-theme="dark"]` 経路のみ視覚確認し、`prefers-color-scheme` 経路は
  §3.1 のトークン網羅スイープでのみ担保する計画だった）。
- View Transitions はアニメーション自体を機械アサートしておらず、opt-in 宣言
  （`@view-transition { navigation: auto; }`）の存在のみを固定している。遷移の
  実際の視覚効果・ページ遷移成功・コンソールエラー無しの確認は §4.2 と同じ
  理由で未実施。
- `base_path` プレフィックス（`/fandhe-frontend`）の都合で、配信ルートを
  生成物の親ディレクトリにする必要がある（`<out>/fandhe-frontend/` を生成し
  `<out>/` を配信ルートにする）。素の `<out>/` 配信では全アセット・全内部
  リンクが 404 になり「ダークモードが壊れている」ように誤読しやすいため、
  再現手順として明記する。

**本節（実ブラウザ確認が本実行環境で不可能だった件）の #924 刷新後の解消状況は
§8.2 を参照。**

## 7. 参照

- 設計文書: `docs/design/docs-site-three-column-redesign.md` §3.2/§3.3/§5/§6
- 関連イシュー: #899（ルート）/ #903（親 Phase 4）/ #905・#907〜#911・#916・
  #918〜#920（対象の刷新一式）
- 関連テスト:
  - `crates/docs-site/src/site_theme.rs`（tests mod、6 件追加 + box-shadow 是正 1 行）
  - `crates/docs-site/tests/layout_render.rs`（4 件追加）
  - `crates/docs-site/tests/site_build.rs`（1 件追加）

## 8. 追補: #924 刷新後の回帰確認（イシュー #961 / Phase 9-1 = #960）

### 8.0 追補の位置づけと受け入れ条件マッピング表

本追補は #924 ツリー（Radix 参照・docs サイト IA 刷新、Phase 1〜8）完了後の
状態に対する回帰記録であり、§1〜§7 は #912（#899 ツリー・3 カラム刷新）
時点の記録として保存する。一次証跡は
`docs/reports/docs-site-visual-regression-960.md`（イシュー #960 / PR #1006、
`2612c9d` で main へマージ済み）であり、本節はその判定結論と根拠の所在を
引くのみで、画像・実測記録を転記しない（同レポートの §4〜§8 参照）。

| #961 受け入れ条件 | 対応節 |
|---|---|
| 回帰確認結果が記録されている | §8.1 / §8.2 |
| 実測課題 A〜G の解消状況が記録されている | §9 |
| 未解消項目があれば根拠と追跡先が記載されている | §10 |

### 8.1 4 観点 + 見出しアンカーの判定

| 観点 | 機械証跡（`cargo test -p fandhe-frontend-docs-site`） | 視覚・実機証跡（#960） | 判定 |
|---|---|---|---|
| ダークモード | `site_theme::tests::every_referenced_fandhe_token_is_defined` / `color_and_shadow_tokens_are_defined_in_all_three_mode_blocks` / `stylesheet_declares_color_scheme_light_dark` / `structural_and_typography_css_contain_no_hardcoded_colors` — 全 ok（#960 §7） | `assets/docs-site-960/p1-top-1440-{light,dark}.png` ほか light/dark 対 9 組（18 枚。P1: 1440/768/375 幅、P2〜P4: 1440/375 幅、#960 §4） | 解消（`data-theme` 属性経路）。`prefers-color-scheme` 経路は機械テストのみで実機確認は未実施（§8.2 参照） |
| View Transitions | `layout_render::docs_page_emits_view_transition_opt_in_style_in_head` / `docs_page_with_assets_emits_view_transition_opt_in_style_in_head` — ok | なし | opt-in 宣言（`@view-transition { navigation: auto; }`）の存在のみ担保。実際の遷移・コンソールエラー確認は未検証（§8.2） |
| SkipNav | `layout_render::docs_page_skip_nav_link_href_matches_content_target_id` / `docs_page_skip_nav_link_is_first_focusable_element_in_body` — ok | なし（DOM 構造での担保） | 退行なし（構造レベルの機械検証） |
| レスポンシブ | `site_theme::tests::stylesheet_base_breakpoint_matches_responsive_contract` / `stylesheet_media_queries_are_ordered_mobile_first` — ok | 1440 / 375 幅は P1〜P7、768 幅は P1 のみ（#960 §4 のトリム記載）・375 幅でのサイドバー折りたたみ表示崩れなしの目視確認（#960 §8） | 解消 |
| 見出しアンカー | `site_build::real_site_build_covers_all_page_kinds_with_shared_layout_contract` — ok | なし | 構造契約は機械テストで担保。sticky ヘッダーによる見出しアンカー回避の実機確認は未検証（§8.2） |

#924 の刷新で新たに増えた JS（テーマトグル・GitHub リンク・検索）が退行を
持ち込んでいないことは、#960 が新設した `crates/docs-site/tests/no_js_contract.rs`
（6 件・全 ok）と JS 無効環境の撮影（`n1-top-nojs-*.png` / `n2-button-nojs-375.png`）
で担保されている（#960 §6）。加えて `cargo test --workspace` exit code 0、
`cargo fmt --all --check` 差分なし、`cargo clippy -p fandhe-frontend-docs-site
--all-targets -- -D warnings` 警告ゼロ（いずれも #960 §7 の実測）を確認済み。

### 8.2 §6（#912 時点の保留事項）の解消状況

判定は「解消 / 部分解消 / 未検証」の 3 値とし、#960 の証跡が実際に担保する
範囲を超えて格上げしない。

| §6 の保留項目 | 現在 | 根拠 |
|---|---|---|
| 実ブラウザ起動不能（Playwright が `ubuntu26.04-x64` 非対応） | **解消** | #960 は Playwright を使わず chromium CLI（`Chromium 150.0.7871.114 snap` の headless `--screenshot`）で撮影した（#960 §2/§3）。実行環境の制約自体を回避する手段が確立された |
| 4 ページ種別 × ブレークポイント × light/dark の視覚確認 | **解消** | #960 §4 の P1〜P7 マトリクス（28 枚） |
| 左ナビ折りたたみのキーボード操作の実機確認 | **部分解消** | CSS 経路の存在は `structural_css_declares_js_independent_toggle_and_dropdown_paths`（#960 §6）が機械固定し、375 幅での表示崩れなしは目視確認済み（#960 §8）。キーボード操作そのもの（Tab 移動・Enter/Space での開閉）の実機確認は未実施 |
| ヘッダードロップダウンの `:focus-within`/`:hover` 実機確認・右端はみ出し確認 | **未検証** | 上記と同じテストは CSS 経路の存在のみを担保する。ホバー/フォーカスの実操作ショット・右端はみ出しの確認は #960 に無い |
| 見出しアンカーの sticky ヘッダー回避の実機確認 | **未検証** | 該当するショット・アサーションが #960 に存在しない |
| View Transitions のページ遷移・コンソールエラー確認 | **未検証** | opt-in 宣言（`@view-transition { navigation: auto; }`）の存在のみが機械テストで担保されている（#960 §7 のテスト 7・8） |
| `prefers-color-scheme` 経路（システム連動ダークモード） | **未検証（機械テストのみ）** | #960 は `<html data-theme="dark">` の直接注入で撮影しており、システム連動経路は撮影対象外（#960 §5 末尾で明記）。機械テスト（`stylesheet_declares_color_scheme_light_dark` 等）のみが担保する |

## 9. 追補: 実測課題 A〜G の before / after

イシュー #924 が記録した刷新前の実測課題（ローカルビルド + ヘッドレス
Chromium、1440x1200）と、#960 が確認した刷新後の判定を対比する。「是正した
Phase / PR」列は `git log origin/main --oneline` で実在確認できたコミット
（issue 番号 + PR 番号が件名に入っている形式）のみを記載する。

| 課題 | before（#924 実測、刷新前） | after（#960 §5 判定） | 是正した Phase / Issue（PR） | 根拠（証跡） |
|---|---|---|---|---|
| A | 約 60 部品が単一 showcase ページに詰め込まれ、右カラム目次が 60 項目のフラットリストで溢れる | **解消** | Phase 3: #941 showcase.rs のページ単位分解（`d0ca7c5`, PR #973）/ #942 component_page.rs 新設（`738515c`, PR #974）/ #943 site/components/ 原稿整備（`37b0ce6`, PR #976）/ #944 CI 契約追随（`89ff79c`, PR #977） | `assets/docs-site-960/p7-components-index-1440-light.png`（カテゴリ別リンク集）・`p2-button-1440-light.png`（個別ページ実体化） |
| B | サイドバーの Components セクションが 60 部品に対し 1 エントリのみ | **解消** | Phase 2: #939 nav.rs の 3 階層スキーマ実装（`e256b67`, PR #968）/ #940 サイドバーのカテゴリ階層描画（`bb8d15c`, PR #970） | `assets/docs-site-960/p1-top-1440-light.png`（サイドバーの `<details>` カテゴリ階層） |
| C | API Reference が内部設計メモのまま公開（issue 番号・「Phase N」・ロードマップ節が露出） | **解消（適用範囲付き）** — 対象は headless-ui-api / pre-styled-ui-api / pre-styled-recipe-api の 3 ページのみ。残り 6 ページ（`component-api` 等）は意図的据え置き（§10.1 参照） | Phase 6: #952 分離方針確定（`3f96d63`, PR #978）/ #953 headless-ui API 再編（`e123310`, PR #984）/ #954 pre-styled-ui API・recipe API 再編（`b3eafab`, PR #986）/ #955 nav.toml 整合・相互リンク（`e4443ca`, PR #987） | `assets/docs-site-960/p4-api-headless-ui-1440-light.png`（issue 番号・ロードマップ節を含まない構成） |
| D | ヘッダー（全幅）と 3 カラム grid で左端が不揃い | **解消** | Phase 5: #949 レイアウト是正（`abf6075`, PR #972） | `assets/docs-site-960/p1-top-1440-light.png`（ブランドリンクとサイドバーの左端 x 座標一致） |
| E | 右カラム目次に見出しも現在地ハイライトもない | **解消** | Phase 5: #950 右カラム目次の改善（`4611427`, PR #975） | `assets/docs-site-960/p1-top-1440-{light,dark}.png`（`ON THIS PAGE` 見出しとアクティブ項目のアクセントバー） |
| F | テーブルが横スクロール可能だがアフォーダンスがない | **解消** | Phase 5: #949 レイアウト是正（`abf6075`, PR #972、テーブルアフォーダンスもこの PR で対応） | `assets/docs-site-960/f-table-dialog-375-tall.png`（列クリップ）+ `crates/docs-site/src/site_theme.rs` の `overflow-x: auto` / `::-webkit-scrollbar*` 宣言（#960 §5 F 行が 2 系統証跡を明記） |
| G | ダークモードトグル・GitHub リンク・検索がいずれも無い（JS を一切出力していない） | **解消** | Phase 5: #951 script.rs 新設・ヘッダー要素追加（`ae59956`, PR #967）/ Phase 7: #957 search_index.rs（`eb779d6`, PR #988）・#958 検索 UI 実装（`fb0e197`, PR #989） | `assets/docs-site-960/p1-top-1440-{light,dark}.png`（Dark/Light ラベルの往復・GitHub リンク・検索ボックスを両テーマで確認） |

判定値は #960 §5 の 3 値（解消 / 解消（適用範囲付き） / 未解消）をそのまま
引き継いでいる。

## 10. 追補: 未解消・意図的な見送りと追跡先

### 10.1 既存文書・既存判定が追跡している事項（新規起票不要）

| 事項 | 位置づけ | 追跡先 |
|---|---|---|
| C の適用範囲外 6 ページ（`component-api` 等）に内部設計記録が残る | 「未解消」ではなく**トリガー付きの意図的据え置き** | `docs/design/docs-site-api-reference-split.md` §3-7「Phase 6 の適用範囲と将来トリガー」 |
| `<1200px` で右目次カラムが `display: none` になる | 本レポート §3.2 で**許容と判定済み**。#960 の 768 / 375 幅ショットでも本文中の見出しは保持され情報欠落が無いことを確認した | 本レポート §3.2（判定は変更しない） |

### 10.2 未起票の追跡候補（起票は `out-of-scope-tracking.md` に従いユーザー承認後）

以下はいずれも **本 PR では起票していない**。ユーザー承認後の Issue 起票を
別途提案する。

| 事項 | 根拠 | 現時点の追跡 |
|---|---|---|
| `site/components-pre-styled-ui.md:26-29` の「Demo 以外の節の充填は Phase 4（#945〜#948）で進めます」という記述が Phase 4 完了後も残存 | 本追補作成時点で `origin/main` の該当行に現存を確認済み。#960 §8 も同事象を是正提案として記録している | 本レポート §10.2 に記録。原稿の書き換えは当時の PR に含めなかった（→ イシュー #1077 で `site/themes.md` の記述を現況へ是正済み） |
| `tools/docs-site/visual-regression.sh` に tall-window 撮影（F 証跡取得手順）が未統合 | #960 §3 / §8。`f-table-*-tall.png` は大きい `--window-size` 高を手動指定するアドホックな chromium 呼び出しで取得しており、通常の再撮影手順（同スクリプト実行）には含まれていない | 同上 |
| §8.2 の「未検証」判定項目（ヘッダードロップダウンの実操作、見出しアンカーの sticky ヘッダー回避、View Transitions の実遷移、`prefers-color-scheme` 経路の実機確認） | 実機の対話操作・メディアクエリのエミュレーションは、本環境で利用可能な撮影手段（chromium CLI の `--screenshot`）では取得できない | 同上 |

## 11. 追補: 参照（#961）

- 一次証跡: `docs/reports/docs-site-visual-regression-960.md`（イシュー #960 /
  PR #1006）・`docs/reports/assets/docs-site-960/`（PNG 28 枚）
- 関連テスト: `crates/docs-site/tests/no_js_contract.rs`（6 件、JS 無効環境の
  構造契約）
- 再現手順: `tools/docs-site/visual-regression.sh`（F 証跡の tall-window 撮影は
  §10.2 の通り未統合）
- 設計文書: `docs/design/docs-site-api-reference-split.md`（§3-7「Phase 6 の
  適用範囲と将来トリガー」）
- 関連イシュー: #924（トラッキング、Radix 参照・docs サイト IA 刷新）/
  #933（親 Phase 9）/ #960（Phase 9-1、実測）/ #961（本追補）

## 12. 追補: Primitives / Themes 2 層構成のビジュアル回帰確認（イシュー #1033）

### 12.0 位置づけと受け入れ条件マッピング

本追補は #1035（ルート、Radix 同型の Primitives / Themes 2 層構成への作り替え）
Phase 1〜5（#1012〜#1029、`fandhe-frontend-headless-ui` = Primitives・
`fandhe-frontend-pre-styled-ui` = Themes の 2 層分離、URL 移行
`/components/<kebab>/` → `/themes/<kebab>/`（#1017）、`/primitives/<kebab>/`
新設（#1021）、Primitives 63 部品の原稿充填（#1024〜#1029）を含む）完了後の
状態に対する回帰記録である。親 #1034「Phase 6: 検証とドキュメント追随」の
子イシュー。§1〜§11 は既存記録として保存し書き換えていない（#961 の先例と
同型）。

| #1033 受け入れ条件 | 対応節 |
|---|---|
| 観点 1〜10 すべての合否と根拠が記録されている | §14 |
| `cargo test -p fandhe-frontend-docs-site` と `cargo run … -- --out dist/` が exit 0 | §15.1 |
| 不合格観点の是正または out-of-scope 記録 | §16 |
| 撮影出力を 1 バイトもコミットしていない | §13（配布方針） |

## 13. 実施環境と配信方法

- Chromium: `/snap/bin/chromium`（snap 版、150.0.7871.114）。`--headless
  --disable-gpu --no-sandbox --screenshot=<path>` で撮影。
- 配信方法: **HTTP のみ**（`python3 -m http.server` / CSP `script-src 'none'`
  を付与する自作サーバ、いずれも `127.0.0.1` バインド）。生成 HTML の
  stylesheet href が `/fandhe-frontend/assets/site.css` という絶対パスであり
  `file://` では解決しないため（計画時点の実測どおり）、`file://` は採らない。
- 出力先: 計画のイシュー本文既定（`_/shots/`）ではなく
  `$HOME/fandhe-docs-site-visual/1033-<タイムスタンプ>/` を使った。理由は
  `_/shots/` を worktree（`.claude/worktrees/<name>/` 配下）で解決すると path
  に `.claude` が含まれ、snap の AppArmor により chromium が無音で書き込み
  失敗するため（`tools/docs-site/visual-regression.sh` 自身が同じ理由で
  ドット始まりパス要素を fail-closed で拒否する既存ガード）。実際に `/tmp`
  直下への書き込みでも同様の問題を実測で確認した（下記参照）: snap 版
  chromium は `/tmp` 自体を snap 専用の private mount にリマップするため、
  ホスト側 `/tmp` へ期待通り書き込まれない（`$HOME` 配下は home interface
  経由で正しく解決する）。
- 成果物（PNG 39 枚・manifest.tsv・chromium ログ）は `$HOME` 配下に生成され
  リポジトリ外であり、`git status` で確認したとおり 1 バイトもコミットして
  いない。

## 14. 観点 1〜10 の合否表

判定語彙は既存レポート §8.2 の 3 値（解消 / 部分解消 / 未検証）に「合格」
「合格（注記付き）」を加えた 5 値とし、「未検証」を「合格」へ格上げしない
（#961 §8.2 の判定原則を継承）。

| # | 観点 | 判定 | 根拠 |
|---|---|---|---|
| 1 | ヘッダー遷移 | **合格** | `grep -o '<a href="[^"]*" class="docs-header-trigger"' dist/primitives/accordion/index.html` で 6 件・`<button>` ゼロ（`<a href>` 化を維持）。機械テスト `nav::tests::header_nav_trigger_links_to_section_index_path` / `site_nav::site_nav_declares_index_path_for_every_section` が ok。ショット `p4-primitives-accordion-1440-light.png` で Primitives タブがアクティブハイライトされていることを目視確認 |
| 2 | ドロップダウン維持 | **合格** | `grep -c 'docs-header-dropdown' dist/assets/site.css` = 11（`:hover`/`:focus-within` 規則を含む）。DOM 存在は各ページで確認済み。**実操作（ホバー/フォーカス）のショットは撮影不能**（headless `--screenshot` にホバー手段が無いため、既存 §8.2 が「未検証」として追跡済みの制約を継承。CSS 生成 + DOM 存在という期待値は満たすため観点自体は合格） |
| 3 | サイドバースコープ | **合格** | 機械テスト `site_build::real_site_sidebar_is_scoped_to_the_current_section`（Guides/Themes/Primitives の 3 代表で他セクション混入ゼロ + Primitives 64 リンク固定）が ok。`p4-primitives-accordion-*.png` / `n2-primitives-accordion-nojs-375.png` で Primitives 配下のみ（Forms A/B/C・Overlay/Disclosure・Navigation・Data Display/Utilities）がサイドバーに表示され、Themes/Guides 等が混入していないことを目視確認 |
| 4 | Primitives ページ | **合格（注記付き）** | 63 件生成・Demo/Features/Anatomy/API Reference（Arguments）/Examples/Accessibility = 63/63（`grep -rl 'id="demo"' dist/primitives \| wc -l` 等で確認）。CSS 変数表は仕様どおり 0/63（`grep -rl 'id="css-variables"' dist/primitives` = 0）。**`data-*` 表は 50/63**（`grep -rl 'id="data-attributes"' dist/primitives` = 50）——`component_page.rs::collect_data_attrs_from_tree` がデモツリーから機械走査する仕様上、デモに `data-scope`/`data-part` が現れない部品では表自体が生成されない導出規則によるものであり退行ではない（Themes 側も 57/107 で同様の部分性）。表が無い代表として `primitives/visually-hidden/` を撮影し（`p6-primitives-visually-hidden-1440-light.png`）、他の 5 節はすべて存在し ToC にも「Data Attributes」項目が現れずページとして破綻していないことを目視確認した |
| 5 | Themes ページ | **合格** | `find dist/themes -mindepth 1 -maxdepth 1 -type d \| wc -l` = 107（索引除く）。CSS 変数表 56/107（`grep -rl 'id="css-variables"' dist/themes` = 56）。#1017 の URL 移行が pure rename（内容 0 変更）であることは `git show 6214804 --stat -M` 系の履歴確認と既存テストで担保済み。`p2-themes-accordion-*.png` で CSS 変数表を持つ代表ページの表示を確認 |
| 6 | 旧 URL | **合格** | `/components/` 配下 108 サブディレクトリ + 索引 1 = 109（`site/redirects.toml` の宣言件数と一致）。`components/button/index.html` で `meta refresh` + `rel=canonical` + `<meta name="robots" content="noindex">` + 静的 `<a href="…/themes/button/">` の 4 要素すべてを直接確認済み（`<script>` タグはゼロ）。機械テスト `redirects.rs` / `no_js_contract::redirect_pages_contain_no_script_and_a_static_fallback_link` が ok。**end-to-end のスクリーンショット証跡は取得していない**——実装時の実測で、CSP `script-src 'none'` 配信下で `meta refresh` ページを headless chromium `--screenshot` で開くと無期限にハングすることを確認した（40 秒 `timeout` でも exit 124、CSP 無し配信・CSP 配信での非リダイレクトページはいずれも数秒で成功する対照実験で原因を CSP + meta-refresh の組み合わせに特定）。HTML 直接検証と既存テストで観点自体は十分に担保されるため判定は合格とし、この撮影不能事実は §16 に記録する |
| 7 | 検索 | **合格（一部未検証）** | `dist/assets/search-index.json` の層別集計（`base_path` を除いた href で判定）: primitives 64（索引 1 + 部品 63）/ themes 108（索引 1 + 部品 107）/ components 0 —— 計画時点の実測どおり。機械テスト `search_index::real_site_search_index_is_deterministic_covers_all_nav_pages_and_matches_html_ids` / `real_site_search_index_does_not_contain_redirect_hrefs` が ok。**検索 UI の結果パネル描画は未検証**（headless `--screenshot` は入力操作ができず `script.rs` に `?q=` 等の URL エントリポイントも存在しない。#1035 の「JS への新規配線を追加しない」方針に従い今回も追加しなかった） |
| 8 | ダークモード | **合格** | `p1`〜`p5`・`p9` の light/dark 対（計 16 組）を目視。Primitives の unstyled デモ（`p4-primitives-accordion-1440-dark.png` 等）はダーク背景でもボタン・テキストのコントラストが保たれ判読可能。`assets/primitives-showcase.css` が `:root[data-theme="light"]` / `@media (prefers-color-scheme: dark)` / `:root[data-theme="dark"]` の 3 ブロックを保有し `site.css` の後段でトークンを再宣言する構成に退行なし |
| 9 | レスポンシブ | **合格** | 375/768/1440 の 3 幅で `p1`（top）・`p2`（themes/accordion）・`p4`（primitives/accordion）・`p7`（primitives 索引）・`p8`（themes 索引）を撮影し、ヘッダー・サイドバー・デモのいずれも崩れなし。375 幅ではヘッダーが「Menu」トグルへ折りたたまれる（`p3-themes-dialog-375-light.png` で確認）。`<1200px` で右目次カラムが消える件は既存レポート §3.2 で許容済みであり判定を変更しない |
| 10 | 既存回帰 | **合格** | `cargo test -p fandhe-frontend-docs-site` 550 件全 green（`#[ignore]` 追加ゼロ、失敗ゼロ）。`site_theme` / `layout_render`（View Transitions・SkipNav）/ `no_js_contract` を含む全テストバイナリが green |

## 15. 実測メモ

#### 15.1 機械検証

```
$ cargo test -p fandhe-frontend-docs-site
（unittests 2 本 + tests/ 配下 24 本 + doc-test 1 本 = 27 実行単位、合計 550 件、全 green。失敗 0・ignored 0）

$ cargo fmt --all --check
（差分なし）

$ cargo clippy -p fandhe-frontend-docs-site --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.87s   # 警告ゼロ

$ cargo run --locked -p fandhe-frontend-docs-site -- --out dist/
fandhe-frontend-docs-site: wrote 195 page(s), 109 redirect(s) and 7 asset(s) to dist/
```

計画時点の事前実測（195 ページ / 109 リダイレクト / 7 アセット、primitives
64・themes 108・components 0 件の search-index 分布、primitives の
`data-*` 表 50/63・css-variables 0/63、themes の `css-variables` 表 56/107）
はすべて実装時点でも再現した。

#### 15.2 実ブラウザ撮影

39 枚（P1〜P11・N1・N2）を撮影し、manifest.tsv にファイル名・URL・幅・高さ・
テーマ・JS 有無・バイト数・SHA256 を記録した（`$HOME` 配下、非コミット）。
当初計画の P6（`data-*` 表を持たない代表・light+dark）は枚数バジェット
（40 枚 / 4.5MB）を優先し light のみへ縮約した。N3（旧 URL の end-to-end
証跡）は §14 観点 6 に記載の理由（CSP + meta-refresh でのハング）により
撮影マトリクスから除外した。

## 16. 残課題・未検証と追跡先

既存レポート §10.2 の項目と重複させず、本追補で新たに判明した増分のみ記載
する。

| 事項 | 位置づけ | 追跡 |
|---|---|---|
| CSP `script-src 'none'` 配信下で `<meta http-equiv="refresh">` ページを開くと headless chromium (`--headless --screenshot`) が無期限にハングする | 観点 6 の end-to-end スクリーンショットは取得不能という**ツール制約の発見**であり、docs サイト自体の退行ではない。`tools/docs-site/visual-regression.sh` から該当撮影（旧 N3）を削除し、理由をスクリプト冒頭コメント・本節へ記録した。観点 6 自体は HTML 直接検証 + 既存テストで合格判定済み（§14） | 本レポート §16（新規 Issue 起票はユーザー承認後、`out-of-scope-tracking.md` に従う） |
| `site/themes.md`（Themes 索引）に残る「Demo 以外の節（Features/Anatomy/…）の充填は Phase 4（#945〜#948）で進めます」という記述 | 既存レポート §10.2 で `site/components-pre-styled-ui.md` として記録済みの項目が、#1018 のリネーム後も `site/themes.md` として引き続き現存することを本追補の撮影（`p8-themes-index-1440-light.png`）で再確認した。原稿の書き換えは Phase 5/6 の対象外であり本 PR に含めない | 既存レポート §10.2 と同一の追跡対象（新規ではないため新規行を追加せず、ここでは継続確認の事実のみ記録） |
| ヘッダードロップダウンの `:focus-within`/`:hover` 実機確認・見出しアンカーの sticky ヘッダー回避・View Transitions の実遷移・`prefers-color-scheme` 経路の実機確認 | §8.2 で「未検証」判定済みの既存項目であり、Primitives/Themes 2 層化によって新たに生じた制約ではない（同じ撮影手段の限界が継続しているだけ） | 既存レポート §10.2 と同一（増分なし） |

## 17. 追補: 参照（#1033）

- 関連イシュー: #1035（ルート）/ #1034（親 Phase 6）/ #1033（本追補）/
  #1012・#1038・#1042・#1017・#1016/#1018・#1039/#1045/#1046・#1020〜#1022・
  #1024〜#1029（Phase 1〜5、対象の刷新一式）
- 関連テスト: `crates/docs-site/tests/site_nav.rs` / `redirects.rs` /
  `no_js_contract.rs` / `primitive_showcase.rs` / `primitive_showcase_xss.rs` /
  `search_index.rs` / `site_build.rs`
- 再現手順: `DOCS_SITE_SHOTS_DIR="$HOME/<任意のパス>" tools/docs-site/visual-regression.sh`
  （出力先は絶対パス・非ドット始まりパス要素であることが必須。§13 参照）

## 18. 追補: 狭幅帯域の折りたたみ目次（イシュー #1080）

§3.2 は「`<1200px` で右目次カラムが `display: none` になりページ内目次への
到達手段が失われる点」を**許容**と判定し、§10.1 も「判定は変更しない」として
追跡を終了していた。本追補はその判定を**撤回せず**、狭幅帯域向けの
JS 非依存な代替到達手段（本文冒頭の折りたたみ目次 `nav.docs-toc-inline`）を
追加した記録である。

### 18.1 実装

- `crates/docs-site/src/layout.rs`: 右目次と同じ `toc_items` ヘルパーから
  導出する `toc_inline()` を新設し、`main.docs-main` の第 1 子（SkipNav の
  スキップ先ターゲットより前）へ配線した。素の `<details>`/`<summary>`
  ディスクロージャで、JS を一切追加しない（`crate::script::SITE_JS` は無変更）。
- `crates/docs-site/src/site_theme.rs`（`STRUCTURAL_CSS`）: `.docs-toc-inline`
  系セレクタを基底帯域で表示、`@media (min-width: 1200px)`（右目次カラムが
  表示に切り替わる帯域）で非表示に切り替え、右目次カラムとの重複表示を防いだ。
- `class="docs-toc"`（`crate::script::SITE_JS` のスクロールスパイが
  `document.querySelector` で掴む唯一のセレクタ）は折りたたみ目次側に一切
  付与しない（専用 class `docs-toc-inline`/`docs-toc-inline-summary` のみ）。
  この不変条件は `crates/docs-site/tests/layout_render.rs::inline_toc_does_not_carry_the_scrollspy_class`
  と `crates/docs-site/src/site_theme.rs` の
  `stylesheet_inline_toc_does_not_reuse_the_scrollspy_selector` が機械固定する。

### 18.2 受け入れ条件の担保

| # | 受け入れ条件 | 担保手段 |
|---|---|---|
| 1 | `<1200px` で目次へアクセスできる代替手段が JS 無効でも動作する | `crates/docs-site/tests/layout_render.rs`（DOM 順・markup の回帰テスト 4 件）/ `crates/docs-site/tests/no_js_contract.rs::inline_toc_provides_a_js_free_heading_navigation_path`（実サイトビルド全体を対象に、右目次カラムを持つ全ページで `<a href="#...">` を含むことを固定） |
| 2 | `>=1200px` で右目次カラムと重複表示にならない | `crates/docs-site/src/site_theme.rs::stylesheet_inline_toc_is_visible_below_and_hidden_at_the_three_column_breakpoint`（両帯域を機械固定）/ `crates/docs-site/tests/site_css_contract.rs`（`TOC_ONLY_CLASSES` へ追加した 2 class の 3 方向契約） |
| 3 | ビジュアル回帰確認を実施しレポートへ記録 | §18.3 参照（本セッションの実行環境では実施不能だったことを記録する） |

### 18.3 ビジュアル回帰の実施結果

本イシューの実装セッションでは `tools/docs-site/visual-regression.sh` を
実行したが、headless chromium が起動段階で GPU プロセス初期化に失敗し
（`GPU process isn't usable. Goodbye.`、AppArmor による D-Bus 拒否ログも
同時発生）、撮影 1 枚目（`p1-top-1440-light`）で `Aborted (core dumped)` と
なり全撮影が失敗した。これは本イシューの変更に起因する退行ではなく、
`docs/ci/ci-runner-requirements.md` が既に指摘している chromium 常設・
sandbox 前提の未解決要件（本スクリプトが CI 化されず手動実行専用に
留まっている理由そのもの）に該当する、実行環境固有の制約である。

このため §6 実装計画が想定していた新規撮影（`n4-top-tocopen-nojs-768`。
折りたたみ目次の展開状態を撮る variant）を含め、本セッションでは
スクリーンショット証跡を取得できなかった。受け入れ条件 3（ビジュアル
回帰の実施・記録）はこの制約込みで本節に記録することで満たし、
chromium が利用可能な環境（通常の開発マシン・snap 制約のない環境等）での
再実行を今後の作業として残す。

markup・CSS の帯域別出し分け自体は §18.1/§18.2 の機械テスト（211 + 44 件の
`layout_render.rs`・`site_theme.rs` 内 unit test、`no_js_contract.rs` の
実サイトビルド検証を含む）で担保済みであり、ビジュアル回帰未実施は
「実装の正しさが未検証」ではなく「視覚的な最終確認手段が本セッションの
実行環境では利用できなかった」ことを意味する。

### 18.4 参照

- 関連イシュー: #1080（本追補）/ #1059（親: docs サイトわかりやすさ・見やすさ改善）
- 関連テスト: `crates/docs-site/tests/layout_render.rs` / `site_css_contract.rs` /
  `no_js_contract.rs` / `crates/docs-site/src/site_theme.rs`（unit test）
- 参照ドキュメント: `docs/design/docs-site-three-column-redesign.md` §3.3a
  （本追補と対になる設計文書側の追記）

## 19. 追補: 対話操作検証手段の導入評価（イシュー #1084）

§8.2・§10.2・§16 が「未検証」と記録してきたヘッダードロップダウンの実操作・
検索結果パネルの描画・見出しアンカーの sticky ヘッダー回避・View Transitions
の実遷移・`prefers-color-scheme` 経路の実機確認について、対話操作可能な
検証手段（Playwright / `chromedriver` + WebDriver 等）の導入可否を調査した
（#1084）。既存節（§1〜§18）は書き換えず、追跡先の更新のみをここに記録する
（#961・#1033 の追補方式と同型）。

- **判定**: 現時点では対話操作検証手段の導入を見送る（Playwright は本環境
  で `ubuntu26.04-x64` 非対応のため #924 時点から状況変化なし、`chromedriver`
  + WebDriver は新規発見の候補だったが本 PoC でセッション確立が失敗し
  未検証 5 項目を 1 件もカバーできなかった）。上記「未検証」判定は本追補
  でも**格上げしない**。
- **今後の追跡先**: §8.2・§10.2・§16 の「未検証」項目、および本節が扱う
  対話操作検証手段の導入可否は、今後 `docs/reports/docs-site-redesign-regression-report.md`
  ではなく `docs/ci/docs-site-interaction-testing-evaluation.md`
  （イシュー #1084）を正とする。同文書 §6 に再評価トリガーを定義した。
- 詳細な候補比較・実測証跡・再評価トリガーは
  `docs/ci/docs-site-interaction-testing-evaluation.md` を参照。

## 21. 追補: tall-window 撮影のスクリプト統合（イシュー #1083）

§10.2 が記録していた「`tools/docs-site/visual-regression.sh` に tall-window
撮影（F 証跡取得手順）が未統合」（同表 2 行目）は**本イシューで解消**した
（§10.2 の表自体は既存記録として書き換えず保存する）。

### 19.1 統合内容

- `tools/docs-site/visual-regression.sh` の撮影マトリクスへ、既定ステップ
  T1（`shoot "t1-themes-dialog-375-tall" "/fandhe-frontend/themes/dialog/" 375
  3000 light js "$PORT_LIGHT"`）を P3 ブロックの直後・P4 ブロックの直前に
  1 枚追加した。撮影対象は `/themes/dialog/`（375 幅・light・JS 有効、既存
  P3 と同一ページ・同一幅）で、高さのみ 812 から 3000 へ引き上げ、Arguments /
  Data Attributes / CSS Variables の各表を viewport 内に収める。
- 新規の関数・環境変数・配信サーバは追加していない。既存 `shoot` 関数（幅・
  高さを引数で受ける）へ高さの大きい値を渡すだけで成立する。配信は既存の
  `$PORT_LIGHT`（通常配信）を再利用し、`$PORT_NOJS`（CSP `script-src 'none'`）
  は使わない（meta refresh との組み合わせで `--screenshot` が無期限ハングする
  既知経路があるため、§16/本ファイル冒頭コメント参照）。
- manifest のスキーマ（8 列: `file`/`url`/`width`/`height`/`theme`/`js`/
  `bytes`/`sha256`）は不変。T1 行は `height=3000` によって tall であることを
  自己記述する。
- 撮影マトリクスの末尾に「T1 がファイル数バジェット（40 枚）の最後の 1 枠を
  占めること、次に撮影を追加する場合は #960 計画 §4.3 の削減順序を先に適用
  すること」を明記するコメントを追加した。

### 19.2 実行結果と実施環境の制約

`bash -n tools/docs-site/visual-regression.sh`（構文チェック）・`shellcheck`
（静的解析、既存/新規とも指摘 0 件）はいずれも合格した。

スクリプトの実行（`DOCS_SITE_SHOTS_DIR="$HOME/fandhe-docs-site-visual/..."
bash tools/docs-site/visual-regression.sh`）は §18.3（イシュー #1080）と
同種の実行環境固有の制約に阻まれた。headless chromium が起動段階で GPU
プロセス初期化に失敗し（`GPU process isn't usable. Goodbye.`、AppArmor に
よる D-Bus 拒否ログも同時発生）、撮影 1 枚目（`p1-top-1440-light`、T1 より
前の既存撮影）で `Aborted (core dumped)` となり、T1 を含む全撮影が実行前に
停止した。追加の chromium フラグ（`--disable-software-rasterizer` /
`--disable-dev-shm-usage`）を単独 chromium 呼び出しで試行しても同一の
`ptrace: Input/output error` / `exit code 133` で再現し、本セッションの
実行コンテナの ptrace/seccomp 制約に起因することを確認した（`shoot` 関数の
chromium 起動オプション自体は変更していない。既存 P1〜P9 の撮影も同一環境で
同様に失敗するため、T1 追加が原因の退行ではない）。

このため、実測による枚数・バイト数バジェット（`tools/docs-site/visual-regression.sh`
末尾の判定: 40 枚超または 4.5MB 超で警告。超過時は上限を実測値ベースで
コメント付きで引き上げる、が本イシューの判断ルール）の検証、
および `t1-themes-dialog-375-tall.png` の目視確認（表がフレーム内に収まり
列が右端でクリップされていること）は本セッションでは実施できなかった。
`docs/ci/ci-runner-requirements.md` が既に指摘している chromium 常設・
sandbox 前提の未解決要件（本スクリプトが CI 化されず手動実行専用に留まって
いる理由そのもの）に該当する既知の制約であり、chromium が正常起動できる
環境（snap AppArmor / seccomp ptrace 制約のない開発マシン等）での再実行と
目視確認を今後の作業として残す。

T1 追加によるロジック面の妥当性（幅・高さの選定根拠、既存関数の再利用、
manifest スキーマ不変）は §21.1 の実装内容とコードレビューで担保しており、
「実装の正しさが未検証」ではなく「視覚的な最終確認手段が本セッションの
実行環境では利用できなかった」ことを意味する（§18.3 と同型の限界）。

### 19.3 再現手順の更新

```bash
DOCS_SITE_SHOTS_DIR="$HOME/<任意のパス>" bash tools/docs-site/visual-regression.sh
```

tall-window 撮影（T1）は上記コマンドの**既定ステップ**として撮影される。
追加の手動 chromium 呼び出しは不要（§11 が記録していた「F 証跡の
tall-window 撮影は §10.2 の通り未統合」という前提は本イシューで解消した。
§11 自体の記述は既存記録として書き換えない）。

### 19.4 参照

- 関連イシュー: #1083（本追補）/ #1059（親: docs サイトわかりやすさ・見やすさ
  改善）/ #1080（同種の実行環境制約の先例、§18.3）
- 対応ファイル: `tools/docs-site/visual-regression.sh` / `docs/guides/browser-testing.md` §9
- 一次記録（凍結・本追補では書き換えない）: `docs/reports/docs-site-visual-regression-960.md`
  §「課題 F」（`f-table-dialog-375-tall.png` のアドホック取得経緯）
