# example オーバーレイのブラウザ実インタラクションテストの常設 CI 化評価（イシュー #1210）

## 1. 背景とトレーサビリティ

- イシュー #1203（`examples/interactive-view-transitions` へ navigation-menu /
  menubar のオーバーレイ実演を追加する）→ PR #1206 が実装した。PR #1206 は
  同時に `wasm-pack test --headless --chrome` の**使い捨てハーネス**（scratch
  ワークスペースへ example 正本を丸ごとコピーし、rlib 化・シナリオ別テスト
  ファイルを一時配置して実行し、実行後は削除する手順）で 9 シナリオを実測し、
  `docs/reports/interactive-view-transitions-overlay-browser-report.md` に
  手順・結果を記録した。同レポート §6 は「常設 CI 化は見送り」と判断し、
  本イシュー #1210 はその**常設化の再評価**を追跡する out-of-scope 記録
  から起票された。
- `docs/guides/browser-testing.md` §3 の既決 Playwright 不採用 →
  `docs/ci/a11y-automation-evaluation.md`（#1076、headless-ui 63 部品の
  ARIA 検証の横断自動化を見送り）→
  `docs/ci/docs-site-interaction-testing-evaluation.md`（#1084、docs サイト
  対話操作検証手段の導入評価。P1 は見送り・P2〔`tools/docs-site/
  visual-regression.sh` の対話操作拡張〕は不採用・P3〔`wasm-pack test
  --headless --chrome` の限定活用〕を採用）という判断チェーンの延長線上に
  本評価が位置づく。#1084 の P3 判断が確立した「Chrome for Testing の
  バージョン固定 + SHA256 検証済みプリビルトバイナリを使い、`crates/
  wasm-full/tests/*_browser.rs` 相当の対象へ限定適用する」パターンを
  PR #1206 の使い捨てハーネスが踏襲している。
- 受け入れ条件との対応:

| # | 受け入れ条件 | 対応節 |
|---|-------------|--------|
| 1 | 採否の判断と根拠を `docs/ci/` の評価文書として記録する | §3・§4（本文書） |
| 2 | 採用する場合は `.claude/rules/ci.md` のツール前提明示・self-hosted runner 規約に従う | §5（本イシューでは不採用のため実装方式メモに留める） |

### 1.1 見送り以降の経過（判断材料の再確認）

本評価着手時点（2026-08-02）で、レポート §6 の見送り判断が前提とした
状況に変化がないかを以下の観点で再確認した。

- **イシュー #1209（再入バグ）は PR #1212 で修正済み**（2026-08-02
  マージ）。`examples/interactive-view-transitions/wasm/src/lib.rs` の
  `sync_shared_overlays()` を `sync_shared_overlays_with(...)` へ拡張し、
  `wire_headless_component` の `on_update` コールバック実行中に同一
  `Rc<RefCell<C>>` へ再入 `try_borrow()` してしまう不具合（レポート §5.2）
  を、呼び出し元が既に保持しているスナップショット参照を渡す設計へ変更する
  ことで解消した。**検証は使い捨てハーネス手順の再実行のみ**（レポート
  §10。9 シナリオ全 PASS）で行われ、恒久テストの追加・CI 変更は一切発生
  しなかった。変更ファイルは example 正本 `lib.rs`・
  `crates/cli/embedded-examples/` 同期・レポート追記・`fandhe-frontend-cli`
  0.3.5 バンプのみである。→ 使い捨てハーネスが「再現可能かつ実用に足る」
  ことが実測で裏付けられ、常設化なしでも回帰検出・修正検証のサイクルが
  機能した実例となった。
- **イシュー #1204/#1205（他 examples へのオーバーレイ実演追加）はいずれも
  クローズ済み**。実際に追加されたのは `examples/headless-pre-styled-ui`
  への navigation-menu / menubar 節のみで、内容は「SSR 初期状態は
  closed・開閉の実挙動は wasm 層の責務」と明記した**静的ショーケース**
  （JS ハイドレーション配線を持たない）である。→ レポート §6 の再評価
  トリガー候補である「`Runtime<C>` 非対応の独自配線がさらに増える」は
  不成立。JS 配線を伴うオーバーレイ実演は現時点でも
  `examples/interactive-view-transitions` の 1 件のみ。
