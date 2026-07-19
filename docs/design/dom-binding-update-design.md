# 実 DOM 束縛点更新・keyed list の設計確定（イシュー #340）

## 1. 目的とトレーサビリティ

本書は #335（v2: AI 開発効率のための実 DOM 更新・契約型化・決定的生成の
トラッキング）配下の #336（feat(phase-1): 実 DOM 直接更新基盤 — 束縛点最小
更新 + keyed list、仮想 DOM 非採用）の**設計確定書**であり、REQ-1（既定
エスケープ）・REQ-11（WASM 完全方式）を実装で担保する後続タスク

- #341 `feat(interactive)`: `update()` の変更フィールド追跡（dirty tracking）API
- #342 `feat(core)`: 束縛点マーキングの Node 木 API
- #343 `feat(wasm-client)`: 束縛点ベースの最小更新への一般化
- #344 `feat(core)`: keyed list プリミティブ
- #345 `feat(wasm-full)`: `set_inner_html` 全置換を束縛点更新 + keyed list へ置換

の実装範囲・API 形状・移行手順を判断可能にするために書く。

**本文書のステータス**: #340 設計確定書。#341〜#345 の実装と本書の記述に
乖離が生じた場合は本書を正とし、PR レビューで指摘する。乖離が実装上
不可避と判明した場合は本書を改訂してから実装を進める。

本書は `docs/design/wasm-full-architecture.md`・`docs/design/hydration-nested-state.md`・
`docs/api/interactive-api.md`・`docs/api/component-api.md` と同じ書式
（ステータス・トレーサビリティ・凍結表・設計判断表・スコープ外表・
セキュリティ不変条件・受け入れ基準対応表）に揃える。

`docs/spec/04-requirements.md` の REQ-1（既定エスケープ）・REQ-11（WASM 完全
方式）に対応する。`docs/spec/` はサブモジュールのため編集禁止であり、
仕様本文側の追随（v2 相当の要件反映）が必要と判断した項目は第 8 節の
スコープ外表に frontend-framework-spec リポジトリへの Issue 提案として
記録するに留める。

## 2. 現状と課題

- `wasm-full/src/dom.rs` の `paint()` は `render_component_html()`（`rws_core::render`
  の出力）を `web_sys::Element::set_inner_html` へ**全置換**で渡す
  （`wasm-full/src/dom.rs:44`）。イベントごとに DOM 全体を文字列化 →
  ブラウザの HTML パーサで再パースする経路であり、フォーカス・入力途中の
  値・スクロール位置・IME 変換状態を破壊する。
- `wasm-full/src/events.rs` は、入力ハンドラで `should_repaint: false`
  を返すことで再描画自体を抑止する暫定運用でこれを回避している
  （`wasm-full/src/events.rs:93` 付近のコメント）。これは症状を避けている
  だけであり、入力以外の更新（カウンタ増減等）では依然として全置換が
  発生する。
- 一方 `wasm-client/src/lib.rs` は既にこの問題を避けた設計になっている。
  `hydrate()` は再構築系 API を一切呼ばず（不変条件 2）、ハンドラ内更新は
  `set_text_content` / `class_list` に限定される（不変条件 3、
  `wasm-client/src/lib.rs:27-31`）。ただし対応するのは `data-hydrate="like"`
  という単一の具象実装のみであり、任意の `Component` へ一般化されていない。
- 本書は `wasm-client` が既に守っている最小更新路線を、`rws-core` の
  Node 木 API・`rws-interactive` の dirty tracking・`wasm-full` の適用層の
  三層に一般化する設計を確定する。構造変化（リストの増減・並べ替え）
  だけは仮想 DOM 的な汎用 diff を採用せず、keyed list という**単一の
  専用経路**に限定する。

## 3. 束縛点対応表の表現（#342 の入力）

### 3.1 属性によるマーキング規約（SSR 出力に現れる形式として凍結）

| 種別 | 属性形式 | 例 | 意味 |
|------|---------|-----|------|
| テキスト束縛 | `data-bind-text="<field>"` | `data-bind-text="counter"` | 要素の唯一のテキスト子ノードが state フィールド `counter` に束縛される |
| 属性束縛 | `data-bind-attr="<attr>:<field>"` | `data-bind-attr="aria-pressed:liked"` | 属性 `aria-pressed` の値が `liked` フィールドに束縛される。1 要素が複数属性を束縛する場合は空白区切りで複数トークンを列挙する（`data-bind-attr="aria-pressed:liked disabled:busy"`） |
| class 束縛 | `data-bind-class="<class>:<field>"` | `data-bind-class="liked:liked"` | `bool` フィールドの真偽で class のオン/オフを切り替える。複数 class も属性束縛と同じ空白区切り規約に従う |
| リスト束縛 | `data-bind-list="<field>"` | `data-bind-list="items"` | 親要素の子ノード列が `field`（keyed list）の現在値の並び順・要素集合に束縛される。子ノード自体は `data-key` を持つ（第 5.1 節）。キー一意性は**当該親の直下子のみ**が対象（子孫にネストした別の `data-bind-list` のキー空間とは独立、第 5.1 節） |

- 属性名は `data-bind-text` / `data-bind-attr` / `data-bind-class` の 3 種に
  固定し、いずれも `core/src/lib.rs:175` の `is_valid_attr_name`（英数字・
  ハイフンで構成される既存ホワイトリスト検証)をそのまま通過する
  形式である（新たな属性名パターンを核の検証ロジックへ追加する必要は
  ない）。
