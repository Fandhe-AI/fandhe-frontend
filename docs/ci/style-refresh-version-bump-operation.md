# 107 部品スタイル調整の semver バンプ・dep-version 追随運用（イシュー #1429）

## 1. 背景・目的

UI 部品スタイル調整イシューツリー（ルート #1420、Phase 1〜13 で Themes 107 部品 + Primitives 63 部品）では `crates/pre-styled-ui/src/`・`crates/headless-ui/src/` を触る PR が 100 本超発生する見込みである。これら 2 クレートは crates.io へ公開済みのため、実体変更 PR は `.claude/rules/coding-rust.md`（イシュー #638）の成文規定によりバンプが必須になり、バンプは `xtask check-dep-versions`（`dep-version-check` ジョブ）による依存元の `version = "..."` 完全一致要求の追随を連鎖させ得る。Phase 1 着手前に運用を確定し、本文書として Phase 1 以降の全 issue から参照できるようにする。

## 2. 事実確認（運用決定の根拠）

- **機械判定と成文規約のずれ**: `version-bump-guard` ジョブが呼ぶ `xtask check-version-bump`（`crates/xtask/src/check_version_bump.rs`）は「`version` が crates.io 既公開バージョン集合に含まれる」場合のみ FAIL する。一方 `.claude/rules/coding-rust.md` は「公開済みクレートの `src/`・`Cargo.toml`・`build.rs` を変更する PR は必ず `version` をバンプする（または `version-bump-exempt:` 宣言で免除する）」と成文規定している。両者の関係は「機械判定は最終防波堤、成文規約が本来のルール」であり、本運用は成文規約に従う。
- **per-PR patch バンプの先例が既に存在する**: 本文書作成時点（2026-08-31）のワークスペースは `crates/pre-styled-ui/Cargo.toml` が `version = "0.47.0"`（crates.io 公開は 0.40.0 系）、`crates/headless-ui/Cargo.toml` が `version = "0.28.6"`（公開 0.28.0）、`crates/wasm-full/Cargo.toml` が `version = "0.7.11"`（公開 0.7.1）であり、未公開のまま PR 毎に patch を積む運用が既に実績を持つ（並列 PR のマージにより上記具体値は本文書公開後も随時進行する。正確な現在値は各 `crates/*/Cargo.toml` の `version` を参照し、本文書中の具体値は参考実績としてのみ扱う）。`crates/pre-styled-ui/Cargo.toml` 冒頭コメント（#1388「依存追随に伴う patch バンプ」）も同型。
- **pre-styled-ui 単体のバンプは dep 追随を発生させない（ただし Cargo.lock 同期は別途必要）**: `version = "..."` 併記で `fandhe-frontend-pre-styled-ui` に依存するクレートは存在しない。`crates/docs-site/Cargo.toml` は `fandhe-frontend-pre-styled-ui = { path = "../pre-styled-ui" }`（version 併記なし）であり、同ファイル冒頭コメントに「本クレートは `publish = false` のため、workspace 内 path 依存に version 併記は不要（`xtask check-dep-versions` のルール 2 は publish 対象クレートのみが対象）」と明記済み。**Themes（`crates/pre-styled-ui/`）のみを触る Phase の PR は他クレートの `version = "..."` 追随こそ不要だが、ルート `Cargo.lock` はワークスペースメンバーである `fandhe-frontend-pre-styled-ui` パッケージ自身の `version` を記録しているため、`Cargo.toml` の `version` を変更したら必ず `cargo metadata --no-deps -q >/dev/null`（または `cargo build`/`cargo check` 等の通常操作）を 1 回実行して `Cargo.lock` を新バージョンへ同期し、更新された `Cargo.lock` をコミットに含める（`fw gate`・CI の `--locked` 系コマンドは `Cargo.lock` と `Cargo.toml` の不一致を fail-closed で検知して失敗する）。詳細な独立手順は §3.4a 参照。**
- **headless-ui のバンプのみ 3 箇所へ連鎖する**: `fandhe-frontend-headless-ui` を `version = "0.28.6"` 併記で依存するのは `crates/wasm-full/Cargo.toml`・`crates/pre-styled-ui/Cargo.toml`・`crates/xtask/Cargo.toml` の 3 箇所。このうち `pre-styled-ui`・`wasm-full` は公開対象クレートのため依存元自身も patch バンプが必要（`dep-version-check` ルール 1: `req == "^" + 依存先の現行 version` の完全一致要求）。`xtask` は `publish = false` のため Cargo.toml の version 要求追随のみでよく、自身のバンプは不要。
- **crates.io 公開デッドロック（#884/#1306）は非該当**: `templates/app`・`templates/app/wasm` は `fandhe-frontend-core`/`fandhe-frontend-app`/`fandhe-frontend-wasm-client` のみに依存し、headless-ui / pre-styled-ui のバンプは `template-app-wasm-smoke` ジョブにも `templates/app/wasm/Cargo.lock` にも影響しない。`examples/headless-pre-styled-ui` は公開済みバージョン（`^0.40.0` 系）への caret 依存であり、未公開の patch バンプの影響を受けない。したがって同時公開フロー（`docs/ci/version-bump-publish-order-gap.md` §10）の適用は不要である。
- **theme.rs → docs-site 契約テストへの依存辺**: `crates/pre-styled-ui/src/theme.rs` のトークン変更は `crates/docs-site/tests/site_css_contract.rs`・`crates/docs-site/tests/site_typography_contract.rs` を落とし得る。この依存辺は #1422・#1423・#1425 の本文には明示済みである一方、**#1424（フォーカスリング・size バリアント規約統一）のみ「該当 golden テスト」という抽象表記に留まり、上記 2 ファイルへの明示参照がない**。#1424 側での補記は本文書公開後にフォローする。

