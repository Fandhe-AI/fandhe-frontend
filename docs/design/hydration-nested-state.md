# ハイドレーション状態注入フォーマット: ネスト構造への一般化（イシュー #163）

## 1. 目的とトレーサビリティ

`docs/api/hydration-state-format.md`（TASK-11.4a・#82、確定版）は REQ-11 の受け
入れ基準を満たすため、ハイドレーション状態注入フォーマットを**単純な値
（数値・文字列・文字列配列）のみ**に限定して凍結し（同書第 2 節・判断 2）、
ネスト構造・オブジェクト・マップ等の複雑な状態への一般化をイシュー #163
（`feat(wasm-full): ハイドレーション状態注入の複雑な状態（ネスト構造等）へ
の一般化`）へ明示的に引き継いだ（同書第 7 節スコープ外表）。

`docs/spec/04-requirements.md:221` も「現時点では単純な値のみ・複雑な状態
への一般化は将来課題」と明記しており、本書はその将来課題の設計・実装を
確定するための成果物である（`docs/spec/` はサブモジュールのため編集禁止。
仕様本文の追随が必要な場合は frontend-framework-spec リポジトリへの Issue
起票を別途検討する）。

**本文書のステータス**: イシュー #163 の設計確定書。`interactive/src/lib.rs`
の `codec::Value`/`encode_value`/`decode_value` 実装と本書の記述に乖離が
生じた場合は本書を正とし、PR レビューで指摘する。

本書は `docs/api/hydration-state-format.md`・`docs/api/interactive-api.md`・
`docs/design/wasm-full-architecture.md` と同じ書式（ステータス・トレーサビリティ・
凍結表・設計判断表・スコープ外表・セキュリティ不変条件・受け入れ基準
対応表）に揃える。

## 2. スコープの確認

- 対象は `rws_interactive::Hydrate` トレイトが既に一般化している
  「フィールド名 → 文字列値」の写像（`Vec<(String, String)>`）の**属性値
  内部**表現の拡張のみである。`Hydrate` トレイト自体・`HYDRATE_ATTR_PREFIX`・
  既存の `codec::encode_list`/`decode_list`（`AppState` が使用）は**一切
  変更しない**（凍結済み API、`docs/api/hydration-state-format.md` 第 3 節）。
- 「トップレベルフィールド 1 個 = `data-hydrate-<field>` 属性 1 個」という
  既存の属性命名規約（同書第 3.1 節）も維持する。ネストは属性**値**の中で
  表現し、属性名パスへの平坦化（例: `data-hydrate-user-address-city`）は
  採用しない（第 4 節・判断 2 で却下理由を記録）。
- 依存クレート追加は**ゼロ**（REQ-11「追加の JSON 等のシリアライズ依存
  なし」・REQ-3 依存上限・`interactive` の外部依存ゼロ制約を維持）。

## 3. フォーマット仕様の凍結

### 3.1 `Value` 型

`rws_interactive::codec::Value`（`interactive/src/lib.rs`）としてネスト
可能な値ツリーを追加する。

```rust
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}
```

- `Map` は `HashMap` ではなく `Vec<(String, Value)>` とし、キー順を挿入順
  のまま保持する（決定的エンコードのため。ハッシュマップの反復順は非決定的
  であり、同一状態から異なるエンコード文字列が生成されるとテスト・
  デバッグ・キャッシュ判定が不安定になる）。キーの重複チェックは行わない
  （アプリ側の `Hydrate` 実装の責務）。

### 3.2 エンコード方式: 長さ明示型（netstring/Bencode 系）の再帰下降

**設計改訂（実装過程での重大な不具合発見）**: 当初案は「先頭 1 文字の型
タグ＋既存 `encode_list`/`escape_item` を子要素へ再帰適用する」方式
だったが、実装・テスト段階でこの方式に**指数的サイズ増大の不具合**が
あることが判明し、採用しなかった（詳細は第 4 節・判断 1 に事後記録）。
本書が確定する方式は、区切り文字のエスケープを一切使わず、**各値の
バイト長を事前に明示する**（netstring/Bencode と同種の）長さ明示型の
再帰下降エンコードである。

