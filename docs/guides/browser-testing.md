# 実ブラウザテスト環境ガイド（TASK-6.3a）

## 1. 目的とトレーサビリティ

- TASK-6.3【Conditional Go 条件 1】（親イシュー #64）は、SSR/SSG 出力との整合・ハイドレーション後の
  イベント発火・状態復元を **実ブラウザ** で正式実証するタスク。`docs/spec/06-roadmap.md` の着手判定に
  おける必須条件である。
- 本イシュー（TASK-6.3a・#65）はその 4 分割の 1 番目で、実ブラウザテストを実行できる環境
  （ローカル + CI）を構築する。後続の TASK-6.3b（#66 `hydration_browser.rs` 実装）・
  TASK-6.3c（#67 実証実行）・TASK-6.3d（#68 検証レポート）がこの環境上で動く。
- `docs/spec/05-tasks.md` TASK-6.3 は検証基盤として `wasm-pack test --headless`（Chromium ヘッドレス）
  を明示している。`docs/api/hydration-api.md` 第 5・7 節も「実ブラウザ検証は TASK-6.3 系へ引き継ぐ」と
  凍結済み。

## 2. 現状（2026-07-17 時点の重要な前提）

**`crates/wasm-client/`（`fandhe-frontend-wasm-client`）クレートは本コミット時点でまだ作成されていない。**
TASK-6.2b（#48 最小ハイドレーション実装）・TASK-6.2c（#49 ハイドレーションテスト整備）が
いずれも open のため、クレート自体の新設は本イシューのスコープに含めない
（`docs/api/hydration-api.md` 第 5 節の引き継ぎ表で #48/#49 のスコープと明記されており、
本イシューで重複作成すると責務混線・コンフリクトを招くため）。

そのため本イシューでは以下の 2 点のみを整備する:

1. `.github/workflows/ci.yml` の `browser-test` ジョブ（crates/wasm-client/ ディレクトリ存在ガード付き）
2. 本ドキュメント（環境ガイド・後続タスクへの引き継ぎ）

`crates/wasm-client/Cargo.toml` への `wasm-bindgen-test` dev 依存追加・
`crates/wasm-client/tests/browser_smoke.rs`（環境実証スモークテスト）の作成は、
`crates/wasm-client/` クレート自体が存在しないため本イシューでは行わない。
**TASK-6.2b（#48）マージ後、TASK-6.3b（#66）着手時に併せて整備すること。**

## 3. テストランナー: `wasm-pack test --headless --chrome`

`docs/spec/05-tasks.md` TASK-6.3 が明示する方式を採用する。`wasm-bindgen-test`
（`wasm_bindgen_test_configure!(run_in_browser)`）で書いたテストを headless Chromium で実行する。

代替案（Playwright ベース E2E）は Node.js 依存・依存面拡大が大きく、仕様の第一指名が
`wasm-pack test` であるため v1 では不採用。TASK-11.5（性能計測）で再検討可能。
横断 a11y 自動検証（axe-core 相当）の導入評価（イシュー #1076、`docs/ci/a11y-automation-evaluation.md`）
も本判断を a11y ツールへ継承し、Playwright ベースの axe-core 実行を同じ理由で見送っている。
docs サイト（`fandhe-frontend-docs-site`）のヘッダードロップダウン・検索結果パネル等の
対話操作検証手段の導入評価（イシュー #1084、`docs/ci/docs-site-interaction-testing-evaluation.md`）
も同様に本判断を継承しており、Playwright は本環境（`ubuntu26.04-x64`）で構造的に
起動不能なことを再実測で確認している。

## 4. CI 構成（`.github/workflows/ci.yml` の `browser-test` ジョブ）

- ランナー: `ubuntu-latest`（GitHub ホストランナーは Chrome / chromedriver プリインストール済み。
  self-hosted へのブラウザ導入コストを回避し、`forbid-unsafe` ジョブとは独立に実行）
