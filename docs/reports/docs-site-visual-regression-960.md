# docs サイト全ページのビジュアル回帰確認と #922 回帰テストの実行（イシュー #960）

親: #933（Phase 9） / トラッキング: #924 / 実装計画: `_/local-plans/960-docs-site-visual-regression.md`

## 1. 目的とトレーサビリティ

#924 が記録した docs サイト刷新前の実測課題 A〜G は、Phase 2〜8 の 30 本超の PR で
個別に是正された。しかし各 PR の検証は「自分が触った層の機械テスト」に閉じており、
刷新後のサイト全体を実ブラウザで通しで見た記録が存在しなかった（#922 の回帰レポート
`docs/reports/docs-site-redesign-regression-report.md` §4.2 は実ブラウザ検証を保留と
記載）。本レポートは以下の受け入れ条件に対応する。

| 受け入れ条件 | 対応節 |
|---|---|
| 3 ブレークポイント × 2 テーマのショットが揃い確認できる | §4 |
| 課題 A〜G の解消をショットで確認 | §5 |
| #922 の回帰テストが通る | §7 |
| `cargo test --workspace` が通る | §7 |
| JS 無効環境で全ページの閲覧・ナビゲーションが成立する | §6 |

## 2. 実行環境

| 項目 | 値 |
|---|---|
| コミット | `519755aa39f70204df907f03555cdc0ffadc1a8f`（origin/main `218739b` 由来ブランチ上のテスト追加コミット） |
| chromium | `Chromium 150.0.7871.114 snap`（CLI headless、`--headless --disable-gpu --no-sandbox --screenshot=...`） |
| rustc | `rustc 1.96.0 (ac68faa20 2026-05-25)` |
| OS | `Linux 7.0.0-27-generic #27-Ubuntu SMP x86_64` |
| 撮影ツール | `pngquant` / `optipng` / `cwebp` / ImageMagick / Pillow はいずれも未導入。枚数と viewport 高（フルページではない）で容量を制御した |

## 3. 再現手順

```bash
# 出力先は絶対パス・非ドットパス要素必須（既定 $HOME 配下、詳細はスクリプト冒頭コメント参照）
bash tools/docs-site/visual-regression.sh
# => $HOME/fandhe-docs-site-visual/<timestamp>/shots/*.png と manifest.tsv を生成
```

スクリプトは (1) `cargo run -p fandhe-frontend-docs-site -- --out ...` で実サイトを
ビルド（内蔵 linkcheck が fail-closed のため成功が全ページ生成の保証）、(2) 全 HTML の
`<html lang="ja">` を `<html lang="ja" data-theme="dark">` へ置換したダーク変種を生成
（置換前後の件数一致を fail-closed 検証）、(3) `Content-Security-Policy: script-src 'none'`
を返す簡易サーバで JS 無効相当の描画を用意、(4) 1 枚ごとにファイル存在・サイズ > 0 を
検証しながら撮影する（chromium は撮影失敗時も 0 終了し得るため無音失敗を成功と誤記録
しない）。3 サーバはいずれも `127.0.0.1` バインドのみ。

F（テーブル横スクロールのアフォーダンス）評価用の 2 枚
（`f-table-dialog-375-tall.png` / `f-table-api-1440-tall.png`）は、通常撮影が
viewport 高固定でテーブルより手前で切れてしまうため、該当ページを広い
`--window-size` 高で撮影しテーブル部分を可視化する追加コマンドで取得した
（スクリプト本体には未統合。§8 の残課題に記載）。

## 4. 撮影マトリクスと画像

すべて `docs/reports/assets/docs-site-960/` に相対パスで配置。合計 28 枚・約 4.5MB
（予算: 40 枚・4.5MB 以内）。

### P1: トップ（D・G・E の一次証跡）

| 幅 | テーマ | 画像 |
|---|---|---|
| 1440 | light | ![p1-1440-light](assets/docs-site-960/p1-top-1440-light.png) |
| 1440 | dark | ![p1-1440-dark](assets/docs-site-960/p1-top-1440-dark.png) |
| 768 | light | ![p1-768-light](assets/docs-site-960/p1-top-768-light.png) |
| 375 | light | ![p1-375-light](assets/docs-site-960/p1-top-375-light.png) |
| 375 | dark | ![p1-375-dark](assets/docs-site-960/p1-top-375-dark.png) |

