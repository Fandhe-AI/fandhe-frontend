# 同一キー内容差分の diff 意味論・API 設計（イシュー #1322）

## 1. 目的とトレーサビリティ

- 本書はベンチ起点トラッキング #1313 → Phase 2 #1315（`keyed_dom` の CSR
  適用性能改善）→ #1321（keyed diff への Update op 導入）の第 1 弾（設計）
  として、#1321 の設計成果物を確定する。後続実装は #1323（`core` への
  `KeyedOp::Update` / 内容比較付き diff 関数の追加）・#1324
  （`wasm-client` の DOM 適用経路への反映と browser テスト）が担う。
- 解決する課題: 現行の `apply_keyed_list`
  （`crates/wasm-client/src/keyed_dom.rs`）はキー列の Insert / Remove /
  Move のみを diff し、**同一キーで内容（テキスト等）だけが変わった更新は
  no-op になる**。このため #1315 のベンチ実装ではプリミティブ直接操作で
  更新を書かざるを得ず、宣言的な keyed list 更新経路に API ギャップが
  存在する。
- 本書は **設計のみ**を成果物とし、コード変更を含まない。REQ-1（既定
  エスケープ）・REQ-11（性能）・`docs/design/dom-binding-update-design.md`
  （以下「更新設計書」）の既存確定設計との整合を保ったまま、#1323・
  #1324 が実装できる粒度まで API 形状・意味論を確定する。

## 2. 現状整理

- `crates/core/src/keyed.rs`: `keyed_list()` 構築 API
  （`data-bind-list` / `data-key` マーカー、構築時 fail-closed 検証）。
  diff 機能は持たない。`Node`（`crates/core/src/lib.rs`）は
  `#[derive(Debug, Clone, PartialEq, Eq)]` 済みであり、部分木同士の
  内容同値比較は外部依存を追加せず `core` 内で完結できる。
- `crates/wasm-client/src/keyed_diff.rs`: 純粋 diff 層。
  `KeyedOp { Remove, Insert, Move }` と `diff_keys(old_keys, new_keys)`
  （キー列の `&str`/`String` 比較のみによる O(n) 2 パス、DOM 非依存で
  native テスト可能）。`pub mod keyed_diff` として公開されている。
- `crates/wasm-client/src/keyed_dom.rs`:
  `apply_keyed_list(document, list_element, new_list_node)` が wasm32
  配線層を担う。**旧キー列を DOM（`data-key` 属性）から読み出して**
  `diff_keys` へ渡し、結果の `KeyedOp` 列を適用する。innerHTML 不使用・
  `RawHtml` fail-closed・URL スキーム / `srcset` / イベントハンドラ属性
  検証・SVG 名前空間対応を備える。
- `crates/wasm-full/src/lib.rs::apply_update_for_dirty`: dirty field
  ごとに `find_list_element` → `apply_keyed_list` を呼ぶ更新駆動。
  構造変化時に束縛点対応表（BindingTable）を再スキャンする。
- 既存確定設計: 更新設計書 §5（keyed list が構造変化を表現できる唯一の
  経路）・§7（仮想 DOM・汎用 diff 非採用の根拠）・§9（セキュリティ
  不変条件）。`docs/policy/intentional-non-adoption.md` §3.1（仮想 DOM、
  評価軸 4 点・再評価トリガー）。
- Phase 2（#1315）の安全設計不変条件: innerHTML 不使用・既存 DOM ノードを
  再生成しない・フォーカス保持。本書の設計はこれらを継承し、緩めない。
- crate バージョン（本書執筆時点）: `fandhe-frontend-core` 0.2.0 /
  `fandhe-frontend-wasm-client` 0.3.0 / `fandhe-frontend-wasm-full` 0.7.1。

## 3. diff 意味論

### 3.1 変更検出

キーが新旧両列に存在し、かつ旧アイテムの `Node` と新アイテムの `Node` が
`PartialEq`（部分木全体の構造的同値比較）で不一致のときにのみ
`KeyedOp::Update` を発行する。変更がないキーは op を一切発行しない
（#1323 受入条件「余分な op を発行しない」に対応）。本設計は差分位置の
再帰的な特定（アイテム内部のどのノードが変わったかのパッチ生成）は
行わない ── アイテム単位で「変更あり／なし」の二値判定のみを行う
（詳細は §3.2 c 案）。

