# keyed list 連続 Insert の行プロトタイプ clone 化設計（イシュー #1385）

## 1. 目的

親 #1383（Phase 3: create の行テンプレート複製化）の後段。前段 #1384
（PR #1398）は core 側に構造同型判定・束縛点パス導出（`fandhe_frontend_core::
keyed::template::derive_item_template`/`text_values`）を実装済みで、本
イシューはその DOM 適用側（`fandhe-frontend-wasm-client`）を実装する。

CSR の初期挿入（例: 1,000 行の keyed list 生成）は、行の構造がすべて
同一（タグ・属性の並びが一致し、テキスト値のみ異なる）であるにも
かかわらず、従来は行ごとに `create_element`/`create_text_node`/
`set_attribute`/`append_child` を個別に呼び出してノード木をゼロから
構築していた。solid / lit / vue 系フレームワークが採用する「静的骨格は
1 回だけ構築し、以後は `cloneNode(true)` + 動的値の書き込みのみ」方式へ
転換し、行あたりの境界呼び出し回数と 1 呼び出しあたりのコスト（タグ名
文字列のマーシャリング・個別ノード生成）を削減する。

## 2. 重要な事前調査結果（計画上の前提訂正）

実装着手前の調査で、**本番経路 `apply_ops_with_items`
（`crates/wasm-client/src/keyed_apply.rs`）は連続 Insert を 1 件ずつ
`create_item` → `insert_before_batch`（長さ 1）で適用していた**ことが
判明した。#1320 が導入した連続 Insert 区間検出 + `DocumentFragment` 集約
は `apply_ops_list`（`#[cfg(test)]`、本番非到達の同型移植テスト）にしか
実装されておらず、本番経路には存在しなかった。

このため本イシューは「行プロトタイプ clone 経路の追加」に加えて、
「連続 Insert 区間検出を本番経路 `apply_ops_with_items` へ導入する」
作業を不可分に含む（プロトタイプ経路の入口が区間そのものであるため）。
`apply_ops`/`apply_ops_list`（`#[cfg(test)]`）は削除せず、doc の
「本番非到達」記述のみ現状に合わせて更新した。

## 3. 設計

### 3.1 層の分担

- **純粋層 `keyed_apply.rs`（native テスト可能）**: `apply_ops_with_items`
  の `KeyedOp::Insert` 処理を `Peekable` による極大区間検出
  （`apply_ops_list` と同じ「index がちょうど 1 ずつ増える極大区間」
  規則）へ置き換えた。区間長 2 以上のとき `derive_item_template` で
  同型判定し、成立時は新設トレイトメソッド
  `KeyedListDom::create_items_from_template` で一括構築、不成立・`None`
  返却時は従来どおり `create_item` を 1 件ずつ適用する。区間全体は
  `insert_before_batch` 1 回で挿入する（既存の DocumentFragment 集約、
  #1320 との併用）。
- **wasm32 配線層 `keyed_dom.rs`（`WebSysKeyedDom`）**: 新設トレイト
  メソッドの web-sys 実装（プロトタイプ構築 → `cloneNode(true)` →
  束縛点書き込み）。

### 3.2 トレイト拡張（`KeyedListDom::create_items_from_template`、`pub(crate)`）

```rust
fn create_items_from_template(
    &mut self,
    template: &fandhe_frontend_core::keyed::template::ItemTemplate,
    items: &[(String, &Node)],
) -> Option<Vec<(String, Self::NewNode)>> {
    let _ = (template, items);
    None
}
```

既定実装は `None`（＝非対応・フォールバック）。`clear_children` と異なり
既定 `None` は「従来経路へ倒れる」fail-safe 方向であり、既存モック
（`CountingDom`/`PoisonedCreateDom`/`MockChildDom` 等）は無改修で通る。

### 3.3 `WebSysKeyedDom::create_items_from_template` の実装

1. `items[0]` を `build_dom_node_with_namespace` で detached 構築して
   プロトタイプとする（既存 `create_item` と同じ構築手段のため、URL
   スキーム・イベントハンドラ・`srcset` 検証はそのまま継承される）。
   プロトタイプ自身を `items[0]` の成果物として使う（clone 1 回を節約）。
2. `items[1..]` は `Node::clone_node_with_deep(true)` 1 回のみで複製し、
   ルート要素の `data-key` をそのアイテム自身のキーへ `set_attribute` で
   上書きしたうえで、`template.text_paths()` が示す束縛点へ
   `text_values` が返すテキスト値を `CharacterData::set_data` で書き込む
   （`innerHTML`/`insertAdjacentHTML` は一切使わない）。
