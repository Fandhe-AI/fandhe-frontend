# 依存グラフ上限値 運用ポリシー

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-3（依存グラフの浅さ・監査可能性、`docs/spec/04-requirements.md` の REQ-3 節）が求める
「なぜその上限値か」「超過したらどうするか」「上限を守れても残るリスクは何か」を明文化するための成果物です。
Conditional Go 条件 2（依存グラフ上限の要件化）に対応し、`docs/spec/05-tasks.md` の TASK-3.1〜TASK-3.3 系列のうち
TASK-3.3（依存グラフ上限値運用ドキュメントの整備）を担います。

TASK-3.3 は 2 段階に分割されていました。

- **TASK-3.3a（Issue #23・完了）**: 上限値の算出根拠・超過時の対応フロー・サプライチェーンリスクの限界を
  明文化した草案の作成（PR #176、コミット 16eee26）
- **TASK-3.3b（Issue #24・本更新で完了）**: 草案のレビュー反映・確定。TASK-3.2（build.rs 保有クレートの機械的列挙）の
  完了状況を踏まえた第 6 節の確定、`fandhe-frontend-server` 実装後の実測値反映、WASM クライアントクレートのスコープ判断
  （Issue #22 コメント由来）、および Conditional Go 条件 2 の解消判定

**本文書のステータス**: 確定（TASK-3.3b、Issue #22/#24）。第 8 節に解消判定の根拠を記録します。

## 2. 上限値と算出根拠

フレームワーク標準構成（コアクレート・SSR サーバー構成）の依存グラフに対し、次の上限値を設定します。

- **依存パッケージ数**: 標準サーバー構成で解決済み依存パッケージ 60 件以内
- **依存グラフ最大深さ**: 6 以内

根拠は PoC-2（マクロ DSL・Leptos 構成）と PoC-3（純 Rust 方式・`fandhe-frontend-server` 相当構成）の実測差です。

| 構成 | パッケージ件数 | 最大深さ | 出典 |
|------|--------------|---------|------|
| PoC-2（マクロ DSL・Leptos 構成） | 202 | 14 | `docs/spec/04-requirements.md` REQ-3 詳細・概要（25 行目） |
| PoC-3（純 Rust 方式・fandhe-frontend-server 相当構成） | 52 | 5 | `docs/spec/04-requirements.md` REQ-3 詳細・受け入れ基準、`docs/spec/03-poc/` |
| 削減率（PoC-2 → PoC-3） | 約 74% 減 | 約 64% 減 | `docs/spec/04-requirements.md` REQ-3 詳細（PoC-2/PoC-3 実測差の記述） |
| **採用上限**（`MAX_PACKAGES` / `MAX_DEPTH`） | **60** | **6** | PoC-3 実測（52 件/深さ 5）に実装拡張分の余裕を加算。`xtask/src/check_deps.rs` |

コアクレート（`fandhe-frontend-core` / `fandhe-frontend-interactive`）は外部依存パッケージ数 0 件であることを別途受け入れ基準としています
（REQ-3 受け入れ基準 1 点目）。`core/Cargo.toml` への外部クレート追加は `.claude/rules/coding-rust.md` により禁止されています。
この「0 件であること」自体は `check-deps`（60/6 判定）とは別に、`check-core-deps` ゲート（Issue #154）が
`check_deps::ZERO_DEP_CRATES`（`fandhe-frontend-core` / `fandhe-frontend-interactive`）を対象に Normal/Dev/Build すべての辺で強制します。

### 現行実測値（TASK-3.3b 確定時点、origin/main 相当）

`cargo run --locked -p xtask -- check-deps --package fandhe-frontend-core --package xtask --package fandhe-frontend-app
--package fandhe-frontend-dist-server --package fandhe-frontend-server` の実行結果は次のとおりです。

```
deps-check: packages=0/60 depth=0/6 result=PASS   (fandhe-frontend-core)
deps-check: packages=0/60 depth=0/6 result=PASS   (xtask)
deps-check: packages=1/60 depth=1/6 result=PASS   (fandhe-frontend-app)
deps-check: packages=21/60 depth=5/6 result=PASS  (fandhe-frontend-dist-server)
deps-check: packages=0/60 depth=0/6 result=PASS   (fandhe-frontend-server)
```

