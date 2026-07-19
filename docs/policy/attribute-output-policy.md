# 属性出力ポリシー（URL スキーム検証・イベントハンドラ属性ブロック）

**本文書のステータス**: 確定（イシュー #373）。

## 1. 目的とトレーサビリティ

`rws-core` の既定エスケープ（`core/src/escape.rs`、OWASP XSS Prevention
Cheat Sheet Rule #1 準拠の 5 文字置換）は属性値コンテキストからの
「脱出」（`"` による breakout 等）を防ぐが、**脱出を伴わない攻撃**は防げ
ない。代表例が URL スキーム経由の XSS である。

```
el("a", vec![("href", user_url)], ...)
```

に `javascript:alert(1)` を渡した場合、5 文字（`"` `'` `<` `>` `&`）の
いずれも含まないため、既定エスケープをそのまま通過して
`href="javascript:alert(1)"` として出力される。これはブラウザ上でリンク
クリック時に任意 JS を実行させる、breakout を伴わない別種の脅威である。

本書は `core/src/escape.rs` の rustdoc「スコープ外」節・イシュー #367 の
out-of-scope 節で明示的に先送りされてきたこの領域について、脅威整理と
対応方針を確定し、`docs/spec/04-requirements.md` REQ-1（既定エスケープ）
の趣旨を「属性出力全体の安全性」へ拡張する形で記録する。関連イシュー:
#367（既定エスケープの範囲確定）。

## 2. 脅威マトリクス

| 脅威 | breakout を伴うか | 既定エスケープで防げるか | 本書での対応 |
|------|---|---|---|
| URL スキーム経由の XSS（`href="javascript:..."` 等） | 伴わない | 防げない | §3.1 URL 許可スキーム検証（採用） |
| イベントハンドラ属性への注入（`onclick="..."`） | 属性値が常に JS 実行コンテキスト | 部分的（breakout は防ぐが、属性自体が存在すれば値がそのまま実行される） | §3.2 `on*` 属性の一律不出力（採用） |
| `style` 属性経由のデータ流出・レガシー CSS 実行ベクタ | 伴わない場合あり | エスケープは breakout を防ぐのみ | `intentional-non-adoption.md` §3.7（非採用。理由: 現代ブラウザでは CSS 経由コード実行ベクタは既に廃止済み。属性値エスケープで breakout は防止済み） |
| `srcset` の複数候補中 1 件が危険スキーム | 伴わない | 防げない（カンマ区切りの複合構文） | §3.1 の拡張・`is_safe_srcset`（1 候補でも不合格なら属性全体をスキップ。`render_into`・`binding_dom.rs`・`keyed_dom.rs` の 3 経路すべてで検証） |
| `<base href>` / `<meta http-equiv="refresh">` 経由の間接的 URL 制御 | 該当なし（別要素からの間接効果） | 対象外（属性値そのものの脅威ではない） | 本書のスコープ外（§6 参照。対策強化が必要なら別 Issue） |

## 3. 採用した対策

### 3.1 URL 属性の許可スキーム検証

**実装**: `core/src/url.rs`（`rws_core::is_safe_url` / `rws_core::URL_ATTRS`
/ `rws_core::is_url_attr`）。

- **正リスト（`URL_ATTRS`）**: `href` / `src` / `action` / `formaction` /
  `xlink:href` / `poster` / `cite` / `data` / `background` / `ping` /
  `dynsrc` / `lowsrc`。属性名の照合は ASCII 大文字小文字非依存。
  `URL_ATTRS` と許可スキームのリストは `core/src/url.rs` を単一の情報源
  とし、`rws-core` から `pub use` された関数・定数を `rws-wasm-client` 等の
  上位クレートが再利用する（コピーを作らない、A05 対策）。
- **許可スキーム（deny by default）**: スキームなしの相対 URL（`/path`・
  `./x`・`?q`・`#frag`・protocol-relative `//host`）、および
  `http` / `https` / `mailto` / `tel`（大文字小文字非依存）。
- **拒否**: 上記以外の全スキーム（`javascript:` / `data:` / `vbscript:` /
  `blob:` / 未知スキーム）。