3. 束縛点走査は `first_child`/`next_sibling`（O(1) 操作）のみで到達し
   `childNodes` の添字アクセスは使わない（`resolve_child_path` ヘルパー）。
   到達ノードは `dyn_ref::<web_sys::CharacterData>()` で検証し、`None` な
   ら全体を `None` にする（fail-safe、ここまで構築した全ノードは
   detached のため実 DOM への副作用はゼロ）。
4. 名前空間: `clone_node` は DOM 標準仕様上ノードの名前空間を保持する
   ため、プロトタイプを `self.namespace` で構築しておけば SVG keyed list
   （SignaturePad の `strokes` 等）への挿入でも clone 先はすべて SVG 名前
   空間のまま複製される（追加の名前空間指定は不要、browser テストで
   固定）。

## 4. フォールバック条件（fail-safe）

以下のいずれかが起きた場合、区間全体を `create_item` による個別生成へ
フォールバックする（部分的に壊れたノードを DOM へ入れない）:

- 区間長が 1（clone の利得がないため `derive_item_template` 自体を呼ばない）
- `derive_item_template` が `None`（非同型混在・`RawHtml` 混入・非
  `Node::Element` ルート）
- `create_items_from_template` が `None`（プロトタイプ構築失敗・
  `clone_node` 失敗・`set_attribute` 失敗・束縛点パスが `Text` へ解決
  できない）

`insert_before_batch` 自体が失敗した場合は、テンプレート経路・個別生成
経路のいずれで構築した場合も、区間内で構築済みだった全アイテムを
未達成スロットとして扱い `resync_required` を立てる（イシュー #1340 の
既存契約と同型）。

## 5. 不変条件（既存契約の継承）

- テキストは Text ノードの値設定（`set_data`）のみで挿入する。
  `innerHTML`/`insertAdjacentHTML` は一切使わない（REQ-1 既定エスケープの
  迂回経路を新設しない）。
- `RawHtml` は core の同型判定で不成立となり本経路へ到達しない
  （到達した場合も `text_values` が `None` で全体フォールバック）。
- 危険 URL スキーム・`srcset`・イベントハンドラ属性はプロトタイプ構築時に
  既存述語で除去され、`clone_node` にも複製されない。
- 途中失敗時、実 DOM へは一切副作用が残らない（全ノードが detached の
  まま破棄される）。

## 6. 実測

### 6.1 native（Rust 側呼び出し回数、`CountingDom` モック）

`apply_ops_with_items_create_1000_rows_from_empty_uses_single_template_batch`
（`crates/wasm-client/src/keyed_apply.rs`）で固定:

| 項目 | 値 |
|------|-----|
| `create_items_from_template` 呼び出し回数 | 1（1,000 行を 1 回で一括構築） |
| `create_item`（個別生成）呼び出し回数 | 0 |
| `insert_before_batch` 呼び出し回数 | 1（#1320 の集約契約） |

区間が保持キーで分断される場合は区間ごとに独立して 1 回ずつ一括構築
されること（`apply_ops_with_items_batches_each_disjoint_insert_run_separately`）、
非同型混在・`RawHtml` 混入では `create_items_from_template` 自体を呼ばず
個別生成へフォールバックすること（`apply_ops_with_items_falls_back_to_create_item_when_run_is_not_isomorphic`/
`apply_ops_with_items_run_with_raw_html_item_falls_back_and_skips_only_that_item`）
も native テストで固定済み。

### 6.2 行あたりの境界呼び出し回数（設計時の見積り）

bench 行（`tr > td(text), td(text)`）1 行あたり、従来の個別生成経路
（`build_dom_node_with_namespace` 経由）:

`create_element`×3 + `create_text_node`×2 + `append_child`×4 +
`set_attribute(data-key)`×1 + `insert_before`×1 = **11 回/行**

プロトタイプ経路（2 件目以降）の見積り: `cloneNode`×1 +
`setAttribute(data-key)`×1 + 束縛点走査（`firstChild`×2 + `nextSibling`×1 +
`firstChild`×1 = 4 相当）+ `instanceof CharacterData` 判定×2（`dyn_ref` が
発行する JS 境界呼び出し）+ `data=` 代入×2 + fragment `appendChild`×1
≈ **9〜11 回**。

**目安（10 → 4 回程度）に対する評価**: 呼び出し**回数**だけでは目安に
届かない見積りであり、主な利得は 1 呼び出しあたりコスト（`cloneNode(true)`
が複数ノードをネイティブに複製し、タグ名文字列のマーシャリングと
個別ノード生成を省く）に移る、という計画時点の想定どおりとなった。