`fandhe-frontend-dist-server`（`dist-server/`）は REQ-3 が本来対象とする「標準サーバー構成（SSR サーバー相当）」の実体です。
`hyper` + `hyper-util` + `http-body-util` + `tokio` の直接構成を採用しており、`axum` / `rust-embed` はいずれも
依存グラフ深さ上限（6）を構造的に超過するため不採用としました（実測根拠は `dist-server/Cargo.toml` のコメント参照）。
`fandhe-frontend-server`（`server/`）はパスマッチングルーティングのみを担う外部依存ゼロのクレートで、SSR/SSG/単一バイナリ配布の
各エントリから共通利用されます。

## 3. 計測の定義と「正」の所在

計測の実体は `xtask` の `check-deps` サブコマンドです。

```bash
cargo run --locked -p xtask -- check-deps --package <NAME> [--package <NAME> ...]
```

定義（`xtask/src/check_deps.rs` の rustdoc と整合）:

- **件数**: `cargo metadata --format-version 1 --filter-platform <host-triple>` の `resolve.nodes` を正とし、
  ルートパッケージから `DepKind::Normal` 辺のみを辿って到達可能な一意パッケージ数（ルート自身を除く）。
  dev 依存は除外する（PoC-3 の `cargo tree -e normal` と整合）
- **深さ**: ルートを深さ 0 とした最長経路長。dev 依存を除いた解決グラフは DAG であるため、メモ化 DFS により
  厳密に算出する（`cargo tree` の `(*)` 重複省略による過小評価を避ける）
- **プラットフォーム**: `--filter-platform` にホストの target triple を渡し、ホストで有効にならない
  cfg 条件付き依存（target-specific な normal edge）を計測から除外する

しきい値の唯一の正は `xtask/src/check_deps.rs` の `MAX_PACKAGES`（60）・`MAX_DEPTH`（6）・`ZERO_DEP_CRATES`
（`fandhe-frontend-core` / `fandhe-frontend-interactive`。`check-core-deps` が参照）定数です。`--locked` 実行を必須とし、CLI 引数・
環境変数・`continue-on-error` 等による緩和経路は意図的に設けません（迂回経路を作らない設計）。

CI 組み込みは `.github/workflows/deps-check.yml` が担い、fail-closed（PASS/FAIL をそのまま CI の成否に伝播）で
運用します。同ワークフローも `--locked` を必須とし、外側の `cargo run` が `Cargo.lock` を書き換えないことを
保証しています。`check-deps`（60/6 判定）・`check-core-deps`（コアクレート外部依存ゼロ判定、Issue #154）は
それぞれ独立した PASS/FAIL 判定を持つ別ステップとして可視化されます（`format_report` / `format_zero_report` の
1 行サマリ契約はテストで固定されています。`xtask/tests/cli_check_deps.rs` / `cli_check_core_deps.rs`）。

## 4. 計測対象パッケージ

現時点の計測対象は次の 5 パッケージです（`.github/workflows/deps-check.yml` と一致）。

- `fandhe-frontend-core`（ディレクトリは `core/`。外部依存ゼロ契約）
- `xtask`（外部依存ゼロ契約）
- `fandhe-frontend-app`（ディレクトリは `app/`。`fandhe-frontend-core` への path 依存のみ）
- `fandhe-frontend-dist-server`（ディレクトリは `dist-server/`。REQ-3 が対象とする「標準サーバー構成」の実体。
  hyper 直接構成、実測 21 packages/depth 5）
- `fandhe-frontend-server`（ディレクトリは `server/`。パスマッチングルーティングのみ、外部依存ゼロ）

`check-core-deps` は引数を取らず、`check_deps::ZERO_DEP_CRATES` と実 workspace メンバーの積集合を xtask 内部で
自動解決します（`fandhe-frontend-interactive` 等の追加時もワークフロー変更は不要です）。

### WASM クライアントクレートのスコープ（Issue #22 コメント由来の判断）

`fandhe-frontend-wasm-full` / `fandhe-frontend-wasm-thin`（CSR・ハイドレーション用のクライアント側クレート）は本ゲートの計測対象に
**含めません**。理由は次のとおりです。

- REQ-3 の受け入れ基準は「標準サーバー構成（SSR サーバー相当）の解決済み依存パッケージ数・依存グラフ最大深さ」を
  対象と明記しており（`docs/spec/04-requirements.md` REQ-3 受け入れ基準 2 点目）、クライアント側で実行される
  WASM バインディング層はこの定義に含まれません
