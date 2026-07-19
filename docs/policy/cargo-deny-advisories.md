# cargo-deny advisories チェックのオンライン CI 運用手順

## 1. 目的・位置づけ

本ドキュメントは REQ-4（AI 生成コード・依存の検証フック、`docs/spec/04-requirements.md`）・TASK-4.3
（`docs/spec/05-tasks.md` 106〜111 行目）の成果物です。PoC-7（`docs/spec/03-poc/ai-self-maintenance/README.md`
114 行目・122 行目）で判明した次の制約への対応として、`cargo deny check advisories`（既知脆弱性データベース
照合）を CI 環境でネットワークアクセス前提のもと運用する手順を確定します。

> `cargo-deny` の `advisories` チェック（既知脆弱性 DB 照合）はオフライン環境では実行できなかった。
> 実運用では CI 環境でのネットワークアクセスを前提に `advisories` を含めるべきである。

TASK-4.1（`templates/default/deny.toml`）・TASK-4.2（`templates/default/.github/workflows/deny.yml`）との
役割分担は次のとおりです。

| チェック | 実行可否（オフライン） | 担当タスク・成果物 |
|---------|----------------------|-------------------|
| `bans` / `licenses` / `sources` | 可（追加ネットワークアクセス不要） | TASK-4.1（`deny.toml`）・TASK-4.2（`deny.yml`） |
| `advisories` | 不可（RustSec Advisory DB 取得が必須） | **TASK-4.3（本ドキュメント）** |

`templates/default/deny.toml` の `[advisories]` セクション・`templates/default/.github/workflows/deny.yml` の
ヘッダコメントは、いずれも「advisories を含めたオンライン CI 運用手順は本ドキュメントに委譲する」と
明記しています。本ドキュメントはその委譲先です。

## 2. 制約の背景