## 3. 決定事項

自動運転での実装のため、既存の成文規約・fail-closed 契約を一切弱めない選択肢を採る。本文書は PR レビュー・マージをもってユーザー承認を得る（本節が承認の扱いの正であり、他に独立した「承認の扱い」節は設けない）。

### 3.1 バンプ粒度: PR 毎に patch バンプ

`crates/pre-styled-ui/src/`・`crates/headless-ui/src/` を変更する PR は、その PR 内で対象クレートの `version` を patch バンプする（例: `0.46.0` → `0.46.1`）。

- 根拠: `.claude/rules/coding-rust.md` の成文規定にそのまま従い、規約変更・`version-bump-exempt:` の解釈拡大を必要としない。§2 の既存実績（0.40.0 系 → 0.47.0 等、具体値は随時進行）と同型であり、バンプは crates.io 公開を伴わない。ただし `Cargo.toml` の `version` 変更は必ずルート `Cargo.lock` の同期（§3.4a）とセットで行う（`Cargo.lock` 未更新のまま `Cargo.toml` だけ変更すると `--locked` を使う既存処理が失敗する）。
- 不採用案: 「Phase 単位でまとめてバンプし個別 PR は `version-bump-exempt:` で免除」は、CSS 変更を免除対象と解釈する規約の弱体化を要するため不採用（§3.2 参照）。「作業ブランチへバンプをまとめて集約」は `implement-issue-tree` の並列 worktree 運用と干渉するため不採用。

### 3.2 `version-bump-exempt:` の CSS 変更への適用: 不可

`css()` / `Theme::to_css` が出力する CSS 文字列は公開クレートの観測可能な振る舞いであり、`.claude/rules/coding-rust.md` が定める免除条件「公開 API に影響しない変更（ドキュメントのみ・内部実装のみ等）」に該当しない。免除が使えるのは rustdoc・コメントのみの変更等、出力に影響しない変更に限る。107 部品のスタイル調整 PR は原則すべて patch バンプの対象である。

### 3.3 CSS 出力変更の破壊的変更判定: 原則 patch、契約破壊のみ minor

- Rust API シグネチャが不変で、意匠（色・余白・影等の値）の調整に留まる変更は patch バンプとする。
- CSS 変数名・クラス名・`data-*` セレクタの削除/改名など、利用者の上書き CSS・セレクタを壊す変更は 0.x の破壊的変更として minor バンプとする（`.claude/rules/coding-rust.md` の既存規定「0.x の破壊的変更はマイナーバンプ」どおり）。
- 107 部品調整の大半は意匠調整（patch）想定だが、Phase 4（focus ring / size バリアント統一、#1424）等でセレクタ変更を伴う場合は minor 判定を個別に検討する。

### 3.4 依存元追随の定型化

バンプを伴う PR では、コミット前に `cargo run -p xtask -- check-dep-versions --fix` を実行し、依存元の `version = "..."` 要求を対象クレートの新バージョンへ追随させる（自動修正できるのは version 不一致のみ。version 欠落〔`req == "*"` かつ依存元が publish 対象〕は `--fix` でも直らないため手動対応）。headless-ui をバンプした場合の連鎖手順は次のとおり（#1388 の先例と同型）:

1. `crates/headless-ui/Cargo.toml` の `version` を patch バンプする。
2. `cargo run -p xtask -- check-dep-versions --fix` を実行し、`crates/wasm-full/Cargo.toml`・`crates/pre-styled-ui/Cargo.toml`・`crates/xtask/Cargo.toml` の `version = "..."` 要求を追随させる。
3. `wasm-full`・`pre-styled-ui` は依存元自身も公開対象クレートのため、これらの `version` も patch バンプする（`xtask` は `publish = false` のためバンプ不要、追随のみで足りる）。
4. `cargo run -p xtask -- check-dep-versions`（`--fix` なし）を再実行し、FAIL が残っていないことを確認する。
5. `cargo check`（ワークスペース全体、または少なくとも `-p fandhe-frontend-headless-ui -p fandhe-frontend-pre-styled-ui -p fandhe-frontend-wasm-full`）を実行してルート `Cargo.lock` を上記 3 クレートの新バージョンへ同期し、更新された `Cargo.lock` をコミットに含める（§3.4a 手順 2〜3 と同じ理由。`Cargo.lock` を置き去りにすると `--locked` を使う既存処理が失敗する）。

pre-styled-ui のみを変更する PR（Themes Phase の大半）は上記 headless-ui 連鎖手順（手順 1〜4）を実行する対象ではない（手順 1 は `crates/headless-ui/Cargo.toml` のバンプであり、pre-styled-ui のみを変更する PR でこれを実行すると変更していない headless-ui を不要にバンプしてしまう）。pre-styled-ui 単体の場合は §3.4a の独立手順に従う。

### 3.4a pre-styled-ui 単体変更時の独立バンプ手順

`crates/pre-styled-ui/src/` のみを変更し `crates/headless-ui/` に変更がない PR（Themes Phase の大半）は、headless-ui のバンプ・依存元追随を一切行わず、次の独立手順のみで完結する（§2 のとおり `fandhe-frontend-pre-styled-ui` に `version = "..."` 併記で依存するクレートが存在しないため、依存元追随は発生しない）。

1. `crates/pre-styled-ui/Cargo.toml` の `version` を patch（または §3.3 の破壊的変更時は minor）バンプする。
2. `cargo check -p fandhe-frontend-pre-styled-ui`（または `cargo metadata --no-deps -q`／`cargo test -p fandhe-frontend-pre-styled-ui` 等、Cargo.lock を書き換える任意の通常操作）を実行し、ルート `Cargo.lock` 内の `fandhe-frontend-pre-styled-ui` パッケージエントリの `version` を新バージョンへ同期する。
3. `git status`／`git diff Cargo.lock` で `Cargo.lock` が更新されていることを確認し、`Cargo.toml` と同一コミットへ含める（`Cargo.lock` を置き去りにしない。`fw gate`・CI の `--locked` 系コマンドは不一致を fail-closed で検知する）。
4. `cargo run -p xtask -- check-dep-versions`（`--fix` なし）を実行し、FAIL が発生しないことを確認する（pre-styled-ui 単体変更では通常 FAIL しないことの再確認）。

### 3.5 crates.io 公開タイミング

Phase 完了ごとに `main` ブランチから `.github/workflows/release.yml`（`workflow_dispatch`、`mode: publish` を明示選択）を実行し、**そのフェーズでバンプされ、かつ crates.io へ未公開のクレートのみ**を、依存順（headless-ui → pre-styled-ui → wasm-full の順序を守りつつ、対象になっているものだけを実行）で公開する。

- **固定 3 クレート同時公開ではない**: Themes のみを触った Phase（pre-styled-ui 単体のバンプ、§3.4a）では headless-ui・wasm-full はバンプされておらず、既に crates.io へ公開済みの現行バージョンのまま変わらない。この 2 クレートに対して `release.yml` の `mode: publish` を実行すると、`verify` ジョブの既公開バージョン検証（`.claude/rules/ci.md` の release ワークフロー節参照）が「version already published」として fail-closed に停止するため、**実行しない**。公開対象は毎回 `git diff`（前回公開コミット以降）または各 `Cargo.toml` の `version` と crates.io sparse index の突合で「バンプ済み・かつ未公開」と確認できたクレートに限定する。
- 複数クレートが対象になる場合（headless-ui をバンプした Phase 等）は、依存元が依存先の新バージョンへ追随済み（§3.4）であることを前提に、headless-ui → pre-styled-ui → wasm-full の順で該当クレートのみ公開する。
- 本件は §2 で確認したとおり `templates/app` 系のビルドに影響しないため #884/#1306 のデッドロックに該当せず、PR ブランチからの先行公開（同時公開フロー、`docs/ci/version-bump-publish-order-gap.md` §10）は適用しない。
- `mode: publish` の明示選択・`CARGO_REGISTRY_TOKEN` のステップ限定注入という既存の承認境界・トークン供給経路は変更しない。