- `wasm-bindgen` / `web-sys` に由来する依存グラフの深さは、ブラウザ API バインディングという領域の構造的特性
  であり、`fandhe-frontend-dist-server` のように代替クレート選定で回避できる性質のものではありません
- `unsafe` 境界としての監査は `docs/policy/unsafe-boundary.md` のスコープであり、本ポリシーの 60/6 上限とは別の
  観点で担保されます

参考実測（TASK-3.3b 確定時点）:

```
deps-check: packages=20/60 depth=9/6 result=FAIL  (fandhe-frontend-wasm-full)
deps-check: packages=13/60 depth=7/6 result=FAIL  (fandhe-frontend-wasm-thin)
```

（Issue #22 コメント記載の #48 時点実測「wasm-client 20 packages / depth 9」は、クレート再編後の
`fandhe-frontend-wasm-full` の現行実測値と一致します。）

この FAIL 表示はあくまで参考値であり、`deps-check` CI（`.github/workflows/deps-check.yml`）はこれらのクレートを
計測対象に含めていないため CI 上の判定には影響しません。上限緩和・WASM 専用の別基準の新設は行わず、
「WASM クライアント向け基準が別途必要か」は本リポジトリの判断で決めず、REQ-3 の対象定義自体の見直しとして
`frontend-framework-spec` リポジトリへの提案事項に留めます（本リポジトリでは既存ゲートを一切緩和しません）。

## 5. 上限超過時の対応フロー

1. **検出**: `deps-check` CI ジョブが FAIL する（`deps-check: packages=<n>/<limit> depth=<n>/<limit>
   result=FAIL` の 1 行サマリが Step Summary に転記される）
2. **原因分析**: `xtask check-deps` の出力・`cargo tree` を用いて、件数・深さ増加の原因となった依存を特定する
3. **原則対応（依存削減）**: 次の優先順で依存削減を検討する
   1. 不要な feature フラグの削減
   2. より依存の浅い代替クレートへの置き換え
   3. 該当機能の自前実装の検討
4. **依存追加が不可避な場合**: `.claude/rules/coding-rust.md` / `.claude/rules/security.md` に従い、
   `cargo metadata` で影響を事前確認し、`build.rs` の有無を確認したうえで、**ユーザー承認**を得てから追加する
5. **上限値自体の見直しが必要な場合**: 本リポジトリ内では変更しません。上限値は REQ-3（`frontend-framework-spec`
   リポジトリ管理）に由来するため、まず同リポジトリへ仕様変更（REQ-3 改訂）を提案し、承認を経たうえで
   `xtask/src/check_deps.rs` の定数変更 PR（レビュー必須）を行います。CI ワークフロー側に一時的な緩和手順・
   スキップ手順は設けません

## 6. build.rs 保有クレートの監査（監査可能性）

REQ-3 の受け入れ基準は「`build.rs` を持つ依存クレートの一覧が、ビルド成果物または CI ログとして機械的に
列挙できること」を求めています。

この機能は TASK-3.2（Issue #19 系列: #20 TASK-3.2a 列挙ロジック実装・#21 TASK-3.2b CI 出力統合）で実装済みです。
実体は `xtask` の `list-build-scripts` サブコマンド（`xtask/src/list_build_scripts.rs`）です。

```bash
cargo run --locked -p xtask -- list-build-scripts --package <NAME> [--package <NAME> ...]
```

出力契約（`format_inventory`、`xtask/tests/cli_list_build_scripts.rs` で固定）は 1 行サマリ
`build-scripts: target=<name> count=<n>` です。この列挙は PASS/FAIL の概念を持たない監査ログであり、
`build.rs` の存在自体は違反ではありません（禁止クレートのブロックは `cargo-deny` 系タスク TASK-4.x のスコープ）。

`.github/workflows/deps-check.yml` は `check-deps` / `check-core-deps` の成否に関わらず（`if: always()`）
本コマンドを実行し、Step Summary に出力します。

実行例（第 4 節の計測対象パッケージに対する実測）:

```
build-scripts: target=fandhe-frontend-core count=0
build-scripts: target=xtask count=0
build-scripts: target=fandhe-frontend-app count=0
build-scripts: target=fandhe-frontend-dist-server count=3   (httparse, libc, fandhe-frontend-dist-server)
build-scripts: target=fandhe-frontend-server count=0
```

## 7. サプライチェーンリスクの限界（安全性主張のスコープ）

