# コンパイルエラー品質の定性レビュー（TASK-5.3）

## 1. 目的

`docs/spec/04-requirements.md` REQ-5「独自 DSL に依存しないプレーン Rust コンポーネント記述」の受け入れ基準 3 点目——

> コンパイルエラーが、マクロ展開後のコードを指す読みにくいメッセージではなく、通常の Rust の型エラーとして表示されること（PoC-3 の定性記録）

——を、製品版 `rws-core` 公開 API（`el` / `text` / `raw_html` / `render` / `Node`）に対して定性レビューとして実測・体系化し、`docs/spec/05-tasks.md` TASK-5.3 の成果物として記録する。

PoC-3（`docs/spec/03-poc/rendering-web-standards/README.md` セクション 8「開発時の反復速度・エラー体験」）は次のように記録している。

> マクロ DSL を使わないため、コンパイルエラーは通常の Rust の型エラーとしてそのまま表示される（`view!` マクロ経由のエラーメッセージのような、マクロ展開後のコードを指す読みにくいエラーに遭遇しなかった）。一方で、ノード木をネストした関数呼び出しで書くため、深いネストでは閉じ括弧の対応を追いにくく、JSX 風マクロの視覚的な構造の分かりやすさは失われる。

本レビューは、この PoC 時点の定性記録を製品版 API に対して再現し、より広い型ミスのバリエーション（7 ケース）で裏付けるものである。

`docs/spec/05-tasks.md` の TASK-5.3 定義は「マクロ非依存という設計選択の妥当性判断を伴うため人間が担当」としている。本レビューでは Claude がエラーケースの作成・rustc 実出力の採取・評価草案の作成までを機械的に行い、**総合所見（設計選択の妥当性の最終判断）は本ドキュメント末尾「人間レビュー承認」セクションで人間が確定する**。

## 2. レビュー方法と環境

- **rustc**: 1.96.0（`ac68faa20 2026-05-25`）
- **cargo**: 1.96.0（`30a34c682 2026-05-25`）
- **対象 API**: `rws-core`（`core/src/lib.rs`、`origin/main` 時点で TASK-5.1 完了済みの公開 API）
- **フィクスチャ**: `core/tests/compile_fail/case01_*.rs` 〜 `case07_*.rs`（意図的にコンパイル不能。`core/tests/compile_fail/README.md` に位置づけと再現手順を記載）
- **採取手順**: ワークスペース外の一時検証クレート（`[dependencies] rws-core = { path = "<repo>/core" }`）を作成し、各フィクスチャの**コード本体のみ**（冒頭の `//!` ドキュメンテーションコメントを除いた部分）を検証クレートの `src/lib.rs` に配置して `cargo check 2>&1` を実行し、標準エラー出力をそのまま記録した。採取した出力中のローカル絶対パスは `<repo>` / `<scratchpad>` に置換してサニタイズしている。
- **再現方法**: `core/tests/compile_fail/README.md` の手順に従う。ただし本ドキュメントの各ケースで引用する `--> src/lib.rs:行:列` の行番号は、フィクスチャ冒頭の `//!` ヘッダーコメントを除いたコード本体を基準にしている。README の手順どおりフィクスチャファイルをそのまま `src/lib.rs` へコピーすると、ヘッダーコメント分（7 行）だけ行番号がずれる（例: ケース 1 は `:5:` ではなく `:12:` 付近になる）。エラーの**内容**（型・トレイト境界・メッセージ文言）は完全に一致し、本レビューの主張はその内容に基づく。

## 3. 評価基準

各ケースを次の 3 観点で評価する。

1. **行番号の正確さ**: エラーがマクロ展開後の合成コードではなく、利用者が書いたソースの該当行・該当式を直接指しているか
2. **マクロ展開痕跡の不在**: `in this macro invocation` / `this error originates in the macro` 等、マクロ展開由来であることを示す文言が出力に含まれないか
3. **`help:` / `note:` の有用性**: rustc が提示する補助情報（型の期待値・訂正候補・借用の提案等）が、次に取るべきアクションを具体的に示しているか

## 4. ケース別レビュー

### ケース 1: 子ノードの型ミス（`Vec<Node>` 期待に `Node` を渡す）

フィクスチャ（`core/tests/compile_fail/case01_child_type_mismatch.rs`）:

```rust
el("div", vec![], text("hi"))
```

実測出力（抜粋、期待エラーコード E0308）:

```
error[E0308]: mismatched types
   --> src/lib.rs:5:23
    |
  5 |     el("div", vec![], text("hi"))
    |     --                ^^^^^^^^^^ expected `Vec<Node>`, found `Node`
    |     |
    |     arguments to this function are incorrect
    |
    = note: expected struct `Vec<Node>`
                 found enum `Node`
note: function defined here
   --> <repo>/core/src/lib.rs:104:8
    |
104 | pub fn el(tag: &'static str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    |        ^^
```