| バリアント | 形式 | 例 |
|-----------|------|-----|
| `Str(s)` | `s{s のバイト長}:{s}` | `Str("hi")` → `s2:hi` |
| `Int(i)` | `i{10 進文字列}e` | `Int(-42)` → `i-42e` |
| `Bool(b)` | `b1`（true）/ `b0`（false） | `Bool(true)` → `b1` |
| `List(items)` | `l{各要素のエンコード結果を連結}e` | `List([Int(1), Int(2)])` → `li1ei2ee` |
| `Map(entries)` | `m{(キーの Str エンコード + 値のエンコード) を連結}e` | `Map([("a", Int(1))])` → `ms1:ai1ee` |

キーは常に `Value::Str` と同じ形式（長さ明示のバイト文字列）でエンコード
する。子要素・キーの境界は「事前に宣言したバイト長」または明示的な終端
記号 `e` のみで決定され、内容に対する走査型のエスケープを一切行わない。

デコード（`decode_value`）はバイト列上のカーソル（読み取り位置）を
保持する再帰下降パーサとして実装し、先頭 1 バイトのタグを読み、
`s` なら直後の長さプレフィックスに従い正確にその分のバイト列を取り出し、
`i`/`l`/`m` なら終端記号 `e` に到達するまで読み進める。`List`/`Map` の
子要素・値はそれぞれ再帰的にデコードする。

### 3.3 既存 codec との関係（後方互換・非依存）