### P2: 部品ページ（Forms 代表、button）

| 幅 | テーマ | 画像 |
|---|---|---|
| 1440 | light | ![p2-1440-light](assets/docs-site-960/p2-button-1440-light.png) |
| 1440 | dark | ![p2-1440-dark](assets/docs-site-960/p2-button-1440-dark.png) |
| 375 | light | ![p2-375-light](assets/docs-site-960/p2-button-375-light.png) |
| 375 | dark | ![p2-375-dark](assets/docs-site-960/p2-button-375-dark.png) |

### P3: 部品ページ（Overlay 代表、dialog。表あり）

| 幅 | テーマ | 画像 |
|---|---|---|
| 1440 | light | ![p3-1440-light](assets/docs-site-960/p3-dialog-1440-light.png) |
| 1440 | dark | ![p3-1440-dark](assets/docs-site-960/p3-dialog-1440-dark.png) |
| 375 | light | ![p3-375-light](assets/docs-site-960/p3-dialog-375-light.png) |
| 375 | dark | ![p3-375-dark](assets/docs-site-960/p3-dialog-375-dark.png) |

### P4: API Reference（headless-ui-api。C・F）

| 幅 | テーマ | 画像 |
|---|---|---|
| 1440 | light | ![p4-1440-light](assets/docs-site-960/p4-api-headless-ui-1440-light.png) |
| 1440 | dark | ![p4-1440-dark](assets/docs-site-960/p4-api-headless-ui-1440-dark.png) |
| 375 | light | ![p4-375-light](assets/docs-site-960/p4-api-headless-ui-375-light.png) |
| 375 | dark | ![p4-375-dark](assets/docs-site-960/p4-api-headless-ui-375-dark.png) |

### F 専用証跡（テーブル部分を viewport 内に収めた追加撮影）

| ページ | 画像 |
|---|---|
| dialog（375 幅、テーブル部分） | ![f-table-dialog](assets/docs-site-960/f-table-dialog-375-tall.png) |
| headless-ui-api（1440 幅、テーブル部分） | ![f-table-api](assets/docs-site-960/f-table-api-1440-tall.png) |

### P5: Guides（quickstart、レスポンシブ確認）

| 幅 | 画像 |
|---|---|
| 1440 | ![p5-1440](assets/docs-site-960/p5-quickstart-1440-light.png) |
| 375 | ![p5-375](assets/docs-site-960/p5-quickstart-375-light.png) |

### P6: Examples（ssr-routing、レスポンシブ確認）

| 幅 | 画像 |
|---|---|
| 1440 | ![p6-1440](assets/docs-site-960/p6-ssr-routing-1440-light.png) |
| 375 | ![p6-375](assets/docs-site-960/p6-ssr-routing-375-light.png) |

### P7: コンポーネント索引（旧 showcase、A の一次証跡）

| 幅 | 画像 |
|---|---|
| 1440 | ![p7-1440](assets/docs-site-960/p7-components-index-1440-light.png) |
| 375 | ![p7-375](assets/docs-site-960/p7-components-index-375-light.png) |

### N1/N2: JS 無効環境（CSP `script-src 'none'` 配信）

| # | ページ | 幅 | 画像 |
|---|---|---|---|
| N1 | トップ | 1440 | ![n1-1440](assets/docs-site-960/n1-top-nojs-1440.png) |
| N1 | トップ | 375 | ![n1-375](assets/docs-site-960/n1-top-nojs-375.png) |
| N2 | button | 375 | ![n2-375](assets/docs-site-960/n2-button-nojs-375.png) |

### 容量予算に伴うトリム（§4.3 の削減順序を適用）

計画の削減順序（(1) P5/P6 を 1440 のみ → (2) P3 を 1440/375 の 2 幅 → (3) 768 幅を
P1/P2 のみ）はスクリプトの既定マトリクスとして最初から反映済みだったが、F 専用の
追加撮影（テーブル可視化 2 枚、計約 0.7MB）を加えたことで合計が予算（4.5MB）を
超過したため、追加で次を間引いた: P1/P2 の 768 幅ダーク・N1 の 768 幅を削除し、
3 ブレークポイントの網羅を保つため P1 の 768 幅ライトのみ残した。

## 5. 課題 A〜G の判定

