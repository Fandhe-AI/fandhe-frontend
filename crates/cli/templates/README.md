# crates/cli/templates/ — 同梱コピー（正本ではない）

このディレクトリはリポジトリルート `templates/`（`default/` / `app/` /
`embed/`）のバイト単位同梱コピーです。**正本はルート `templates/` のまま**
であり、本ディレクトリを直接編集しないでください。

## なぜ必要か

`crates/cli/src/new_template.rs` は `fw new` が展開するテンプレート群を
`include_str!` でバイナリへコンパイル時埋め込みしていますが、`include_str!`
はクレートディレクトリ（`crates/cli/`）の外を参照できません。ルート
`templates/` を直接参照すると `cargo package` / `cargo publish` の tarball
検証（クレートディレクトリ外ファイルの同梱禁止）が失敗します
（`fandhe-frontend-cli` を crates.io へ公開する際に発覚）。

これを解決するため、`templates/default/tools/npm-asset-build/` が
`tools/npm-asset-build/`（イシュー #316）の同梱コピーであるのと同じ
「正本 + 同梱コピー + ドリフト検知テスト」運用を採用し、`crates/cli/src/`
から参照可能な位置（本ディレクトリ）へ正本のコピーを置いています。

## 同期を保証する仕組み

ルート `templates/<name>/` と `crates/cli/templates/<name>/` の乖離は
`crates/cli/tests/template_publish_copy_drift.rs` が両ディレクトリを
再帰走査してバイト単位比較し、`cargo test -p fandhe-frontend-cli` で
機械的に検出します。手動同期に頼りません（`.claude/rules/ci.md` の
cargo-deny pin ドリフト検知と同じ運用方針）。

## 変更手順

テンプレートの内容を変更する場合は、必ず**ルート `templates/<name>/` を
先に変更**し、その後 `crates/cli/templates/<name>/` へ同じ変更を反映して
ください（`cp -R templates/<name> crates/cli/templates/<name>` で全体を
再同期しても構いません）。反映を忘れると
`template_publish_copy_drift.rs` が CI で失敗します。
