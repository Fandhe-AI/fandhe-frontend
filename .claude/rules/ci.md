# CI 規約

## Runner 方針

- GitHub Actions の CI ジョブは `runs-on: ubuntu-latest` 等の GitHub ホステッドランナー（標準スペック）を既定とする（ユーザー指示 2026-08-07。public リポジトリのため標準ホステッドランナーは無料・分数消費なし）。本方針は組織 runner 方針（[Fandhe-AI/actions `docs/runner-policy.md`](https://github.com/Fandhe-AI/actions/blob/main/docs/runner-policy.md)、ユーザー決定 2026-08-07: public リポジトリは GitHub ホステッド〔ubuntu〕・private は self-hosted）の本リポジトリ（public）への適用である。組織方針の詳細は同文書を正とし、本節では書き写さず参照する（ドリフト防止）
- **`runs-on` は `ubuntu-latest` のみを使う（ユーザー指示 2026-08-10）**: `windows-latest` / `macos-latest` 等の他 OS ランナー、`ubuntu-24.04` のようなイメージ固定、`ubuntu-24.04-arm` 等の arm 変種、`${{ matrix.os }}` のような非リテラル指定（matrix による複数 OS 展開を含む）はいずれも使わない。この指示に伴い、唯一の非 ubuntu ワークフローだった `.github/workflows/fw-new-windows-verify.yml`（`workflow_dispatch` 専用の Windows 実機検証ハーネス、イシュー #413／#1236 で `windows-latest` へ移行済みだった）は削除した。同ワークフローが担っていた `fw new` の非 Unix（`#[cfg(not(unix))]`）分岐の検証は、Linux CI の `cargo test`（`crates/cli/tests/new_e2e.rs` の `#[cfg(not(unix))]` 分岐）による論理検証が引き続き担う（実機検証の喪失は既知のトレードオフ。過去の実測記録は `docs/reports/fw-new-windows-verification-report.md` に残す）。本規則の機械強制の対象は `.github/workflows/`（本リポジトリ自身の CI）のみとし、`templates/**/.github/workflows/`（`fw new` が生成するユーザープロジェクト向けテンプレート）は従来どおりスコープ外判断を維持する（現状すべて `ubuntu-latest` であることは確認済み）
- 新規ジョブ追加時もホステッドランナーを使用し、`runs-on: self-hosted` を使わない。larger runner（有料の大型ホステッドランナー）も使わない
- **唯一の例外（codex-review の codex 実行ジョブ、ユーザー承認済み 2026-08-07）**: `codex-review`（`.github/workflows/codex-review.yml`、イシュー #1275/PR #1278 で導入済み）が呼び出す reusable workflow の codex 実行ジョブのみ、self-hosted な codex 専用 runner（`runner-label` 既定値 `codex`）の使用を認める。適用条件（fork PR での実行拒否・sudo 不在の fail-closed 検証等）・wrapper の書き方・例外が及ばない範囲は [Fandhe-AI/actions `docs/codex-review-runner-exception.md`](https://github.com/Fandhe-AI/actions/blob/main/docs/codex-review-runner-exception.md) に従い、本節では詳細を書き写さない。例外は codex 実行ジョブに閉じる: `post_feedback` ジョブ（`post-feedback-runner-label`）は資格情報に触れないため wrapper で `ubuntu-latest` を明示済みであり、この例外を根拠に他ジョブ・他ワークフローを self-hosted 化しない。上記以外で self-hosted が必要になった場合は `docs/runner-policy.md` の更新から始める
- 旧方針（`runs-on: self-hosted` 既定、ユーザー指示 2026-07-18）は本指示で廃止。既存ワークフロー YAML の `runs-on: self-hosted` はホステッドランナーへ順次移行済み（`ci.yml` の `forbid-unsafe`/`clippy-wasm32`〔イシュー #1228〕・`release.yml` の `verify`/`publish`〔イシュー #1233〕は移行完了、`runner-maintenance.yml` はプール保守自体が不要になったため廃止〔イシュー #1237〕。全ワークフロー移行完了済み、詳細は `docs/ci/hosted-runner-migration.md` §6）
- self-hosted 前提だった箇所（ツールの事前導入・共有 `CARGO_TARGET_DIR`・runner イメージ常設要件）は移行時に「クリーンな使い捨て VM（ジョブごとに初期化、共有キャッシュなし）」前提へ読み替える。ホステッドランナーではツールは毎ジョブ導入が必要になるため、各ワークフローの存在チェック付きインストール（安全網）は移行後むしろ主経路となる。削除・弱体化しない
- キャッシュ戦略（`actions/cache` 採否・キャッシュキー設計）・ツール導入方針・移行順序の正は `docs/ci/hosted-runner-migration.md`（イシュー #1225）とし、詳細は同文書へ譲り本節では二重管理しない
- 本方針（`runs-on` は `ubuntu-latest` リテラル単一）は同じく `crates/xtask/tests/workflow_runner_policy.rs`（`workflows_run_only_on_ubuntu_latest`）が `.github/workflows/*.yml` に対して fail-closed に機械強制する。判定は「許容形の列挙」ではなく**反転判定**とする: コメント除去後の内容に `runs-on` の文字列が現れる行は、唯一の許容形 `runs-on: ubuntu-latest`（キーのクォート・キーと `:` の間の空白・値のクォートのみ表記揺れとして許容）に一致しない限りすべて違反とする。これにより block sequence 形・`labels:` mapping 形・flow mapping 形・アンカー/エイリアス・tag 指定など**認識器が知らない表記は自動的に違反側へ倒れる**（許容形を列挙する形では、未知表記が「`runs-on` 行ではない」として素通りする fail-open になり、PR #1301 の codex レビューで空白形・クォート形の 2 通りが実際に指摘された）。副作用として `run:` スクリプト中に `runs-on` と書いた行も違反になるが、`self-hosted` 禁止契約と同じく意図的な厳格性であり、歴史的経緯の言及はコメントへ書けば足りる。reusable workflow の呼び出しジョブ（job-level `uses:`）は `runs-on` を持たず本テストの射程外であり、下記 codex-review 例外はその射程外で成立している（本規則は OS・イメージ選択に関するもので、既承認の codex 例外を撤回しない）
- 本方針（`runs-on: self-hosted` の禁止）は `crates/xtask/tests/workflow_runner_policy.rs` が `.github/workflows/*.yml` のコメント除去後全文に対して fail-closed に機械強制する（イシュー #1239）。歴史記録としての言及はコメントでのみ許容される。codex-review 例外と本テストは両立する: wrapper（`.github/workflows/codex-review.yml`）は `runner-label` を reusable workflow 側の既定値経由で参照するのみで、YAML の非コメント行に `self-hosted` の文字列が現れないため、除外リスト追加・テスト弱体化なしに例外が成立している。将来 wrapper へ `self-hosted` リテラルを書く変更（例: `post-feedback-runner-label: self-hosted`）はこのテストが FAIL させるが、それは方針違反の検知として正しい挙動であり、テスト側へ除外を追加して回避しない

## Runner 環境と一時領域の前提

- 旧 self-hosted 環境では共有 `CARGO_TARGET_DIR=/cargo-target` が使われ、テストフィクスチャのクレート名衝突・キャッシュ誤命中への対策が必要だった。ホステッドランナーはジョブごとにクリーンな VM だが、ローカル開発・`actions/cache` 等でのキャッシュ再利用でも同種の問題は起こり得るため、以下の防御策は削除・弱体化しない
- フィクスチャ専用 `CARGO_TARGET_DIR` を明示指定する（例: `crates/cli/tests/negative_cases.rs` / `crates/cli/tests/raw_html_lint_e2e.rs`、PR #264）
- **ワークフロー YAML 側で明示指定するフィクスチャ専用 `CARGO_TARGET_DIR`・生成物パスは `/tmp` 固定パスではなく `RUNNER_TEMP`（`${{ runner.temp }}`）配下に置く**（イシュー #659）。`RUNNER_TEMP` はジョブ開始・終了時に runner が自動清掃するため、`/tmp` 固定パスのようにジョブ間残置が恒久蓄積しない（`template-app-wasm-smoke` ジョブの是正が先例、`docs/ci/ci-runner-requirements.md` §8 参照）。テストコード側の `env!("CARGO_TARGET_TMPDIR")` 固定方針（イシュー #637）と対をなす。ホステッドランナーでも `RUNNER_TEMP` は提供されるため本原則は不変

## ツール前提の明示

- runner に常設が保証されないツール（wasm-bindgen-cli / wasm-pack / cargo-deny / clippy component / Chrome 等）に依存するステップは以下のいずれかを実行する
  - 存在チェック付きインストール（`command -v` / `where` 等で確認してから `cargo install` 等を実行）
  - ワークフロー YAML に明示的な前提コメント（例: `# 要: wasm-pack がインストール済み`）
- **共有 `CARGO_TARGET_DIR` と `cargo package`/`cargo publish` 検証ビルドの分離（イシュー #1192）**:
  `cargo package` / `cargo publish`（`--dry-run` 含む）の検証ビルドは packaged
  コピー（path 依存が剥がされ crates.io registry 版の依存に解決される）を
  ビルドするため、旧 self-hosted 環境の共有 `CARGO_TARGET_DIR`
  （`/cargo-target`）で実行すると cdylib+rlib クレート（wasm-thin/wasm-full/
  wasm-client。`crate-type = ["cdylib", "rlib"]` の rlib はメタデータハッシュ
  サフィックスなしの固定ファイル名で出力される cargo の仕様）の rlib を
  registry 依存内容で上書きし、後続のワークスペースビルドが fingerprint
  fresh 判定で汚染済み rlib をそのままリンクして
  「multiple different versions of crate」（E0277/E0599）の flaky を
  引き起こす（PR #1164/#1180/#1186/#1187 で実際に観測、再現手順はイシュー
  #1192 コメント参照）。`cargo package`/`cargo publish` を実行するワーク
  フロー（`release.yml` の `verify`/`publish` ジョブ）は必ず専用
  `CARGO_TARGET_DIR`（`RUNNER_TEMP` 配下、イシュー #659 の配置原則）を
  明示指定して共有 target dir から隔離する（根本対策）。加えて `ci.yml` の
  `forbid-unsafe`/`test`/`gate-self-apply` ジョブは cargo 実行前に無ハッシュ
  cdylib rlib（3 種）を削除する自己修復ガードステップを持ち、対策導入前の
  既存汚染や他ワークフロー起因の汚染からも回復する（多層防御）。ガード
  ステップの削除対象は固定ファイル名のみとし、glob・`rm -rf` は用いない
  （A01 パストラバーサル・広域削除の防止、`security.md` 参照）。この 2 層の
  対策宣言は `crates/xtask/tests/workflow_shared_target_contract.rs` が
  fail-closed に固定しており、削除・弱体化しない。**イシュー #1226 で
  ホステッドランナー前提へ契約を再設計済み**: 既存 5 テストは全件維持し
  （ホステッドの使い捨て VM では共有ディスク汚染の動機は消えるが
  `actions/cache` 復元時に同型汚染が再発し得るため）、加えて
  「`target` をキャッシュするジョブへのガードステップ必須化（ci.yml）」
  「release.yml での `target` キャッシュ禁止」の 2 契約を新設した
  （イシュー #1227 で `ci.yml` の `clippy` ジョブへ `actions/cache` が実導入
  され、以降この 2 契約は vacuous ではなく実ワークフローに対して実効判定
  している。Phase 2 以降の追加キャッシュ導入時もガード欠落・target
  キャッシュを即座に検知する）。
  詳細は `docs/ci/hosted-runner-migration.md` §2.1(d) 参照。**PR #1244 レビュー
  指摘（イシュー #1226）を受けた追加強化**: 新設 2 契約は「ガードステップの
  マーカー文字列がジョブ本文のどこかに存在するか」の vacuous な判定では
  なく、(1) ガードステップが `actions/cache` 復元ステップより**後段**に
  あること（復元はそのステップ実行時に起きるため、先行するガードは復元後の
  汚染を除去できない）、(2) ガードが削除するディレクトリ参照（環境変数名/
  リテラルパス）がキャッシュされているディレクトリ参照と**完全一致**する
  こと（別ディレクトリを掃除するだけの no-op ガードを弾く）を検証する。
  target 検出ヒューリスティックも大小文字を区別しない形へ強化し、
  `path: ${{ env.CARGO_TARGET_DIR }}` のような env 参照形（`target` の語が
  変数名の一部として大文字でのみ現れる）の見逃しを防ぐ。ステップ検出も
  ステップ名の完全一致に限定し、無関係なステップ内のコメント引用による
  誤判定を防ぐ。**自己レビュー追補**: 新設契約は、一致したガードステップに
  対して既知 3 ジョブ（`forbid-unsafe`/`test`/`gate-self-apply`）と同じ完全性
  チェック（無ハッシュ cdylib rlib 3 種すべてを削除・`rm -rf` 不使用）も適用
  する（1 種類しか削除しない不完全なガードがすり抜けるのを防ぐ）。ガード
  本体の抽出は 1 パス 1 行の継続行形式・単一行 `rm -f "a" "b" "c"` 形式の
  いずれにも対応する。**PR #1244 に対する Bugbot 再指摘（イシュー #1226）を
  受けた追加修正**: (1) `name:` を持たず `- uses: actions/cache@...` から
  始まる nameless 形式のステップは行頭 `- ` を剥がさない判定では検知漏れに
  なっていたため、`- ` プレフィックスを剥がしてから `uses:` を判定するよう
  修正した。(2) 「キャッシュ復元ステップより後段にガードを置く」順序契約
  は、復元を一切行わない `actions/cache/save`（書き込み専用アクション）
  ステップの後段にもガードを要求してしまい、「restore → guard → build →
  save」という正しい構成を偽陽性で FAIL させていたため、save 専用ステップ
  を ci.yml 側の順序契約の対象から除外した（release.yml 側の「target を
  キャッシュしない」禁止契約は save 専用ステップも引き続き検出対象のまま
  弱体化しない）。(3) `path: "target/"` のようなクォート付きリテラルは
  クォートを剥がさないまま比較していたため、クォート無しのガード削除
  パスとの完全一致比較が常に失敗し、正しくカバーしているガードを誤って
  「カバーしていない」と判定していたため、比較前にクォートを除去する
  よう修正した。
- **`docs-site.yml` の paths フィルタ契約（イシュー #899/#913）**: docs サイトの
  骨格 CSS（`assets/site.css`）は #905 以降ビルド生成物であり、生成元は
  `crates/docs-site/src/site_theme.rs` と `crates/pre-styled-ui`（`Theme::to_css`）。
  このため `crates/pre-styled-ui/**` / `crates/headless-ui/**` /
  `crates/interactive/**` は showcase 限定の例外ではなく**全ページに影響する
  paths 必須項目**である。レンダラ側（core / app / server）は従来どおり
  paths 対象外（反映が必要なときは `workflow_dispatch`）。
- **`docs-site.yml` の verify ステップ契約（イシュー #944/#951/#957/#1016/#1017/#1018/#1021/#1022）**: `site/**` の
  glob は `site/themes/*.md`（イシュー #1017 で `site/components/*.md` から
  移行）と `site/primitives/*.md`（イシュー #1021）を包含するため、部品ページ
  追加時に paths への個別エントリ追加は不要（イシュー #944 で検証済み）。`site/redirects.toml`
  （イシュー #1016、旧 URL 互換のリダイレクトページ生成機構）も同じ `site/**`
  glob に包含されるため、同様に paths への個別エントリ追加は不要（同一 glob
  包含の再確認）。`docs/**` は `docs/internal/`
  も包含する。`docs/internal/` は `site/nav.toml` 未登録のためサイトへは出力され
  ないが、変更時に再ビルドは走る（無害な過剰トリガーであり、paths からの除外は
  しない）。dist sanity check（`verify: dist sanity check` ステップ）の `test -f`
  対象は #944/#951/#957/#1016/#1017/#1018/#1021/#1022 で拡張され、現在は `index.html` / `assets/site.css` /
  `assets/site.js`（#951、`src/script.rs` 生成）/ `assets/search-index.json`
  （#957、`src/search_index.rs` 生成）/ 代表部品ページ（`themes/button/index.html`。
  イシュー #1017 で `components/button/index.html` から移行）/ `themes/index.html`
  （#1018、`/components/pre-styled-ui/` から移設した新索引ページ本体）
  / `assets/pre-styled-ui.css`（`showcase::STYLESHEET_REL_PATH`）/ 代表リダイレクト
  ページ 3 件（`components/index.html`、#1016、`src/redirect.rs` 生成。
  `components/button/index.html`、#1017 が追記した 107 件の代表。
  `components/pre-styled-ui/index.html`、#1018、旧索引 URL のリダイレクト生成物）/
  `primitives/index.html`（#1021、Primitives セクション索引）/
  `primitives/accordion/index.html`（#1021、代表 Primitives 部品ページ）/
  `assets/primitives-showcase.css`（イシュー #1022、`src/primitive_showcase/`
  が生成。#1021 が「本イシュー完了時点では CSS を持たない」として先送り
  していた test -f を、Primitives 63 部品の Demo 供給に伴い追加した）である。
  いずれも
  fail-closed（欠落時にジョブを落とし、空サイト・アセット欠落の公開を防ぐ）であり、
  この `test -f` 群は削除・弱体化しない。生成物の**内容**検証（CSS トークン網羅性・
  検索インデックスの決定性/エスケープ/サイズ上限・ページ総数・リダイレクトページの
  4 要素網羅と fail-closed 検証、Primitives の Anatomy/`data-*` 表網羅・
  scope 一致・`[data-scope=`/`[data-part=` 不在）は
  `crates/docs-site/tests/`（`site_css_contract.rs` / `site_typography_contract.rs` /
  `search_index.rs` / `site_nav.rs` / `site_build.rs` / `redirects.rs` /
  `no_js_contract.rs` / `primitive_showcase.rs` / `primitive_showcase_xss.rs` /
  `primitives_nav.rs` / `primitives_catalog.rs` / `wrap_state.rs`〔Primitives
  63 部品 と Themes 107 部品の層をまたぐラップ状態の 4 バケット分割検知、
  イシュー #1064〕/ `highlight.rs`〔フェンスコードブロックの軽量シンタックス
  ハイライト（`src/highlight.rs`）の XSS エスケープ・CSS トークン網羅性・
  全域性契約、イシュー #1078〕）が担い、yml・ci.md では
  二重管理しない（ページ件数・部品数を ci.md へ書かないのはこの二重管理回避のため）。
- **`fw gate`（`crates/cli/src/gate.rs`）系のツール（clippy component / cargo-deny / wasm32-unknown-unknown rustup target）**: `tools/ci/ensure-gate-tools.sh` を標準ブートストラップとする（イシュー #292。wasm32 target の常設は `lint_wasm32` チェック向けにイシュー #1174 で追加）。CI（`.github/workflows/ci.yml` の test ジョブ・`gate-self-apply` ジョブ）・ローカル開発・AI 自己保守フックのいずれも `fw gate` 実行前にこのスクリプトを前置する運用を推奨する。バージョン固定・SHA256 チェックサム検証はスクリプト側に一元化し、CI ワークフロー側との二重管理でドリフトさせない。前置されなかった場合でも `fw gate` 側のプリフライト検出（`docs/design/gate-design.md` §2.3a・§2.6）が「環境エラーであること」を決定的なメッセージ（是正コマンド付き）で示し、コード起因の FAIL との区別を保つ
- **`fw gate --project .` の自己適用常時実行（イシュー #400・#1116）**: `.github/workflows/ci.yml` の `gate-self-apply` ジョブが PR ごと・main push ごとに `fw gate --project .` 自己適用（#372/PR #382 で PASS 化）を実行し、`gate_result: "PASS"` の継続を保証する。イシュー #1116 で `gate` の終了コードに `3`（`gate_result: "ERROR"`、実行環境にツールが無いだけの不合格）が追加されたが、ジョブの判定は `version-bump-guard` と同じ「終了コードは 0/非 0 のみを見て、種別判定は JSON 出力本文の grep に任せる」設計を踏襲する: `if PIPELINE; then exit 0; fi`（`set -euo pipefail` を維持したまま条件式内で pipefail 失敗による即時中断を回避する）で非 0 終了を捕捉した後、`"gate_result":"ERROR"` の有無で環境エラーとコード起因 FAIL（`BLOCKED`）を CI アノテーションとして区別する。終了コードの値（`PIPESTATUS` 等）を読み分ける実装は複雑化を避けるため採らない（詳細は `docs/design/gate-design.md` §4・§6）
- **cargo-deny 導入パターンの統一（イシュー #314）**: cargo-deny を導入する全ワークフロー（`tools/ci/ensure-gate-tools.sh`・`templates/default/.github/workflows/deny.yml`・`docs/policy/cargo-deny-advisories.md` のサンプルワークフロー）は「バージョン固定 + SHA256 チェックサム検証済みプリビルトバイナリ」パターンに統一する（`cargo install` によるソースからの任意最新版コンパイルは行わない）。バージョン・SHA256 の pin の正は `tools/ci/ensure-gate-tools.sh` の `CARGO_DENY_VERSION` / `CARGO_DENY_SHA256` のみとし、テンプレート・docs はスタンドアロン配布物のため同パターンをインラインで複製する。3 箇所の pin 値が乖離しないことは `crates/xtask/tests/template_deny_workflow.rs` のドリフト検知テストが `cargo test -p xtask` / CI で強制する（手動同期に頼らない）
- **`templates/app`（`fw new --template app`）の crates.io バージョン依存化（イシュー #412/#493）**: `templates/app` は fandhe-frontend-core/-app/-interactive/-wasm-client への vendor 同梱を廃止し、通常の crates.io バージョン依存へ切り替えた（`docs/design/template-vendor-to-version-switch.md`）。このため `crates/cli/tests/new_gate_e2e.rs` の app テンプレート分（`fw_new_app_template_output_passes_fw_gate` 等、生成プロジェクトの `cargo build`/`fw gate` を実行する e2e）と `templates/app/Cargo.lock`・`templates/app/wasm/Cargo.lock` の再生成は、crates.io へのネットワークアクセスと registry キャッシュを前提とする（vendor 同梱時のオフライン決定性は失われた）。これらを実行する CI ジョブは、runner が crates.io（`https://static.crates.io`・`https://index.crates.io`）へ到達可能であることを前提とする（到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない）。**バンプ先バージョンが crates.io へ未公開のウィンドウ中（イシュー #895）**: `crates/cli/tests/new_gate_e2e.rs` の app テンプレート e2e 2 件は `apply_patch_template_smoke`（同ファイル）が `xtask patch-template-smoke`（イシュー #885、`template-app-wasm-smoke` ジョブが導入した機構と同一）をサブプロセスとして再利用し、未公開の依存のみ `[patch.crates-io]` フォールバックを適用してからテスト対象プロジェクトの `fw gate` を実行する（version-bump-guard・`template_vendor_drift` テストとの三すくみを smoke ジョブと同型に回避する。詳細は `docs/ci/version-bump-publish-order-gap.md` §「実装結果（イシュー #895）」）
- **`examples/ssr-routing`（`fw new --example ssr-routing`）の crates.io バージョン依存前提（イシュー #499/#500）**: `examples/ssr-routing` は fandhe-frontend-core/-app/-server への crates.io バージョン依存で完結する正本サンプルであり、vendor 同梱を持たない。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_ssr_routing_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`cargo run`/`fw gate` を実行する e2e）は `templates/app` 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **`examples/dist-server-docker`（`fw new --example dist-server-docker`）の crates.io バージョン依存前提（イシュー #502）**: `examples/dist-server-docker` も同様に fandhe-frontend-core/-app/-dist-server への crates.io バージョン依存で完結する正本サンプルであり、vendor 同梱を持たない。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_dist_server_docker_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`fw gate` を実行する e2e）も `ssr-routing` 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **examples e2e の実行時間計測と `CARGO_TARGET_DIR` 共有（イシュー #505）**: `crates/cli/tests/new_gate_e2e.rs` の examples e2e 5 件（`fw_new_example_ssr_routing_output_passes_fw_gate` / `fw_new_example_ssg_blog_output_passes_fw_gate` / `fw_new_example_dist_server_docker_output_passes_fw_gate` / `fw_new_example_interactive_view_transitions_output_passes_fw_gate` / `fw_new_example_headless_pre_styled_ui_output_passes_fw_gate`）は `example_shared_target_dir()`（同ファイル）が返す共有 `CARGO_TARGET_DIR` を `fw gate` と `cargo run` smoke の双方に明示指定し、examples 間で crates.io 依存のビルドキャッシュを共有する。上記「フィクスチャ専用 `CARGO_TARGET_DIR` を明示指定する」原則の例外ではなく、対象を絞った適用である（`negative_cases.rs` 等の欠陥注入フィクスチャは同名パッケージを異内容で再利用するため引き続き専用ディレクトリを使う。examples はパッケージ名が相互に一意でリーフクレートが毎回新規展開されるため偽陰性リスクがない。根拠は `new_gate_e2e.rs::example_shared_target_dir` の doc コメント参照）。`.github/workflows/ci.yml` の `test` ジョブは「`fw new` 生成直後の `fw gate` PASS 構成保証（examples 除く、`--skip example_`）」と「examples gate e2e 5 件の実行と時間計測（`example_` フィルタ）」の 2 ステップに分割し、後者の所要時間を CI ログで常時可視化する。判定基準（examples e2e による時間増が +10 分超なら `examples-gate-e2e` ジョブへ分離）に対し、#505 時点の実測（4 件合計・ステップ全体で約 8 秒）は閾値未達のためジョブ分離は行っていない（#609 で 5 件目を追加後も同一判断基準を適用する）。恒常的に 10 分超となった場合は ci.yml のコメントに従いジョブ分離を再検討する
- **`examples/headless-pre-styled-ui`（`fw new --example headless-pre-styled-ui`）の crates.io バージョン依存前提（イシュー #609）**: `examples/headless-pre-styled-ui` は当初 `fandhe-frontend-headless-ui` が crates.io 未公開のため path 依存の意図的な例外だった（イシュー #552）が、前提クレート公開（イシュー #608）を受けて fandhe-frontend-core/-headless-ui（推移的に -interactive）への crates.io バージョン依存へ切り替え、`fw new --example` に登録した（イシュー #609）。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_headless_pre_styled_ui_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`cargo run`/`fw gate` を実行する e2e）は他の examples 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **一時領域の配置固定・自動クリーンアップ（イシュー #637）**: `crates/cli/tests/support/mod.rs::scratch_root` / `crates/cli/tests/scenarios/common.rs::scratch_root` / `crates/cli/tests/new_e2e.rs::unique_scratch_dir` / `crates/cli/tests/new_gate_e2e.rs`（`unique_scratch_dir` / `example_shared_target_dir`）が展開する一時プロジェクト・共有 `CARGO_TARGET_DIR` は、コンパイル時に確定する `env!("CARGO_TARGET_TMPDIR")`（`<target>/tmp`。CI では `/cargo-target/tmp`、ローカルでは `target/tmp`）へ確定配置し `/tmp` へは置かない（cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリのコンパイル時のみであり、実行時の `std::env::var` 参照は cargo の仕様上常に失敗するため使わない。かつてこの事実誤認により実行時フォールバック＝`/tmp` へ恒常的にリークしていた）。共有 target（`example_shared_target_dir`）・生成プロジェクト（`fw-new-gate-e2e-*`）は「所有者テストの特定が不安定」なため `ScratchProject` の Drop ガードでは消さず、`new_gate_e2e.rs::cleanup_stale_scratch`（PID 生存判定＝`/proc/<pid>` の存在確認、**かつ** mtime が `STALE_MIN_AGE`（1 時間）を超えていることを削除の必須 AND 条件とする。`/proc` が使えない環境では mtime 判定のみにフォールバックする）が次回実行時に旧世代（新配置先・旧配置先 `/tmp` の双方）を回収し蓄積を有界化する。PID 生存確認のみに依存しない理由（PR #648 CI 障害の根本原因、イシュー #637 追補）: `/cargo-target` はコンテナ化された複数 CI ジョブ間で共有され得るが、PID の生存確認は **PID 名前空間ローカル** の判定であるため、別ジョブ（別 PID 名前空間）から見ると「他ジョブが現に `fw new`/`fw gate` を実行中の scratch ディレクトリ」の PID も非存在＝stale と誤判定されドリフト削除されてしまう（実際に `fw_new_app_template_output_passes_fw_gate` が cargo working directory 消失で FAILED になった）。mtime を必須の追加条件にすることで、たとえ PID 判定が誤って「非生存」と示しても直近に作成・更新されたディレクトリは保護される。ラン内キャッシュ共有（#505）の意図は不変
- **crates.io 公開用 release ワークフロー（イシュー #514/#513）**: `.github/workflows/release.yml` は `workflow_dispatch` 起点で単一クレートを crates.io へ公開する。既公開バージョン検証ステップは `command -v curl` で存在チェックしてから sparse index（`https://index.crates.io`）を取得するため、curl 未導入 runner では環境エラーとして明示停止する（自動インストールは行わない）。`cargo package`/`cargo publish`（dry-run 含む）は `https://index.crates.io`・`https://static.crates.io` への到達性を前提とし、到達不可の場合は環境エラーとして扱う（他の crates.io バージョン依存ワークフローと同様、テストの弱体化で対処しない）。`CARGO_REGISTRY_TOKEN` はリポジトリ Secrets からのみ供給し、`mode: publish` を選択したステップの `env:` にのみ限定注入する（ログへは出力しない）。誤操作対策として `mode` の既定値は `dry-run-only`（安全側）とし、実公開は明示的に `mode: publish` を選ぶ運用とする
- **`version-bump-guard` ジョブ（イシュー #638）**: `.github/workflows/ci.yml` の `version-bump-guard` ジョブ（`if: github.event_name == 'pull_request'` のみ実行、main push ではスキップ）は `xtask check-version-bump`（`crates/xtask/src/check_version_bump.rs`）を呼び出し、公開済みクレート（`crates/*` のうち `publish = false` を持たないもの）の `src/`・`Cargo.toml`・`build.rs` が変更されているのに `Cargo.toml` の `version` が crates.io 既公開バージョンのままの PR を検知する（headless-ui 0.1.0 公開直後にバージョンバンプなしの破壊的変更がマージされ main を赤にした事故、PR #611 → 復旧 PR #634、が動機）。crates.io sparse index（`https://index.crates.io`）への到達性を前提とし、`command -v curl` 相当の存在チェック（`check_version_bump::query_index` 内）・curl 非 0 終了・想定外 HTTP status はすべて `environment error: ` プレフィックス付きで fail-closed に扱う（到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない。他の crates.io バージョン依存ワークフローと同じ方針）。HTTP 200 系だが body が空/パース不能で 1 バージョンも抽出できない応答（sparse index の異常応答）も `Published([])` として PASS 扱いにせず `environment error: ` として fail-closed にする（イシュー #638 PR #647 レビュー指摘、`query_index` 参照）。`curl` 呼び出しには `--connect-timeout 10 --max-time 30` を付け、ジョブ自体にも `timeout-minutes: 10` を設定し、`index.crates.io` へのリクエストがハングしても runner を無期限に占有しない（同レビュー指摘）。ジョブは xtask の stderr を `environment error: ` プレフィックス有無で判定し、「runner/ネットワーク起因」と「コード起因（バンプ漏れ）」を CI アノテーションとして区別する（`gate-self-apply` と同型）。この判定パイプラインは `if PIPELINE; then ... fi` の形（`set -euo pipefail` のまま `-e` を維持）で組む: `set -uo pipefail`（`-e` を含めない）で `PIPESTATUS` を後段で読む旧実装は、GitHub Actions のデフォルト起動オプション由来の `-e` が `set` で解除されずに残るため、パイプライン失敗時に判定へ到達する前にジョブが中断してしまう不具合があった（同レビュー指摘、修正済み）。さらに「コード起因（バンプ漏れ）」の断定は 1 行サマリの `result=FAIL` 行の有無で確認し、`cargo metadata`/`git diff` 自体の失敗（`CommandFailed`、例: `origin/<base>` 未 fetch）を誤って「バンプせよ」と注釈しない（同レビュー指摘）。誤検知の抑制手段として、PR 本文に `version-bump-exempt: <crate-name>`（同一行に理由を続けて記載）を宣言すると当該クレートのみ免除される。免除はクレート名の完全一致でのみ成立し、包括免除（マーカーのみ・名前なし）は認めない（security.md A05、`coding-rust.md` 参照）。PR 本文・`github.base_ref` はワークフロー内で `env:` 経由のみで受け渡し、シェルへ直接展開しない（script injection 対策）。**cargo-semver-checks 導入評価（イシュー #656）**: 本ゲートが検証しない公開 API の意味論的な semver 互換性を cargo-semver-checks で補完する案は評価済みであり、運用実績の集計・コスト分析の結果、現時点では導入見送りと結論した（再評価トリガーを含め詳細は `docs/ci/cargo-semver-checks-evaluation.md` 参照）
- **`dep-version-check` ジョブ（イシュー #657）**: `.github/workflows/ci.yml` の `dep-version-check` ジョブ（push・PR 双方で常時実行。`version-bump-guard` と異なり `if: github.event_name == 'pull_request'` は付けない）は `xtask check-dep-versions`（`crates/xtask/src/check_dep_versions.rs`）を呼び出し、workspace 内メンバー間の `path + version` 併記依存について依存元の `version = "..."` 要求が依存先の現行 `version` へ追随しているかを検知する。headless-ui 0.1.0 → 0.2.0 バンプ時、依存元（pre-styled-ui / wasm-full / xtask）の `version = "..."` 追随が sed による手動一括変更を要した実績（`version-bump-guard` の是正メッセージによる注意喚起のみでは機械検知手段がなかった、PR #647 out-of-scope）が動機。判定は `cargo metadata --no-deps` のみで完結し、`version-bump-guard`（crates.io sparse index 照会あり）と異なりネットワーク照会を一切行わないため push・PR 双方で常時実行できる。判定ルールは 2 つ: (1) version 宣言があるエッジは `req == "^" + 依存先の現行 version` の完全一致のみ PASS（古い version・`=` ピン・部分指定はいずれも FAIL、3 要素完全表記を機械的に固定）、(2) version 宣言がない（`req == "*"`）エッジは、依存元が publish 対象（`check_version_bump::published_crates_from_cargo_metadata` と同じ fail-closed 判定）かつ kind が normal/build の場合のみ FAIL（dev は `cargo publish` 時に自動除去されるため対象外）。既定（引数なし）は検知のみで 1 件でも FAIL があれば終了コード 1。`--fix` は version 不一致（ルール 1）のみを自動修正するローカル向けオプトイン手段で、依存元 Cargo.toml 内で書き換え対象の `version = "<旧>"` 行を一意に特定できない場合（未対応の req 表記・候補 0/複数件）は一切書き換えず fail-closed にエラー終了する（部分書き込みをしない。全編集位置の特定完了後に一括適用する設計）。書き込み先は `cargo metadata` の `manifest_path` が workspace_root 配下であることを検証してからに限定する（パストラバーサル防止、security.md A01）。version 欠落（ルール 2）は `--fix` でも自動修正されず、残留すれば終了コード 1 のままとなる（`cargo publish` が実際に失敗する構成を安易に隠さないため）。1 行サマリは `dep-version-check: crate=<依存元> dep=<依存先> kind=<normal|dev|build> req=<req> actual=<version> result=<PASS|FAIL>`（`grep '^dep-version-check:'` で CI アノテーション生成側が抽出できる契約）。CLI 契約の回帰テストは `crates/xtask/tests/cli_check_dep_versions.rs`。`version-bump-guard` の是正メッセージ（`crates/xtask/src/main.rs`）も「バンプ後は `cargo run -p xtask -- check-dep-versions --fix` で依存元の version 要求を自動追随できる」旨を案内する
- **横断 a11y 自動検証（axe-core 相当）導入評価（イシュー #1076）**: `crates/headless-ui/`（63 部品）の WAI-ARIA 検証を横断で機械強制する axe-core 相当ツールの導入は、npm 経路の構造的な受け入れ不可（REQ-12 allowlist）・既決の Playwright 不採用の継承・サプライチェーン方針との非整合を理由に、現時点では見送りと結論した（詳細・再評価トリガーは `docs/ci/a11y-automation-evaluation.md` 参照）
- **example オーバーレイのブラウザ実インタラクションテスト常設 CI 化評価（イシュー #1210）**: `examples/interactive-view-transitions/wasm` の navigation-menu / menubar オーバーレイ実演（PR #1206 が `wasm-pack test --headless --chrome` の使い捨てハーネスで実測、`docs/reports/interactive-view-transitions-overlay-browser-report.md`）の常設 CI 化は、中核ロジックが `crates/wasm-full/tests/overlay_close_browser.rs` 等で CI 常設済みであること・example 正本へのテスト同梱が `embedded-examples` バイト一致同期と cli semver バンプ連鎖を誘発すること・再現手順が既にレポートで文書化済み（#1209 修正検証 PR #1212 で実際に再現できた実績あり）であることを理由に、現時点では見送りと結論した（詳細・候補比較・再評価トリガーは `docs/ci/example-overlay-browser-interaction-testing-evaluation.md` 参照）
- **aarch64 self-hosted runner による Docker WASM 再ビルド検証の CI 常設化評価（イシュー #1216）**: イシュー #450（PR #1214）で実測した aarch64 実機（Apple Silicon macOS ホスト上の Docker Engine）での Docker マルチステージ WASM 再ビルド（`docs/reports/docker-wasm-rebuild-acceptance-report.md` §5a）の CI 常設化は、org スコープの self-hosted runner プール（20 台稼働確認済み）が全台 `X64` ラベルで aarch64 Linux インスタンスを 1 台も含まないこと・`WASM_BINDGEN_VERSION` のバージョン文字列ドリフトは `crates/xtask/tests/wasm_bindgen_version_sync.rs` が x86_64/aarch64 両分岐とも既に `cargo test` 時点で fail-closed 検知していること（aarch64 側 `WASM_BINDGEN_SHA256` の同期は当初この回帰テストの対象外という既知のギャップがあったが、イシュー #1218 で第 3 のテスト `dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive`（既知 SHA256 値との突合）を追加し解消済み）を理由に、現時点では見送りと結論した（詳細・候補比較・再評価トリガーは `docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md` 参照）。**前提変更の注記（イシュー #1238）**: 見送り根拠の中心だった self-hosted runner プールの aarch64 不在は、CI runner 方針のホステッドランナー既定への反転（#1220）で判断基盤ごと前提が変化した（public リポジトリ無料の arm64 ホステッドランナー `ubuntu-24.04-arm` 等が利用可能）。再評価の実施は本規約のスコープ外であり、詳細は同評価文書の追記節を参照する。**さらなる前提変更（2026-08-10）**: `runs-on` を `ubuntu-latest` 単一へ限定するユーザー指示（本節冒頭）により、この再評価経路で想定していた arm64 ホステッドランナー（`ubuntu-24.04-arm` 等）の利用は現方針では選択肢から外れた。aarch64 CI 常設を再検討する場合は Runner 方針の変更から始める
- **`clippy-wasm32` ジョブ（イシュー #1160）**: `.github/workflows/ci.yml` の `clippy-wasm32` ジョブは wasm32-unknown-unknown target で 3 wasm クレート（wasm-full / wasm-client / wasm-thin）の `cargo clippy --all-targets --locked -- -D warnings` を実行し、host target のみの `clippy` ジョブでは検知できない `#[cfg(target_arch = "wasm32")]` ゲート配下（browser テスト含む）の警告を fail-closed に検知する（イシュー #1140/PR #1147 のすり抜けが動機）。clippy は check のみで wasm-bindgen 後処理を起動しないため wasm-bindgen-cli は不要。対象閉包に dist-server（build.rs ネスト WASM ビルド）を含まないため `FANDHE_FRONTEND_WASM_BUILD` のオプトアウトも不要。workspace ルート `clippy.toml` の disallowed-methods（REQ-1）も wasm32 ゲート配下へ適用される
- **Fandhe-AI/actions 新規 15 コミット分の機能の採用可否評価（イシュー #1288）**: `docs-site.yml` の deploy ジョブは `Fandhe-AI/actions/.github/workflows/pages-deploy.yml`（reusable workflow）呼び出しへ置換済み。`rust-base-ci.yml`（本リポジトリの `ci.yml` カスタム構成・cargo-deny 導入パターン非整合）・`lint-docs.yml`（npm 経路、REQ-12 非整合）・`cargo-tool-install`（適用先なし）・`idempotent-issue`（自動起票ジョブ現存せず）の 4 件は見送り。判断根拠・比較評価・再評価トリガーの詳細は `docs/ci/actions-new-feature-adoption-evaluation.md` 参照
- **version バンプ PR と crates.io 公開の順序ギャップ（イシュー #884・実装 #885）**: `templates/app` が crates.io バージョン依存する公開済みクレート（core / app / interactive / wasm-client）の `src/` を変更する PR では、version-bump-guard・`template_vendor_drift` テスト・`template-app-wasm-smoke` ジョブの三すくみにより、バンプ先バージョンが crates.io へ未公開の間は smoke が必ず fail する構造的デッドロックが生じる（PR #872 で release.yml のマージ前ブランチ公開が 2 回発生した実例あり）。採用案（smoke ジョブへの `[patch.crates-io]` 依存解決フォールバック追加）は `xtask patch-template-smoke`（`crates/xtask/src/patch_template_smoke.rs`）として実装済み（イシュー #885）: `.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブが「fw new」直後に実行し、生成プロジェクトの直接依存バージョンを crates.io sparse index へ照会（`check_version_bump::query_index` を再利用）して、未公開バージョンのみ `[patch.crates-io]`（checkout 済みリポジトリの `crates/<dir>` への path 参照）へ切り替える。1 行サマリ契約 `template-app-wasm-smoke: dep=<crate> version=<v> resolution=<crates-io|path-override>` により発動有無を可視化し、`resolution=path-override` 発生時は `::warning::` アノテーション + Step Summary 転記でサイレントな弱体化にしない。index 到達不可・異常応答は `environment error: ` プレフィックス付きで fail-closed（version-bump-guard と同型の判定パイプライン）。緩和用の workflow_dispatch input・環境変数は設けていない。crates.io 公開の承認境界（`release.yml` の `mode: publish` 明示選択）は不変。CLI 契約の回帰テストは `crates/xtask/tests/cli_patch_template_smoke.rs`。3 案比較・設計判断の詳細は `docs/ci/version-bump-publish-order-gap.md` を参照。**案 3（マージ後 crates.io 公開の自動化）再評価（イシュー #896）**: 案 2 運用実績（観察期間実質ゼロ、マージ後 release.yml 実行 0 件）を踏まえて再評価し、承認境界・トークン供給経路の論点に状況変化がないため現時点では見送りを継続すると結論した（再評価トリガー・詳細は同文書 §9 参照）

## `ci-complete` 集約ジョブと ruleset 必須チェック

- `.github/workflows/ci.yml` の `ci-complete` ジョブは ci.yml の全ジョブを `needs:` に列挙し、`if: always()` で全結果を検証する集約ジョブである（`success` 以外は FAIL。`skipped` の許容は `version-bump-guard`〔`if: pull_request` の条件付きジョブ〕のみに限定し、他ジョブの skip は検知して FAIL する fail-closed 設計。条件付きジョブを増やす場合は許容リストへの明示追加が必要）
- ruleset `main-protection` の必須チェックはこの集約により `ci-complete` + `deps-check`（別 workflow のため `needs` にできず単独維持）+ `codex-review / codex` の 3 件へ集約されている。ci.yml のジョブ追加・改名時に ruleset の required_status_checks を追随更新する必要はない
- **ci.yml へジョブを追加するときは必ず `ci-complete` の `needs:` へ追加する**（忘れると当該ジョブの失敗が必須チェックに反映されない。レビューで確認する）

## ワークフロー YAML の規約

- ステップ名（`name:` フィールド）に「: 」を含める場合はクォートで囲む（例: `name: "test: verify escaping"` ）
- 理由: 過去に構文エラーで CI 全滅の実績（PR #264 で修正）
- YAML の仕様上、構造化された値（コロン含む）はクォート必須