| 課題 | 期待する状態 | 判定 | 根拠 |
|---|---|---|---|
| A | 60 部品の単一 showcase ページが解消し 1 部品 1 ページ | **解消** | P7 の索引ページ（`p7-components-index-1440-light.png`）はカテゴリ別リンク集で、60 部品のフラット掲示ではない（原稿 `site/components-pre-styled-ui.md` は見出し 8 個・150 行）。P2/P3（button/dialog）が個別ページとして実体化していることを 1440 幅で確認 |
| B | サイドバー Components がカテゴリ階層で展開 | **解消** | `p1-top-1440-light.png` のサイドバーで Typography/Forms/Interactive/Data Display/Utilities/Charts の `<details>` カテゴリ見出しを確認（p2/p3 の 1440 幅でも同型） |
| C | API Reference が利用者向けで issue 番号・Phase 表記が出ない | **解消（適用範囲付き）** | `p4-api-headless-ui-1440-light.png` は issue 番号・ロードマップ節を含まない利用者向け構成。ただし Phase 6 の適用対象は headless-ui-api / pre-styled-ui-api / pre-styled-recipe-api の 3 ページのみ（`docs/design/docs-site-api-reference-split.md` §3-7）。残り 6 ページ（`component-api` 等）は将来トリガー付きで意図的に据え置きであり、本レポートは範囲外ページを撮影対象にしていない |
| D | ヘッダーと 3 カラム grid の左端が揃う | **解消** | `p1-top-1440-light.png` でブランドリンク（`fandhe-frontend`）左端とサイドバー左端の x 座標が一致 |
| E | 右目次に見出しと現在地ハイライト | **解消** | `p1-top-1440-light.png` / `-dark.png` の `ON THIS PAGE` 見出しとアクティブ項目のアクセントバーを確認 |
| F | テーブルに横スクロールのアフォーダンス | **解消** | `f-table-dialog-375-tall.png` / `f-table-api-1440-tall.png` で Arguments / Data Attributes 表の列が viewport 幅で切れており、水平スクロールを要する構造であることを確認（`--hide-scrollbars` は使用していない） |
| G | ダークトグル・GitHub リンク・検索があり、ダークが機能 | **解消** | `p1-top-1440-light.png`（Dark ラベル）と `p1-top-1440-dark.png`（Light ラベル・全面ダーク配色）で往復を確認。GitHub リンク・検索ボックスも両テーマで表示 |

判定は「解消」「解消（適用範囲付き）」「未解消」の 3 値。本 PR で新たな未解消事項は
検出しなかった。既知の据え置き事項（C の範囲外 6 ページ、索引ページの陳腐化した
Phase 4 完了後の記述）は §8 に記録する。

ダークモードは `<html data-theme="dark">` の直接注入で撮影しており、これはテーマ
トグル（`crates/docs-site/src/script.rs`）がクリック時に設定するのと同一の属性
経路である。`prefers-color-scheme` によるシステム連動の経路は
`crates/docs-site/src/site_theme.rs` の `stylesheet_declares_color_scheme_light_dark`
等の既存機械テストが別途担保しており、本レポートのスコープではない。

## 6. JS 無効環境の検証

`n1-top-nojs-*.png` / `n2-button-nojs-375.png`（`Content-Security-Policy: script-src 'none'`
配信）で、検索ボックス・テーマトグルが非表示のまま、ヘッダーナビ・サイドバー・本文・
右目次を含む全レイアウトが破綻なく閲覧できることを確認した。これを一回性の目視で
終わらせず、`crates/docs-site/tests/no_js_contract.rs`（6 テスト）として恒久固定した。

| テスト名 | 検証内容 |
|---|---|
| `no_generated_page_uses_javascript_scheme_links` | `javascript:` スキームの href/src が全ページに存在しない |
| `no_generated_page_uses_inline_event_handler_attributes` | `onclick` 等のインラインイベントハンドラ属性が存在しない |
| `site_js_is_loaded_as_single_deferred_external_script` | `assets/site.js` が `defer` 付き外部 `<script src>` 1 本のみで読み込まれる（唯一の例外は FOUC 抑止インラインブートストラップ `INLINE_THEME_BOOTSTRAP`） |
| `search_block_and_theme_toggle_default_to_hidden` | `div.docs-search` / `.docs-theme-toggle` が既定 `hidden` |
| `sidebar_and_header_and_prev_next_navigation_uses_static_anchor_hrefs` | サイドバー・ヘッダーナビ・prev/next の各ブロックが静的 `<a href>` を持つ |
| `structural_css_declares_js_independent_toggle_and_dropdown_paths` | 生成 CSS が `:checked` によるサイドバー開閉・`:hover`/`:focus-within` によるヘッダードロップダウン開閉の経路を保持する |

