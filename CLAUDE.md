# CLAUDE.md

## Overview

Rust 製フロントエンドフレームワーク。AI 時代のセキュリティリスク低減を目的に、プレーンな HTML / JavaScript / CSS を尊重しつつ SSR / SPA / SSG / トランジションなどモダン機能を網羅する。部分埋め込みの最小構成からフル機能構成までのグラデーションを持ち、単一実行ファイルでのデプロイ（Docker 想定）を目標とする。

- 正式名称は `fandhe-frontend`（確定、2026-07-19）。決定記録・新旧マッピング表は `docs/design/framework-naming.md` を参照。crate 名は `fandhe-frontend-*`（アニメーション演算基幹のみ `fandhe-animation`）、リポジトリは `Fandhe-AI/fandhe-frontend`。`publish = false` の docs-site・xtask を除く全 14 クレートが crates.io 公開済みで、実体変更時の semver バンプは `version-bump-guard` が強制する（`.claude/rules/coding-rust.md`）。新規クレートの CI 組み込み・初回公開は `docs/ci/version-bump-publish-order-gap.md` §11 の手順に従う
- 仕様書は [Fandhe-AI/fandhe-frontend-spec](https://github.com/Fandhe-AI/fandhe-frontend-spec) を `docs/spec/` サブモジュールとして取り込み管理
- 開発は `docs/spec/06-roadmap.md` のマイルストーン MS-1〜MS-5 に従う（最初のタスクは TASK-1.1: `fandhe-frontend-core` 既定エスケープの製品化）
- 計画クレート: `fandhe-frontend-core`（描画コア・外部依存ゼロ）/ `fandhe-frontend-app` / `fandhe-frontend-server`（SSR/SSG）/ `fandhe-frontend-wasm-client`・`fandhe-frontend-wasm-full`（WASM/CSR）/ `fandhe-frontend-interactive`（状態管理）/ `fandhe-frontend-headless-ui`（headless UI コンポーネント層、#520）/ `fandhe-frontend-pre-styled-ui`（pre-styled UI コンポーネント層、#520）/ `fandhe-animation`（プラットフォーム非依存のアニメーション演算基幹。外部依存ゼロ・`forbid(unsafe_code)`、将来別リポジトリへ切り出し前提、#2365/#2371）/ `fandhe-frontend-animation`（Web アダプタ。`fandhe-animation` + wasm-bindgen / web-sys / js-sys のみに依存し `fandhe-frontend-wasm-full` には依存しない、#2365/#2417）/ `fandhe-frontend-wireframe-ui`（blocks.pm 参照のローファイ・モノクロワイヤーフレーム UI コンポーネント層。headless-ui/pre-styled-ui とは独立した第 3 の UI 層、SSR 専用・非インタラクティブ、#2599/#2600/#2603）/ `xtask`（CI 計測）/ `fandhe-frontend-cli`（`fw` コマンド・AI 自己保守フック、REQ-13）

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
│   ├── design/               # 設計文書（アーキテクチャ・参照方針・docs サイト設計・部品規約の決定記録）。参照方針の正は shadcn-reference-adoption-policy（chakra-ui / Radix Themes / shadcn-ui の 3 者を主基準、競合は部品ごと判断、golden 純追加、headless は ark-ui 維持）と motion-reference-adoption-policy。コンポーネント対応表の正は component-coverage-map.md（ark-ui / chakra-ui / Radix UI / shadcn/ui の 4 参照軸）。reference-screenshots/ の命名・再取得手順・出典管理は同ディレクトリの README.md を正とする
│   ├── api/                  # API 仕様（component-api / hydration-api / hydration-state-format 等）
│   ├── guides/               # 利用者向けガイド（embedding-guide / npm-asset-build / browser-testing / wasm-full-features / pre-styled-ui-motion-feature〔pre-styled-ui `motion` feature（既定 off）の有効化手順・無効時ゼロコスト保証、#2416〕等）
│   ├── policy/               # 規約・セキュリティポリシー（unsafe-boundary / dependency-graph-policy / cargo-deny-advisories / intentional-non-adoption 等）
│   ├── ci/                   # CI・runner 運用の評価と決定記録（hosted-runner-migration / version-bump-publish-order-gap〔§11 新規クレートの CI 組み込み・初回公開チェックリスト〕/ wasm-opt・アロケータ・cargo-semver-checks 等の導入評価 / browser-test-duration-regression-analysis 等）。各評価の結論と再評価トリガーは各文書を正とする
│   ├── reports/              # 実測・受け入れレポート（perf-browser-report / *-acceptance-report / docs-site-redesign-regression-report 等）
│   ├── internal/             # docs サイト（site/nav.toml）非掲載の内部設計記録（*-implementation-notes.md。docs/api/ から実装経緯・進行管理記述を分離、イシュー #953/#954。分離基準の正は docs/design/docs-site-api-reference-split.md。本リポジトリは public であり「サイト非掲載」は「非公開」を意味しない。`pre-styled-ui-golden-test-update-guide.md`〔`crates/pre-styled-ui/tests/*_css.rs` golden テストの更新手順・部品対応表、#1427〕・`wireframe-ui-golden-test-update-guide.md`〔`crates/wireframe-ui/tests/golden_css.rs` + `tests/golden/`（値ベース全単射検証、ソーステキスト走査は不採用）の golden テスト更新手順・49 部品対応表（全件整備済み）、#2666〕も同ディレクトリに置く）
│   └── spec/                 # 仕様サブモジュール (fandhe-frontend-spec)
│       ├── 01-brainstorm.md
│       ├── 02-poc-plan.md
│       ├── 03-poc/           # PoC-1〜7 成果物（rendering-web-standards が中核）
│       ├── 04-requirements.md  # MoSCoW 要件・受け入れ基準
│       ├── 05-tasks.md         # タスク分解（依存・工数）
│       └── 06-roadmap.md       # MS-1〜MS-5・着手判定
├── site/                     # docs サイト原稿（crates/docs-site が SSG でビルド。site/assets/ は #905 で廃止済み、骨格 CSS はビルド生成）
│   ├── index.md
│   ├── guides.md               # Guides セクショントップページ原稿（配下 6 ガイドへの索引、イシュー #1009）
│   ├── api.md                  # API Reference セクショントップページ原稿（クレート別 10 ページ索引、イシュー #1009。イシュー #1156 で `docs/api/server-api.md`（`generate_assets` 等 SSG API）が加わり 9 → 10）
│   ├── themes.md                # Themes（`fandhe-frontend-pre-styled-ui`）セクショントップページ原稿（凡例 + カテゴリ別リンク集、イシュー #943。旧 `site/components-pre-styled-ui.md` をイシュー #1018 で `/themes/` へ移設・改称。Primitives（`/primitives/`）へのリンクをイシュー #1021 で追加）
│   ├── themes/                 # 部品ページ原稿（`/themes/<kebab>/` 1 ページ = 部品 1 件、イシュー #943。イシュー #1017 で `site/components/` から `site/themes/` へ移行、旧 URL は `site/redirects.toml` で互換維持。台帳は `docs/design/docs-site-component-pages.md` §3、登録の正は `site/nav.toml`、ページ数の期待値は `crates/docs-site/tests/site_nav.rs`）
│   ├── primitives.md            # Primitives（`fandhe-frontend-headless-ui`）セクショントップページ原稿（凡例 + カテゴリ別リンク集、イシュー #1021）
│   ├── primitives/              # 部品ページ原稿（`/primitives/<kebab>/` 1 ページ = 部品 1 件、イシュー #1021）。台帳は `crates/docs-site/src/primitives_catalog.rs`（イシュー #1020）、登録の正は `site/nav.toml`、三方突合は `crates/docs-site/tests/primitives_nav.rs`
│   ├── blocks.md                # Blocks セクショントップページ原稿（shadcn/ui Blocks 相当の既存部品合成例、イシュー #2088。イシュー #2733 でカテゴリ属性を導入し、手書き「掲載済み」箇条書きを撤去してイントロ文のみへ縮小。索引本文（区分 → カテゴリの階層見出し）は `crate::blocks::all_blocks()` レジストリからビルド時生成する）
│   ├── blocks/                  # block ページ原稿（`/blocks/<kebab>/` 1 ページ = block 1 件、イシュー #2088）。レジストリは `crates/docs-site/src/blocks/`（トップレベル `mod.rs` の `all_blocks()` が 4 区分〔marketing/application/ecommerce/docs〕サブモジュールの `blocks()` を連結する構成へイシュー #2734 でカテゴリ別モジュール分割済み、`docs/design/docs-site-blocks-section.md` §18 参照）、登録の正は `site/nav.toml`、三方突合は `crates/docs-site/tests/blocks_nav.rs`。各原稿は手書き「Rust コード」節（rust 言語フェンス）を持ち、対応する `crates/docs-site/src/blocks/<section>/<category>/<snake>.rs` の `// blocks-code:begin`/`:end` マーカー内容との一致を `crates/docs-site/tests/blocks_code_drift.rs` が固定する。block 追加時に `CLAUDE.md`・`.claude/rules/ci.md` を編集する追記は行わない（イシュー #2736。内容の正は本ディレクトリの原稿と上記レジストリ、設計判断の索引は `docs/design/docs-site-blocks-section.md` §19）
│   ├── wireframes.md            # Wireframes セクショントップページ原稿（`fandhe-frontend-wireframe-ui` の部品ページ索引、イシュー #2607。全 49 部品を掲載済み）
│   ├── wireframes/              # 部品ページ原稿（`/wireframes/<kebab>/` 1 ページ = 部品 1 件、イシュー #2607）。レジストリは `crates/docs-site/src/wireframes/mod.rs`、登録の正は `site/nav.toml`、三方突合は `crates/docs-site/tests/wireframes_nav.rs`
│   ├── nav.toml               # ナビゲーション構成マニフェスト。`[[section]]` は全セクションで `index_path`（セクショントップページの出力 URL パス）が必須（イシュー #1010/#1038。パース時に当該セクション配下の実在 `page.path` との完全一致を検証、不一致は fail-closed）。セクション構成は宣言順（＝ヘッダー並び順）で Getting Started / Guides / Examples / Primitives / Themes / Blocks / Wireframes / API Reference の 8 つ（イシュー #2088 で Blocks が Themes の直後・API Reference の直前へ新設。イシュー #2607 で Wireframes が Blocks の直後・API Reference の直前へ新設）。Primitives（`/primitives/`、75 部品、`fandhe-frontend-headless-ui` 相当）・Themes（`/themes/`、123 部品、`fandhe-frontend-pre-styled-ui` 相当）・Blocks（`/blocks/`、既存部品の合成例。新規部品は追加しない）・Wireframes（`/wireframes/`、`fandhe-frontend-wireframe-ui` 相当。独立した第 3 の UI 層、SSR 専用・非インタラクティブ）の 4 層構成である
│   └── redirects.toml         # 旧 URL 互換のリダイレクト宣言（`[[redirect]]` の `from`/`to`、イシュー #1016）。`crates/docs-site/src/redirect.rs` がパース・`nav.toml` との突合検証を行い `meta refresh` 案内ページを生成する。`nav.toml` とは意図的に別ファイル（判断根拠は `docs/design/docs-site-primitives-themes-split.md` §4 と該当 PR 本文）。旧 URL `/components/*` 125 件（123 部品 + `/components/` + `/components/pre-styled-ui/`）が移転案内ページとして維持され、この生成機構は `nav.toml` 非登録・検索インデックス非掲載（そもそも収集経路に載らない設計であり、`search_index`/`linkcheck`/`nav.toml` ページ数契約への除外述語を持ち込まない）
├── examples/
│   ├── ssr-routing/          # SSR + ルーティング正本サンプル・examples 規約の初例（crates.io バージョン依存、イシュー #499）
│   ├── ssg-blog/             # SSG（generate_pages）による静的ブログ正本サンプル（crates.io バージョン依存、イシュー #501）
│   ├── dist-server-docker/  # 単一バイナリ配布 + Docker 正本サンプル（crates.io バージョン依存、イシュー #502。外部依存のため build.rs の WASM ビルドステージは自動スキップされ WASM は出荷されない。dist-server のワークスペース内ビルド〔ルート Dockerfile 等〕限定の最小インタラクティブ構成は #2329/#2330）
│   ├── interactive-view-transitions/  # 状態管理（fandhe-frontend-interactive）+ View Transitions 正本サンプル（イシュー #503。wasm-full feature 実指定・Phase 4 アニメーション配線の実演、#2330/#2525）
│   ├── headless-pre-styled-ui/  # headless-ui / pre-styled-ui コンポーネントショーケース（crates.io バージョン依存、`fw new --example` 対応、イシュー #609）
│   └── wireframe-ui/          # wireframe-ui Phase 1〜8・全 49 部品ショーケース正本サンプル（crates.io バージョン依存、`fw new --example` 対応、イシュー #2667。`fandhe-frontend-wireframe-ui` は headless-ui/pre-styled-ui 非依存の第 3 の UI 層のため `fandhe-frontend-app`/`-server` にも依存しない最小構成）
├── docker/                     # コンテナ定義（製品配布用 `Dockerfile` とは別。開発ループ専用）
│   └── dev/                    # 開発用 Docker イメージ・compose 定義
│       ├── Dockerfile         # Rust toolchain + wasm32 + 開発ツール一式。`make docker-dev-build` で構築
│       └── compose.yml        # `make docker-dev-build` / `make docker-dev` で利用
├── .github/
│   ├── workflows/              # CI ワークフロー（ci.yml / deps-check.yml / musl-smoke.yml / image-size.yml / ai-review.yml / docs-site.yml / release.yml / update-external.yml）
│   └── required-status-checks.json  # ruleset `main-protection` の required_status_checks の正のマニフェスト（`{context, integration_id}` の一覧。`gh api rulesets/<id>` の出力から生成、手書きしない。ワークフローとの整合は `crates/xtask/tests/workflow_required_checks_manifest.rs`、live ruleset との整合は `xtask check-ruleset-sync`〔`dep-version-check` ジョブ〕が検証する。イシュー #2325、`.claude/rules/ci.md` §「`ci-complete` 集約ジョブと ruleset 必須チェック」参照）
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
├── animation/     # fandhe-animation: アニメーション演算基幹（外部依存ゼロ・`forbid(unsafe_code)`。プラットフォーム非依存の値・イージング・タイムライン計算のみを担い、将来別リポジトリへの切り出しを前提とした設計。`fandhe-frontend-animation` が唯一の依存元、`docs/design/animation-core-architecture.md` 参照。雛形は #2371 で追加）
├── app/           # fandhe-frontend-app: モード非依存の共通コンポーネント
├── server/        # fandhe-frontend-server: SSR/SSG エントリ
├── wasm-client/   # fandhe-frontend-wasm-client: クライアントランタイム基盤
├── wasm-full/     # fandhe-frontend-wasm-full: CSR/ハイドレーション フルセット（配線群別 feature 31 件 + 別枠 feature 9 件〔position/stagger/animation-driver/view-transitions/view-transition-name/animate/layout-animation/view-transition-preset/presence〕+ scope feature 16 件、既定 on、#2326/#2327/#2396/#2397/#2398/#2400/#2403/#2515/#2516/#2517/#2518/#2519/#2520/#2521/#2532/#2533/#2535/#2536/#2538/#2539/#2540/#2541/#2542/#2544/#2550。`default-features = false` 利用者の移行手順は `docs/guides/wasm-full-features.md`）
├── wasm-thin/     # fandhe-frontend-wasm-thin: CSR/ハイドレーション 最小構成
├── frontend-animation/  # fandhe-frontend-animation: Web アニメーションアダプタ（`fandhe-animation` の演算結果を wasm-bindgen / web-sys / js-sys で DOM/Web Animations API へ適用する層。`fandhe-frontend-wasm-full` には依存しない独立クレートで、`crates/wasm-full/` が optional 依存として取り込む配線層を担う。依存方向は `fandhe-animation ← fandhe-frontend-animation ← wasm-full(optional)`、`docs/design/animation-core-architecture.md` 参照。雛形は #2417 で追加。typewriter/scramble のフレーム計算・DOM 書き込み（`text_animation`）は #2532 で追加。carousel のドラッグ + spring スナップ演算（`carousel`）は #2541 で追加。カスタムカーソルの spring 追従演算・rAF 駆動（`cursor::CursorFollower`/`CursorAnimator`）は #2542 で追加。keyed list 削除行の退場ゴースト配置・実測 CSS アニメーション時間経過後の除去（`presence`）は #2544 で追加）
├── dist-server/   # fandhe-frontend-dist-server: 単一実行ファイル配布サーバー（配布 WASM は `src/wasm_dist_features.rs` の最小インタラクティブ構成 6 feature、#2329。`build.rs` と `wasm-full/tests/bundle_size.rs` が `#[path]` 共有）
├── headless-ui/   # fandhe-frontend-headless-ui: headless UI コンポーネント層（anatomy・data-*・WAI-ARIA、イシュー #520/#522）
├── pre-styled-ui/ # fandhe-frontend-pre-styled-ui: pre-styled UI コンポーネント層（headless-ui 上層のスタイル済み部品、イシュー #520/#546）
├── wireframe-ui/  # fandhe-frontend-wireframe-ui: blocks.pm 参照のローファイ・モノクロワイヤーフレーム UI 部品層（SSR 専用・非インタラクティブ。`fandhe-frontend-core` のみに依存し、headless-ui/pre-styled-ui とは独立した第 3 の UI 層）。共通基盤（`Size`・`props` の共通型・`--fw-wire-*` モノクロトークン・`wireframe_css()`）と SVG ラインアートアイコン基盤（`icon.rs`）の上に全 49 部品を持つ。部品一覧・`fw-wire-*` class 命名規約・`Option<Node>` スロット規約（§11.4）と部品ごとの意図的な逸脱は `docs/design/wireframe-ui-architecture.md` を、golden テストの更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を正とする。実在ブランドのロゴ・商標を模した SVG は持ち込まない
├── docs-site/     # fandhe-frontend-docs-site: docs サイトジェネレータ（`publish = false`・配布物に含めない開発者/CI 用ツール。外部クレート依存ゼロで、内部 path 依存は core/app/server/pre-styled-ui/wireframe-ui のみ）。サイト骨格 CSS は `fandhe-frontend-pre-styled-ui` の `Theme::to_css` から、`assets/site.js` と検索インデックス（`assets/search-index.json`）はビルド時に生成する。JS ハイドレーションは行わない（無 JS 制約）。このためヘッダーのドロップダウンは pre-styled-ui の `menu` を使わず、素の `nav`/`ul`/`li`/`a` + CSS の `:hover`/`:focus-within` で組み、`role`/`aria-expanded`/`aria-haspopup` を付けない。文書リンク集に操作メニューの意味論を与えないためで、詳細は `src/nav.rs` の `header_nav` の rustdoc にある。サイドバーは現在ページのセクションへ限定し、該当なしのときは全セクション描画へフォールバックする（公開サイトのため可視性はアクセス境界ではない）。フェンスコードのハイライト（`src/highlight.rs`）は「トークンを連結すると常に入力と一致する」全域性で REQ-1 の既定エスケープを保つ。ページ種別ごとの供給元は次のとおり: Themes/Primitives の部品ページは `src/component_page.rs`（`Layer` で層を切り替え、Primitives は CSS 変数表を省略）、Blocks は `src/blocks/`、Wireframes は `src/wireframes/`。後 2 者は `build.rs` から独立分岐で呼ばれ、Markdown 本文の最初の `h2` の直前へ生成節を挿入する。旧 URL 互換のリダイレクトは `src/redirect.rs`（`site/redirects.toml`）が生成する。インライン目次は `class="docs-toc"`（スクロールスパイの唯一のセレクタ）を共有しない。各不変条件は `crates/docs-site/tests/` の契約テスト（`*_nav.rs` の三方突合・`*_contract.rs`・`blocks_code_drift.rs`・`layout_render.rs`・`wrap_state.rs` 等）が固定する
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
不変。`animation`/`frontend-animation` の `[directories.*]` 宣言はそれぞれ
#2371/#2417 で crate 雛形と同一 PR で有効化済み。`wasm-full.depends_on` へ
`frontend-animation` の辺は #2403/#2517 で追加済み（`crates/wasm-full/Cargo.toml`
の依存は引き続き `optional = true` だが、既定 on の `animation-driver` feature
（`dep:fandhe-frontend-animation`）が `default` 配列に列挙されたため、
`fw structure` が突き合わせる `cargo metadata` の既定解決〔`resolve.nodes[].deps`〕
にも現れるようになった。対称な `allowed_dependents = ["wasm-full"]` も
`directories.frontend-animation` へ宣言済み）。

## 委譲方針（必読）

main セッションは**指揮・統合・ユーザー対話に専念**し、調査・実装・テスト・レビューは sub-agent へ委譲して main のコンテキスト消費を抑える。詳細は `.claude/rules/delegation.md`（調査・設計）と `.claude/rules/delegation-impl.md`（作成・編集）を参照。

### パスベース切り替え表

| 対象パス | 委譲先 Agent |
|---------|-------------|
| `crates/core/` `crates/interactive/` | core-builder |
| `crates/headless-ui/` `crates/pre-styled-ui/` | core-builder |
| `crates/animation/` | core-builder |
| `crates/wireframe-ui/` | core-builder |
| `crates/app/` `crates/server/` | server-builder |
| `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `static/` | wasm-builder |
| `crates/frontend-animation/` | wasm-builder |
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
| implement | core-builder | sonnet | `crates/core/` `crates/interactive/` `crates/animation/` `crates/wireframe-ui/` — 描画・状態管理・アニメーション演算・ワイヤーフレーム UI コア（`forbid(unsafe_code)` 域） |
| implement | server-builder | sonnet | `crates/app/` `crates/server/` — SSR / SSG / ルーティング |
| implement | wasm-builder | sonnet | `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `crates/frontend-animation/` `static/` — CSR / ハイドレーション / WASM / Web アニメーションアダプタ |
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
| `ci.md` | CI 規約（GitHub ホステッドランナー既定・`runs-on` は `ubuntu-latest` 単一・ai-review の codex ジョブのみ self-hosted 例外・共有 CARGO_TARGET_DIR 対策・ツール前提の明示。frontmatter の `paths` により `.github/**`・`crates/**`・`site/**`・`templates/**`・`examples/**`・`Cargo.toml`・`docs/ci/**` 等を読んだときだけ読み込まれる） |

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