本ポリシーが担保するのは「依存数・深さの相対的な浅さ＝監査コストの低減」であり、次の点について
過大な安全性主張は行いません。

- `build.rs`・手続きマクロによる任意コード実行は、Cargo エコシステム全体に共通する構造的リスクであり、
  本フレームワーク単体の実装方針では解消できません（PoC-1 の空白 B 判定、PoC-2 の逆転発見:
  `docs/spec/04-requirements.md` 25 行目・28 行目）
- 「上限（60 件/深さ 6）を満たしていれば安全」という読み替えは誤りです。上限は監査対象の絶対量を
  抑制するものであり、個々の依存クレートに内在する暗黙実行リスクそのものを排除するものではありません
- メモリ安全性の保証範囲は `core` / `interactive`（`#![forbid(unsafe_code)]` を設定したクレート）に
  限定されます。WASM バインディング層・FFI 依存クレートの残存リスクは `docs/policy/unsafe-boundary.md` の
  スコープです
- 第 4 節で WASM クライアントクレートを本ゲートのスコープ外と記載していますが、これは既存ゲートの
  緩和ではありません。`deps-check` は元々 WASM クレートを対象にしておらず、本ドキュメントはその事実を
  明文化したに過ぎません

## 8. Conditional Go 条件 2 の解消判定

TASK-3.3b（Issue #22/#24）として、以下のレビュー観点を消化しました。

- [x] 算出根拠（第 2 節の数値表）が `docs/spec/04-requirements.md` の記述と一致していることを確認した
- [x] 超過時の対応フロー（第 5 節）が実際の運用として実行可能であることを確認した（依存追加承認フロー
      `.claude/rules/coding-rust.md` / `.claude/rules/security.md` との整合を含む）
- [x] TASK-3.2（build.rs 列挙）完了を受け、第 6 節を実コマンド名・出力形式・実行例で確定記述に更新した
- [x] `fandhe-frontend-server` 実装を受け、第 2 節・第 4 節に標準サーバー構成（`fandhe-frontend-dist-server` / `fandhe-frontend-server`）の
      実測値を反映した（`.github/workflows/deps-check.yml` の計測対象にも `fandhe-frontend-server` を追加した）
- [x] WASM クライアントクレート（`fandhe-frontend-wasm-full` / `fandhe-frontend-wasm-thin`）のスコープ判断（Issue #22 コメント由来）を
      第 4 節に明文化した

**判定**: Conditional Go 条件 2（依存グラフ上限の要件化）は運用として確立しました。しきい値の根拠（第 2 節）・
fail-closed な CI 強制（`check-deps` / `check-core-deps`、第 3 節）・build.rs 監査ログ（`list-build-scripts`、
第 6 節）・計測対象の妥当性（標準サーバー構成のみを対象とし WASM は明示的にスコープ外とする、第 4 節）が
すべて実装・文書化・稼働済みであるためです。最終確認は `docs/spec/06-roadmap.md` が定める MS-1 完了時レビュー
（レビューポイント節）で改めて行われます。

**判定日**: 2026-07-17（TASK-3.3b 確定コミット時点）。

## 9. MAX_DEPTH 上限値・計測方法の再検証（イシュー #298）

### 9.1 経緯

TASK-9.1a（rust-embed 統合設計、`docs/design/dist-server-design.md` 4.3 節）の検討過程で、
本ポリシー第 2 節が引用する「PoC-3 実測: 52 件/深さ 5」という値と、現行の計測実装
（`xtask/src/check_deps.rs` のメモ化 DFS、第 3 節参照）による再計測値が一致しない
ことが発見されました。PR #210 の対象外節に記録されたまま Issue 化されていなかった
事項を、out-of-scope 全数集約でイシュー #298 として起票し、本節でその再検証結果を
確定させます。

### 9.2 齟齬の実体

- PoC-3 の「52 件/深さ 5」（第 2 節の数値表）は `cargo tree -e normal` の**目視インデント
  段数**による計測値であり、`(*)` による重複サブツリーの省略を伴います
- 現行の `xtask` 計測（第 3 節）は `cargo metadata` の `resolve.nodes` に対する
  **メモ化 DFS による厳密な最長経路長**であり、`(*)` 省略による過小評価が起きません
- `docs/design/dist-server-design.md` 4.2 節（表・参考行）は、PoC-3 と同一の依存構成
  （axum + tokio）を現行アルゴリズムで再計測した結果を「50 件/深さ **9**」と記録して
  います。旧基準（cargo tree 目視）の「52/5」と現行基準（メモ化 DFS）の「50/9」は
  **直接比較不能**であり、両者の差は計測方法の違いに起因するものであって、現行実装の
  不具合ではありません