`cargo deny check advisories` は [RustSec Advisory Database](https://github.com/rustsec/advisory-db) を
git 経由（または cargo-deny の advisory-db キャッシュ機構経由）で取得し、依存グラフの各クレート・
バージョンと照合します。この DB 取得にはネットワークアクセスが必須であり、オフライン環境では
実行そのものが失敗します（PoC-7 実測）。

このため、本フレームワークの標準テンプレート（`templates/default/deny.toml`・
`templates/default/.github/workflows/deny.yml`）では、ネットワークアクセスを前提にしない
ローカルでのゲート実行を次の 3 チェックに限定しています。

```bash
cargo deny check bans licenses sources --config deny.toml
```

`advisories` を含めた完全なチェックは、ネットワークアクセスが保証される CI 環境でのみ実行する
運用とし、その手順を以下に定めます。

## 3. オンライン CI の前提条件

- **ネットワークアクセス**: CI ランナーから次の 2 か所への HTTPS アクセスが必要です。
  - `crates.io`（`cargo install` によるツールチェーン取得・依存解決）
  - `github.com`（RustSec Advisory DB の取得元。cargo-deny は内部で advisory-db リポジトリを
    クローンまたは更新します）
- **cargo-deny のバージョン固定**: `templates/default/.github/workflows/deny.yml`・
  `tools/ci/ensure-gate-tools.sh`（本フレームワーク自身の CI・イシュー #314）と同一の、
  バージョン固定 + SHA256 チェックサム検証付きプリビルトバイナリを用います。`cargo install
  cargo-deny` によるソースからの任意最新版コンパイルは、advisory-db のフォーマット変更や
  コマンド仕様変更を予期せず取り込むリスクがあるため避けます。バージョン・SHA256 の pin の正は
  `tools/ci/ensure-gate-tools.sh` の `CARGO_DENY_VERSION` / `CARGO_DENY_SHA256` であり、
  本ドキュメント・テンプレートの pin 値がそこからドリフトしていないことは
  `crates/xtask/tests/template_deny_workflow.rs` が `cargo test -p xtask` / CI で強制検知します。
- **第三者製 Action を使わない**: `templates/default/.github/workflows/deny.yml` の方針
  （`EmbarkStudios/cargo-deny-action` 等の第三者 Action を採用しないサプライチェーン方針）を
  オンライン運用でも踏襲します。Action を参照する場合はフル SHA 固定とします。

## 4. 実行手順

### 4.1 単発実行

```bash
cargo deny check advisories --config deny.toml
```

`deny.toml` の `[advisories]` セクション（`templates/default/deny.toml` 47〜61 行目）は cargo-deny の
既定値のまま（`ignore = []`）とし、個別クレートの除外を追加しません。

### 4.2 advisory-db の事前取得（任意・高速化目的）

CI ジョブを繰り返し実行する場合、advisory-db を都度フル取得すると実行時間が伸びます。次のコマンドで
事前取得・更新できます。

```bash
cargo deny fetch
```

**注意**: 事前取得・キャッシュはあくまで実行時間短縮のための最適化であり、DB の鮮度を犠牲にしては
なりません。CI 環境のキャッシュ機構（例: `actions/cache`）を用いる場合も、キャッシュキーに
日付や advisory-db の最新コミットハッシュを含める等、鮮度が古いキャッシュを無期限に使い続けない
設計にします。キャッシュヒットのみで advisory-db を一切更新しない運用は、新規に公開された
脆弱性を検出できなくなるため避けます。

## 5. CI ワークフロー例（コード例）

以下は `templates/default/.github/workflows/deny.yml`（bans/licenses/sources を fail-closed で
強制する既存ワークフロー）と同一の設計方針（`permissions: contents: read` のみ、Action はフル SHA
固定、`concurrency` で重複実行を抑止、`continue-on-error` や `|| true` を設けない fail-closed 設計）を
`advisories` チェック向けに拡張したコード例です。テンプレートへの実装組み込み自体は本タスク
（TASK-4.3、ドキュメント化タスク）のスコープ外とし、別途 Issue 化の要否をユーザーに諮ります
（第 7 節参照）。

```yaml
name: deny-advisories

# advisories（既知脆弱性 DB 照合）は RustSec Advisory DB のネットワーク取得を要するため
# bans/licenses/sources（templates/default/.github/workflows/deny.yml）とは別ジョブ・
# 別スケジュールで運用する（docs/policy/cargo-deny-advisories.md 参照）。
on:
  pull_request:
  push:
    branches: [main]
  schedule:
    # コード変更がなくても新規 advisory の公開でジョブが失敗し得るため、
    # 定期実行そのものが本チェックの本質である（毎日 UTC 00:00 に実行）。
    - cron: "0 0 * * *"

permissions:
  contents: read

concurrency:
  group: deny-advisories-${{ github.ref }}
  cancel-in-progress: true

jobs:
  deny-advisories:
    name: REQ-4 known-vulnerability gate (cargo-deny advisories)
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@4cda84d5c5c54efe2404f9d843567869ab1699d4
        with:
          toolchain: stable

      - name: Install cargo-deny
        # tools/ci/ensure-gate-tools.sh・templates/default/.github/workflows/deny.yml
        # と同一の「バージョン固定 + SHA256 検証済みプリビルトバイナリ」パターン
        # （イシュー #314）。pin 値のドリフトは xtask テストが強制検知します。
        run: |
          set -euo pipefail
          CARGO_DENY_VERSION="0.19.8"
          CARGO_DENY_SHA256="70e769ae3872e34d45132b17040859175e11401dc12dddb0303e0b8c7d088f3f"

          existing_version=""
          if existing_version="$(cargo deny --version 2>/dev/null)"; then
            existing_semver="$(awk '{print $2}' <<<"${existing_version}")"
            if [ "${existing_semver}" = "${CARGO_DENY_VERSION}" ]; then
              echo "cargo-deny ${CARGO_DENY_VERSION} は導入済みのためインストールをスキップする"
              exit 0
            fi
          fi

          archive="cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl.tar.gz"
          url="https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/${archive}"
          tmp_dir="$(mktemp -d)"
          curl -sSfL -o "${tmp_dir}/${archive}" "${url}"
          echo "${CARGO_DENY_SHA256}  ${tmp_dir}/${archive}" | sha256sum -c -
          tar xzf "${tmp_dir}/${archive}" -C "${tmp_dir}"
          install_dir="${tmp_dir}/cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl"
          echo "${install_dir}" >> "${GITHUB_PATH}"

      - name: Run cargo deny check (advisories)
        # fail-closed: 終了コードをそのまま CI の成否に伝播させる。
        # continue-on-error・|| true は設けない。
        run: |
          cargo deny check advisories --config deny.toml
```

`schedule:`（定期実行）を含める点が bans/licenses/sources 用ワークフローとの本質的な違いです。
コードの変更がなくても、新規に RustSec Advisory が公開された時点で既存依存が該当し得るため、
プッシュ・PR 契機だけでは検出漏れが生じます。定期実行によって「コード変更なしで新規失敗する」
運用こそが本チェックの目的です。

## 6. 検出時の対応フロー

1. **第一選択（依存の更新・差し替え）**: 該当クレートを修正版に更新するか、代替クレートへ
   置き換えます。多くの場合はこれで解消します。
2. **`[advisories] ignore` への追加（緩和手続き）**: どうしても即時の更新・置き換えができない場合に
   限り、`deny.toml` の `[advisories] ignore` に該当 RUSTSEC ID を追加して一時的に無視できます。
   ただしこれは既存 `deny.toml` のセキュリティ不変条件（`templates/default/deny.toml` 12〜15 行目）
   と同じ考え方で、次を必須とします。
   - 追加理由・影響評価（該当脆弱性が本フレームワークの利用形態で実害となるか）・解消期限を
     `ignore` エントリの直上にコメントで記録する
   - **人間の承認を必須とする**。AI エージェントが自己判断で `ignore` へ追加してはなりません
     （`.claude/rules/security.md`・`.claude/rules/coding-rust.md` の依存追加承認フローと同じ扱い）
   - 期限を過ぎても解消していない場合は Issue で追跡し、放置しません
     （`.claude/rules/out-of-scope-tracking.md` 準拠）
3. **DB 側の誤検知が疑われる場合**: RustSec Advisory Database 側への異議申し立て・修正 PR を
   検討します。本リポジトリ側での独自の除外判定ロジックは追加しません。

## 7. 運用上の注意

- **定期実行の失敗は放置しない**: `schedule:` トリガーによる失敗はコードレビューの文脈を伴わないため
  埋もれやすい傾向があります。失敗を検知した場合は Issue を起票し、`.claude/rules/out-of-scope-tracking.md`
  のフローに従って追跡します。
- **ネットワーク断とポリシー違反を区別する**: advisory-db の取得自体が失敗した場合（ネットワーク断・
  GitHub 側の障害等）と、advisories 照合の結果ポリシー違反が検出された場合は、原因が異なります。
  前者は CI ランナー・ネットワーク環境の問題であり、後者は依存クレートの脆弱性対応が必要な問題です。
  ログ・エラーメッセージを確認し、取得失敗であれば再実行を検討し、ポリシー違反であれば第 6 節の
  対応フローに進みます。
- **fail-open にしない**: `continue-on-error: true` や `|| true` によってジョブ失敗を握りつぶす
  実装は行いません。取得失敗時にジョブが失敗すること自体は、検証未実施のまま「安全」の誤ったシグナルを
  出さないための意図した挙動です。
- **テンプレートへの実装組み込みは別スコープ**: 本ドキュメントはコード例の提示に留め、
  `templates/default/.github/workflows/deny.yml` への `advisories` ステップの実装追加は行いません
  （第 5 節参照）。実装組み込みを Issue 化するかどうかはユーザー承認を得てから判断します。

## 8. 関連ファイル

- `templates/default/deny.toml`（TASK-4.1・`[advisories]` セクション）
- `templates/default/.github/workflows/deny.yml`（TASK-4.2・bans/licenses/sources の CI 強制）
- `crates/xtask/tests/template_deny_config.rs` / `crates/xtask/tests/template_deny_workflow.rs`（テンプレートの回帰テスト）
- `docs/spec/03-poc/ai-self-maintenance/README.md`（PoC-7・advisories オフライン実行不可の実測根拠）
- `docs/spec/04-requirements.md`（REQ-4）・`docs/spec/05-tasks.md`（TASK-4.3）