- 行番号の正確さ: 利用者コードの該当式（`text("hi")`）をそのまま指している。○
- マクロ展開痕跡: なし。○
- `help:` の有用性: `expected`/`found` の型が明示され、`el` の関数シグネチャへの `note` も付く。呼び出し側で `vec![text("hi")]` に直すべきことが直接わかる。○

### ケース 2: `Into<String>` 未実装型（`text(42)`）

フィクスチャ（`case02_into_string_not_implemented.rs`）:

```rust
text(42)
```

実測出力（抜粋、期待エラーコード E0277）:

```
error[E0277]: the trait bound `String: From<{integer}>` is not satisfied
   --> src/lib.rs:5:10
    |
  5 |     text(42)
    |     ---- ^^ the trait `From<{integer}>` is not implemented for `String`
    |     |
    |     required by a bound introduced by this call
    |
    = help: `String` implements trait `From<T>`:
              From<&String>
              From<&mut str>
              From<&str>
              From<Box<str>>
              From<Cow<'_, str>>
              From<char>
    = note: required for `{integer}` to implement `Into<String>`
note: required by a bound in `text`
   --> <repo>/core/src/lib.rs:126:21
    |
126 | pub fn text(s: impl Into<String>) -> Node {
    |                     ^^^^^^^^^^^^ required by this bound in `text`
```

- 行番号の正確さ: `text(42)` の呼び出し箇所を直接指す。○
- マクロ展開痕跡: なし（`impl Trait` のトレイト境界エラーとして通常通り表示）。○
- `help:` の有用性: `String` が実装する `From<T>` の一覧が提示され、`42.to_string()` 等への修正方針が読み取れる。○

### ケース 3: 動的タグ名（`&'static str` 制約とライフタイム）

フィクスチャ（`case03_dynamic_tag_name.rs`）:

```rust
let t = String::from("div");
el(&t, vec![], vec![])
```

実測出力（抜粋、期待エラーコード E0597）:

```
error[E0597]: `t` does not live long enough
 --> src/lib.rs:6:8
  |
5 |     let t = String::from("div");
  |         - binding `t` declared here
6 |     el(&t, vec![], vec![])
  |     ---^^-----------------
  |     |  |
  |     |  borrowed value does not live long enough
  |     argument requires that `t` is borrowed for `'static`
7 | }
  | - `t` dropped here while still borrowed
```

- 行番号の正確さ: 束縛箇所（5 行目）と借用箇所（6 行目）の両方を正確に指す。○
- マクロ展開痕跡: なし。○
- `help:` の有用性: 「`'static` で借用される必要がある」という要求が明示される。`el` のタグ名が `&'static str` 固定であること（動的タグ名注入を型で防ぐ設計、不変条件 5）に利用者が気づける形で表示されている。○
- 備考: このケースは型ミスというより設計上の制約（タグ名の静的固定）に起因するエラーだが、通常のライフタイムエラーとして表示され、マクロ由来の特殊な文言は一切現れない。

### ケース 4: 属性タプルの型ミス

フィクスチャ（`case04_attr_tuple_type_mismatch.rs`）:

```rust
el("div", vec![("class", 3)], vec![])
```

実測出力（抜粋、期待エラーコード E0308）:

```
error[E0308]: mismatched types
 --> src/lib.rs:5:30
  |
5 |     el("div", vec![("class", 3)], vec![])
  |                              ^ expected `&str`, found integer
```

- 行番号の正確さ: 該当リテラル（`3`）の列位置まで正確。○
- マクロ展開痕跡: なし。○
- `help:` の有用性: 短く簡潔。`expected`/`found` のみだが型ミスの箇所が一目でわかり追加情報は不要なレベル。○

### ケース 5: 子 `Vec` への異型混在

フィクスチャ（`case05_children_vec_type_mismatch.rs`）:

```rust
vec![text("a"), "b"]
```

実測出力（抜粋、期待エラーコード E0308）:

```
error[E0308]: mismatched types
 --> src/lib.rs:5:21
  |
5 |     vec![text("a"), "b"]
  |                     ^^^ expected `Node`, found `&str`
```

- 行番号の正確さ: `vec!` マクロ自体は標準ライブラリのものだが、エラーは利用者コードの要素式（`"b"`）を直接指しており、`rws-core` 側のマクロ展開痕跡は存在しない。○
- マクロ展開痕跡: なし（`vec!` は Rust 標準マクロであり、展開痕跡ではなく通常の要素型推論のエラー）。○
- `help:` の有用性: `text("b")` へのラップ忘れであることがそのまま読み取れる。○

### ケース 6: 存在しない enum バリアント参照

フィクスチャ（`case06_nonexistent_variant.rs`）:

```rust
Node::Raw("x".to_string())
```

実測出力（抜粋、期待エラーコード E0599）:

```
error[E0599]: no variant, associated function, or constant named `Raw` found for enum `Node` in the current scope
 --> src/lib.rs:5:11
  |