### 3.2 適用意味論（採用案・却下案）

採用案は「実装単純性」と「既存不変条件（DOM ノード再生成禁止・フォーカス
保持）との両立」の 2 点で選定する。

| 案 | 内容 | 判定 |
|----|------|------|
| a. アイテム全置換 | 変更検出したアイテムを `Remove` + `Insert` 相当で丸ごと作り直す | **却下**。「既存 DOM ノードを再生成しない」不変条件（更新設計書 §7.2、#1315/#1321 受入条件）に反し、フォーカス・入力途中の値・IME 変換状態が失われる |
| b. テキストのみ更新 | アイテムルート要素は保持し、テキストノードの差し替えのみ行う | **却下（限定案として記録）**。`class` / `disabled` 等の属性変更に更新経路がなく、解決したい API ギャップ（「内容変更全般が no-op になる」）が部分的に残存する |
| c. **浅い in-place 更新（深さ 1 打ち切り）** | アイテムルート要素の同一性（DOM ノード参照）を維持したまま、(1) 子ノード列は新旧で不一致なら detached コンテナ上で `build_dom_node` 相当のプログラム的構築を先に行い構築成功を確認し、(2) 構築成功後にのみルート要素の属性差分を `setAttribute`/`removeAttribute` で適用し、続けて旧子ノード列を新しい子ノード列と交換する | **採用** |
| d. ノード全体の再帰 diff | アイテム内部の任意深さのノードを新旧比較し、変更箇所だけへ最小パッチを当てる（morphdom 型） | **却下**。仮想 DOM・汎用 diff 相当であり、§4 の評価軸に基づき非採用（`docs/policy/intentional-non-adoption.md` §3.1 と同じ判断軸） |

c 案の詳細:

- **ルート属性の差分適用**: 旧ノードの属性集合と新ノードの属性集合を比較し、
  削除された属性は `removeAttribute`、追加・変更された属性は
  `setAttribute` で適用する。`data-key` / `data-bind-list`
  （更新設計書 §5.1 の予約属性）は比較・書き換えの対象外とする（§5 不変
  条件 3 参照）。属性値の検証規則（URL スキーム・`srcset` 候補分割・
  イベントハンドラ属性のブロック）は既存の `build_dom_node`
  （`crates/wasm-client/src/keyed_dom.rs`）が新規ノード構築時に適用する
  規則と同一のものを、Update 経路の属性適用にもそのまま適用する
  （検証規則を複製せず共有する）。
- **子ノード列の再構築（detached 構築 → 検証 → 交換の順序、イシュー #1330
  レビュー指摘 P0 を受けて確定）**: 子ノード列（テキスト・要素混在を含む）
  が新旧で一致しない場合、**新しい子ノード列を DOM 未接続（detached）の
  コンテナ（例: `DocumentFragment` または未接続の一時要素）上へ
  `build_dom_node` によるプログラム的構築（`create_element` →
  `set_attribute` → `set_text_content` → `append_child`）で組み立てる**。
  この構築は既存 DOM に一切触れずに行う。`build_dom_node`
  は子孫に `RawHtml` が含まれる場合や要素生成・属性設定に失敗した場合に
  `None` を返しうる（更新設計書 §5.2・§9）ため、構築フェーズで `None` が
  1 件でも発生した場合は直ちに構築を中断し、detached コンテナごと破棄
  する。既存の子ノード列・ルート属性のいずれにも変更を加えず、当該
  アイテムの Update 適用全体（属性差分適用を含む）を諦めて他アイテムへの
  適用へ進む（§6 不変条件 6 の fail-closed 契約。旧 DOM 削除を構築成功の
  確認より先に行わない）。
  新しい子ノード列の構築が**すべて**成功した場合にのみ、ルート属性の
  差分適用（`setAttribute`/`removeAttribute`）を行い、続けて既存の子ノード
  をすべて削除して detached コンテナの子ノード列を `append_child` で
  移し替える（構築と検証が完了した内容だけを旧 DOM と交換する。
  「削除してから構築」ではなく「構築・検証してから削除して交換」の順序を
  常に守る）。テキストのみの変更（例: `<span>1</span>` → `<span>2</span>`）
  は、この手順でも結果として `set_text_content` 相当の 1 回の DOM 操作に
  帰着する。子要素の内部に HTML パーサを一切経由しない（更新設計書 §9
  不変条件 1 の延長）。
