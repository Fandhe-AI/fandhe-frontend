# CLAUDE.md

## Overview

Rust 製フロントエンドフレームワーク。AI 時代のセキュリティリスク低減を目的に、プレーンな HTML / JavaScript / CSS を尊重しつつ SSR / SPA / SSG / トランジションなどモダン機能を網羅する。部分埋め込みの最小構成からフル機能構成までのグラデーションを持ち、単一実行ファイルでのデプロイ（Docker 想定）を目標とする。

- 正式名称は `fandhe-frontend`（確定、2026-07-19）。決定記録・新旧マッピング表は `docs/design/framework-naming.md` を参照。crate 名は #441 で `rws-*` から `fandhe-frontend-*` へ改名済み。リポジトリ名は #439 で `Fandhe-AI/fandhe-frontend` へ改名済み。全 9 クレート（fandhe-frontend-core / -interactive / -app / -server / -wasm-client / -wasm-full / -wasm-thin / -dist-server / -cli）は v0.1.0 で 2026-07-20 に crates.io へ公開済み。加えて `fandhe-frontend-headless-ui` / `fandhe-frontend-pre-styled-ui`（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成、親トラッキング #520 で新設）も v0.1.0 で crates.io へ公開済み（イシュー #608）
- 仕様書は [Fandhe-AI/fandhe-frontend-spec](https://github.com/Fandhe-AI/fandhe-frontend-spec) を `docs/spec/` サブモジュールとして取り込み管理
- 開発は `docs/spec/06-roadmap.md` のマイルストーン MS-1〜MS-5 に従う（最初のタスクは TASK-1.1: `fandhe-frontend-core` 既定エスケープの製品化）
- 計画クレート: `fandhe-frontend-core`（描画コア・外部依存ゼロ）/ `fandhe-frontend-app` / `fandhe-frontend-server`（SSR/SSG）/ `fandhe-frontend-wasm-client`・`fandhe-frontend-wasm-full`（WASM/CSR）/ `fandhe-frontend-interactive`（状態管理）/ `fandhe-frontend-headless-ui`（headless UI コンポーネント層、#520）/ `fandhe-frontend-pre-styled-ui`（pre-styled UI コンポーネント層、#520）/ `xtask`（CI 計測）/ `fandhe-frontend-cli`（`fw` コマンド・AI 自己保守フック、REQ-13）

## Repository Structure

```
fandhe-frontend/
├── CLAUDE.md
├── README.md
├── .editorconfig             # エディタ間のインデント・改行統一（Rust は rustfmt 既定 4 スペースと一致させる補助）
├── rust-toolchain.toml       # channel = "stable" を単一真実源とする toolchain 宣言（イシュー #1273。CI 各ワークフローの Fandhe-AI/actions/rust-toolchain-setup が rustup show で同期）
├── .cargo/config.toml        # wasm32-unknown-unknown ターゲット限定の `opt-level = "s"`（REQ-11 gzip 200KB 上限対応、イシュー #1647。native ビルドへは波及させない設計、`docs/ci/wasm-opt-adoption-evaluation.md` 追記節参照）。`Dockerfile` は明示 `COPY .cargo ./.cargo` で同梱する
├── Makefile                  # 開発タスク（setup / build / test / fmt / lint / gate / bench / bench-cross / docs / docker-dev-build / docker-dev）の入口
├── lefthook.yml              # pre-commit / commit-msg フック定義（npm 非依存）。lefthook 導入済み環境で `make setup` が有効化、ローカル検証用
├── skills-lock.json          # npx skills add の導入記録
├── bench/                    # フレームワーク横断ベンチハーネス v2（正は bench/PROTOCOL.md。旧 _/bench/ が git 管理外で喪失した教訓から git 管理下に再構築。SSR 8 種 / CSR 7 種 / payload。npm 依存は --ignore-scripts + lockfile 固定、CI 非常設〔REQ-12、PROTOCOL §5〕、実行入口は make bench-cross）
│   ├── PROTOCOL.md           # 比較対象リスト・ワークロード定義・実行手順・公平性注記の正
│   ├── ssr/                  # SSR render-to-string 比較（xtask bench-ssr と同一ワークロード。fandhe は xtask、他 7 種は run_ssr.mjs）
│   ├── csr/                  # CSR create/update/clear 比較（playwright-core + システム chromium。fandhe は csr/fandhe/ の wasm アプリ、workspace path 依存）
│   └── payload/              # 配布物 gzip サイズ計測（measure.mjs）
├── docs/
│   ├── design/               # 設計文書（gate-design / wasm-full-architecture / structure-manifest / docs-site-three-column-redesign / docs-site-component-pages〔部品ページ IA、#938〕 / docs-site-api-reference-split〔利用者向け API と内部設計記録の分離基準、#952〕 / docs-site-search-design〔全文検索、#956〕 / docs-site-primitives-themes-split〔Primitives / Themes 2 層分割、#1037〕 / keyed-insert-template-clone-design〔連続 Insert の行プロトタイプ cloneNode + 束縛点書き込み一括生成、#1385〕 / color-token-system〔色トークン体系、#1422〕 / pre-styled-ui-scale-tokens〔radius / shadow / spacing / z-index トークンのスケール整備、#1423〕 / pre-styled-ui-focus-ring-and-size-conventions〔フォーカスリング実装手段・size バリアント名称/既定/保有判定基準の共通規約、#1424〕 / pre-styled-ui-size-and-color-palette-axes〔size / ColorPalette 軸の段階数・名称決定とトークン・列挙型定義、#1678〕 / pre-styled-ui-interaction-visual-language〔hover / disabled / transition の共通ビジュアル言語、#1425〕 / wasm-full-feature-gating-evaluation〔headless-ui / wasm-full の部品別 feature gating によるサイズ削減の導入評価、#1973〕 / shadcn-reference-adoption-policy〔shadcn/ui を 4 本目の参照軸として位置づけ、既存部品への適用原則（補完参照・判定基準）を決定、#2003〕 等）。component-coverage-map.md は ark-ui / chakra-ui に Radix UI を加えた 3 参照軸のコンポーネント対応表の正（イシュー #937。Radix 側の一次記録は radix-primitives-inventory.md / radix-themes-survey.md）。reference-screenshots/ は参考サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes）と本リポジトリ docs サイトの部品スクリーンショット置き場であり、命名規約・再取得手順・出典管理・サイズ方針は同ディレクトリの README.md を正とする（イシュー #1428）
│   ├── api/                  # API 仕様（component-api / hydration-api / hydration-state-format 等）
│   ├── guides/               # 利用者向けガイド（embedding-guide / npm-asset-build / browser-testing 等）
│   ├── policy/               # 規約・セキュリティポリシー（unsafe-boundary / dependency-graph-policy / cargo-deny-advisories / intentional-non-adoption 等）
│   ├── ci/                   # CI・runner 運用（ci-runner-requirements〔旧 self-hosted 方針時代の常設要件・実測記録、イシュー #1238 で位置づけ変更〕 / perf-browser-harness / cargo-semver-checks-evaluation / version-bump-publish-order-gap / a11y-automation-evaluation / docs-site-interaction-testing-evaluation〔docs サイト対話操作検証手段の導入評価、#1084〕 / example-overlay-browser-interaction-testing-evaluation〔example オーバーレイのブラウザ実インタラクションテスト常設 CI 化評価、#1210〕 / aarch64-docker-wasm-rebuild-ci-evaluation〔aarch64 self-hosted runner による Docker WASM 再ビルド検証の CI 常設化評価、#1216。ホステッド移行〔#1220〕で前提変化の追記あり、#1238〕 / hosted-runner-migration〔ホステッドランナー移行設計、#1225〕 / actions-new-feature-adoption-evaluation〔Fandhe-AI/actions 新規 15 コミット分の機能の採用可否評価、#1288〕 / wasm-opt-adoption-evaluation〔クライアント payload 削減のための wasm-opt（binaryen）導入評価・ビルドプロファイル実測比較、#1327。#1388 の panic・fmt 縮減記録、#1969 の dist-server 経路 wasm-opt 実測・twiggy 内訳記録、#1972 の CI・Dockerfile への binaryen 実導入見送り決定記録を含む〕 / wasm-allocator-adoption-evaluation〔wasm 向けアロケータ差し替え（dlmalloc → lol_alloc/talc）の導入評価、#1389。payload 削減は実測できたが update 経路の op_ms 悪化・unsafe 境界・保守状況のいずれかで両候補とも見送り。#1408 の再評価（talc 条件付き充足）を経て #1412 で talc 採否のユーザー判断（系統 2〔`WasmArenaTalc`〕不採用確定・系統 1〔`WasmDynamicTalc`〕見送り継続）を追記〕 / style-refresh-version-bump-operation〔107 部品スタイル調整（イシュー #1420）における semver バンプ粒度・dep-version 追随・crates.io 公開タイミングの運用決定、#1429〕）
│   ├── reports/              # 実測・受け入れレポート（perf-browser-report / *-acceptance-report / docs-site-redesign-regression-report 等）
│   ├── internal/             # docs サイト（site/nav.toml）非掲載の内部設計記録（*-implementation-notes.md。docs/api/ から実装経緯・進行管理記述を分離、イシュー #953/#954。分離基準の正は docs/design/docs-site-api-reference-split.md。本リポジトリは public であり「サイト非掲載」は「非公開」を意味しない。`pre-styled-ui-golden-test-update-guide.md`〔`crates/pre-styled-ui/tests/*_css.rs` golden テストの更新手順・部品対応表、#1427〕も同ディレクトリに置く）
│   └── spec/                 # 仕様サブモジュール (fandhe-frontend-spec)
│       ├── 01-brainstorm.md
│       ├── 02-poc-plan.md
│       ├── 03-poc/           # PoC-1〜7 成果物（rendering-web-standards が中核）
│       ├── 04-requirements.md  # MoSCoW 要件・受け入れ基準
│       ├── 05-tasks.md         # タスク分解（依存・工数）
│       └── 06-roadmap.md       # MS-1〜MS-5・着手判定
├── site/                     # docs サイト原稿（crates/docs-site が SSG でビルド。site/assets/ は #905 で廃止済み、骨格 CSS はビルド生成）
│   ├── index.md
│   ├── guides.md               # Guides セクショントップページ原稿（配下 4 ガイドへの索引、イシュー #1009）
│   ├── api.md                  # API Reference セクショントップページ原稿（クレート別 10 ページ索引、イシュー #1009。イシュー #1156 で `docs/api/server-api.md`（`generate_assets` 等 SSG API）が加わり 9 → 10）
│   ├── themes.md                # Themes（`fandhe-frontend-pre-styled-ui`）セクショントップページ原稿（凡例 + カテゴリ別リンク集、イシュー #943。旧 `site/components-pre-styled-ui.md` をイシュー #1018 で `/themes/` へ移設・改称。Primitives（`/primitives/`）へのリンクをイシュー #1021 で追加）
│   ├── themes/                 # 部品ページ原稿（`/themes/<kebab>/` 1 ページ = 部品 1 件、イシュー #943。イシュー #1017 で `site/components/` から `site/themes/` へ移行、旧 URL は `site/redirects.toml` で互換維持。台帳は `docs/design/docs-site-component-pages.md` §3、登録の正は `site/nav.toml`、ページ数の期待値は `crates/docs-site/tests/site_nav.rs`）
│   ├── primitives.md            # Primitives（`fandhe-frontend-headless-ui`）セクショントップページ原稿（凡例 + カテゴリ別リンク集、イシュー #1021）
│   ├── primitives/              # 部品ページ原稿（`/primitives/<kebab>/` 1 ページ = 部品 1 件、イシュー #1021）。台帳は `crates/docs-site/src/primitives_catalog.rs`（イシュー #1020）、登録の正は `site/nav.toml`、三方突合は `crates/docs-site/tests/primitives_nav.rs`
│   ├── nav.toml               # ナビゲーション構成マニフェスト。`[[section]]` は全セクションで `index_path`（セクショントップページの出力 URL パス）が必須（イシュー #1010/#1038。パース時に当該セクション配下の実在 `page.path` との完全一致を検証、不一致は fail-closed）。セクション構成は宣言順（＝ヘッダー並び順）で Getting Started / Guides / Examples / Primitives / Themes / API Reference の 6 つ。Primitives（`/primitives/`、63 部品、`fandhe-frontend-headless-ui` 相当）と Themes（`/themes/`、110 部品、`fandhe-frontend-pre-styled-ui` 相当）の 2 層構成である
│   └── redirects.toml         # 旧 URL 互換のリダイレクト宣言（`[[redirect]]` の `from`/`to`、イシュー #1016）。`crates/docs-site/src/redirect.rs` がパース・`nav.toml` との突合検証を行い `meta refresh` 案内ページを生成する。`nav.toml` とは意図的に別ファイル（判断根拠は `docs/design/docs-site-primitives-themes-split.md` §4 と該当 PR 本文）。旧 URL `/components/*` 112 件（110 部品 + `/components/` + `/components/pre-styled-ui/`）が移転案内ページとして維持され、この生成機構は `nav.toml` 非登録・検索インデックス非掲載（そもそも収集経路に載らない設計であり、`search_index`/`linkcheck`/`nav.toml` ページ数契約への除外述語を持ち込まない）
├── examples/
│   ├── ssr-routing/          # SSR + ルーティング正本サンプル・examples 規約の初例（crates.io バージョン依存、イシュー #499）
│   ├── ssg-blog/             # SSG（generate_pages）による静的ブログ正本サンプル（crates.io バージョン依存、イシュー #501）
│   ├── dist-server-docker/  # 単一バイナリ配布 + Docker 正本サンプル（crates.io バージョン依存、イシュー #502）
│   ├── interactive-view-transitions/  # 状態管理（fandhe-frontend-interactive）+ View Transitions 正本サンプル（イシュー #503）
│   └── headless-pre-styled-ui/  # headless-ui / pre-styled-ui コンポーネントショーケース（crates.io バージョン依存、`fw new --example` 対応、イシュー #609）
├── docker/                     # コンテナ定義（製品配布用 `Dockerfile` とは別。開発ループ専用）
│   └── dev/                    # 開発用 Docker イメージ・compose 定義
│       ├── Dockerfile         # Rust toolchain + wasm32 + 開発ツール一式。`make docker-dev-build` で構築
│       └── compose.yml        # `make docker-dev-build` / `make docker-dev` で利用
├── tools/                      # CI・開発スクリプト
│   ├── ci/                    # CI 用ブートストラップ（ensure-gate-tools.sh）
│   └── hooks/                 # Git hooks スクリプト（lefthook 実行対象）
│       └── commit-msg-check.sh # Conventional Commits 形式検証（npm 依存なし、REQ-12 整合）
├── templates/
│   ├── default/
│   │   ├── deny.toml         # 標準プロジェクトテンプレート同梱の cargo-deny 設定（TASK-4.1 / REQ-4）
│   │   ├── structure.toml    # fw gate が唯一の情報源として読む構造マニフェスト（生成直後 fw gate PASS 保証、イシュー #351）
│   │   ├── tools/
│   │   │   └── npm-asset-build/  # NPM 静的アセットゲートの同梱コピー（正本は tools/npm-asset-build/、イシュー #316）
│   │   └── .github/
│   │       └── workflows/
│   │           ├── deny.yml            # 禁止クレート追加を CI でブロックするテンプレートワークフロー（TASK-4.2 / REQ-4）
│   │           └── npm-asset-gate.yml  # NPM 静的アセットゲート（install.sh 経由）のテンプレートワークフロー（REQ-12, イシュー #316）
│   ├── app/                   # `fw new --template app`（イシュー #378/#411）: fandhe-frontend-core/fandhe-frontend-app 依存の拡充テンプレート（wasm ビルド込み CSR 完全実体を同梱）
│   │   ├── src/main.rs       # Loader・束縛点 API（bind_text/keyed_list）・render の実体サンプル
│   │   ├── wasm/              # CSR wasm ビルド用の独立ワークスペース（glue クレート app-csr-wasm、root の依存グラフから隔離）
│   │   ├── tools/wasm/build.sh # wasm ビルド手順（wasm-bindgen-cli バージョン整合の fail-closed 検証込み）
│   │   └── static/embed.html # CSR マウント骨格（templates/embed/embed.html の同梱コピー、build.sh 実行後に動作）
│   └── embed/                 # `fw new --template embed`（イシュー #410）: 静的単一ファイルの部分埋め込み構成（cargo パッケージなし）
│       ├── embed.html        # TASK-7.1a（#52）正本（templates/app/static/embed.html と同一）
│       └── structure.toml    # fw gate 静的専用（asset-only）モードの明示宣言（role = "asset" のみ、crate キーなし）
└── .claude/
    ├── agents/               # カテゴリ別 sub-agent 定義
    ├── rules/                # 委譲・コーディング・セキュリティ規約
    ├── skills/               # npx skills add 導入スキル
    └── settings.json         # SessionStart / PostToolUse hooks
```

全メンバークレートは `crates/` 配下に配置する（イシュー #436）:

```
crates/
├── core/          # fandhe-frontend-core: 描画コア・外部依存ゼロ
├── interactive/   # fandhe-frontend-interactive: 状態管理コア
├── app/           # fandhe-frontend-app: モード非依存の共通コンポーネント
├── server/        # fandhe-frontend-server: SSR/SSG エントリ
├── wasm-client/   # fandhe-frontend-wasm-client: クライアントランタイム基盤
├── wasm-full/     # fandhe-frontend-wasm-full: CSR/ハイドレーション フルセット
├── wasm-thin/     # fandhe-frontend-wasm-thin: CSR/ハイドレーション 最小構成
├── dist-server/   # fandhe-frontend-dist-server: 単一実行ファイル配布サーバー
├── headless-ui/   # fandhe-frontend-headless-ui: headless UI コンポーネント層（anatomy・data-*・WAI-ARIA、イシュー #520/#522）
├── pre-styled-ui/ # fandhe-frontend-pre-styled-ui: pre-styled UI コンポーネント層（headless-ui 上層のスタイル済み部品、イシュー #520/#546）
├── docs-site/     # fandhe-frontend-docs-site: docs サイトジェネレータ（外部クレート依存ゼロ・内部 path 依存のみ〔core/app/server/pre-styled-ui〕・配布物に含めない開発者/CI 用ツール）。サイト骨格 CSS を `fandhe-frontend-pre-styled-ui` の `Theme::to_css` から生成する（イシュー #899 で 3 カラム刷新、詳細は `docs/design/docs-site-three-column-redesign.md`）。ヘッダーのセクション別ドロップダウンは `src/nav.rs` の `header_nav` が生成するが、`fandhe-frontend-pre-styled-ui` の `menu` は意図的に不採用（素の `nav`/`ul`/`li`/`a` + CSS の `:hover`/`:focus-within` のみで構成し、`role`/`aria-expanded`/`aria-haspopup` を付与しない）。根拠は (1) WAI-ARIA `menu` ロールは操作コマンドリスト向けであり文書リンク集ナビへの転用はスクリーンリーダー利用者へ「操作可能なメニュー」と誤って伝える意味論不整合、(2) `menu` の `data-state` 開閉が wasm-full の JS 配線（hydration）前提であり JS ハイドレーションを行わない docs-site では動作しない無 JS 制約、の 2 点（詳細は `src/nav.rs` の `header_nav` rustdoc「イシュータイトルとの差分」節、`docs/design/docs-site-three-column-redesign.md` §3.5）。#919 のコミットタイトル「pre-styled-ui menu によるドロップダウン付きメニュー」は実装のこの不採用判断と食い違うため、読み替えの根拠として本記述を参照する。サイドバー（`src/nav.rs::sidebar`）は現在ページの属するセクション 1 件へスコープ限定して描画する（イシュー #1013/#1042。`Nav::section_for_path` が唯一の解決経路。nav 中のどの `page.path` にも一致しないときは全セクション描画へフォールバックする意図的な fail-open であり、公開静的サイトのためサイドバー可視性はアクセス境界ではない）。加えて、サイト骨格 CSS に加え、素の JS 単一ファイル（`assets/site.js`、`src/script.rs`）をビルド時生成し、`<head>` の FOUC 抑止インラインスニペットとあわせてヘッダーのテーマトグル・GitHub リンクを実装する（イシュー #951）。全文検索インデックス（`assets/search-index.json`）は `src/search_index.rs` がビルド時に生成する（イシュー #957）。フェンスコードブロックの軽量シンタックスハイライトは `src/highlight.rs`（外部依存ゼロ・JS ハイドレーションなし。`crate::markdown::parse_fence` から呼ばれ、トークンを `<span class="token-*">` で包んで `crate::text` を経由して出力する。全域性不変条件〔連結すると常に入力と一致する〕により REQ-1 の既定エスケープが構造的に保たれる、イシュー #1078）が担う。`src/search_index.rs::collect_text_into` はこの `token-*` span を語境界の空白挿入から除外する（span 化前と同じ連結結果を保つため。キーワード・リテラルに隣接する語句が索引上で分断されない不変条件）。部品ページ（`/themes/<kebab>/`。イシュー #1017 で `/components/<kebab>/` から移行）は `src/component_page.rs`（雛形レンダラ・`SPEC_TABLES` レジストリ、イシュー #942）が Demo〔`src/showcase.rs`〕・Anatomy・`data-*` 属性表・CSS 変数表（機械導出）と原稿データを合成して生成する。原稿データ（Features / API Reference 引数表 / Examples / Accessibility）の供給元は `src/component_specs/`（カテゴリ別サブモジュール。Forms は #945）と、イシュー単位のフラットな原稿モジュール `src/component_specs_overlay.rs`（#946）/ `src/component_specs_nav_data.rs`（#947）/ `src/component_page_specs_948.rs`（#948）に分かれる。旧 URL 互換のリダイレクトページ生成機構（`site/redirects.toml`、イシュー #1016）は `src/redirect.rs` が担い、`nav.toml` とは独立した宣言ファイルから `meta refresh` + `rel=canonical` のクロームなし案内ページを生成する（`search_index`/`linkcheck`/`nav.toml` ページ数契約への除外述語を持ち込まない設計）。Primitives 台帳（`src/primitives_catalog.rs`、63 部品 + 基盤 9 件。headless-ui ソースとのドリフトは `tests/primitives_catalog.rs` が fail-closed に検知、イシュー #1020）。Primitives（63 部品）と Themes（110 部品）の層をまたぐラップ状態（同名/別名委譲・rustdoc 言及のみ・独自実装の 4 バケット分割）は `tests/wrap_state.rs` が fail-closed に機械可視化する（イシュー #1064、判別規約は `docs/design/docs-site-primitives-themes-split.md` §6a）。`component_page.rs` はイシュー #1021 で `Layer`（Themes/Primitives）による層パラメータ化を経ており、Primitives 層（`/primitives/<kebab>/`）は CSS 変数表を恒常的に省略し Demo ラッパ class を `primitives-showcase` に切り替える。Primitives 63 部品の Demo（Anatomy・`data-*` 属性表の機械導出元）は `src/primitive_showcase/`（カテゴリ別 6 submodule: `forms_a` / `forms_b` / `forms_c_date_status` / `overlay_disclosure` / `navigation` / `data_display_utilities`、イシュー #1022）が headless-ui のパート関数のみで供給し、専用 CSS（`assets/primitives-showcase.css`）は headless-ui のマークアップへスタイルを到達させない（`[data-scope=`/`[data-part=` セレクタを持たない）デモ枠中和のみを担う。原稿レジストリ `src/primitive_specs/` は Phase 5（#1024〜#1029）完了により 6 カテゴリ別サブモジュール（`forms_a` / `forms_b` / `forms_c_date_status` / `overlay_disclosure` / `navigation` / `data_display_utilities`）が 63 部品分の原稿（Features / API Reference 引数表 / Examples / Accessibility の 4 節）を供給し、`SPEC_TABLES` へ集約している。`< 1200px` で右目次カラム（`aside.docs-toc-aside`）が非表示になる代替として、`src/layout.rs` の `toc_inline()` が本文冒頭に素の `<details>`/`<summary>` 折りたたみ目次（`nav.docs-toc-inline`、`>= 1200px` は `site_theme.rs` 側で非表示）を出力する（イシュー #1080）。`class="docs-toc"`（`src/script.rs` のスクロールスパイが `document.querySelector` で掴む唯一のセレクタ）は共有しない不変条件が `crates/docs-site/tests/layout_render.rs` で機械固定されている
├── cli/           # fandhe-frontend-cli: `fw` コマンド（structure.toml のスキーマ・パース・生成、REQ-13）
│   ├── templates/          # `fw new --template` 埋め込み用の同梱コピー（正本はルート `templates/`。`new_template.rs` が `include_str!` で吸収、乖離は `tests/template_publish_copy_drift.rs` が検知）
│   └── embedded-examples/  # `fw new --example` 埋め込み用の同梱コピー（正本はルート `examples/`。パッケージ名は置換せず正本と全ファイルバイト一致、乖離は `tests/example_publish_copy_drift.rs` が検知、イシュー #500）
└── xtask/         # CI 計測用の開発者ツール
```

ルート `Cargo.toml` は `members = ["crates/*"]`（glob）。リポジトリ自身の
`structure.toml` は各 `[directories.<name>]` に `path = "crates/<name>"` を
宣言し、依存宣言の論理名（`<name>`）とは独立して実配置を表す
（`docs/design/structure-manifest.md` §2.2.0a 参照）。`fw new` が生成する
ユーザープロジェクト（`templates/`）は `path` を使わないフラット配置のまま
不変。

## 委譲方針（必読）

main セッションは**指揮・統合・ユーザー対話に専念**し、調査・実装・テスト・レビューは sub-agent へ委譲して main のコンテキスト消費を抑える。詳細は `.claude/rules/delegation.md`（調査・設計）と `.claude/rules/delegation-impl.md`（作成・編集）を参照。

### パスベース切り替え表

| 対象パス | 委譲先 Agent |
|---------|-------------|
| `crates/core/` `crates/interactive/` | core-builder |
| `crates/headless-ui/` `crates/pre-styled-ui/` | core-builder |
| `crates/app/` `crates/server/` | server-builder |
| `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `static/` | wasm-builder |
| `crates/xtask/` `crates/cli/` `.github/` `Dockerfile` `deny.toml` `templates/` | tooling-builder |
| `docs/`（spec 以外）・CLAUDE.md | docs-writer |
| `docs/spec/`（読み取り調査） | explorer |
| テスト実行・失敗分析 | test-runner |
| レビュー | reviewer / security-auditor |

### model 配分表

| 用途 | model |
|------|-------|
| 複雑な横断判断・アーキテクチャ設計 | opus または fable（fable は特に大規模設計・横断判断の最上位 tier） |
| 調査・生成・実装・レビュー | sonnet |
| 機械的集計・lint・ドキュメント更新 | haiku |

## Sub-agents

`.claude/agents/<category>/<name>.md` に定義。

| カテゴリ | subagent_type | model | 役割 |
|---------|---------------|-------|------|
| research | explorer | sonnet | コードベース・`docs/spec/` 横断調査（読み取り専用） |
| research | reference-researcher | sonnet | 外部仕様（Rust / WASM / Web 標準 / 依存クレート）調査 |
| implement | core-builder | sonnet | `crates/core/` `crates/interactive/` — 描画・状態管理コア（`forbid(unsafe_code)` 域） |
| implement | server-builder | sonnet | `crates/app/` `crates/server/` — SSR / SSG / ルーティング |
| implement | wasm-builder | sonnet | `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `static/` — CSR / ハイドレーション / WASM |
| implement | tooling-builder | sonnet | `crates/xtask/` / CI / Dockerfile / cargo-deny / 単一バイナリ配布 / AI 自己保守フック |
| testing | test-runner | sonnet | `cargo test` / XSS 回帰 / wasm テストの実行と失敗分析 |
| quality | reviewer | sonnet | 仕様準拠・アーキテクチャ整合・Rust イディオムのレビュー |
| quality | security-auditor | sonnet | OWASP・XSS エスケープ保証・`unsafe` 境界・依存監査 |
| quality | linter | haiku | rustfmt / clippy / frontmatter の機械的チェック |
| docs | docs-writer | haiku | README / CLAUDE.md / docs/（spec 除く）の更新 |

## Rules

`.claude/rules/` に定義。

| ファイル | 内容 |
|---------|------|
| `delegation.md` | 調査・設計フェーズの委譲原則・パスベース切り替え |
| `delegation-impl.md` | 作成・編集フェーズの委譲マッピング |
| `coding-rust.md` | Rust 規約（既定エスケープ厳守・`forbid(unsafe_code)`・依存上限 60 件/深さ 6・core 外部依存ゼロ） |
| `security.md` | OWASP Top 10・秘密情報混入防止・サプライチェーン対策 |
| `japanese-style.md` | 日本語出力スタイル |
| `conventional-commits.md` | Conventional Commits 詳細規約（scope 一覧含む） |
| `code-comment-style.md` | コメント規約（役割・責務・呼び出し文脈・`// SAFETY:` を埋め込む） |
| `out-of-scope-tracking.md` | 実装対象外の追跡規約（スコープ外事項を Issue 化して放置しない） |
| `ci.md` | CI 規約（GitHub ホステッドランナー既定・`runs-on` は `ubuntu-latest` 単一・codex-review の codex ジョブのみ self-hosted 例外・共有 CARGO_TARGET_DIR 対策・ツール前提の明示） |

## Current Skills

`npx skills add Fandhe-AI/agent-cli-skills` で導入（`skills-lock.json` で追跡）。

- **コミット・PR**: create-commit / create-pr / implement-review / implement-review-pr
- **Issue**: create-issue / create-issue-tree / update-issue-tree / implement-issue / implement-issue-tree
- **計画・ドキュメント**: create-plan / update-docs / comment-code
- **Project v2**: project-init / project-add-items / project-create-issues / project-update-items / project-view-status / project-sync-issues / project-archive-done
- **.claude 体系**: init-claude / update-claude / sync-skills-lock / contribute-skill / update-reference
- **リファレンス**: rust / github-docs / commitlint / lefthook / editorconfig

## Conventions

- **日本語**: やりとり・ドキュメント・コミット/PR 本文は日本語（`japanese-style.md`）
- **Conventional Commits**: create-commit スキルを使用。`--no-verify` 禁止（`conventional-commits.md`）。commit-msg フック（`tools/hooks/commit-msg-check.sh`）でローカル検証が自動実行される
- **ローカル hooks**: lefthook による pre-commit / commit-msg フック（`make setup` で導入、`--no-verify` 禁止は従来どおり）
- **開発タスク**: `make help` で全ターゲット一覧が見られる（setup / build / test / fmt / lint / gate / bench / bench-cross / docs / docker-dev-build / docker-dev）
- **セキュリティレビュー**: コミット・PR 前に security-auditor による OWASP チェック必須（`security.md`）
- **ユーザー承認フロー**: 実装は計画承認後（implement-issue）。依存クレート追加・Issue 起票は事前承認必須
- **`docs/spec/` は編集禁止**: サブモジュール。仕様変更は fandhe-frontend-spec リポジトリで行う
- **スコープ外事項**: 放置せず Issue 化を提案（`out-of-scope-tracking.md`）

## hooks（settings.json）

- **SessionStart**: 日本語・委譲・Conventional Commits・`--no-verify` 禁止・core 厳守事項のリマインダーを表示
- **PostToolUse**（Edit|Write）: `.rs` ファイル編集後に `rustfmt` で自動整形（jq / rustfmt 不在時はスキップ）