- レポート §6 の見送り根拠 4 点（論理の中核は `crates/wasm-full/tests/
  overlay_close_browser.rs` 等で CI 常設済み・example smoke の先例は
  build/run/gate までに限られる・example 正本へのテスト同梱が
  embedded-examples バイト一致同期〔`example_publish_copy_drift`〕と
  `fandhe-frontend-cli` の semver バンプ連鎖を誘発する・再現手順は
  レポート §4/§10 と `docs/guides/browser-testing.md` §10 で文書化済み）
  は、上記再確認の結果、**いずれも現在も有効**であることを確認した。

## 2. 候補比較

| 案 | 内容 | 評価 |
|---|---|---|
| C1 | example 正本（`examples/interactive-view-transitions/wasm/`）へ `wasm-bindgen-test` の dev 依存とテストファイルを直接同梱し、`browser-test` ジョブへステップ追加して常設実行する | `crates/cli/embedded-examples/` のバイト一致同期（`example_publish_copy_drift` テスト）と `fandhe-frontend-cli` の semver バンプ（#638 規約）を毎回誘発する。テスト補助コードの混入で example の教材性（「動く最小構成を読む」という利用者体験）を損なう。**棄却** |
| C2 | 正本を汚さず、xtask 等が scratch コピー + パッチ適用（PR #1206 のハーネス手順相当）を機械生成し `browser-test` ジョブ内で実行する恒久 CI ジョブ | 正本汚染は回避できるが、パッチ適用コード自体の保守負担が発生し、example ワークスペースの crates.io 依存フルビルド分だけ CI 時間が増える。現時点で JS 配線を伴う example は 1 件のみ（§1.1）であり、専用の恒久ジョブ・保守対象を新設するコストに見合う回帰対象規模ではない。**棄却** |
| C3 | 恒久ハーネス（scratch コピー生成スクリプト）を `tools/` 配下へローカル手動ツールとして常備し、CI 化はしない（`tools/docs-site/visual-regression.sh` と同クラスの位置づけ） | 再現手順は既にレポート §4・§10 と `docs/guides/browser-testing.md` §10 で文書化済みで、#1209 修正検証（PR #1212）でも手順どおり再現できた実績がある。スクリプト化による増分価値（コマンド 1 回で再実行できる利便性）はあるが、常設 CI 化の可否判定という本イシューの受け入れ条件には直接寄与しない。**見送り（再評価トリガー成立時の第一候補として記録）** |
| C4 | 現状維持（使い捨てハーネスによる都度実測・レポートへの記録のみ、常設 CI 化なし） | §1.1 の再確認・§3 の根拠により妥当。**採用** |

## 3. 結論と根拠

**見送り継続**とする。

1. **中核ロジックは CI 常設済み**: オーバーレイ閉鎖制御（Escape / 外側
   クリック / opt-out / 未知 scope no-op / XSS 経路）の中核ロジックは
   `crates/wasm-full/tests/overlay_close_browser.rs` 等が CI で常設実行
   している。#1209 で発見された実バグは wasm-full 本体ではなく example
   側の統合コード（`nav_overlays` モジュール）に閉じており
   （レポート §5.3）、中核ロジックの CI 担保がこの種のバグを取りこぼす
   構造的欠陥があるわけではない。
2. **example smoke の先例を超える**: 既存の examples e2e
   （`crates/cli/tests/new_gate_e2e.rs`）は `cargo build`/`cargo run`
   起動確認・`fw gate` 通過確認までであり、example 固有のブラウザ実
   インタラクション常設テストはこの先例のスコープを超える。
3. **同期・バンプ連鎖の回避**: C1 の棄却理由のとおり、example 正本への
   テスト同梱は `embedded-examples` バイト一致同期と cli semver バンプの
   連鎖を誘発し、しかも CI で実行しない限りそのテスト自体が headless-ui /
   wasm-full の更新のたびにサイレントに腐る risk がある。
4. **再現可能性は文書で担保済み**: レポート §4・§10 に使い捨てハーネスの
   全差分・実行コマンド・環境情報が埋め込まれており、#1209 修正検証
   （PR #1212）で実際にこの手順のみで再実行・全 PASS 確認ができた
   （§1.1）。常設化しなくても再現性は損なわれていない。
5. **再評価トリガーはいずれも未成立**: #1209 は使い捨てハーネス自身が
   発見した初回の統合バグであり、PR #1212 の修正**以降の再発**ではない。
   #1204/#1205 で追加された実演は JS 配線を持たない静的ショーケースであり
   「独自配線の増加」トリガーにも該当しない（§1.1）。

依存追加・CI 変更・クレート変更のいずれも伴わない（ユーザー承認が必要な
操作は発生しない）。

## 4. 再評価トリガー