- **打ち切りの意味**: 「深さ 1」とはアイテムルート要素の属性は差分適用、
  子孫は変更があれば丸ごと再構築するという意味であり、子孫のさらに内側
  だけを選択的に更新する再帰比較は行わない。これにより diff 実装は
  「ルート属性比較 1 回 + 子ノード列同値判定 1 回」に収まり、コード量・
  レビューコストを小さく保つ。

### 3.3 制約（フォーカス保持の範囲）

- c 案では、変更されたアイテムの**内部**にフォーカスがある場合、子ノード
  列が再構築されるため内部フォーカスは保持されない（アイテムルート要素
  自体の同一性は保持されるため、アイテム外からアイテムへのフォーカスの
  戻り先特定は可能だが、アイテム内部の特定要素へのフォーカス位置までは
  保持しない）。編集中の入力値等、フォーカス・キャレット位置の保持が必須
  なノードは、本設計のスコープ外とし、既存の束縛点更新経路
  （`data-bind-text` / `data-bind-attr`、更新設計書 §3・§4）を使う指針を
  実装ガイドとして #1323／#1324 に引き継ぐ。
- ネストした `keyed_list` を含むアイテム（アイテム内部に別の `keyed_list`
  呼び出しがある場合）は、外側の keyed list の Update 判定において内側
  keyed list 部分木も含めて `Node` の構造的同値比較に含まれる。すなわち
  内側リストの内容が変われば外側アイテムも「変更あり」と判定され、子ノード
  列再構築（内側リストの DOM ノードも再生成）が起きる。内側 keyed list を
  外側の Update 判定から独立させ、内側リスト自体の diff 結果を再利用する
  最適化は本書のスコープ外とする（内側リストが独立した dirty field として
  駆動される既存の `apply_update_for_dirty` 前提とは別に、外側アイテムの
  Update が発生した場合の扱いを明確化するに留める）。

### 3.4 op 発行順・既存互換

- 既存 3 op（`Insert` / `Remove` / `Move`）の発行順・内容は不変とする
  （#1323 受入条件「既存テスト無変更」）。`diff_keys`
  （キー列のみを見る既存関数）のシグネチャ・挙動・テストは変更しない。
- `Update` の発行位置は、既存の 2 パス（Remove パス → Insert/Move パス）
  の**後**に第 3 パスとして追加する: 2 パス終了後の「新しい並び」に
  対して、新旧両方のキー列に存在する保持キー（`Remove` パスで削除されな
  かったキー）**すべて**について、当該キーに対し第 2 パスで `Move` が
  発行されたか否かに関わらず、対応する旧アイテム `Node` と新アイテム
  `Node` を比較し、不一致であれば `KeyedOp::Update` を発行する（イシュー
  #1330 レビュー指摘 P1 を受けて確定。「位置が変わらないキーのみ」への
  限定は行わない）。発行順序はキーの新しい並び順（`new_keys` の順序）に
  固定する。判定条件は「保持キーであるか」の一意な条件のみであり、
  実装解釈の余地を残さない。
- `Move` されたアイテムについても内容変更が同時に起きうる（キーは同じだが
  位置も内容も変わるケース）。この場合は上記の通り `Move` 発行に加えて
  同一キーへの `Update` も発行する（1 キーに対し `Move` と `Update` の
  両方が起きうることを許容する。位置決定と内容適用は独立した関心であり、
  `Move` 1 種類に内容更新の意味を混在させない）。op 列全体における
  順序は、`Move` は第 2 パス（`new_keys` 順）で先に発行され、`Update` は
  第 3 パス（`new_keys` 順）で後に発行されるため、**同一キーの `Move` と
  `Update` は隣接しない**（両パスの間に他キーの `Move`/`Insert` op が
  挟まりうる）。具体例: `old_keys = ["a", "b"]`・`new_keys = ["b", "a"]`
  で、`a` は位置のみ変更（内容一致）、`b` は位置と内容の両方が変更された
  場合、期待される op 列は `[Move{key: "b", ...}, Update{key: "b"}]`
  （`a` は Move も Update も発行しない。新しい並び順が `["b", "a"]`
  のため `b` が先に処理される）。`old_keys = ["a", "b"]`・
  `new_keys = ["a", "b"]`（Move 対象なし）で両キーとも内容変更がある
  場合は `[Update{key: "a"}, Update{key: "b"}]`（`new_keys` 順）となる。

