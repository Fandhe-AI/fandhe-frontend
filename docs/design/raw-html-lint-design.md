# `raw_html()` 検出ゲートの頑健化: 方式比較と選定（イシュー #157/#158）

## 1. 背景・脅威モデル

REQ-1（既定エスケープ）の唯一の許容迂回経路は `fandhe_frontend_core::raw_html()` である
（`crates/core/src/lib.rs`）。`fw gate`（TASK-13.3、#138）の `default_escape_check`
（`crates/cli/src/gate.rs`）は当初、次のテキスト走査方式（PoC-7 由来の「マーカー方式」）
のみで未レビュー呼び出しを検出していた。

- `raw_html()` 呼び出しの同一行・直前行に `// ESCAPE-REVIEWED: <根拠>` という
  **コメント**があれば「レビュー済み」として通過させる。

この方式には次の脅威がある。

1. **偽装（spoofing）**: コメントはコンパイラに一切検証されない。レビューを
   経ずに `// ESCAPE-REVIEWED: ...` と書くだけで検出をすり抜けられる。
   AI エージェント・人間のいずれであっても、レビュープロセスを経由したという
   「事実」を保証する仕組みがコメント文字列にはない。
2. **見落とし（false negative）**: テキスト走査は識別子の文字列一致に依存する
   ため、リネーム import（`use fandhe_frontend_core::raw_html as fragment;` → `fragment(...)`)
   のような呼び出しを構造的に見逃し得る。

本イシューは、上記脅威に対して偽装不能・見落としに強い検出機構へ頑健化する。

## 2. 方式比較

| 方式 | 偽装耐性 | stable toolchain | 依存追加（REQ-3: 上限 60 件/深さ 6） | 判定 |
|------|---------|-------------------|--------------------------------------|------|
| A. コメントマーカー（旧方式） | なし（コメントは任意に書ける） | ○ | 0 | 単独では不採用（本イシューの起点） |
| B. **`clippy::disallowed_methods`（`clippy.toml` 設定）** | **あり（コンパイラの HIR パス解決に基づく。コメントでは抑止不能）** | ○ | **0** | **採用** |
| C. dylint 等のカスタム lint（`fandhe_frontend::unreviewed_raw_html` 相当） | あり | ×（`rustc_private`/nightly 前提が一般的） | 重量依存の追加が必要 | 不採用（REQ-3 違反・依存追加は事前承認が必要） |
| D. rustc ドライバ自作・`register_tool` | あり | ×（unstable API 前提） | 保守コスト大 | 不採用 |
| E. `syn` 等による AST 走査を `cli` へ実装 | 中（属性解析は可能だがパス解決はできない） | ○ | `fandhe-frontend-cli` は外部依存ゼロ方針（`crates/cli/Cargo.toml` 明記）に抵触 | 不採用 |

### 選定理由（方式 B）

`clippy::disallowed_methods` は設定ファイル（`disallowed-methods`）で「呼んでは
いけない関数のフルパス」を宣言でき、**コンパイラが実際に解決した呼び出し先
（HIR 上の `DefId`）に基づいて**警告を出す。これは次の性質を持つ。

- コメントで抑止できない（`clippy::disallowed_methods` 自体を沈黙させるには
  ソース上に検証可能な属性 `#[allow(...)]`/`#[expect(...)]` を書く必要があり、
  「レビューを経た証跡」を残す運用と両立する）。
- `use fandhe_frontend_core::raw_html as fragment;` のようなリネーム import 経由の呼び出し
  も、コンパイラが最終的に解決する定義元パスが同一であるため検出できる
  （実証: `crates/cli/tests/raw_html_lint_e2e.rs`
  `renamed_import_call_is_still_rejected_by_clippy`）。
- 追加の外部クレート依存が不要（clippy 自体は既存のツールチェーン構成要素）。
- stable Rust 上で動作する（`rustc 1.96.0` で検証済み）。