## 7. 機械テスト結果

```
$ cargo test -p fandhe-frontend-docs-site --test no_js_contract
running 6 tests
test structural_css_declares_js_independent_toggle_and_dropdown_paths ... ok
test site_js_is_loaded_as_single_deferred_external_script ... ok
test sidebar_and_header_and_prev_next_navigation_uses_static_anchor_hrefs ... ok
test search_block_and_theme_toggle_default_to_hidden ... ok
test no_generated_page_uses_javascript_scheme_links ... ok
test no_generated_page_uses_inline_event_handler_attributes ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#922 が追加した回帰テスト 11 件（すべて `cargo test -p fandhe-frontend-docs-site` の
実行ログ内で `... ok` を確認済み）:

1. `site_theme::tests::every_referenced_fandhe_token_is_defined` — ok
2. `site_theme::tests::color_and_shadow_tokens_are_defined_in_all_three_mode_blocks` — ok
3. `site_theme::tests::stylesheet_declares_color_scheme_light_dark` — ok
4. `site_theme::tests::structural_and_typography_css_contain_no_hardcoded_colors` — ok
5. `site_theme::tests::stylesheet_base_breakpoint_matches_responsive_contract` — ok
6. `site_theme::tests::stylesheet_media_queries_are_ordered_mobile_first` — ok
7. `docs_page_emits_view_transition_opt_in_style_in_head` — ok
8. `docs_page_with_assets_emits_view_transition_opt_in_style_in_head` — ok
9. `docs_page_skip_nav_link_href_matches_content_target_id` — ok
10. `docs_page_skip_nav_link_is_first_focusable_element_in_body` — ok
11. `real_site_build_covers_all_page_kinds_with_shared_layout_contract` — ok

`cargo test --workspace`: **exit code 0**（crates.io 到達性を前提とする examples/app
テンプレートの e2e を含め全件 pass。到達性起因の failure は発生しなかった）。

追加ゲート:

```
$ cargo fmt --all --check         # 差分なし
$ cargo clippy -p fandhe-frontend-docs-site --all-targets -- -D warnings   # 警告ゼロ
```

## 8. 所見と残課題

- **索引ページの陳腐化した記述**: `site/components-pre-styled-ui.md:26-29` に
  「Demo 以外の節（Features / Anatomy / …）の充填は Phase 4（#945〜#948）で進めます」
  という、Phase 4 完了後は陳腐化した文言が残っている。本 PR は検証タスクのため原稿を
  書き換えていない。#961（本レポートの `docs-site-redesign-regression-report.md` への
  統合）または別 Issue での是正を提案する（ユーザー承認後に起票、
  `out-of-scope-tracking.md` 準拠）。
- **F 専用撮影のスクリプト未統合**: `f-table-*-tall.png` は
  `tools/docs-site/visual-regression.sh` 本体ではなく、大きい `--window-size` 高を
  手動指定するアドホックな `chromium` 呼び出しで取得した（通常撮影は viewport 高
  固定のためテーブルより手前で切れる）。恒久的な再現性のため、スクリプトへの統合
  （テーブルを含むページ専用の「tall window」撮影オプション追加）を将来の改善として
  記録する。
- **375px でのサイドバー折りたたみの表示崩れなし**: `max-height` + チェックボックス
  開閉（#916）は JS 無効でも到達可能であることを目視確認した。追加の不具合は
  検出しなかった。
- **C の適用範囲外 6 ページ**: `docs/design/docs-site-api-reference-split.md` §3-7 の
  意図的な据え置きであり、本レポートでは「未解消」ではなく設計判断として記録するに
  留めた。

## 9. 変更対象外（このレポートで触っていないもの）

- `docs/reports/docs-site-redesign-regression-report.md`（#961 の担当）
- 公開済みクレート（`crates/headless-ui/` / `crates/pre-styled-ui/` 等）の
  `src/` ・ `Cargo.toml`