- chromedriver: ランナー内蔵のものを `CHROMEDRIVER="$CHROMEWEBDRIVER/chromedriver"` で明示指定
  （wasm-pack による実行時の chromedriver 自動ダウンロードを防ぐサプライチェーン対策）
- wasm-pack: バージョン固定（v0.13.1）+ SHA256 チェックサム検証付きの公式リリースバイナリ
  ダウンロード
- 第三者製 action（rust-cache / install-action 等）は新規追加しない。既存ワークフローと同じ
  SHA 固定の `actions/checkout` のみ使用する
- **ディレクトリ存在ガード**: `crates/wasm-client/Cargo.toml` の有無を最初のステップで判定し、
  存在しない間は後続ステップ（wasm32 target 追加・wasm-pack 導入・テスト実行）をすべて
  スキップする。`crates/wasm-client/` が追加された時点で自動的に有効化される

## 5. ローカル実行手順（`crates/wasm-client/` 追加後）

```bash
# 1. wasm32 ターゲットの追加（初回のみ）
rustup target add wasm32-unknown-unknown

# 2. wasm-pack の導入（未導入の場合）
cargo install wasm-pack --locked

# 3. ローカルの chromedriver パスを指定して実行
CHROMEDRIVER=/path/to/chromedriver wasm-pack test --headless --chrome wasm-client
```

Chrome/Chromium と対応する chromedriver がローカルに必要（バージョン整合に注意）。

## 6. トラブルシュート

| 症状 | 対処 |
|------|------|
| `chromedriver` が見つからない | `CHROMEDRIVER` 環境変数でパスを明示指定する（自動ダウンロードには依存しない） |
| CI で `browser-test` ジョブがスキップされる | `crates/wasm-client/Cargo.toml` が存在するか確認（#48 未マージの間は意図した挙動） |
| wasm-pack のチェックサム検証失敗 | バージョンアップ時にチェックサム更新を忘れていないか確認する（`.github/workflows/ci.yml` 内にハードコード） |
| ローカルとCIでブラウザテスト結果が異なる | Chrome バージョン差異の可能性。CI 側のバージョンを基準とする |

## 7. TASK-6.3b〜d への引き継ぎ事項

| 事項 | 引き継ぎ先 |
|------|-----------|
| `crates/wasm-client/Cargo.toml` への `wasm-bindgen-test` dev 依存追加 | TASK-6.3b（#66）着手時（#48 マージ後） |
| `crates/wasm-client/tests/browser_smoke.rs`（環境実証スモークテスト） | TASK-6.3b（#66）着手時 |
| `hydration_browser.rs`（ハイドレーション実証テスト本体） | TASK-6.3b（#66） |
| 実証実行・不具合修正 | TASK-6.3c（#67） |
| 検証レポート（Conditional Go 条件 1 解消判定） | TASK-6.3d（#68） |
| 実ブラウザ性能計測ハーネス（本環境の再利用） | TASK-11.5 系（#85〜#88） |
| WASM 経路 XSS テスト（`xss_escape_wasm.rs`）の本環境への統合 | TASK-1.3 系（#90〜#92） |
| CI ビルドキャッシュ導入（第三者 action の採否判断） | 必要になった時点で新規イシューを提案（`out-of-scope-tracking.md`。勝手起票はしない） |

## 8. セキュリティ考慮事項（OWASP Top 10 観点）

- **A05 セキュリティ設定ミス**: ワークフローは `permissions: contents: read` の最小権限を維持。
  シークレット参照なし。`run:` への `${{ }}` 展開による script injection 経路を作らない
  （外部入力を補間しない）
- **A08 ソフトウェア・データ整合性（サプライチェーン）**: wasm-pack はバージョン固定 + SHA256
  チェックサム検証付き導入。chromedriver はランナー内蔵バイナリを明示指定し実行時の自動
  ダウンロードを封じる。action は既存の SHA 固定 `actions/checkout` のみ・第三者製 action の
  新規追加なし。`submodules: false` でチェックアウト面を最小化