- 属性**値**（`<field>` 名・`<attr>:<field>` の組）は既定エスケープ
  （`rws_core::render` の属性値エスケープ）をそのまま経由して出力される。
  フィールド名は `&'static str` としてコンパイル時に固定される値であり
  （第 3.3 節）、実行時の外部入力から組み立てられることはない。
- コロン `:` ・空白は区切り文字として予約する。フィールド名にコロン・
  空白を含めることは許容しない（`#342` 実装時に構築 API 側でアサート
  する。実行時に外部から改ざんされる経路がないため `panic!`/`debug_assert!`
  ではなく型設計 — `&'static str` 定数のみを受理 — で防ぐ）。

### 3.2 `data-hydrate-*`（#163）との役割の直交性

- `data-hydrate-<field>`（`interactive::HYDRATE_ATTR_PREFIX`、
  `interactive/src/lib.rs:130`）は**状態値そのもの**をハイドレーション時に
  復元するための注入である（`codec::Value` によるエンコード、
  `docs/design/hydration-nested-state.md`）。
- `data-bind-*` は**更新先ノード位置**のマーキングであり、値を運ばない
  （値は state 側にのみ存在する）。両者は同一要素・別要素いずれにも
  独立して付与でき、どちらか一方の有無がもう一方の動作に影響しない。
- クライアント側は起動時（`hydrate()` 相当の初期化）に `data-bind-*` を
  持つ全ノードを 1 回だけ `query_selector_all` で走査し、
  `Vec<(&'static str, BindingTarget)>`（`field` → 束縛点リスト）の対応表を
  構築してメモリ上に保持する。この走査パターンは `rws_core::find_attr_values`
  （`core/src/lib.rs:230`）・`wasm-client` が `data-nav` に対して行う走査と
  同一の考え方（属性値を手掛かりにした DOM 走査、HTML 再パースを伴わない）
  を踏襲する。

#### 3.2a 実装確定（#343、`wasm-client/src/binding.rs`・`binding_dom.rs`）

- 対応表のキーである `field` は、DOM 走査で `data-bind-*` 属性値から読んだ
  **実行時 `String`**（`BindingSpec::field`）にならざるを得ない（属性値は
  ブラウザの DOM API から得るためコンパイル時定数にできない）。一方
  `DirtyTracked::dirty_fields()`（#341）は `&'static str` の有限集合を返す。
  両者の突き合わせは**文字列比較**（`==`）で行う。`&'static str` 側は
  コンパイル時に確定した有限集合であり外部入力からの偽装余地がないため、
  この比較で「対応表側の走査キーが動的文字列である」ことによる安全性の
  低下は生じない（第 3.3 節の設計原理を維持したまま、突き合わせ方式のみを
  確定させたもの）。
- `data-bind-attr` トークンの属性名（`<attr>` 部分）は消費側（#343）の
  fail-closed 検証を経由する: 英数字・`-`・`_` のみを許可し、大小文字を
  無視して `on` で始まる名前を拒否する
  （`wasm-client/src/binding.rs::is_valid_binding_name`）。これは
  `setAttribute("onclick", value)` のような呼び出しが状態値を実行可能な
  イベントハンドラへ昇格させる経路を遮断するための、`core/src/bind.rs` の
  モジュール docs が明記する「消費側の契約」の履行である。URL スキーム等
  「値の内容」の検証は本節・第 9 節の確定通り導入しない（既存 SSR 経路と
  同等の残存リスク）。

### 3.3 core API 形状（#342 実装確定）

当初案（`bind_text(field, value) -> Node` / `bind_attr(...) -> (String, String)` /
`bind_class(...) -> Option<String>`）は実装検討の結果、次の 2 点の課題が
判明したため、下記の確定形へ改訂する。

- (a) `bind_text` が「マーカー属性 + テキスト子」を単一 `Node` で表すには
  タグ名・呼び出し側属性を合わせて受け取る必要があり、`field`/`value` のみの
  シグネチャでは要素を構築できない。
- (b) 同一要素へ複数の `bind_attr`/`bind_class` を適用すると `data-bind-attr`/
  `data-bind-class` 属性が要素内で重複し、ブラウザが先頭のみを採用して
  残りの束縛が黙って欠落する（第 9 節・fail-closed 方針に反する）。トークン
  合成を明示的な複数束縛版関数に分離することでこれを構造的に防ぐ。

```rust
// rws-core: 束縛点マーキングのヘルパー群（core/src/bind.rs、#342 で追加）

/// 束縛点マーカー属性名（#343 が走査する契約値。§3.1 で凍結）
pub const BIND_TEXT_ATTR: &str = "data-bind-text";
pub const BIND_ATTR_ATTR: &str = "data-bind-attr";
pub const BIND_CLASS_ATTR: &str = "data-bind-class";

/// "attr:field" トークンを合成する（data-bind-attr の値用）
pub fn bind_attr_token(attr: &'static str, field: &'static str) -> String;
/// 複数束縛の空白区切り合成（同一要素の data-bind-attr 重複を構造的に防ぐ）
pub fn bind_attr_tokens(bindings: &[(&'static str, &'static str)]) -> String;
/// "class:field" トークン（data-bind-class の値用）。複数版も同様に提供
pub fn bind_class_token(class: &'static str, field: &'static str) -> String;
pub fn bind_class_tokens(bindings: &[(&'static str, &'static str)]) -> String;

/// テキスト束縛付き要素を構築する。マーカー属性は呼び出し側 attrs の後ろへ
/// 決定的な順序で付加し、子は Node::Text(value) の 1 つのみとする
/// （§3.1「唯一のテキスト子」不変条件を構築で保証する）。
pub fn bind_text(
    tag: &'static str,
    attrs: Vec<(&str, &str)>,
    field: &'static str,
    value: impl Into<String>,
) -> Node;
```