## 3. 採用方式の骨子

1. **`clippy.toml`**（workspace ルート・`templates/default/`）に
   `disallowed-methods = [{ path = "fandhe_frontend_core::raw_html", reason = "..." }]` を
   宣言する。`clippy::disallowed_methods` は設定時 warn-by-default の lint だが、
   `fw gate` の `lint` チェック（`cargo clippy --locked --all-targets -p <crate> -- -D warnings`、
   `--all-targets` はイシュー #315 で追加）と CI の `-D warnings` によりエラー化される。
2. **正当なオプトインは属性のみ**とする。

   ```rust
   #[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: <レビュー根拠>")]
   fandhe_frontend_core::raw_html(trusted_fragment)
   ```

   `#[expect]`（Rust 1.81 以降、stable。本リポジトリのツールチェーンは
   `rustc 1.96.0` で利用可能）は「この lint が実際に発火することを期待する」
   属性であり、次の 2 つの性質を持つ。

   - 呼び出しが実在する限り `disallowed_methods` の発火を吸収し警告を出さない
     （オプトインとして機能する）。
   - **呼び出しが後で削除・変更され lint が発火しなくなると
     `unfulfilled_lint_expectations` が新たな警告を出す**（実証:
     スパイク検証で `raw_html` 呼び出しを他の式に差し替えたところ
     `this lint expectation is unfulfilled` エラーを確認済み）。これにより
     「レビュー済みマーカーだけが残り、実際のリスクのある呼び出しは
     既に消えている」という残置マーカーの陳腐化も検出できる（旧コメント方式
     にはこの自己検証性がない）。

   属性はコード（diff レビューで可視）であり、`unknown_lints`（存在しない
   lint 名を `allow`/`expect` すると警告になる）により偽の lint 名を書く偽装も
   実質的に困難である。

3. **多層防御として `default_escape_check`（テキスト走査）を廃止せず改修する**。
   - clippy が見ない領域（`cfg` で除外されたコード等）に対する保険として、
     テキスト走査は残す。受理条件を「コメントマーカー」から「呼び出し開始行
     自体、または呼び出し直前に隙間なく連なる属性グループ列のいずれかに
     `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED:
     ...")]` 属性が含まれること」へ変更した（`crates/cli/src/gate.rs`
     `line_has_reviewed_expect_attribute`・`reviewed_attribute_covers_call`）。
     イシュー #1116 で「同一行・直前行」の 1 行限定判定から属性ブロック単位の
     判定へ拡張し、rustfmt が `reason = "..."` を複数行へ折り返した属性・
     `#[rustfmt::skip]` 等の重ね掛けも受理できるようにした
     （`docs/policy/raw-html-review-gate.md` §1）。単独のコメントはもはや
     受理しない（回帰テスト: `scan_file_rejects_comment_only_marker_as_spoofable`・
     `fw_gate_still_blocks_comment_only_spoofed_marker`）。
   - **走査の「コード文脈限定」への精密化（イシュー #372）**: 当初の走査は
     `raw_html` の全出現（コメント・文字列リテラル内を含む）を対象としていた
     ため、`fw gate` を本リポジトリ自身（doc コメントで `raw_html()` に言及する
     `interactive`/`wasm-*` 系クレート、メッセージ文言・テストフィクスチャ
     文字列を持つ `cli` 自身）へ適用すると自己参照誤検知で恒常 BLOCKED になる
     問題があった。コメント・文字列リテラル・文字リテラルの内側と、識別子
     サフィックス（`..._raw_html(` 等）を字句規則上呼び出しになり得ない文脈と
     して除外する状態機械（`code_context_mask`）へ精密化し解消した。除外は
     偽陽性のみを削り偽陰性を生まないため主防御・多層防御の構成は変えない
     （詳細は `docs/design/gate-design.md` §2.2a）。
   - **ブランケット抑止の監査**を追加する。`#![allow(clippy::disallowed_methods)]`
     や `#![expect(clippy::disallowed_methods)]`（ファイル・モジュール冒頭の
     内部属性、ファイル全体の検出を一括無効化する）は、呼び出し個別のレビュー
     宣言とは独立に一律違反として列挙する（`BLANKET_DISALLOWED_METHODS_MARKERS`）。
   - **`lint` チェック自体の fail-closed 強化**: `clippy.toml` が削除・エントリが
     欠落すると `disallowed_methods` が沈黙し「検出項目なし」の黙示的 PASS に
     なり得る。`cargo clippy` を起動する前に `clippy.toml` の存在と
     `fandhe_frontend_core::raw_html` エントリの包含をテキスト検証し、欠落時は clippy を
     起動せず `lint` チェックを failed にする（`clippy_policy_check`,
     security.md A05）。

