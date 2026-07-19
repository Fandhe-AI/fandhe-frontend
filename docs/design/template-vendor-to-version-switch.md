# テンプレート vendor 同梱 → バージョン依存への切替手順（イシュー #412）

## 背景

`templates/app`（`fw new --template app`、イシュー #378）は、fandhe-frontend-core /
fandhe-frontend-app が `publish = false`（crates.io 未公開）のため、生成プロジェクトへ
ソースを vendor 同梱（`templates/app/vendor/fandhe-frontend-core`・
`templates/app/vendor/fandhe-frontend-app`）し、path 依存させている
（選定根拠は `docs/design/fw-new-design.md` §3a）。

vendor 同梱は正本の写しであり、本来は「暫定措置」である。fandhe-frontend-core /
fandhe-frontend-app が crates.io へ公開された時点で、テンプレートは通常のバージョン
依存（`fandhe-frontend-core = "X.Y.Z"`）へ切り替えるべきである。本文書はその切替を
実施する際のチェックリストを定める。

## トリガー条件

**「vendor 同梱対象クレート（fandhe-frontend-core / fandhe-frontend-app）の正本 `Cargo.toml` から
`publish = false` が解除される」** をトリガーとする。

- crates.io の公開状態そのものへの実行時問い合わせ（ネットワークアクセス）
  は行わない。オフライン決定性（`.claude/rules/security.md` サプライチェーン
  対策）と矛盾するため。
- `publish = false` の解除はリポジトリ内で完結して検知できる代理指標であり、
  crates.io への実際の公開作業に先立って行われる必要条件である。
- トリガーの機械検知は `cli/tests/template_vendor_drift.rs` の
  `vendor_to_version_switch_trigger_has_not_fired` テスト（canary）が担う。
  正本 `core/Cargo.toml` / `app/Cargo.toml` のいずれかで `publish = false` が
  非コメント行として存在しなくなると、このテストが是正手順付きメッセージで
  FAIL する。

## 切替チェックリスト

トリガーが成立した（= canary テストが FAIL した）ら、以下の順に作業する。

1. **前提確認**

   fandhe-frontend-core / fandhe-frontend-app が crates.io へ公開済みで、公開バージョン `X.Y.Z` が
   確定していることを確認する。公開作業自体・公開バージョンの決定は本手順の
   範囲外（別途ユーザー承認のもとで実施する）。

2. **依存宣言の切替**

   `templates/app/Cargo.toml` の `[dependencies]` を書き換える。

   ```toml
   [dependencies]
   fandhe-frontend-core = "X.Y.Z"
   fandhe-frontend-app = "X.Y.Z"
   ```

   バージョンは厳密固定を基本とし、テンプレートの決定性（同一入力 → 同一
   出力）を保つ。あわせて `[workspace] members = ["."]` の維持要否を
   再評価する（vendor path 依存の自動編入防止という当初の存在理由が
   切替後は消えるため。ただし「生成プロジェクトを root workspace に
   巻き込まない」という別の目的が残る場合は維持する）。

3. **vendor 削除**

   - `templates/app/vendor/` ディレクトリを削除する。
   - `cli/src/new_template.rs` の `APP_TEMPLATE_FILES`（または同等の定義）
     から vendor 配下エントリ（`fandhe-frontend-core`/`fandhe-frontend-app` の各ソースファイル・
     `Cargo.toml`）を除去する。
   - `cli/tests/new_e2e.rs` のドリフト検知（埋め込みマニフェストと
     `templates/<name>/` の 1:1 検証）が整合を強制する。

4. **Cargo.lock 再生成**

   crates.io の registry エントリ（`source` + `checksum` 付き）で
   `templates/app/Cargo.lock` を再生成する。`new.rs::replace_exact` が使う
   needle（`fandhe-frontend-template-app`）の出現回数が引き続き 1 であることを確認する
   （`docs/design/fw-new-design.md` §4 に fail-closed 検証の実例あり）。

5. **テスト更新**

   `cli/tests/template_vendor_drift.rs` から以下を削除する。

   - vendor drift テスト（バイト一致検証）:
     `vendored_fandhe_frontend_core_src_is_byte_identical_to_source_crate`・
     `vendored_fandhe_frontend_core_cargo_toml_has_no_external_dependencies`・
     `vendored_fandhe_frontend_app_src_is_byte_identical_to_source_crate`・
     `vendored_fandhe_frontend_app_cargo_toml_points_at_vendored_fandhe_frontend_core`
   - canary テスト:
     `vendor_to_version_switch_trigger_has_not_fired`・
     `vendored_crates_not_covered_by_known_map`

   共有ファイル同一性テスト
   （`default_and_app_templates_share_identical_bytes_for_common_files`）は
   vendor 同梱と無関係のため維持する。

6. **CI 前提の変化を明記**

   `fw gate` e2e（`cli/tests/new_gate_e2e.rs`）の app テンプレート分は
   crates.io からの取得（ネットワークアクセス）が必要になり、vendor 同梱時
   のオフライン決定性が失われる。self-hosted runner のネットワーク・
   registry キャッシュ前提を `.claude/rules/ci.md`（ツール前提の明示）の
   規約に従って追記する。

7. **セキュリティ再評価**

   生成プロジェクトの依存が「同梱ソース」から「crates.io 取得物」へ変わる
   ため、以下を実施する（`.claude/rules/security.md` 準拠）。

   - `templates/app/deny.toml`（`advisories`/`sources`）の実効範囲確認
   - REQ-3（60 件 / 深さ 6）の `cargo metadata` 再計測
   - `build.rs` 有無の確認

8. **文書更新**

   `docs/design/fw-new-design.md` の以下を切替後の状態へ更新する。

   - §3a（vendor 同梱の選定根拠）: 切替が完了した旨を追記
   - §9（非目標）: 「crates.io 公開後の vendor → バージョン依存への切替」の
     項目を「イシュー #412 で実施済み」へ更新

## 本文書の対象外

- 実際の切替実施（本文書はトリガー成立時に参照するチェックリストであり、
  本文書自体の新設・改訂だけでは切替は行わない）
- crates.io への公開作業そのもの・公開バージョンの決定
- `templates/default`（fandhe-frontend-core / fandhe-frontend-app に依存しないため対象外）