- `field`/属性名/class 名を `&'static str` に固定するのは、`Node::Element.tag`
  （`core/src/lib.rs:80` 以降）が `&'static str` 固定であることと同じ設計
  原理である。動的文字列（実行時に組み立てた `String`）をフィールド名として
  受理しないことで、束縛点対応表の走査キーが常にコンパイル時に確定した
  有限集合であることを型で保証し、任意文字列注入によるフィールド偽装の
  余地を構造的に排除する。
- `bind_text` は `tag`/呼び出し側 `attrs` を受け取り、`Node::Element` の
  子として `data-bind-text` 属性と `Node::Text(value)` を同時に生成する
  要素構築ヘルパーとする（当初案の `field`/`value` のみのシグネチャから
  改訂）。
- 属性・class 束縛は要素構築ヘルパーを持たず、トークン合成関数
  （`bind_attr_token`/`bind_attr_tokens`/`bind_class_token`/`bind_class_tokens`）
  + 既存 `el`/タグヘルパーの組み合わせで表現する（例:
  `el("button", vec![("aria-pressed", "false"), (BIND_ATTR_ATTR, &bind_attr_token("aria-pressed", "liked"))], ...)`）。
  同一要素へのテキスト + 属性 + class の複合束縛も既存 API の合成だけで書け、
  マーカー属性の重複を作らない。
- `is_valid_attr_name` の検証は既存の `render()` 経路（属性出力時の
  ホワイトリスト検証、`core/src/lib.rs:320` 付近）へそのまま委ねる。新しい
  検証ロジックを追加しない（既存の防御を再利用する）。
- SSR 出力の決定性: 束縛点マーキングを使わない既存の `Node` 構築
  （`el`/`div`/`text` 等）の出力には**一切影響しない**（`bind_*` は既存
  ヘルパーに属性・テキストを付加するオプトイン API であり、未使用時の
  `render()` 出力はバイト単位で不変）。これを凍結条件とする
  （`core/tests/xss_escape.rs` の `bind_points::existing_node_construction_output_is_unaffected`
  で回帰固定）。

## 4. 更新の種別（#341・#343 の入力）

### 4.1 4 種別への固定

DOM 変異は以下の 4 種別に限定し、これ以外の DOM 変異経路を持たないことを
不変条件とする（第 7 節）。

| 種別 | 適用 Web API | 対象 |
|------|-------------|------|
| テキスト更新 | `Node.textContent`（`set_text_content`） | `data-bind-text` を持つ要素の子テキスト |
| 属性更新 | `Element.setAttribute`（`set_attribute`） | `data-bind-attr` で指定された属性 |
| class 更新 | `Element.classList`（`DomTokenList`、`class_list`） | `data-bind-class` で指定された class |
| 構造変化 | keyed list 専用手続き（第 5 節） | `data-key` を持つ子リストの挿入・削除・並べ替え |

### 4.2 dirty tracking API 形状（#341 の入力・設計判断確定）

`docs/api/interactive-api.md` 第 3 節で `Component::update` は
`fn update(&mut self, action: Self::Action)`（戻り値なし）として**既に
凍結済み**である。このシグネチャを変更すること（戻り値方式で変更
フィールド集合を返す案）は凍結 API への破壊的変更となり、`rws-wasm-full`
（TASK-11.2、既に `Component` へ依存する実装）を巻き込む影響が大きい
ため**採用しない**。

代わりに、`update()` と**対になる別関数**として dirty tracking を提供する
方式を採用する。

```rust
// rws-interactive: dirty tracking API 形状案（#341 で追加）
pub trait DirtyTracked: Component {
    /// `update()` によって直前の呼び出しで変更されたフィールド名の集合。
    /// フィールド名は `&'static str`（第 3.3 節と同じ設計原理）。
    fn dirty_fields(&self) -> &[&'static str];
}
```

- `Component` トレイト自体（凍結済み、`docs/api/interactive-api.md` 第 3 節）
  は変更しない。dirty tracking が不要な既存実装（`AppState` 等）は
  `DirtyTracked` を実装しなくてよい。
- 呼び出し側（`wasm-full` の適用層）は `update()` の直後に
  `dirty_fields()` を呼び、返された集合と第 3 節の束縛点対応表を突き合わせて
  該当ノードのみ更新する。無関係フィールドの束縛点には触れない
  （#343 受け入れ条件「無関係ノードの DOM 変異ゼロ」に対応）。
- `dirty_fields()` の実装は、各 `update()` 実装がアクション処理の中で
  変更したフィールド名を `&'static str` の集合（`Vec<&'static str>` 等、
  重複排除は呼び出し側の責務としない — 束縛点対応表側の走査は同一
  フィールドへの複数回の更新適用が冪等であることを前提とする）へ
  積み上げる形とする。マクロ生成は提供しない（#163 の判断 6 と同じ理由:
  proc-macro 依存追加は REQ-3 に反する。手書き実装とする）。

### 4.3 更新駆動の流れ（固定）

```
dispatch（文字列 → Action、既存 #341 以前の経路） →
  Component::update（凍結 API、状態を変更） →
    DirtyTracked::dirty_fields（変更フィールド集合） →
      束縛点対応表から該当エントリのみ抽出 →
        種別ごとに set_text_content / set_attribute / class_list を適用