- 将来 `crates/wasm-client/tests/browser_smoke.rs` を追加する際は、`set_text_content` 等のテキスト API
  のみを使用し、`raw_html()` / `set_inner_html` の直接使用・HTML 文字列組み立てを行わないこと
  （`docs/api/hydration-api.md` 第 6 節の不変条件に整合）

## 9. docs サイトのビジュアル回帰撮影（`tools/docs-site/visual-regression.sh`）

- **位置づけ**: 本ガイド §3 の `wasm-pack test --headless`（ハイドレーション実証）とは別系統で、
  docs サイト（`crates/docs-site`）の見た目を実ブラウザで撮影する手動スクリプト（イシュー #960）。
  chromium の常設が self-hosted runner に保証されないため（`docs/ci/ci-runner-requirements.md`
  の未解決要件に依存する）、CI ジョブ化はしていない。ローカル開発者・test-runner が手動実行する。
- **前提ツール**: chromium（または chromium-browser）/ python3 / cargo / ss。いずれか不在の場合は
  `environment error:` を出して fail-closed に停止する（自動インストールは行わない）。
- **実行例**:

  ```bash
  DOCS_SITE_SHOTS_DIR="$HOME/fandhe-docs-site-visual/$(date +%Y%m%d-%H%M%S)" \
    bash tools/docs-site/visual-regression.sh
  ```

- **出力先の制約**: `DOCS_SITE_SHOTS_DIR` は絶対パスであること、かつパス要素にドット始まり
  ディレクトリ（`.claude/...` 等）を含まないことが必須（worktree の `.claude/...` 配下では snap の
  AppArmor により chromium が無音で書き込み失敗するため）。未指定時の既定値は
  `$HOME/fandhe-docs-site-visual/<timestamp>/`。
- **出力構成**: `shots/*.png`（撮影画像）/ `manifest.tsv`（8 列: `file`/`url`/`width`/`height`/
  `theme`/`js`/`bytes`/`sha256`）/ `logs/`（ビルドログ・chromium ログ）。撮影物はリポジトリへ
  コミットしない。
- **tall-window 撮影（イシュー #1083）**: `t1-themes-dialog-375-tall` は通常の撮影マトリクスの
  **既定ステップ**として常に撮影される（オプションではない）。目的は狭幅（375px）でテーブルが
  横方向にクリップされていることの証跡取得で、通常撮影（viewport 高 812/900/1024）は表に到達する
  前に切れてしまうため、`/themes/dialog/` を高さ 3000 で撮影して表全体をフレーム内へ収めることを
  狙っている（初回の目視確認は本イシューの実装セッションでは実行環境の制約により未実施。
  `docs/reports/docs-site-redesign-regression-report.md` §19.2 参照。高さが不足していると判明した
  場合はスクリプト内のリテラル値を引き上げて再実行すること）。追加の手動 chromium 呼び出しは
  不要（`bash tools/docs-site/visual-regression.sh` の実行だけで再取得できる）。
- **トラブルシュート**:

  | 症状 | 対処 |
  |------|------|
  | `environment error: chromium ... not found` | chromium（または chromium-browser）を導入する。自動インストールはしない |
  | 出力ディレクトリ関連の `environment error` | `DOCS_SITE_SHOTS_DIR` を絶対パス・ドット始まりディレクトリなしの値へ変更する（既定値を使えば発生しない） |
  | 撮影がハングして戻らない | CSP（`script-src 'none'`）配信下で `<meta http-equiv="refresh">` を含むページ（旧 `/components/*` 移転案内等）を新たに撮影対象へ追加していないか確認する。既知のハング経路のためスクリプトのマトリクスには含めていない |

- **セキュリティ注記（OWASP Top 10 観点）**:
  - **A01 アクセス制御の不備**: 配信サーバは `127.0.0.1` バインドのみ（外部公開しない）。
  - **A09 ログ・監視の不備**: `manifest.tsv` にはユーザー名を含む絶対パス（`$HOME` 等）を残さず、
    出力ディレクトリ相対パスのみを記録する。撮影物・ログは非コミットとし、リポジトリへ残置しない。