### 3.6 theme.rs → docs-site 契約テストの依存辺の明記

`crates/pre-styled-ui/src/theme.rs` を変更する issue には `crates/docs-site/tests/site_css_contract.rs`・`crates/docs-site/tests/site_typography_contract.rs` への影響を明記する。#1422・#1423・#1425 は明記済み。#1424 は「該当 golden テスト」の抽象表記に留まり明示参照がないため、本文書公開後に補記コメントを追加する。

## 4. 部品 PR 用チェックリスト（雛形）

Phase 1 以降の各部品 PR は以下を満たすことを確認する。

- [ ] 対象クレート（`fandhe-frontend-pre-styled-ui` および/または `fandhe-frontend-headless-ui`）の `version` を patch（意匠調整のみ）または minor（セレクタ・CSS 変数名等の破壊的変更、§3.3）でバンプした
- [ ] headless-ui をバンプした場合は `cargo run -p xtask -- check-dep-versions --fix` を実行し、`wasm-full` / `pre-styled-ui` / `xtask` の依存元追随・バンプ連鎖（§3.4）を完了した
- [ ] `Cargo.toml` の `version` を変更したクレート分について、ルート `Cargo.lock` の同期（§3.4 手順 5／§3.4a 手順 2〜3）を実行し、更新された `Cargo.lock` を同一コミットに含めた
- [ ] `cargo test -p fandhe-frontend-pre-styled-ui` および/または `cargo test -p fandhe-frontend-headless-ui` が green
- [ ] `theme.rs` を変更した場合は `cargo test -p fandhe-frontend-docs-site`（`site_css_contract` / `site_typography_contract`）が green
- [ ] golden テスト更新手順は #1427 の手順に従う
- [ ] `version-bump-exempt:` を使う場合は「公開 API に影響しない変更」であることを確認する（CSS 出力を変える変更には使わない、§3.2）

## 5. version 行コンフリクトの解決手順（並列 PR 間）

複数の部品 PR が同一クレートの `version` 行を並行してバンプするため、rebase 時に `Cargo.toml` の `version = "..."` 行がコンフリクトし得る。以下の手順で機械的に再解決する（どちらの番号を取るかで迷わない）。

1. `git rebase origin/main` 等でコンフリクトが発生したら、`main` 側の現在の `version` 値を確認する。
2. 自分の変更が patch バンプであれば `main` の現在値の patch を +1 した値を採用する（例: `main` が `0.46.3` なら `0.46.4`）。minor バンプ（§3.3 の破壊的変更）であれば `main` の現在値の minor を +1 し patch を 0 に戻す。
3. 他の行（自分が変更した実装・CSS）はそのまま残す。
4. 解決後、`cargo run -p xtask -- check-dep-versions` を再実行し、依存元追随が崩れていないことを確認する。

## 6. 再評価トリガー

以下のいずれかに該当する場合、本運用（特に §3.1 の per-PR patch バンプ）を見直す。

- 1 Phase あたりの `version` 行コンフリクト解決（§5）が慢性的に発生し、実装速度のボトルネックになった場合
- headless-ui のバンプ連鎖（§3.4）が想定より高頻度に発生し、`wasm-full`/`pre-styled-ui` の不要な patch バンプが蓄積した場合
- crates.io 公開間隔（§3.5）が開きすぎ、未公開バージョン番号が実運用上追いづらくなった場合

## 7. 関連文書

- `.claude/rules/coding-rust.md`（semver バンプ必須の成文規定、免除条件）
- `.claude/rules/ci.md`（`version-bump-guard` / `dep-version-check` ジョブの機械判定契約）
- `docs/ci/version-bump-publish-order-gap.md`（crates.io 公開順序ギャップ・同時公開フロー §10）
- `crates/xtask/src/check_version_bump.rs` / `crates/xtask/src/check_dep_versions.rs`（機械判定の実装）
- ルート #1420、Phase 親 #1430/#1443/#1514/#1552/#1579/#1588/#1600/#1612/#1624/#1635/#1646/#1658/#1669