- **`escape_item`/`unescape_item`・`ITEM_SEP`（U+001F）・`ESCAPE_CHAR`
  （`\`）・`encode_list`/`decode_list` は一切変更せず、`Value` codec も
  これらを一切呼び出さない。** `Value` codec は完全に独立した長さ明示型
  エンコードとして実装したため、`AppState` 等の既存 `Hydrate` 実装が
  出力する `encode_list` の結果は本イシューの変更後も**バイト単位で
  不変**である（後方互換の維持、回帰テスト
  `value_codec_addition_does_not_change_encode_list_output` で固定）。
- 区切り文字を U+001E（Record Separator）等へ追加する案・既存
  `encode_list` を再帰適用する案は**いずれも採用しない**（第 4 節・
  判断 1 に不採用の詳細な経緯を記録）。長さ明示型を採用したことで、
  U+001F の HTML 属性内保持実証（PoC-5）への依存自体が不要になった
  （文字列内容に区切り文字が含まれていても、長さが事前に分かっている
  ため走査・エスケープが一切不要）。
- `List`/`Map` の子要素・キーはそれぞれ「事前に宣言したバイト長」で
  厳密に境界が確定するため、子の内容（タグに使われる文字・数字・`e`・
  `:` 等を含む任意の文字列）が親の境界を偽装することは構造的にできない
  （エスケープに頼らない安全性論証）。

### 3.4 ネスト深さの上限（DoS 耐性）とサイズの線形性

`decode_value` は再帰呼び出しの深さを追跡し、`MAX_VALUE_DEPTH`（`32`）を
超える入力に対して `Err(ValueDecodeError::DepthExceeded)` を返す
（`unwrap()`/`panic!` を使わず、スタックオーバーフローによるクラッシュを
防ぐ）。深さのカウントは `List`/`Map` へ 1 段再帰するたびに 1 加算する。

`32` は通常のアプリ状態（ネストしたフォーム・設定オブジェクト等、数段
程度のネスト）を十分許容しつつ、無制限の深さを弾く値として選定した。より
精密な上限が必要になった場合は Issue 化して見直す（YAGNI）。

エンコード・デコードとも、各バイトを高々 1 回処理する線形時間・線形サイズ
のアルゴリズムである（第 4 節・判断 1 の不具合はこの性質が破れていた
ことが原因であり、本方式ではネスト段数に対してエンコード結果のサイズが
線形に留まることを回帰テスト `encoded_size_grows_linearly_with_nesting_depth`
で固定している）。深さ制限と既存の `MAX_ATTR_VALUE_LEN`（64 KiB、
`wasm-full/src/hydration.rs`）の総量上限を組み合わせることで、全体の
デコードコストは入力サイズに対し線形に抑えられる。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | ネスト構造のエンコードは、既存 `encode_list`/`escape_item` の再帰適用ではなく、長さ明示型（netstring/Bencode 系）の再帰下降方式を採る | **実装過程で発見した重大な不具合**: 「子要素を再帰的に `encode_value` した文字列を、親レベルで `escape_item`（バックスラッシュ・U+001F をエスケープ）してから連結する」当初案は、ネストが 1 段深くなるごとに、それ以前の段で導入されたバックスラッシュ・区切り文字が**再度エスケープされ、エスケープ文字の出現数が毎段倍増**する（深さ D で符号化結果のサイズが O(2^D) に発散する）。`decode_value_rejects_excessive_nesting_without_panicking`（許容深さ + 10 段程度のネスト）の実装・テスト時にこの発散が発現し、エンコード処理自体が実用不能な時間・メモリを要することが判明した（`decode` 側の深さチェック以前に `encode_value` の呼び出し自体が発散するため、深さ制限では防げない）。長さ明示型は境界確定にエスケープを必要とせず、エンコード・デコードとも入力サイズに対して線形であるため、この問題が構造的に発生しない |
| 2 | 属性名パスへの平坦化（`data-hydrate-user-address-city` 等）は採用せず、ネストは属性値の中で表現する | ハイフンを含むフィールド名との曖昧性（`user-name` は 1 フィールドか `user.name` か区別できない）、配列内オブジェクト（`items[0].name`）を属性名だけでは表現できないこと、属性数がネスト段数に応じて爆発すること、の 3 点から不採用とする |
| 3 | `Map` は `Vec<(String, Value)>` とし `HashMap` を使わない | ハッシュマップの反復順は非決定的であり、同一状態から異なるエンコード文字列が生成されると、テストの再現性・デバッグ容易性を損なう。キー順の保持は決定的エンコードの前提条件 |
| 4 | `escape_item`/`unescape_item`・`encode_list`/`decode_list`（`AppState` 等が使用する既存 API）は一切変更せず、`Value` codec からも一切呼び出さない | 既存出力への影響（後方互換の破壊）を避ける。長さ明示型を独立実装として採用したことで、既存 codec との結合自体がゼロになり、判断 1 の不具合の再発可能性も構造的に排除される |
| 5 | `decode_value` に `MAX_VALUE_DEPTH`（`32`）を設け、超過は panic ではなく `Err` を返す | 改ざんされうるクライアント入力（`data-hydrate-*` 属性値）による深い再帰でのスタックオーバーフロー（DoS、A05 相当）を防ぐ（`.claude/rules/coding-rust.md` のエラーハンドリング規約）。長さ明示型はサイズの指数的発散（判断 1）を構造的に回避しているため、深さ制限は純粋にスタック消費（再帰呼び出し段数）の防御に専念できる |
| 6 | `Hydrate` の derive マクロは提供しない。`Value` との相互変換はアプリ側の手書き実装とする | proc-macro クレート追加は依存増（REQ-3）であり、`core` 外部依存ゼロ・依存上限 60 件/深さ 6 の制約に反する。マクロ DSL 回避方針（REQ-5 系の設計判断の踏襲）とも整合する |
| 7 | `ValueDecodeError` から `HydrateError::InvalidValue` への変換は `From` トレイトではなく `into_hydrate_error(attr: &str)` メソッドで提供する | `HydrateError::InvalidValue` は属性名（`attr`）を保持する必要があるが、`ValueDecodeError` 自体はどの属性のデコードで発生したかを知らない（呼び出し側だけが知っている）。`From` の実装では属性名を渡せないため、明示的なメソッドとする |

## 5. `codec::Value`/`encode_value`/`decode_value` の API 表面

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `Value` | `pub enum Value { Str(String), Int(i64), Bool(bool), List(Vec<Value>), Map(Vec<(String, Value)>) }` | ネスト可能なハイドレーション値ツリー |
| `MAX_VALUE_DEPTH` | `pub const MAX_VALUE_DEPTH: u32 = 32` | `decode_value` が許容する最大ネスト深さ |
| `ValueDecodeError` | `pub enum ValueDecodeError { Empty, UnknownTag(char), InvalidLength, InvalidUtf8, InvalidInt, InvalidBool, InvalidMapKey, UnexpectedEnd, DepthExceeded, TrailingData }` | `decode_value` の失敗種別 |
| `ValueDecodeError::into_hydrate_error` | `pub fn into_hydrate_error(self, attr: &str) -> HydrateError` | アプリの `Hydrate::from_hydration_attrs` 実装が `?` で使える変換ヘルパ |
| `encode_value` | `pub fn encode_value(value: &Value) -> String` | `Value` を 1 属性値文字列へエンコードする（サーバー側責務） |
| `decode_value` | `pub fn decode_value(input: &str) -> Result<Value, ValueDecodeError>` | [`encode_value`] の逆変換（クライアント側責務）。`unwrap()`/`panic!` を使わない |

アプリの `Hydrate` 実装は、ネスト構造を持つフィールドについて
`hydration_attrs()` 内で `Value` へ変換して `encode_value` を呼び、単一の
`data-hydrate-<field>` 属性として出力する。`from_hydration_attrs()` では
`decode_value` の結果（`Value`）をアプリ固有の型へ変換する（`match` に
よる型チェック、型不一致は `HydrateError::InvalidValue` として扱う）。
`wasm-full/tests/nested_hydration_state.rs` の `NestedState`/`UserProfile`
がこの往復パターンの実装例を提供する。

## 6. テスト観点

- **ラウンドトリップ**: `Str`/`Int`/`Bool` の単純値、深いネスト（`Map` の
  中に `List`、その中に `Map`）、空 `Map`/空 `List`、境界値（`i64::MIN`/
  `MAX`）、日本語・絵文字を含む文字列（`interactive/src/lib.rs` の
  `codec::tests` モジュール）。
- **敵対的入力**（すべて `Err`、panic しない）: 未知の型タグ、空入力、
  `Int`/`Bool` ペイロードのパース失敗、`Map` キー位置への非文字列混入、
  長さプレフィックスが残り入力を超える改ざん、終端記号 `e` 欠落、末尾の
  余分なバイト列、`MAX_VALUE_DEPTH` 超過の深いネスト、非 ASCII な
  マルチバイト文字が型タグ位置に来る場合のバイト境界安全性。
- **線形性の回帰確認**: `MAX_VALUE_DEPTH` 段のネストを持つ値のエンコード
  結果サイズが線形（数百バイト程度）に収まること
  （`encoded_size_grows_linearly_with_nesting_depth`、第 4 節・判断 1 で
  発見した指数的サイズ増大バグの再発防止）。
- **後方互換**: `Value` codec の追加後も既存 `encode_list`/`decode_list`
  の出力・挙動が完全に不変であること（`interactive/src/lib.rs` の
  `value_codec_addition_does_not_change_encode_list_output`）。
- **統合テスト**: ネスト構造を持つ独自の `Hydrate` 実装
  （`wasm-full/tests/nested_hydration_state.rs::NestedState`）での
  `restore_state` ラウンドトリップ・改ざん値・過度なネスト・型不一致の
  各ケース。
- **XSS 回帰**: ネスト値（`Map` in `Map`）経由でも `render_for_hydration`
  の既定エスケープが貫通すること、および `Value` codec 自体は HTML
  エスケープを一切行わない契約であることの両方
  （`interactive/tests/xss_escape.rs`）。

## 7. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `Hydrate` の derive マクロ提供 | 要望が出た場合に別 Issue を提案（第 4 節・判断 6） |
| `AppState` 自体のネスト化（デモ用途変更） | 別途（本書はネスト codec の提供のみが対象） |
| `docs/spec/04-requirements.md:221` の「将来課題」文言の更新 | frontend-framework-spec リポジトリへの Issue 起票をユーザーへ提案 |
| U+001E 等の追加区切り文字による属性値表現の実ブラウザ実証 | 不採用（第 3.3 節・第 4 節・判断 1）のため引き継ぎ不要。将来 U+001F 以外の区切り方式が必要になった場合は改めて設計する |

## 8. セキュリティ不変条件

`docs/api/hydration-state-format.md` 第 8 節の不変条件をそのまま継承し、
ネスト値 codec 固有の不変条件を追加する。

1. **既定エスケープの一貫性（REQ-1）**: ネスト値を含む `data-hydrate-*`
   属性の出力も `render_for_hydration` → `rws_core::render()` の既定
   エスケープを必ず経由する。`codec::Value`/`encode_value`/`decode_value`
   は新たなエスケープ迂回経路（`raw_html()` の使用・HTML 文字列直接
   組み立て）を作らない。**`Value` codec 自体は HTML エスケープを一切
   行わない**（区切り文字・エスケープ文字のみを対象とした構造的
   エスケープに専念し、HTML としての安全性は render 層の責務のまま
   分離を保つ）。
2. **改ざん耐性（A08 相当）**: `data-hydrate-*` は信頼できないクライアント
   入力として扱う。`decode_value` は `Result` ベースとし panic しない。
   項目境界の偽装は、境界がエスケープではなく事前宣言されたバイト長・
   明示的な終端記号のみで確定する設計（第 3.2〜3.3 節）により構造的に
   防がれる。
3. **DoS 耐性（A05 相当）**: `MAX_VALUE_DEPTH`（32）により深い再帰による
   スタックオーバーフローを `Err` で遮断する。長さ明示型はエンコード・
   デコードとも入力サイズに対して線形（第 3.4 節・第 4 節・判断 1）で
   あり、既存の `MAX_ATTR_VALUE_LEN`（64 KiB）と組み合わせることで
   全体のデコードコストが入力サイズに対し線形に収まる。
4. **サプライチェーン（REQ-3・REQ-11）**: 依存クレート追加はゼロ（`serde`
   等は使用しない）。`interactive` の外部依存ゼロ・`forbid(unsafe_code)`
   を維持する。
5. **エラー・ログの機微情報非露出（A09 相当）**: `ValueDecodeError`/
   `HydrateError` の `Display` 実装に、復元対象の実際の状態値・内部パス等
   の機微情報を含めない（属性名・理由コードのみ、英語）。

## 9. REQ-11 受け入れ基準との対応表

| 受け入れ基準 | 満たす設計要素 |
|-------------|----------------|
| ネストしたオブジェクト等の複雑な状態のシリアライズ・注入方式が設計・実装されていること | `codec::Value`（長さ明示型の再帰ツリー）＋`encode_value`/`decode_value`（第 3〜5 節） |
| 追加の JSON 等のシリアライズ依存なしに成立すること | 長さ明示型（netstring/Bencode 系）の自作エンコードのみで実装（第 3.2〜3.3 節）、依存クレート追加ゼロ |
| 既存の単純な値のみのフォーマット（`docs/api/hydration-state-format.md`）との後方互換 | `escape_item`/`unescape_item`・`encode_list`/`decode_list` を無変更のまま維持し、`Value` codec からも一切呼び出さない（第 3.3 節・第 4 節・判断 4、回帰テストで固定） |

## 10. 関連文書との整合確認

- `docs/api/hydration-state-format.md` 第 2 節・第 7 節が本書（イシュー #163）
  への引き継ぎとして明記した記述と整合する。同書自体は改訂せず、本書が
  独立した拡張として参照する形を取る。
- `docs/api/interactive-api.md` 第 3〜4 節の `Hydrate`・`HYDRATE_ATTR_PREFIX`・
  `HydrateError`・既存 `codec::encode_list`/`decode_list` の凍結記述を
  そのまま引用し、本書側で再定義・変更していない。
- `interactive/src/lib.rs`（実装）の `codec::Value`/`encode_value`/
  `decode_value`/`MAX_VALUE_DEPTH`/`ValueDecodeError` と本書第 3〜5 節の
  記述が一致することを確認済み。