## 10. examples のオーバーレイ実演の実測検証（イシュー #1203）

- **位置づけ**: `examples/*/wasm`（例: `examples/interactive-view-transitions/wasm`）
  が独自実装するアプリ側の配線コード（`Runtime<C>` に載らないコンポーネントの
  ハイドレーション・オーバーレイ配線ラッパー等）は、`crates/wasm-full/tests/` の
  browser テストがカバーする wasm-full 本体の論理とは別に、example 固有の統合
  ギャップを持ちうる。`docs/reports/interactive-view-transitions-overlay-browser-report.md`
  はこの種のギャップ（navigation-menu/menubar のオーバーレイ実演における
  Escape/外側クリック閉鎖が click 経路では機能しない再入バグ）を実ブラウザ実測で
  発見した記録である。
- **採用した検証方式（使い捨てハーネス・非常設 CI）**: §3 の `wasm-pack test
  --headless --chrome` の応用として、対象 example の `wasm/` ディレクトリを
  `<repo>/target/tmp/<作業名>/wasm/` へコピーし（イシュー #637 の配置規約に
  整合、example 正本は一切変更しない）、コピー側にのみ次の変更を加える:
  1. `[lib] crate-type` へ `"rlib"` を追加（`tests/` からクレートの公開関数を
     呼べるようにする）。
  2. `[dev-dependencies] wasm-bindgen-test = "0.3"` を追加。
  3. `tests/support.rs`（共通フィクスチャ・イベント合成ヘルパー、
     `crates/wasm-full/tests/overlay_close_browser.rs`/`keynav_browser.rs`
     と同型の `create_placeholder`/`keydown_event`/`pointerdown_event`/
     `RemoveOnDrop` パターンを踏襲）と、シナリオごとの
     `tests/<name>_NN_<scenario>.rs` を追加する。
  4. アプリ側が `thread_local!` で状態を共有する設計（例: 複数コンポーネント間で
     1 個の `OverlayCloseController` を共有する `SHARED_OVERLAY`）を持つ場合、
     `wasm-pack test` は `tests/*.rs` の**ファイル単位**で別々の `.wasm`
     バイナリを生成することを利用し、シナリオ 1 件につきテストファイル 1 個へ
     分割する（同一 `.wasm` インスタンス内で複数テストを実行すると
     `thread_local` の状態が前のテストから残留し、偽陰性/偽陽性の原因になる）。
  5. 実測後、`target/tmp/<作業名>/` を削除する（撮影物・ハーネスはコミットしない）。
  常設 CI 化を見送った理由・再評価トリガーは同レポート §6 を参照。
  `crates/cli/embedded-examples/` のバイト一致同期・`fandhe-frontend-cli` の
  semver バンプ連鎖を避けるため、example 正本へテストを同梱しない点が
  `crates/wasm-full/` 本体の browser テスト運用との違いである。採否確定の
  評価記録は `docs/ci/example-overlay-browser-interaction-testing-evaluation.md`
  （イシュー #1210）を参照。
- **フィクスチャ構築の注意（同レポート §5.2 参照）**: `data-hydrate-*` を
  持たない `id` のみの root 要素は復元失敗 → 既定状態へのフォールバック
  描画という安全側経路を通るが、`wire_keynav` 等 DOM 上の `data-scope`/
  `data-part` 属性を頼りに対象範囲を確定するロジック（例:
  `[data-scope="navigation-menu"][data-part="root"]` を `closest()` で
  探索する）は、実 SSR が root 要素自体に付与する anatomy 属性を
  フィクスチャ側でも明示的に再現しないと無言で no-op になる。フィクスチャの
  `create_placeholder` を素の `div` のまま使い回さず、対象コンポーネントの
  `root()` 関数が出力する属性と揃えること。
