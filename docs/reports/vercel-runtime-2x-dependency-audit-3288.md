# `vercel_runtime 2.4.1` 依存グラフ計測レポート（イシュー #3288）

## 1. 結論

`vercel_runtime 2.4.1` を `examples/vercel-ssr` 相当の構成で依存に加えた場合、
REQ-3（標準サーバー構成で解決済み依存パッケージ 60 件以内・依存グラフ最大深さ
6 以内）を**構造的に超過**します。件数・深さのいずれも上限超過であり、
`vercel_runtime` の feature では依存を削減できません。

この結果は `docs/design/vercel-deployment-strategy.md` §7 の再評価トリガー
「`vercel_runtime 2.x` の依存木が REQ-3 上限を超える、`build.rs` を持つ依存が
見つかる、cargo-deny の advisories に違反する、のいずれかが判明したとき（→
案 a の併用を取り下げて案 c 単独にする）」のうち、件数・深さ超過の条件に
該当します。これを受けて同文書を改訂し、案 a（`vercel_runtime` の併用）を
取り下げて案 c（SSG + Build Output API + `--prebuilt`）単独を Vercel 上の
既定方式として確定しました。

## 2. 検証環境

- 実行日: 2026-09-27
- `rustc 1.98.1 (48a229cea 2026-09-01)` / `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- 計測はリポジトリ外の一時ディレクトリ（`mktemp -d` 相当のスクラッチ領域）で
  行い、計測完了後に削除しています。本リポジトリの `crates/*` や
  `examples/*` には `vercel_runtime` を一切追加していません。
- ホスト target: `aarch64-apple-darwin`（macOS）。Vercel のビルド環境相当
  として `x86_64-unknown-linux-gnu` も別途計測しています。

## 3. 計測の定義

`crates/xtask/src/check_deps.rs`（REQ-3 の計測実装）と同一の定義を用いています。

- **件数**: ルートパッケージから通常依存（`DepKind::Normal`、`dep_kinds[].kind`
  が `null`）の辺のみを辿って到達可能な一意パッケージ数（ルート自身を除く）。
  dev 依存は除外。
- **深さ**: ルートを深さ 0 とした最長到達経路長。メモ化 DFS で算出（`cargo
  tree` の `(*)` 重複省略による過小評価を避けるため）。
- **プラットフォーム**: `cargo metadata --format-version 1 --filter-platform
  <triple>` でホスト（または対象プラットフォーム）向けに解決されない
  cfg 条件付き依存を除外。

## 4. 再現手順

一時ディレクトリに次の `Cargo.toml` を持つ独立クレートを作成しました
（`[workspace]` を明示し、リポジトリの workspace に取り込まれないようにして
います）。

```toml
[package]
name = "vercel-dep-audit-3288"
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace]

[dependencies]
vercel_runtime = "=2.4.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
fandhe-frontend-core = "0.4.3"
fandhe-frontend-app = "0.2.6"
fandhe-frontend-server = "0.2.6"
```

実行コマンド:

```bash
cargo generate-lockfile
cargo metadata --format-version 1 --filter-platform x86_64-unknown-linux-gnu > meta-linux.json
cargo metadata --format-version 1 --filter-platform aarch64-apple-darwin > meta-darwin.json
```

得られた `cargo metadata` の JSON（`packages[]` / `resolve.nodes[]`）に対し、
`check_deps.rs` と同じアルゴリズム（BFS による到達可能集合・メモ化 DFS に
よる最長経路）を実装した短いスクリプトで件数・深さを算出しました
（スクリプト自体はリポジトリに含めていません）。

最長経路の確認には `cargo tree -e normal --target x86_64-unknown-linux-gnu -i
unicode-ident` を使い、経路上の各クレートを目視で裏付けています。

`build.rs` の有無は `packages[].targets[].kind == ["custom-build"]` を走査して
列挙しました。

## 5. 計測結果

| 対象 | パッケージ数 | 最大深さ | 判定（上限 60 / 6） |
|---|---|---|---|
| 一時クレート全体（`x86_64-unknown-linux-gnu`。Vercel のビルド環境相当） | 66 | 11 | FAIL（件数・深さとも超過） |
| 一時クレート全体（`aarch64-apple-darwin`。ローカル） | 71 | 11 | FAIL |
| `vercel_runtime` 単体のサブツリー（linux） | 62 | 10 | 単体でも FAIL |
| `vercel_runtime` 単体のサブツリー（darwin） | 67 | 10 | 単体でも FAIL |

`Cargo.lock` は 78 パッケージを解決しています（`fandhe-frontend-core` /
`-app` / `-server` の 3 クレートおよびそれ自身とルートを含む総数）。

### 最長経路の例（linux）

```
vercel_runtime → hyper-util → futures-util → futures-macro (proc-macro)
  → proc-macro2 → unicode-ident
```

深さは `vercel_runtime` を深さ 1 として数えるため、上記経路は深さ 6 に
達し、`tokio`（`tokio-macros` 経由で `syn` → `proc-macro2` → `unicode-ident`）
を辿る経路も同様に深さ 6 を超えます。

### feature で依存を削れない根拠

`vercel_runtime 2.4.1` の `Cargo.toml` は `default = []` であり、`axum` /
`actix` の 2 つのみが optional feature です。`tokio`（`full`）・`hyper`
（`full`）・`hyper-util`（`full`）・`tower`・`serde_json`・`lazy_static` 等の
実行時ランタイム一式は feature 無関係の無条件依存として宣言されています。
`tokio/full` が `tokio-macros → syn → quote → proc-macro2 → unicode-ident` の
連鎖を引き込み、これだけで深さ 6 を超えます。これは `crates/dist-server/
Cargo.toml` が axum を不採用とした理由（フルスタック HTTP フレームワークの
推移的依存の重さ）と同じ構造です。

### `build.rs` を持つ依存

- linux: `httparse`・`libc`・`parking_lot_core`・`proc-macro2`・`quote`・
  `serde`・`serde_core`・`serde_json`・`zmij`（9 件）
- darwin: 上記に `system-configuration-sys`（`hyper-util` の `full` feature
  経由で有効になる macOS 固有の system proxy 検出クレート）が加わり 10 件

### ライセンス

`cargo metadata` の `packages[].license` を集計した結果は次のとおりです:
MIT / Apache-2.0 / `MIT OR Apache-2.0` / `(MIT OR Apache-2.0) AND
Unicode-3.0`（`unicode-ident`）/ `Unlicense OR MIT`（`memchr`）/ `Apache-2.0 /
MIT`（`fnv`）。いずれも OR 選択により `examples/ssr-routing/deny.toml` の
allow リスト（`MIT`, `Apache-2.0`, `Unicode-3.0`, `BSD-3-Clause`）で充足
できました（§6 参照）。

### edition・`unsafe`

`vercel_runtime 2.4.1` は edition 2024 です（toolchain 1.85 以上が必要。
本リポジトリの `rust-toolchain.toml` の `channel = "stable"` は現行 stable
（1.98.1）のため充足します）。`src/ipc_utils.rs` に `unsafe` ブロックが 1
箇所あります（第三者クレート内。本リポジトリの `forbid(unsafe_code)` 境界
〔`docs/policy/unsafe-boundary.md`〕は `crates/core`/`crates/interactive` を
対象としており、この事実自体が既存の境界を弱めるものではありません）。

## 6. cargo-deny の確認結果

`examples/ssr-routing/deny.toml` を一時クレートへ複製して実行しました
（ポリシー自体は変更していません）。

```
$ cargo deny check bans licenses sources
bans ok, licenses ok, sources ok
```

（一時クレート自身に `license = "MIT"` を明示した後の結果です。未指定の
状態では一時クレート自身が `unlicensed` エラーになりましたが、これは
実在しない計測用パッケージの体裁の問題であり、`vercel_runtime` 側の
依存には起因しません。）

```
$ cargo deny check advisories
advisories ok
```

advisories はネットワーク到達性を前提とするチェックですが、本環境では
到達可能で実行でき、既知の脆弱性は検出されませんでした（RustSec Advisory
DB 照会時点、2026-09-27）。

`bans` の `multiple-versions = "warn"` により `base64` の 2 版共存
（`v0.22.1` と `v0.23.1`）が警告として出力されましたが、`deny` 設定では
ないため判定には影響しません。

## 7. 結論（再掲）

件数（66〜71 件、上限 60）・深さ（11、上限 6）のいずれも大幅に超過して
おり、`build.rs` を持つ依存が 9〜10 件（`-sys` クレートを含む）追加されます。
一方で cargo-deny（bans/licenses/sources/advisories）はすべて PASS しました。
`docs/design/vercel-deployment-strategy.md` §7 の再評価トリガーのうち
「依存木が REQ-3 上限を超える」「`build.rs` を持つ依存が見つかる」の 2 点が
該当したため、案 a（`vercel_runtime` の併用）を取り下げ、案 c（SSG + Build
Output API + `--prebuilt`）単独を Vercel 上の既定方式として確定しました
（決定の反映は同文書を参照）。

## 8. 参照

- `docs/design/vercel-deployment-strategy.md`（本レポートの計測結果を受けて
  §4・§5・§6・§7・§8 を改訂）
- `crates/xtask/src/check_deps.rs`（本レポートが準拠した計測定義）
- `examples/ssr-routing/deny.toml`（cargo-deny 設定の複製元）