### 9.3 再検証の結論

自動運転・安全側判断として、既存ゲートを一切弱めない前提で次の 4 点を確定します。

1. **計測方法（メモ化 DFS）は変更しません**。`cargo tree` 目視は `(*)` 重複省略により
   深さを過小評価するため、REQ-3 が求める監査可能性には現行の厳密な最長経路長計測が
   適切です。実装の欠陥ではなく、根拠側（PoC-3 目視値）が旧基準だったと整理します
2. **`MAX_DEPTH = 6` / `MAX_PACKAGES = 60` の値は変更しません**。現行計測定義のもとで、
   REQ-3 が対象とする標準サーバー構成の実体 `fandhe-frontend-dist-server` が実測で上限内であり
   （下記 9.4 の再実測参照）、上限値としての運用実効性が裏付けられているためです
3. **根拠の再アンカー**: 「PoC-3 実測 52/5 + 余裕」という第 2 節の説明は現行計測定義と
   比較不能なため、根拠を「旧基準（cargo tree 目視）の PoC-3 値」と「現行基準
   （メモ化 DFS）の再計測値・現行実測値（`fandhe-frontend-dist-server`: 21/5）」を区別して読む
   ことを本節で明文化します。第 2 節・第 3 節の記述自体（TASK-3.3b 確定版）は既に
   この区別を反映済みであり、齟齬の説明が欠けていたのは `xtask/src/check_deps.rs` の
   `MAX_PACKAGES` / `MAX_DEPTH` 定数の rustdoc（PoC-3 値のみを根拠として記載）でした。
   本イシューにあわせて同 rustdoc にも旧基準/現行基準の区別を追記します
4. **仕様（REQ-3）側への注記提案はユーザー承認事項に留めます**。`docs/spec/04-requirements.md`
   の REQ-3 受け入れ基準にある「PoC-3 実績: 52 件/深さ 5 を基準に」という記述には
   計測基準の注記が望ましいですが、`docs/spec/` は編集禁止（サブモジュール）かつ
   Issue 起票は事前承認必須のため、frontend-framework-spec への計測基準注記 Issue の
   起票は本節では**提案のみ**とし、起票自体は行いません

### 9.4 再検証時点の実測値（イシュー #298 判定時点）

```
cargo run --locked -p xtask -- check-deps --package fandhe-frontend-core --package xtask \
  --package fandhe-frontend-app --package fandhe-frontend-dist-server --package fandhe-frontend-server
```

```
deps-check: packages=0/60  depth=0/6 result=PASS  (fandhe-frontend-core)
deps-check: packages=0/60  depth=0/6 result=PASS  (xtask)
deps-check: packages=1/60  depth=1/6 result=PASS  (fandhe-frontend-app)
deps-check: packages=21/60 depth=5/6 result=PASS  (fandhe-frontend-dist-server)
deps-check: packages=2/60  depth=2/6 result=PASS  (fandhe-frontend-server)
```

`fandhe-frontend-server` は第 2 節記載時点（0 件/深さ 0）から 2 件/深さ 2 へ増加していますが、
上限（60/6）に対して十分な余裕があり、判定結果（PASS）に変わりはありません。この
差分自体が「実測値は経時的にドリフトしうるため定期的な再検証が必要」という本節の
趣旨を裏付けます。参考実測（`fandhe-frontend-wasm-full`: 20/60・9/6・FAIL、`fandhe-frontend-wasm-thin`:
13/60・7/6・FAIL）は第 4 節記載値と一致し、変化なしを確認しました。

### 9.5 axum / rust-embed 系スタックへの含意（スコープ外・条件付き整理）

`docs/design/dist-server-design.md` 4.3 節のとおり、axum / rust-embed 系スタックを標準サーバー
構成として採用する場合、現行アルゴリズムでは深さ 9 前後となり現行上限（6）で構造的に
FAIL します。この構成を将来採用するには、frontend-framework-spec 側での REQ-3 改訂
（計測基準を踏まえた上限値の再設計）が前提となります。本イシューはこの前提整理の
確認に留まり、上限値自体の変更判断は行いません（第 7 節「対応外」参照）。

**判定日**: 2026-07-18（イシュー #298 再検証確定時点）。