### 3.5 計算量・SSR/SSG への影響

- キー照合は既存どおり O(n)（`old_keys.len() + new_keys.len()`）。内容比較
  パスは保持キー（Remove されなかったキー）それぞれについて対応する新旧
  `Node` 部分木の `PartialEq` 判定を行うため、計算量は
  O(Σ 保持キーの部分木サイズ) が追加される。全体としてリストサイズに線形の
  計算量に留まり、既存の「仮想 DOM を導入しない」判断（更新設計書 §7.1）の
  前提（変更フィールド数に比例する計算量）を破らない。
- `render()`（SSR/SSG/CSR 初回マウント共通のレンダラ）の出力・シグネチャは
  一切変更しない。本設計は CSR 側の差分適用（`wasm-client`）にのみ関わる。

## 4. API 形

### 4.1 責務分担（受入条件の中核）

- **op 生成（diff・内容比較）= `fandhe-frontend-core`**。DOM 非依存で
  `cargo test -p fandhe-frontend-core` の native 単体テストとして検証
  でき、`forbid(unsafe_code)`・外部依存ゼロの制約下に置く。
- **DOM 適用 = `fandhe-frontend-wasm-client`**。`web-sys` を用いた実 DOM
  操作（要素生成・属性適用・`insert_before`・削除）に閉じる。browser
  テスト（`wasm-pack test --headless`）で検証する。

### 4.2 採用案: 明示 API + core への diff 移管

- **core（`crates/core/src/keyed.rs` へ追加）**:
  - `KeyedOp`（`Remove` / `Insert` / `Move` / **`Update`**）を core 側で
    新設する。既存 3 variant は wasm-client 側 `KeyedOp` と等価な情報を
    持つ（`Update` variant のみ新規: `key: String` と、適用に必要な新旧
    `Node` 参照または複製済み `Node` を持つ。具体的なフィールド型・
    所有権設計（`&Node` 借用か `Node` 複製か）は #1323 側の実装確定事項
    とする）。
  - `diff_keys(old_keys: &[String], new_keys: &[String]) -> Vec<KeyedOp>`
    （キー列のみを見る既存互換版）を core へ移設する。
  - 内容比較付きの新設 diff 関数（例: `diff_keyed_items(old_items:
    &[(String, Node)], new_items: &[(String, Node)]) -> Vec<KeyedOp>`。
    正式名称は #1323 で確定）を追加し、§3 の意味論（3 パス: Remove →
    Insert/Move → Update）を実装する。
- **wasm-client（`crates/wasm-client/src/keyed_diff.rs`）**: 型定義・
  `diff_keys` の実体を core からの re-export へ置き換える
  （`pub use fandhe_frontend_core::keyed::{KeyedOp, diff_keys};` 相当）。
  型の同一性を保つことで既存の呼び出し側コンパイル互換を維持する。
- **wasm-client（`crates/wasm-client/src/keyed_dom.rs`）**: 既存の
  `apply_keyed_list(document, list_element, new_list_node)` のシグネチャ・
  挙動は**不変**とする（DOM 上の `data-key` 列からしか旧内容を得られない
  ため、引き続き `Update` は発行されない）。`Update` に対応する新規関数
  （例: `apply_keyed_list_with_previous(document, list_element,
  previous_list_node, new_list_node)`。正式名称は #1324 で確定）を追加し、
  呼び出し側が直近の `Node`（前回描画時の keyed list ノード）を保持して
  いる場合にのみ内容比較付き diff を使う。
- **wasm-full（`crates/wasm-full/src/lib.rs`、#1324 で反映）**: `Runtime`
  が field ごとに直近適用済み keyed list の `Node` を保持し、新 API へ
  渡す。保持がない初回描画・保持が破棄されたフォールバック時は既存の
  `apply_keyed_list`（DOM 読み出し）経路を使う。

### 4.3 却下案: 暗黙差分（DOM からの旧内容読み出しで Update 判定）

`apply_keyed_list` が現行どおり DOM（`data-key` 属性・`textContent` 等）
から旧内容を読み出して `Update` 判定まで行う案は却下する。理由:

1. op 生成が wasm-client に残り、§4.1 の責務分担（op 生成は core）に
   反する。
2. DOM を真実源とする内容比較は、ブラウザの属性値正規化（例: 数値属性の
   表現ゆれ）・空白文字の扱いの影響を受け、決定性（更新設計書 §7.4「決定性」
   の評価軸）を損なう。
3. DOM 読み出しに依存する比較ロジックは native 単体テストで検証できず、
   `cargo test -p fandhe-frontend-wasm-client` のみでは browser テスト
   （実行コストの高い経路）を必ず経由する形になる。

### 4.4 後方互換（0.x マイナーの範囲）

- `KeyedOp` への `Update` variant 追加は、呼び出し側の非網羅
  （exhaustive でない）`match` を壊しうる破壊的変更である。イシュー #638
  規約（0.x の破壊的変更はマイナーバンプ）に従い、#1323 実装時に
  `fandhe-frontend-core` を 0.2.0 → 0.3.0 へ、#1324 実装時に
  `fandhe-frontend-wasm-client` を 0.3.0 → 0.4.0 へ、それぞれバンプする
  ことを実装手順として明記する。依存元クレート（`fandhe-frontend-wasm-full`
  等）の `version = "..."` 要求追随は `xtask check-dep-versions --fix`
  （`.claude/rules/ci.md` `dep-version-check` ジョブ参照）で機械的に行う。
- 既存の公開 API（`keyed_list()` / `apply_keyed_list()` /
  `diff_keys()` のシグネチャと挙動）は不変のまま維持する。破壊的変更は
  `KeyedOp` の variant 追加のみに閉じる。

## 5. 仮想 DOM との境界（`intentional-non-adoption.md` §3.1 評価軸での確認）

`docs/policy/intentional-non-adoption.md` §3.1 が定める 4 つの評価軸に
照らし、本設計が仮想 DOM・汎用 diff の再導入に該当しないことを確認する。

- **明示性**: 対象は `data-key` で特定された keyed list の直下アイテム
  1 件に閉じる。どのノードが Update 対象になりうるかは `grep -r
  'data-bind-list\|data-key'` で静的に把握できる既存の明示性を維持する。
  仮想 DOM のようにコンポーネントツリー全体が diff 対象になるわけではない。
- **決定性**: 判定は `Node` の構造的 `PartialEq`（値の同値性）のみに
  基づき、要素種別の推定・key のヒューリスティック推定を行わない。同じ
  新旧 `Node` 対に対して常に同じ op 列が決定的に得られる。
- **機械検証可能性**: §3 の op 発行規則（3 パス固定・発行順固定）は
  native 単体テストで op 列を決定的に固定できる（§6 テスト計画）。
- **コンテキスト消費**: 追加実装は「アイテムルート属性の差分適用 1 パス
  + 子ノード列同値判定 1 パス」に限定され、汎用 diff アルゴリズム
  （reconciliation）のような新規の大きな抽象層を追加しない。

**結論**: 本設計は「keyed list 専用経路の拡張（アイテム単位の浅い
in-place 更新）」であり、ノード木の任意位置を対象とする再帰 diff
（仮想 DOM・morphdom 型汎用 diff）は導入しない。`docs/policy/
intentional-non-adoption.md` §3.1 の再評価トリガー（perf-browser ゲート
での REQ-11 受け入れ基準の恒常的未達）には該当しない。

## 6. セキュリティ不変条件（更新設計書 §9 の継承）

Update 適用経路も更新設計書 §9 の既存不変条件をすべて継承し、緩めない。

1. **HTML 非解釈**: 子ノード列の再構築・テキスト差し替えは
   `set_text_content` / プログラム的な要素構築のみを経由し、
   `innerHTML` / `insertAdjacentHTML` を一切使わない（更新設計書 §9
   不変条件 1 の延長）。
2. **属性適用は `setAttribute`/`removeAttribute` のみ**: URL スキーム・
   `srcset` 候補分割・イベントハンドラ属性（`on*`）のブロックは、既存の
   `build_dom_node` が新規ノード構築時に適用する検証規則と同一のものを
   Update 経路の属性差分適用にも適用する（§3.2 c 案、規則の複製ではなく
   共有関数の再利用とする）。