```

- リストの増減・並べ替えが必要な変更フィールドは、通常の 3 種別
  （テキスト/属性/class）では扱わず、必ず第 5 節の keyed list 経路を
  通る（`dirty_fields()` にリストフィールド名が含まれる場合、束縛点
  対応表側でそのフィールドが keyed list 束縛点として登録されている
  ことを呼び出し側が判別する）。

## 5. keyed list プリミティブ（#344 の入力）

### 5.1 SSR 出力形式（#344 実装確定）

当初案 `keyed_list(field, items) -> Node` は、(a) 親要素のタグ名・呼び出し
側属性を受け取れない、(b) fail-closed（キー衝突・欠落で `Err`）を戻り値
なしで表現できない、の 2 点で実装不能であったため、実装（`core/src/keyed.rs`）
着手前に本節を以下の確定形へ改訂する（#342 が第 3.3 節で行ったのと同じ
「設計書を先に改訂してから実装する」手順）。

```rust
// rws-core::keyed（#344 で追加。core/src/keyed.rs）
pub const BIND_LIST_ATTR: &str = "data-bind-list";
pub const KEY_ATTR: &str = "data-key";

pub enum KeyedListError {
    EmptyKey { index: usize },
    DuplicateKey { first_index: usize, duplicate_index: usize },
    NonElementItem { index: usize },
    ReservedAttr { attr: &'static str },
}

pub fn keyed_list(
    tag: &'static str,
    attrs: Vec<(&str, &str)>,
    field: &'static str,
    items: Vec<(String, Node)>,
) -> Result<Node, KeyedListError>;
```

- 各子ノードは元の属性列の末尾へ `data-key="<key>"` を付加した要素として
  レンダリングされる。`key` は文字列（アプリ側が一意性を保証する）。
- 親要素は呼び出し側 `attrs` の末尾へ `data-bind-list="<field>"` を付加した
  要素として構築される。子の並び順が `field` の現在値の並び順と一致する
  ことを表す（`data-bind-*` 系列の第 4 の種別として、第 3.1 節の表に
  追記済み）。
- `field` は `&'static str` 固定（第 3.3 節と同じ設計原理: 実行時文字列
  によるフィールド偽装の型レベル遮断）。`key` は実行時データ（`String`）
  であり、一意性はアプリ側責務 + 本関数の検証で fail-closed。
- キー一意性検査は**直下の子のみ**が対象（同一親の直下子スコープに限定。
  子孫にネストした別の `keyed_list` 呼び出しのキー空間とは独立）。
- 出力は通常の `Node::Element` 木であり、新しい `Node` バリアント・新しい
  レンダリング経路・新しいエスケープ処理を追加しない。

### 5.2 fail-closed の定義（#344 受け入れ条件、実装確定）

| 異常系 | 挙動 |
|--------|------|
| キー欠落（`keyed_list` の要素に空文字列キーを渡す） | `keyed_list()` **構築時点**で `KeyedListError::EmptyKey` を返す。不正な `Node` はそもそも構築されない |
| キー衝突（同一親の直下子内で `key` が重複） | `keyed_list()` 構築時点で `KeyedListError::DuplicateKey` を返す。同上 |
| 子が `Node::Element` でない（`Text`/`RawHtml`） | `data-key` を付与できないため `keyed_list()` 構築時点で `KeyedListError::NonElementItem` を返す |
| 予約属性の手渡し（`data-key`/`data-bind-list` を呼び出し側 `attrs` に含める） | `keyed_list()` 構築時点で `KeyedListError::ReservedAttr` を返し、マーカー属性の重複・偽装を防ぐ |
| クライアント側でキー照合に失敗（改ざん等により `data-key` が想定外の値） | 該当ノードを更新対象から除外し、束縛点対応表の再構築（フルスキャン）にフォールバックする。個別ノードの不整合が全体のクラッシュへ波及しない設計とする（#343/#345 のスコープ） |

いずれの場合も `unwrap()`/`panic!` は使わない
（`.claude/rules/coding-rust.md` のエラーハンドリング規約）。

**改訂理由**: 当初案「`render()` 時点で `Result::Err`」は、モード非依存
レンダラ `render(&Node) -> String`（SSR/SSG/CSR 全層が共通使用する凍結 API、
第 10 節の出力一致保証の土台）が戻り値型を変更すると破壊的変更になる
こと、および infallible な `render()` が残る限り検証迂回経路が残ることから、
**構築時点の `Err`（不正な keyed list ノードをそもそも表現不能にする）へ
強化**する。「衝突した HTML を出力しない」という fail-closed の目的は
より早い段階（ノード構築時点）で満たされ、`render()` のシグネチャは
一切変更しない。

### 5.3 CSR 側の最小 DOM 操作

- キー照合により、新しい `field` の値（キー付きリスト）と現在の DOM 上の
  `data-key` 列を比較し、**挿入・削除・移動のみ**を行う。**汎用 diff
  アルゴリズム（morphdom 型の任意ノード比較）は実装しない**。構造変化は
  この keyed list 経路が唯一である。
- 新規挿入ノードの構築は `innerHTML`/`insertAdjacentHTML` を使わず、
  `Node` 木（`keyed_list` が生成した子 `Node`）から
  `document.create_element` → `set_attribute`（属性ごと）→
  `set_text_content`（テキスト子ごと）→ `append_child` という
  プログラム的構築で行う。これは第 4.1 節のテキスト更新不変条件
  （HTML パーサを経由しない）を新規ノード挿入にも延長するものである。
- 移動（並べ替え）は既存 DOM ノードの参照を保持したまま
  `Node.insertBefore`（`web_sys::Node::insert_before` 相当）で位置のみを
  変更し、ノードの再生成を行わない（フォーカス・入力状態の保持、
  第 2 節の課題に対応）。
- SSR / SSG 出力一致保証: `keyed_list` は既存の `Node` 木 API の一部として
  実装するため、`render()` の出力は SSR・SSG・CSR 初回マウントで同一の
  関数（`rws_core::render`）を経由し続ける。決定性は他の `Node` 種別と
  同じ既存保証（`docs/design/wasm-full-architecture.md` 等の凍結表）を
  継承する。

## 6. `set_inner_html` 全置換の移行方針（#345 の入力）

本節は #345 が実施する将来の移行手順を**記述**するものであり、本書
（#340）自体はコード変更を含まない。現時点の防御（`should_repaint` 等）は
本書によって一切弱体化されない。

### 6.1 撤去手順

1. `wasm-full/src/dom.rs` の `paint()`（`set_inner_html` 全置換、
   `wasm-full/src/dom.rs:44`）を、第 4.3 節の更新駆動フローに置き換える。
2. `wasm-full/src/events.rs` の `should_repaint: false` による入力時再描画
   抑止（`wasm-full/src/events.rs:93` 付近）は、束縛点更新が実 DOM を
   再構築しないため入力状態を破壊しなくなり、暫定運用として不要になる。
   ただし「入力ハンドラは呼ばれたが対応する束縛点更新が存在しない」
   ケースの扱いは #345 実装時に個別確認する（本書は撤去を許可する設計
   条件を示すのみで、撤去自体は #345 のスコープ）。

### 6.2 `mount_csr`（初回マウント）の `set_inner_html` の扱い（設計判断確定）

初回マウント時は DOM 上に既存ノードが存在しない（保持すべきフォーカス・
入力状態がない）ため、`set_inner_html` を使うこと自体は第 2 節の課題
（既存状態の破壊）を引き起こさない。したがって、初回マウント用の
`set_inner_html` 呼び出しは**撤去せず存置する**。ただし以下の条件で
明示的な限定 API として文書化する。

- `rws_core::render(component.view())` の出力（既定エスケープ済み HTML
  文字列）**のみ**を渡す用途に限定した内部関数（例:
  `mount_initial(root: &Element, html: &str)`）として切り出し、汎用の
  DOM 更新経路（第 4〜5 節）とは呼び出し元を分離する。
- 上記関数以外から `set_inner_html` を呼ばないことを、#345 実装時の
  受け入れ条件（コードレビュー・grep 確認）に加える。

### 6.3 移行順序と段階条件

依存関係に従い `#341 → #342 → #343 → #344 → #345` の順に直列で実装する
（`interactive` の dirty tracking が確定しないと `core` の束縛点 API の
消費側要件が固まらず、`core` の束縛点 API が確定しないと `wasm-client`
一般化・`wasm-full` 移行が着手できないため）。各段階で以下を非劣化条件
とする。

- XSS 回帰テスト（`interactive/tests/xss_escape.rs` 等、SSR/SSG/CSR/WASM
  各経路）が削除・弱体化されないこと。
- ハイドレーション roundtrip テスト（#163 系、
  `wasm-full/tests/nested_hydration_state.rs`）が非劣化であること。
- WASM バンドルサイズ上限（REQ-11）に対する影響を各段階で計測すること。

### 6.4 #345 実装確定（設計の乖離改訂）

本節は #345 実装時に判明し、上記 6.1〜6.3 の記述から乖離が生じた事項を
先行して確定する（`.claude/rules/coding-rust.md` の「乖離時は文書を先に
改訂」規約に従う）。

#### 6.4.1 `should_repaint` は撤去する（6.1 の確定）

6.1 は撤去可否を「#345 実装時に個別確認」としていたが、実装の結果、
束縛点更新（`set_text_content`/`set_attribute`/`class_list`）は**冪等**
かつ**変更フィールド数に比例するコスト**であるため、input イベントでも
毎回適用してよいと確定した。`should_repaint`（`wasm-full/src/events.rs`
の `ActionRef`）はフィールドごと撤去し、`click`/`input` いずれのイベント
後も同一の更新経路（6.4.4 参照）を通す。

#### 6.4.2 `value` 属性は「属性 + live value プロパティ」の両方を更新する

`set_attribute("value", ...)` は HTML 属性（要素の初期値）のみを更新し、
ブラウザの live value プロパティ（`HTMLInputElement.value`）には反映
されない（DOM 仕様上の既知の非対称性）。これにより、例えば
`add_item` 後に入力欄をクリアする操作が `set_attribute` だけでは効かない。

`wasm-client::binding_dom::apply_one` は、束縛先が `value` 属性かつ対象
要素が `HtmlInputElement` へダウンキャストできる場合、`set_attribute` に
加えて `HtmlInputElement::set_value` も呼ぶ。**現在の live value と新しい
値が等しい場合は `set_value` を呼ばない**（等値ガード）。これは、ユーザー
が入力中の値と状態側の値が一致している限り（＝自分自身の入力による
更新である限り）不要な `set_value` 呼び出しを避け、キャレット位置が
飛ぶ事故を防ぐための不変条件である。

#### 6.4.3 keyed list の安定キー戦略と `remove_item` の id 化

`rws_interactive::AppState` の動的リストは、当初 index をキーとする案が
想定されたが、中間削除でキーがずれる（削除後、後続項目の index が
1 つずつ繰り上がり、別項目のキーと衝突する）ため、`AppState` に
`item_ids: Vec<u64>`（`items` と同じ長さ・順序で対応する安定キー）・
`next_item_id: u64`（単調増加カウンタ）を追加した。`item_ids`/
`next_item_id` は `dirty` と同様の**描画同期メタデータ**であり、
`PartialEq` 比較対象外とする。

ハイドレーション（`from_hydration_attrs`）については、`item_ids` を
`data-hydrate-item-ids` 属性（`items` と同じ `codec::encode_list` 方式で
数値文字列の配列をエンコード）として運び、SSR が keyed list の `data-key`
として実際に出力した id 列と一致させる（イシュー #345 レビュー指摘の
是正。当初案は `0..items.len()` への決定的再割当てのみだったが、これは
中間削除で `item_ids` に欠番が生じている場合に SSR 出力済み `data-key`
と乖離し、ハイドレーション直後の最初の構造変化で変更されていない既存
ノードまで誤って破棄・再生成してしまう欠陥があった。`wasm-full` 側の
`BindingTable` 再スキャン（第 6.4.1 節・`Self::wire` 相当）はテキスト/
属性/class の束縛点対応表のみを再構築するものであり、keyed list の
`data-key` ↔ `item_ids` 対応はこの再スキャンでは救済されない）。

復元した `data-hydrate-item-ids` 属性値は「`items` と同じ長さ」「全要素が
`u64` としてパース可能」「重複なし（`keyed_list` のキー一意性契約）」の
3 条件をすべて満たす場合のみ採用する。クライアント制御下の属性値は
改ざんされうる（本クレートの不変条件 3）ため、条件を満たさない場合は
`0..items.len()` への決定的再割当てへ安全側フォールバックする（panic
しない）。`next_item_id` は採用した `item_ids` の最大値 + 1（空の場合は
0）とし、将来の新規追加 id が既存 id と衝突しないようにする。

これに伴い、`Action::RemoveItem` の payload を index（`usize`）から安定
id（`u64`）へ変更する。keyed 更新では既存 DOM ノードの `data-payload`
（削除ボタンの payload）が自動的には再描画されないため、index のままだと
中間削除後に別項目の削除ボタンが stale な index を指し続け、意図しない
項目が削除される事故が構造的に起こり得る。id 化によりこれを防止する。

#### 6.4.4 keyed list の DOM 適用層は `wasm-client` に置く（2 層構成）

`rws-wasm-client`（#343 で新設済み）に以下 2 モジュールを追加する。

- `wasm-client/src/keyed_diff.rs`（純粋層・native テスト可）: 「現在の
  DOM 上のキー列」と「新しい `Node` 木のキー列」の 2 つの文字列列から、
  最小の操作列（`KeyedOp::{Remove, Insert, Move}`）を計画する DOM 非依存
  関数 `diff_keys`。
- `wasm-client/src/keyed_dom.rs`（`wasm32` 配線層）: 操作列を実 DOM へ
  適用する `apply_keyed_list`。挿入ノードは `rws_core::Node` 木から
  `create_element`/`set_text_content`/`append_child` によりプログラム的に
  構築し、`innerHTML`/`insert_adjacent_html` を一切使わない。移動は
  既存ノード参照を保持したまま `insert_before` のみで行う（再生成しない
  ためフォーカス・入力途中の値が保持される）。`Node::RawHtml` 子は
  fail-closed で skip し、`console` へ英語固定文言の警告を出す。

`wasm-full` は `rws-wasm-client` を workspace path 依存として追加し、
これらを rlib 経由で消費する（`structure.toml` の
`[directories.wasm-client]`（新設）・`[directories.wasm-full].depends_on`
へ追加）。外部クレートの追加はゼロ。

**シンボル衝突への対応**: `rws-wasm-client` は独自の `#[wasm_bindgen] pub
fn hydrate`/`mount_csr`（REQ-6 最小ハイドレーション方式のデモ、#48）を
既に公開しており、`rws-wasm-full` も独自に `#[wasm_bindgen] pub fn
hydrate`/`mount`（`entry.rs`）を公開する。両クレートを 1 つの wasm
バイナリへ静的リンクすると、`wasm-bindgen` の "describe" シンボル
（クレートで名前空間分離されない）が重複しリンクエラーになる。この衝突を
避けるため、`rws-wasm-client` に `wasm-bindgen-exports` feature（既定
on）を新設し、`wiring` モジュール（`hydrate`/`mount_csr` の
`#[wasm_bindgen]` エクスポート）をこの feature でゲートする。
`rws-wasm-full` は `default-features = false` で依存することでこの
feature を無効化し、シンボル衝突を構造的に回避する。本クレートを単体で
利用する既存の呼び出し元（feature 既定 on）には影響しない。

#### 6.4.5 `wasm-full::Runtime` の更新駆動

`Runtime<C>` の型境界を `C: Component + DirtyTracked + BindingSource +
'static` へ拡張する（`entry.rs` の `AppState` 具象化は `wasm-client` 側の
`impl BindingSource for AppState` により成立する。孤児則により実装先は
`wasm-client` 一択）。

`Runtime::wire` は dispatch 成功後、`dirty_fields()` を読み、
`BindingTable::apply_dirty`（テキスト・属性・class）と、dirty field ごとに
`wasm-client::{find_list_element, find_keyed_list_node, apply_keyed_list}`
（keyed list、field が実際に `data-bind-list` を持つ場合のみ）の両方を
試みる。束縛点対応表（`BindingTable`）はイベント配線時に 1 回 `scan` し、
keyed list の構造変化が発生した呼び出しに限り**対応表を再スキャン**する
（挿入された新規ノード内の束縛点を拾うため。第 5.2 節の「フォールバック
としてのフルスキャン再構築」と同じ機構を、通常の構造変化後更新にも転用
する形になる）。

`wasm-full/src/dom.rs::paint` は `mount_initial` へ改名し、6.2 の限定 API
方針どおり初回マウント・ハイドレーション CSR フォールバックからのみ
呼ばれる（イベント後更新からは呼ばれない）。

## 7. 仮想 DOM・汎用 diff 非採用の根拠

### 7.1 性能

毎イベントで全 `Node` 木を再構築し文字列化 → ブラウザが再パースする
現行 `paint()` 経路（計算量は木サイズに比例）に対し、束縛点更新は
変更フィールド数に比例する計算量で完結する。仮想 DOM の diff アルゴリズム
自体（新旧木の比較コード）を実装・保守する必要がなく、WASM バンドル
サイズ上限（REQ-11）・glue コード行数の増加を避けられる。

### 7.2 DOM 状態の保持

実 DOM ノードを再構築しないため、フォーカス位置・入力途中のテキスト値・
スクロール位置・IME 変換状態が構造的に保持される。現行の
`should_repaint: false` による回避策（第 6.1 節）は対症療法であり、本設計
はその根本原因（DOM 再構築そのもの）を解消する。

### 7.3 XSS 面の構造的縮小

更新経路が `set_text_content` / `set_attribute` / `class_list` / keyed list の
プログラム的ノード構築（第 5.3 節）に閉じるため、HTML 文字列を実行時に
組み立てて再パースする経路が消える。これにより「`raw_html()` 以外に
エスケープ迂回が存在しない」という REQ-1 の保証を、**実行時の規律
（レビューで守る）から API 形状（呼び出せる関数の集合そのものが HTML
文字列を受け付けない）へ引き上げる**。第 8 節の不変条件がこれを凍結する。

### 7.4 AI 開発前提の評価軸（#335 に対応）

- **明示性**: 束縛点が `data-bind-*` 属性として HTML に現れるため、
  `grep -r 'data-bind-'` や静的解析で「どの DOM ノードがどの状態フィールド
  に依存するか」を人間・AI いずれも即座に確認できる。仮想 DOM の diff
  結果はランタイムでしか観測できない。
- **決定性**: 更新は「dirty フィールド集合 → 束縛点対応表引き → 種別
  ごとの API 呼び出し」という一本道であり、diff アルゴリズムの内部
  ヒューリスティック（key 推定・要素種別推定等）に依存しない。
- **機械検証可能性**: `fw gate` やブラウザテストで「特定の束縛点のみが
  更新され、無関係ノードが変異していないこと」を DOM 属性ベースで
  アサーションできる（#343・#345 の受け入れ条件）。
- **コンテキスト消費**: 汎用 diff アルゴリズムを実装しないことで
  コード量が小さく保たれ、AI がこの経路を読み解く・レビューする際の
  コンテキスト消費が小さい。

### 7.5 却下案の記録

| 却下案 | 却下理由 |
|--------|---------|
| 仮想 DOM（React 型、新旧木の diff + パッチ適用） | diff アルゴリズム自体の実装・保守コストが発生し、REQ-3（依存上限）・REQ-11（バンドルサイズ）に対する圧が大きい。第 7.4 節の明示性（束縛点が属性として現れない）も失われる |
| 汎用 diff（morphdom 型、実 DOM 同士の比較） | 実 DOM ノード比較のヒューリスティックはブラウザの実装差・要素種別により挙動が変わりやすく、決定性（第 7.4 節）を損なう。構造変化は keyed list という単一の明示的経路に絞る方が AI にとって扱いやすい |
| signal ベースの細粒度リアクティビティ（Solid 型） | ランタイムの依存追跡機構（signal グラフ構築・購読）自体が新たな抽象層であり、`forbid(unsafe_code)`・外部依存ゼロの `core`/`interactive` の制約下では自作コストが大きい。#352（意図的非採用の横断記録）側で棲み分けて記録する（本書は Phase 1 の実装判断として signal を採用しない理由を述べるに留め、横断的な位置づけの記録は #352 が担う） |

## 8. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| #341〜#345 のコード実装（`core`/`interactive`/`wasm-client`/`wasm-full` への実装反映） | 各対応イシュー（本書はいずれも設計のみ） |
| Loader trait（Phase 2 相当の非同期データ取得層） | 別途 Phase として起票（本書スコープ外） |
| `structure.toml` / `fw impact` / `fw gate` の束縛点・Loader・`fw new` への追従 | #353 で完了。束縛点の `fw impact` 影響反映・`fw gate` への新チェック追加は非採用と判断（`docs/design/impact-analysis-design.md` §7、`docs/design/gate-design.md` §7 参照）。`fw new` 生成物の `root` 慣習は `structure.toml` スキーマ v1 で正式化した（`docs/design/structure-manifest.md` §2.2） |
| 意図的非採用の横断記録（仮想 DOM・ファイルベースルーティング・HMR・signal 一括） | #352（本書第 7.5 節は Phase 1 実装判断としての却下記録であり、#352 の横断記録とは別建てで併存する） |
| `wasm-thin` の JS グルー側更新経路への同方針（束縛点更新・keyed list）の適用可否 | #376 で非採用確定（`docs/policy/intentional-non-adoption.md` §3.10） |
| `docs/spec/` 本文（REQ-1/REQ-11 等）への v2 要件反映 | frontend-framework-spec リポジトリへの Issue 提案をユーザーへ別途提案する（本書はサブモジュール編集禁止のため直接反映しない） |

## 9. セキュリティ不変条件

1. **テキスト更新経路は HTML 解釈されない（受け入れ条件の中核）**:
   テキスト束縛（`data-bind-text`）の更新は `Node.textContent`
   （`set_text_content`）のみを経由する。DOM 仕様上 `textContent` への
   代入は常にプレーンテキストとして扱われ、HTML マークアップとして
   パースされない（新しい子ノードは単一のテキストノードとして生成
   される）。`innerHTML` / `insertAdjacentHTML` はテキスト更新経路
   （第 4 節）からは一切呼ばない。これにより、テキスト補間の更新に
   関する XSS は構造的に不可能となる（既定エスケープの実行時規律に
   加え、呼び出せる API そのものが HTML 注入を受け付けない）。
2. **属性更新は `setAttribute` のみを経由する**: `data-bind-attr` の
   更新対象属性は core の属性名ホワイトリスト（`is_valid_attr_name`、
   第 3.3 節）で許可された名前と整合する。実行時の属性値更新は
   `core::render()` を再度経由しない ── `dirty_fields()` が返す
   フィールド名に対応する束縛点へ、アプリケーション状態が保持する
   生の（未エスケープの）値をそのまま `setAttribute` に渡す
   （第 4.3 節の更新フロー）。`setAttribute` は値を HTML として
   パースしないため、マークアップ注入によるコンテキスト脱出は
   構造的に発生しない（不変条件 1 と同種の防御）が、これは値の
   *内容*を検証・無害化するものではない。したがって `href`/`src`
   等 URL 系属性への `javascript:` スキーム注入や、任意の属性名
   （`on*` を含む）に対する値の設定自体は、本設計の範囲では防止
   されない。この残存リスクは SSR 経路（`core::render()` の属性値
   エスケープも HTML メタ文字のエスケープに留まり、URL スキームの
   妥当性は検証しない）と同一であり、本設計によって新たに拡大される
   ものではないが、既定エスケープが「安全化済みの値を渡している」
   という意味では**ない**点を明記する。URL スキーム検証・`on*`
   属性名の追加ホワイトリスト制限が必要かどうかは既存の属性出力
   ポリシー全体（SSR 側を含む）の見直しとして扱うべき事項であり、
   本書では新たな検証機構を導入しない（スコープ外、第 8 節）。
3. **keyed list の新規ノード構築は `Node` 木からのプログラム的構築のみ**
   （第 5.3 節）とし、HTML 文字列を経由しない。挿入されるノードも
   テキスト子は `set_text_content` で設定され、不変条件 1 と同じ保証を
   継承する。
4. **`raw_html()` は本経路から呼ばない**。束縛点更新・keyed list 更新
   いずれの経路にもエスケープ迂回オプトインを組み込まない。新たな
   エスケープ迂回経路を作らない（REQ-1、`.claude/rules/coding-rust.md`）。
5. **束縛点属性・キー属性の値は既定エスケープ済み出力にのみ現れる**:
   `data-bind-*`/`data-key` は `rws_core::render()` の属性値エスケープを
   経由して出力される。フィールド名は `&'static str`（第 3.3 節）に
   固定され、実行時の外部入力から構築されないため、属性名自体への
   注入面も存在しない。
6. **fail-closed（A05 相当）**: keyed list のキー衝突・欠落・非 Element
   子・予約属性の混入は `keyed_list()` 構築時点で `Err` とし（`render()`
   のシグネチャは変更しない）、`unwrap()`/`panic!` を使わない（第 5.2 節）。
   クライアント側のキー照合失敗も安全側（更新を適用しない）に倒す。
7. **エラー・ログの機微情報非露出（A09 相当）**: dirty tracking・束縛点
   対応表構築・keyed list 照合のいずれのエラーメッセージも、内部状態
   の値そのものを含めず英語・固定文言とする（`wasm-client` 不変条件 6・
   `docs/design/hydration-nested-state.md` 第 8 節と同じ方針の継承）。
8. **サプライチェーン（REQ-3・REQ-11）**: #341〜#345 のいずれも依存
   クレート追加をゼロとする前提で設計している（`core`/`interactive` の
   外部依存ゼロ・依存上限 60 件/深さ 6 を維持）。依存追加が必要と判明
   した場合は事前にユーザー承認を得る（`.claude/rules/coding-rust.md`）。

## 10. 受け入れ基準対応表

| イシュー #340 の受け入れ条件 | 対応する本書の節 |
|------------------------------|------------------|
| `docs/design/` に設計文書を追加し、後続タスクの実装範囲・API 形状・移行手順が判断可能である | 第 3 節（束縛点対応表・core API 形状）・第 4 節（更新種別・dirty tracking API 形状）・第 5 節（keyed list API 形状・fail-closed 定義）・第 6 節（`set_inner_html` 撤去手順・移行順序） |
| セキュリティ不変条件（テキスト更新経路は HTML 解釈されない）を明記している | 第 9 節・不変条件 1（`set_text_content` が DOM 仕様上 HTML パースを経由しないことの明示） |

## 11. 関連文書との整合確認

- `docs/design/wasm-full-architecture.md` の冒頭ステータス節に、本書への
  移行予告を追記する（既存記述の削除・改変は行わない。第 6 節の内容が
  将来 #345 で反映されることのみを追記する）。
- `docs/api/interactive-api.md` 第 3 節の `Component` 凍結表
  （`fn update(&mut self, action: Self::Action)`、戻り値なし）を変更せず、
  第 4.2 節の dirty tracking は対になる別トレイト（`DirtyTracked`）として
  提供する設計とした。
- `docs/design/hydration-nested-state.md` が確立した `data-hydrate-*` の
  役割（状態値の注入）と、本書が新設する `data-bind-*`（更新先ノード
  位置のマーキング）の役割分担を第 3.2 節で明示した。
- `core/src/lib.rs` の `is_valid_attr_name`（属性名ホワイトリスト）・
  `find_attr_values`（属性値走査、`data-hydrate`/`data-nav` が使用する
  既存パターン）を、本書の `data-bind-*` 走査（第 3.2 節）・属性検証
  （第 3.3 節）がそのまま再利用する設計とした。