**本セッションでの計測範囲の限定（既知の未実施事項）**: 計画 §5.2 が
挙げていた「JS 側 API（`createElement`/`cloneNode`/`insertBefore` 等）を
一時的にラップして回数を実測する」browser テストは、実装のコア（native
呼び出し回数固定・browser 上での機能正当性）を優先した結果、本セッション
では追加しなかった（native 側の呼び出し回数固定テストで代替）。実際の
境界呼び出し回数の実測、および `bench/csr` を用いた create/update/clear
op_ms の before/after 比較（`bench/PROTOCOL.md` §3）も、`bench/csr/`
の npm 依存（`node_modules/`）が本セッションの実行環境に未導入だった
ため実施していない。いずれもフォローアップとして Issue 化を提案する
（追跡候補: 「境界呼び出し実測 browser テストと bench/csr 実測の追補」）。

## 7. 受け入れ基準対応表

| # | 基準 | 状態 |
|---|------|------|
| 1 | 同型判定成立時、先頭アイテムを 1 回構築しプロトタイプ化、2 件目以降は `clone_node_with_deep(true)` + 束縛点パス走査で構築、`childNodes` 添字アクセス不使用 | 達成（§3.3、`resolve_child_path`） |
| 2 | 既存の DocumentFragment 集約と併用し、区間 1 件につき実 DOM への挿入は 1 回 | 達成（`insert_before_batch` を区間単位で 1 回のみ呼ぶ） |
| 3 | 同型不成立・束縛点不整合・clone/属性書き込み失敗時は個別生成へフォールバックし、部分的に壊れたノードを DOM へ入れない | 達成（§4、全経路 detached のまま破棄） |
| 4 | 行あたりの境界呼び出し回数の削減を実測し記録 | 一部達成: native 呼び出し回数（Rust 側）は実測・固定済み（§6.1）。ブラウザ JS 境界呼び出しの直接実測は未実施（§6.2 末尾、フォローアップ候補） |
| 5 | 既存 keyed list テスト全件 + 新規テスト（同型/非同型/属性付き/ネスト）が通る | 達成（native 8 件・browser 8 件を新規追加、既存回帰は無改修で通過） |
| 6 | XSS 回帰テストを削除・弱体化しない | 達成（clone 経路版の XSS 回帰テストを新規追加） |
| 7 | `bench/csr` の create op_ms が改善する（目標: preact 圏） | 未実施: 実行環境に `bench/csr` の npm 依存が未導入のため本セッションでは計測していない（フォローアップ候補） |

## 8. テスト

- native: `crates/wasm-client/src/keyed_apply.rs` の `mod tests` に 8 件
  追加（1,000 行一括構築の呼び出し回数固定・区間長 1 判定・非同型
  フォールバック・テンプレート構築失敗フォールバック・区間分断・
  `RawHtml` 混入・`insert_before_batch` 失敗時の `resync_required`・
  ネストした `data-bind-list` の `invalidated_nested_fields` 収集）。
- browser: `crates/wasm-client/src/keyed_dom.rs` の `mod tests` に 8 件
  追加（同型一括挿入・静的属性保持・深さ 3 のネスト束縛点・非同型
  フォールバック・XSS 回帰・危険属性除去・SVG 名前空間・with_previous
  経路での既存ノード同一性保持）。`CHROMEDRIVER=/usr/bin/chromedriver
  wasm-pack test --headless --chrome crates/wasm-client` で全件 PASS を
  確認済み（新規 8 件を含む計 42 件）。

## 9. 対象ファイル

| パス | 変更内容 |
|------|---------|
| `crates/wasm-client/src/keyed_apply.rs` | `KeyedListDom::create_items_from_template` 追加、`apply_ops_with_items` の `Insert` arm を区間処理へ置換、`CountingDom` へカウント実装・native テスト追加 |
| `crates/wasm-client/src/keyed_dom.rs` | `WebSysKeyedDom::create_items_from_template`・`write_template_text_paths`・`resolve_child_path` 追加、browser テスト追加 |
| `crates/wasm-client/Cargo.toml` | 0.6.0 → 0.6.1（patch） |
| `crates/wasm-full/Cargo.toml` | 依存 `fandhe-frontend-wasm-client` 追随、0.7.10 → 0.7.11（patch） |
| `templates/app/wasm/Cargo.toml` / `crates/cli/templates/app/wasm/Cargo.toml.embed` | `fandhe-frontend-wasm-client = "0.6.1"` へ同期 |
| `docs/design/keyed-insert-template-clone-design.md`（本ファイル） | 新規 |