3. **`RawHtml` は部分木ごと fail-closed**: keyed list アイテム内に
   `RawHtml` ノードが含まれる場合の扱いは既存の `keyed_list()` 構築時
   検証（`KeyedListError::NonElementItem` 等、更新設計書 §5.2）を継承し、
   Update 経路のために新たな迂回を作らない。
4. **予約属性は Update 対象外**: `data-key` / `data-bind-list`
   （更新設計書 §5.1）は §3.2 の属性差分適用の比較対象・書き換え対象から
   除外する。Update 経路がキー照合契約自体を書き換えられないことを構造的
   に保証する。
5. **`raw_html()` を呼ばない**: 束縛点更新・keyed list Insert/Remove/Move
   と同様、Update 経路にもエスケープ迂回オプトインを組み込まない（REQ-1）。
6. **fail-closed**: 新旧 `Node` の比較・属性適用に失敗した場合（想定外の
   属性名・DOM API 呼び出し失敗等）は当該アイテムの Update 適用を諦め、
   他アイテムへの適用を妨げない（更新設計書 §9 不変条件 6 と同じ設計、
   `unwrap()`/`panic!` を使わない）。§3.2 の子ノード列再構築は
   detached コンテナ上での構築・検証をすべて終えてから旧 DOM と交換する
   順序を守り、構築途中の失敗（`build_dom_node` が `None` を返す場合を
   含む）で旧 DOM の子ノード・ルート属性が失われないことを構造的に
   保証する（旧 DOM を消してから作り直す実装は許容しない）。
7. **ログの機微情報非露出**: Update 適用時の警告ログは固定英語文言のみとし、
   キー値・アイテム内容（アプリ状態）を含めない（更新設計書 §9 不変条件 7
   の継承）。

## 7. テスト計画（#1323／#1324 受入基準へのマッピング）

- **core（#1323）**:
  - 同一キー内容変更のみ（構造変化なし）で `Update` 1 件のみが発行される
    ことを固定するテスト。
  - 内容変更なし（新旧 `Node` が完全一致）で op がゼロ件になることを固定
    するテスト。
  - `Insert` / `Remove` / `Move` との混在ケース（例: あるキーは削除、
    別キーは新規挿入、別キーは移動かつ内容変更）で、無関係キーへの
    余分な op が発行されないことを固定するテスト。
  - 既存の `diff_keys` テスト（`crates/wasm-client/src/keyed_diff.rs` の
    move 元テスト群）は core への移設後もすべて無変更で通過することを
    確認する。
- **wasm-client（#1324）**:
  - 同一キー・新テキスト内容での `set_text_content` 相当の DOM 更新を
    browser テストで確認する。
  - `<script>` タグ相当のペイロードを内容変更として渡した際に HTML として
    解釈されないことを確認する XSS 回帰テスト（既存の keyed_dom XSS
    回帰テストと同型）。
  - Update 対象アイテムのルート要素が `is_same_node` で同一 DOM ノードの
    まま保たれる（再生成されない）ことを確認するテスト。
  - Update 実測のオーバーヘッドが既存操作（#1315 実測 1.32ms 相当）の
    2 倍以内に収まることを確認する性能回帰テスト（perf-browser ハーネス
    経由）。
- **スコープ外**: #1319／#1320 の性能施策一般、`wasm-thin` への適用
  （`docs/policy/intentional-non-adoption.md` §3.10 の非採用判断を継承し
  対象外）、束縛点更新（`crates/wasm-client/src/binding.rs`）自体の変更。

## 8. 受け入れ基準対応表

| イシュー #1322 の受け入れ条件 | 対応する本書の節 |
|-------------------------------|------------------|
| 採用案と却下案の根拠が書かれたレビュー可能な設計文書が存在する | §3.2（diff 適用意味論の 4 案比較）・§4.3（API 形の却下案） |
| core / wasm-client の責務分担（op 生成は core、DOM 適用は wasm-client）を明記している | §4.1・§4.2 |

## 9. 関連文書との整合確認

- `docs/design/dom-binding-update-design.md`（§5.3 に本書への前方参照を
  追記済み）。既存本文の削除・改変は行わない。
- `docs/policy/intentional-non-adoption.md` §3.1 は編集していない
  （本書 §5 が再評価トリガー非該当であることを論証するに留める）。