5 |     Node::Raw("x".to_string())
  |           ^^^ variant, associated function, or constant not found in `Node`
```

- 行番号の正確さ: 該当識別子（`Raw`）を直接指す。○
- マクロ展開痕跡: なし。○
- `help:` の有用性: △ ── 今回の実測では、正しいバリアント名 `RawHtml` への訂正候補（"did you mean" 相当の提案）は表示されなかった。`Raw` と `RawHtml` の編集距離がやや大きいため rustc の類似名検出が働かなかったと考えられる。エラー自体は明確だが、修正候補の提示という点では他ケースよりやや弱い。

### ケース 7: 参照渡し忘れ

フィクスチャ（`case07_missing_reference.rs`）:

```rust
let node = text("hi");
render(node)
```

実測出力（抜粋、期待エラーコード E0308）:

```
error[E0308]: mismatched types
   --> src/lib.rs:6:12
    |
  6 |     render(node)
    |     ------ ^^^^ expected `&Node`, found `Node`
    |     |
    |     arguments to this function are incorrect
    |
note: function defined here
   --> <repo>/core/src/lib.rs:311:8
    |
311 | pub fn render(node: &Node) -> String {
    |        ^^^^^^
help: consider borrowing here
    |
  6 |     render(&node)
    |            +
```

- 行番号の正確さ: 呼び出し箇所を正確に指す。○
- マクロ展開痕跡: なし。○
- `help:` の有用性: `consider borrowing here` により `&node` への具体的な修正案が diff 形式で提示される。7 ケース中最も親切な `help:` の例。○

## 5. 総合所見（マクロ非依存設計の妥当性判断のための整理）

7 ケースすべてにおいて、以下が実測により確認された。

- **マクロ展開痕跡は 1 件も観測されなかった**（`in this macro invocation` 等の文言は全ケースで不在）。`rws-core` が手続きマクロ・宣言マクロ DSL を経由しない素の Rust 関数・enum で構成されているという設計（TASK-5.1 実装）がそのままエラー体験に反映されている。
- **行番号・列位置は全ケースで利用者コードの該当箇所を正確に指していた**。マクロ展開後の合成コード上の位置を指す事例はなかった。
- **`help:`/`note:` による訂正提案は 7 ケース中 6 ケースで具体的だった**（型の期待値、トレイト実装候補、借用提案）。唯一弱かったのはケース 6（存在しない enum バリアント）で、類似名の訂正候補が提示されなかった。ただしエラーメッセージ自体（バリアント不在・列挙体名）は明確であり、可読性を損なうものではない。
- これらは PoC-3 の定性記録（「`view!` マクロ経由のエラーメッセージのような、マクロ展開後のコードを指す読みにくいエラーに遭遇しなかった」）と整合する結果であり、製品版 API でも同様の特性が再現されることを裏付ける。

一方で、PoC-3 が指摘した既知のトレードオフ（ノード木のネストを関数呼び出しで書くため、深いネストで閉じ括弧の対応を追いにくい）は、コンパイルエラーの品質とは別軸の可読性課題として Issue #164（`feat(core): ノード木記述の可読性向上（ヘルパー関数・インデント規約）`）で追跡中であり、本レビューのスコープ外とする。

以上の実測結果は、REQ-5 受け入れ基準 3 点目を支持する材料として整理したものである。**マクロ非依存という設計選択自体の妥当性に関する最終判断は、次節の人間レビュー承認によって確定する。**

## 6. 人間レビュー承認

- **承認者**: admin@fandhe.com
- **承認日**: 2026-07-19
- **承認内容**: 上記「総合所見」がマクロ非依存設計（REQ-5）の妥当性判断として妥当であることを確認し、`docs/spec/05-tasks.md` TASK-5.3 の完了として承認する。
- **承認時の再検証**: 承認に先立ち、7 ケースすべてを main HEAD 時点の `rws-core` に対して §2 の手順で再実測し、エラー内容・文言・利用者コード側の行列位置が本ドキュメントの記録と完全一致することを確認した。唯一の差分は `note: function defined here` が指す `core/src/lib.rs` 側の定義行番号（`el`: 104→160、`text`: 126→182、`render`: 311→333。レビュー採取後のタグショートカット追加等による定義位置のずれ）であり、エラー品質の評価には影響しない。

## 7. 既知のトレードオフ・スコープ外事項

- 深いネストの可読性課題（閉じ括弧対応の追いにくさ）は Issue #164 で追跡。本レビューはコンパイルエラー品質のみを対象とし、可読性向上策の設計には立ち入らない。
- `compile_fail` フィクスチャの自動テスト化（`trybuild` 等によるコンパイル失敗の CI 回帰保証）は、依存クレート追加の事前ユーザー承認が必要（REQ-3 依存上限・`core` 外部依存ゼロ規約）なため本タスクのスコープ外とした。必要性が生じた場合は別 Issue として提案する。