- **スキーム抽出**: ブラウザの寛容な URL パースを模倣した過剰側安全設計。
  判定前に ASCII タブ・改行（`\t` `\n` `\r`）を全位置で除去し、先頭の
  C0 制御文字・空白をトリムした上で、`/` `?` `#` `\` のいずれよりも前に
  現れる `:` までの区間が `[a-zA-Z][a-zA-Z0-9+.\-]*` に一致する場合のみを
  スキームとみなす。これにより `java\tscript:`・`\u{0}javascript:`・
  ` javascript:` のような偽装形を遮断しつつ、`/path/a:b` のような相対
  URL 中のコロンをスキーム区切りと誤認しない。
- **`srcset` の扱い**: カンマ区切りの複数候補を持つ特殊構文のため
  `URL_ATTRS` には含めず、`core/src/url.rs` の `is_safe_srcset`（単一の
  情報源）が候補分割（カンマ区切り→各候補の先頭空白区切りトークンを URL
  部分として抽出）と `is_safe_url` 適用を行う。1 候補でも不合格なら属性
  全体をスキップする（部分的な書き換えは決定性を損なうため行わない）。
  `render_into`・`binding_dom.rs`・`keyed_dom.rs` の 3 経路すべてが
  `is_safe_srcset` を参照し、判定ロジックを重複させない（イシュー #373
  レビュー指摘対応: 従来は `render_into` にのみインライン実装されており、
  wasm-client の実 DOM 直接更新経路では `srcset` が `URL_ATTRS` 非該当
  ゆえに未検証だった）。

**適用箇所（3 経路に同一保証）**:

1. **`render_into`（`core/src/lib.rs`）**: SSR・SSG・CSR いずれのモードも
   共通で通る `rws_core::render()` の内部実装。`URL_ATTRS` 該当属性の値が
   `is_safe_url` 不合格の場合、`srcset` の値が `is_safe_srcset` 不合格の
   場合は、属性ごと出力をスキップする（既存の不正属性名スキップ・不正
   タグ名スキップと同型の fail-closed 挙動。panic させない）。
2. **`rws-wasm-client` の実 DOM 直接更新経路**（`binding_dom.rs` の
   `apply_one` / `keyed_dom.rs` の `build_element`）: `render()` を通らず
   `set_attribute` を直接呼ぶ経路が存在するため、render 時検証だけでは
   不十分。両関数とも `rws_core::is_url_attr` / `rws_core::is_safe_url`、
   および `srcset` については `rws_core::is_safe_srcset` を通し、不合格の
   場合は `set_attribute` を呼ばない。`binding_dom.rs` は
   さらに `remove_attribute` で既存の（束縛前に設定されていた）属性値を
   除去する（fail-closed。古い安全値が残ることによる不整合も避ける）。

検証は常に**生の属性値**に対して行う。`escape_html_into` によるエスケープ
後の文字列を判定対象にしない（エスケープは別コンテキスト向けの文字置換で
あり、スキーム判定の対象を歪めるため）。

### 3.2 イベントハンドラ属性の一律ブロック

**実装**: `core/src/url.rs`（`rws_core::is_event_handler_attr`）。

本フレームワークのインタラクションモデルは `data-hydrate` /
`data-bind-*` マーキングと dispatch（`docs/api/interactive-api.md`）で
あり、インライン JS（`onclick` 等）は設計上の正規経路に存在しない。
属性名が ASCII 大文字小文字非依存で `on` から始まり、かつ `on` の後に
1 文字以上続く場合（`on` 単体は対象外。HTML 標準に `on` という名前の
イベントは存在しない）、値によらず出力しない。

**適用箇所**: `render_into`（`core/src/lib.rs`）・`binding_dom.rs` の
`apply_one`・`keyed_dom.rs` の `build_element`（§3.1 と同一の 3 箇所）。

`on*` 始まりの正当なカスタム属性が必要な場合は `data-*` を使う運用と
する。本対策は制限の追加のみであり、既定エスケープの迂回経路の新設には
当たらない。

## 4. 正リストの所在（単一の情報源）

- `URL_ATTRS` 定数・許可スキーム判定ロジック・`is_event_handler_attr`:
  `core/src/url.rs`（`rws-core` の公開 API として `pub use` 済み）
- 利用者コード・他クレートからの利用は必ず `rws_core::{is_safe_url,
  is_url_attr, is_event_handler_attr, URL_ATTRS}` を経由し、独自にリストを
  複製しない。

## 5. 運用: 属性がスキップされた場合の診断

`core` はログ機構を持たないため、URL 属性・イベントハンドラ属性が
スキップされても実行時ログは出力されない（現状構造で自然にログ経由の
情報露出を回避している、A09 対策）。開発時に「属性が消えた」ことに
気づくには以下を確認する。

- 出力 HTML に該当属性（例: `href=`）自体が存在するかを確認する。
- 属性値が `javascript:` 等の危険スキームでないか、またはタブ・改行・
  制御文字による偽装形になっていないかを確認する（§3.1 のスキーム抽出
  規則参照）。
- `on*` で始まる属性名を意図的に使っていないか確認する（`data-*` へ
  置き換える）。

将来診断機能（開発モードでのみ有効な警告出力等）を追加する場合は、
属性値そのものをログに含めない設計とすること（A09 対策の継続）。

**`fw gate` との連携（弱体化の機械検出、イシュー #401）**: 本書が定める
不変条件（`URL_ATTRS` 正リスト・許可スキーム 4 種・ガード関数 4 種の
定義/呼び出し）は `core/tests/xss_escape.rs` の回帰テストに加え、
`fw gate url_validation_check`（`cli/src/gate.rs`、詳細は
`docs/design/gate-design.md` §2.4）が保険層としてテキスト走査で検出する。
検出対象は (1) `role != "core"` ディレクトリでの未ガード DOM 属性設定
呼び出しの新規追加、(2) `role = "core"` ディレクトリでの `URL_ATTRS`/
許可スキームの緩和、(3) ガード関数呼び出しの削除の 3 種（U1〜U3）。
`fw gate` 実行時に `url_validation_check` が `passed: false` を返した
場合、本書 §3 の対策のいずれかが弱体化された可能性を疑い、該当ファイルの
`file:line` を出力から特定して復元する。

## 6. スコープ外（放置しない事項）

- `<base href>` / `<meta http-equiv="refresh">` 経由の間接的 URL 制御の
  追加対策: これらは対象属性そのものの脅威ではなく、ページ内の別要素が
  ナビゲーション先へ間接的に影響する経路であり、本書の対象（属性値の
  URL スキーム検証）とは性質が異なる。対策強化が必要と判断された場合は
  別 Issue として提案する（`.claude/rules/out-of-scope-tracking.md` 準拠）。
  本件は `docs/policy/intentional-non-adoption.md` §3.18 に再評価トリガー
  付きで登録済み。
- `fw gate` への「URL 属性検証の弱体化検出」ゲート追加: イシュー #401 で
  `url_validation_check` として実装済み（上記§5 参照、
  `docs/design/gate-design.md` §2.4）。clippy `disallowed-methods` による
  `web_sys::Element::set_attribute` の主防御化・`web_sys` 型付きセッター
  （`set_href`/`set_src` 等）の検出は同イシューのスコープ外として別 Issue
  候補に残す。
- `templates/default/` 側への周知ドキュメント反映: 標準プロジェクト
  テンプレート利用者向けの説明追加は本書のスコープ外。別 Issue 候補。

## 7. 参照

- `core/src/url.rs`（`is_safe_url` / `URL_ATTRS` / `is_url_attr` /
  `is_event_handler_attr` の実装・ユニットテスト）
- `core/src/lib.rs`（`render_into` のクレート冒頭不変条件 8・9、適用箇所）
- `core/src/escape.rs`（既定エスケープの守備範囲、スコープ外節から本書への参照）
- `core/tests/xss_escape.rs`（`mod url_scheme_xss`、SSR/SSG/CSR 経路の回帰テスト）
- `wasm-client/src/binding_dom.rs`（実 DOM 属性束縛更新経路の適用）
- `wasm-client/src/keyed_dom.rs`（keyed list プログラム的構築経路の適用）
- `wasm-client/tests/binding_browser.rs`（実ブラウザでの束縛更新回帰テスト）
- `wasm-full/tests/xss_escape_wasm.rs`（実ブラウザでの `set_inner_html` 経由 DOM 読み戻し回帰テスト）
- `docs/policy/intentional-non-adoption.md` §3.5〜§3.9（非採用項目の評価軸・再評価トリガー）
- `docs/api/component-api.md`（属性出力の検証仕様への参照）
- `.claude/rules/coding-rust.md`（既定エスケープ厳守・`forbid(unsafe_code)`・依存上限）
- `.claude/rules/security.md`（OWASP Top 10 チェック）