以下のいずれかが発生した場合、常設 CI 化（C2）または恒久ハーネススクリプト
の常備（C3）を再検討する。

- example 統合経路に起因するバグが、**PR #1212 の修正以降に再発**した場合
  （#1209 と同種の統合ギャップ起因バグの 2 件目以降）。
- examples へ **JS 配線（`wire_headless_component`/`wire_keynav` 等の
  hydrate/wire 呼び出し）を伴う** UI 部品実演が新規追加された場合。ただし
  #1204/#1205 のような、SSR 初期状態の表示のみを示す静的ショーケース
  （JS ハイドレーション配線を持たないもの）は対象外とする。
- `fandhe-frontend-wasm-full` 側の overlay/keynav 関連 API に破壊的変更が
  入り、example 側の追随漏れが実際に検出（build 失敗・実挙動不一致等）
  された場合。
- 使い捨てハーネスの手動再構築が短期間に反復して必要になり、都度のコスト
  （scratch ワークスペース構築・パッチ適用・実行・後始末）が常設化コスト
  （§2 C2/C3 の保守負担・CI 時間増）を上回る兆候が観測された場合。この
  場合はまず C3（ローカルツール常備）を第一候補として検討する。

## 5. 導入する場合の実装方式メモ（本イシューでは実装しない）

受け入れ条件 2 への対応として、将来 C2 または C3 を採用する場合に従うべき
要件を記録する。**本イシューではこれらを実装しない**（§3 のとおり見送り
継続のため）。

- **self-hosted runner 前提**（`.claude/rules/ci.md`「Runner 方針」）:
  新規ジョブも `runs-on: self-hosted` を既定とし、GitHub ホステッド
  ランナーを使わない。
- **ツール前提の明示**（同「ツール前提の明示」節）: `browser-test` ジョブ
  が確立済みの「バージョン固定 + SHA256 チェックサム検証済み Chrome for
  Testing / chromedriver」パターンを踏襲し、`cargo install` や実行時の
  任意最新版取得は行わない。wasm-bindgen-cli もバージョン整合の
  fail-closed 検証を伴う既存パターン（`templates/app/tools/wasm/build.sh`
  等）に合わせる。
- **`CARGO_TARGET_DIR` の隔離**（同「Self-hosted 環境の前提」節、
  イシュー #659/#1192）: 生成物・一時ワークスペースは `/tmp` 固定パスでは
  なく `RUNNER_TEMP`（`${{ runner.temp }}`）配下の専用ディレクトリへ配置
  する。`cargo package`/`cargo publish` 系の汚染問題（#1192）と同様、共有
  `CARGO_TARGET_DIR`（`/cargo-target`）を直接使わない。
- **`environment error: ` プレフィックスの fail-closed 判定**
  （`version-bump-guard`・`gate-self-apply` と同型）: ツール不在・
  ネットワーク到達不可等の環境要因と、テスト失敗（コード起因）を CI
  アノテーション上で区別する。
- **`embedded-examples` 同期・cli semver バンプへの対応**（C1 を採る場合。
  ただし C1 自体は §2 で棄却済み）: テストファイルを example 正本へ追加
  する場合は `crates/cli/tests/example_publish_copy_drift.rs` の同期契約と
  `fandhe-frontend-cli` の semver バンプ（`.claude/rules/coding-rust.md`
  該当節）を PR チェックリストへ明記する。

## 6. セキュリティ考慮事項（OWASP Top 10 観点）

- **A08 サプライチェーン**: 将来 C2/C3 を採用する場合も「バージョン固定 +
  SHA256 チェックサム検証済みプリビルトバイナリ」パターン（`browser-test`
  ジョブの既存規約）を必須要件とし、`cargo install` や実行時自動ダウン
  ロードによる任意最新版取得は禁じる（§5）。
- **A05 セキュリティ設定ミス**: 本評価自体はドキュメントのみの変更であり、
  `.github/workflows/**`・`deny.toml`・secrets は変更しない。
- **A01 アクセス制御 / パストラバーサル**: 将来の実装方式メモ（§5）は
  一時ワークスペースを `RUNNER_TEMP` 配下の専用ディレクトリへ限定する
  ことを要件化しており、固定パスへの無制御な書き込み・削除（`rm -rf` 等の
  広域削除）を避ける既存規約（`.claude/rules/ci.md`）を踏襲する。
- **A09 ログ・監視 / 機微情報**: 本文書・関連コミットに `$HOME`・
  ユーザー名を含む絶対パスやトークン等の機微情報を含めない。
