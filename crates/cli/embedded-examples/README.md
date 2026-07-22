# crates/cli/embedded-examples/ — 同梱コピー（正本ではない）

このディレクトリはリポジトリルート `examples/`（`ssr-routing/` /
`ssg-blog/` / `dist-server-docker/` / `interactive-view-transitions/` /
`headless-pre-styled-ui/`）のバイト単位同梱コピーです。**正本は
ルート `examples/` のまま**であり、本ディレクトリを直接編集しないでください。

## なぜ必要か

`crates/cli/src/new_template.rs` は `fw new --example <name>` が展開する
サンプル群を `include_str!` でバイナリへコンパイル時埋め込みしていますが、
`include_str!` はクレートディレクトリ（`crates/cli/`）の外を参照できません。
ルート `examples/` を直接参照すると `cargo package` / `cargo publish` の
tarball 検証（クレートディレクトリ外ファイルの同梱禁止）が失敗します
（`fandhe-frontend-cli` を crates.io へ公開する際に発覚した制約、
`crates/cli/templates/README.md` 参照）。

これを解決するため、`crates/cli/templates/` が `templates/`（イシュー #316/
#378）の同梱コピーであるのと同じ「正本 + 同梱コピー + ドリフト検知テスト」
運用を `examples/` にも適用し、`crates/cli/src/` から参照可能な位置
（本ディレクトリ）へ正本のコピーを置いています（イシュー #500）。

`crates/cli/templates/` は cargo が自動検出する `examples` ターゲット名
（`[[example]]`）と衝突しない `embedded-examples/` という名前を使っています。
`crates/cli/examples/` を使うと cargo が example ターゲットとしてコンパイル
対象に取り込んでしまうため、意図的に別名にしています。

## `--template` との違い（置換なし）

`templates/` 配下は `fw new` がプロジェクト名へパッケージ名を置換します
（`new_template.rs::Template::substituted_files`）が、`embedded-examples/`
配下は**置換しません**（`Template::substituted_files` は空配列）。

`examples/ssr-routing/tests/routing.rs` は
`env!("CARGO_BIN_EXE_fandhe-frontend-example-ssr-routing")` でバイナリ名を
直接参照するため、`Cargo.toml` のパッケージ名だけを置換すると
`CARGO_BIN_EXE_*` が未定義になり生成直後の `cargo test` がコンパイル不能に
なります。`examples/` は「雛形の生成」ではなく「正本サンプルの取得」であり、
生成物が正本 `examples/<name>/` と全ファイルバイト一致になることが決定性・
ドリフト検知の観点で最も強い保証になります（`templates/embed/` と同じ整理）。

## 同期を保証する仕組み

ルート `examples/<name>/` と `crates/cli/embedded-examples/<name>/` の乖離は
`crates/cli/tests/example_publish_copy_drift.rs` が両ディレクトリを
再帰走査してバイト単位比較し、`cargo test -p fandhe-frontend-cli` で
機械的に検出します。手動同期に頼りません（`.claude/rules/ci.md` の
cargo-deny pin ドリフト検知と同じ運用方針）。

## 変更手順

サンプルの内容を変更する場合は、必ず**ルート `examples/<name>/` を
先に変更**し、その後 `crates/cli/embedded-examples/<name>/` へ同じ変更を
反映してください（`Cargo.toml` は `Cargo.toml.embed` へのリネームが必要な
点に注意。`cp -R examples/<name> crates/cli/embedded-examples/<name>` で
全体を再同期したあと `mv .../Cargo.toml .../Cargo.toml.embed` しても
構いません）。反映を忘れると `example_publish_copy_drift.rs` が CI で
失敗します。