## 4. 多層防御の構成（まとめ）

| 層 | 実体 | 役割 | 偽装耐性 |
|----|------|------|---------|
| 主防御 | `clippy::disallowed_methods`（`fw gate` `lint`・CI `clippy` ジョブとも `cargo clippy --all-targets -- -D warnings`、イシュー #299/#315） | コンパイラのパス解決に基づく偽装不能な検出。`--all-targets` によりテストターゲット内呼び出しも検出範囲に含む | 高 |
| 保険層 | `default_escape_check`（テキスト走査、`crates/cli/src/gate.rs`） | clippy が見ない領域・`fw gate` 単体実行時の二次防御 | 中（属性方式化により旧方式より向上） |
| 監査層 | ブランケット抑止検出（`default_escape_check` 内） | オプトイン地点・一括無効化を `file:line` で可視化 | — |
| ゲート設定の健全性 | `clippy_policy_check`（`lint` チェック） | `clippy.toml` 欠落による沈黙化を fail-closed で検出 | — |

## 5. 残余リスク

- **レビュー済みラッパ関数経由の呼び出し**: `#[expect(...)]` を付けたラッパ
  関数内で `raw_html()` を呼び、そのラッパを別の場所から呼び出す構成の場合、
  ラッパの呼び出し元は lint 対象外になる（disallowed_methods は
  `fandhe_frontend_core::raw_html` への直接呼び出しにのみ反応する）。これは方式 B の
  構造的な限界であり、監査層（ブランケット抑止検出・レビュー地点の
  `file:line` 列挙）による可視性確保で緩和する。ラッパ関数の新設自体は
  コードレビューで検知されることを前提とする。
- **self-hosted CI runner の clippy コンポーネント不在**: `crates/cli/tests/raw_html_lint_e2e.rs`
  は clippy 起動失敗時に明示メッセージ付きで `panic!` する（沈黙スキップ
  しない）ため、不在は CI 失敗として顕在化する。

## 6. 受け入れ条件との対応

| 受け入れ条件 | 対応 |
|-------------|------|
| コメント偽装で回避不能であることの実証 | `crates/cli/tests/raw_html_lint_e2e.rs::comment_only_spoofed_marker_is_still_rejected_by_clippy`（実 clippy 起動） |
| リネーム import 経由の呼び出しも検出 | `crates/cli/tests/raw_html_lint_e2e.rs::renamed_import_call_is_still_rejected_by_clippy` |
| CI で未レビュー呼び出しをブロック | `.github/workflows/ci.yml` `clippy` ジョブ（主防御）+ `test` ジョブの独立ステップ「REQ-1 raw_html() 偽装回避不能テスト (issue #159)」（`raw_html_lint_e2e` を明示実行し受け入れ条件を可視化） |
| 正当なオプトイン経路の保全 | `crates/cli/tests/raw_html_lint_e2e.rs::reviewed_expect_attribute_call_is_accepted_by_clippy` |

運用手順（属性の書式・レビュー規約）は `docs/policy/raw-html-review-gate.md` を参照。
