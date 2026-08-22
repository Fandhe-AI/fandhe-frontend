//! keyed list プリミティブ（構造変化の唯一の経路、イシュー #344）。
//!
//! `docs/design/dom-binding-update-design.md`（#340 設計確定書）第 5 節が
//! 定める、実 DOM 直接更新方針（イシュー #336）における「リストの挿入・
//! 削除・並べ替え（構造変化）を表現できる唯一の経路」。汎用 diff・仮想 DOM
//! は実装しない（同書第 5・7 節で確定済み）。
//!
//! SSR/SSG 出力には [`BIND_LIST_ATTR`]（`data-bind-list="<field>"`）が
//! リスト親要素に、[`KEY_ATTR`]（`data-key="<key>"`）が各子要素に現れる。
//! `wasm-full` の CSR 側（イシュー #343/#345）はこの 2 属性を走査してキー
//! 照合を行い、`set_inner_html` による全置換ではなく最小の DOM 操作
//! （insert/remove/move）を適用する契約になっている。**本モジュールが
//! 生成するのはこの属性形式の `Node` 木のみ**であり、DOM 適用そのものは
//! #343/#345 のスコープ（本モジュールの責務外）。一方、**op 生成（diff・
//! 内容比較）は本モジュールの責務**である（イシュー #1323、
//! `docs/design/keyed-update-op-design.md` §4.1 が確定）: [`KeyedOp`] /
//! [`diff_keys`] / [`diff_keyed_items`] がその実体であり、
//! `fandhe-frontend-wasm-client` の `keyed_diff` モジュールはこれらを
//! 消費する側（#1324 で re-export へ置換予定）に位置づけが変わる。
//!
//! # 不変条件（本クレート冒頭 doc の不変条件 1・2 の継承）
//!
//! [`keyed_list`] は既存の [`crate::Node`] 木を組み立てるだけであり、新しい
//! `Node` バリアント・新しいレンダリング経路・新しいエスケープ処理を追加
//! しない。出力される `data-key`/`data-bind-list` の属性値は
//! [`crate::render`] の既定エスケープを常に経由する（不変条件 1）。エスケープ
//! を迂回する経路は本モジュールには存在しない（不変条件 2 を弱めない）。
//! [`diff_keys`] / [`diff_keyed_items`] は純粋なキー列・`Node` 木の比較のみを
//! 行い、HTML 文字列の組み立て・レンダリングを一切行わないため、この不変
//! 条件に影響しない。

use crate::Node;

/// リスト束縛のマーカー属性名。
///
/// keyed list の親要素に付与され、値はリスト化対象のフィールド名
/// （`&'static str`）。`wasm-full`（#343/#345）はこの属性を走査してリスト
/// 親要素を特定する契約値であり、値は本モジュールの `render()` 出力上で
/// 固定される（設計書 §3.1 で凍結）。
pub const BIND_LIST_ATTR: &str = "data-bind-list";

/// キー属性名。
///
/// keyed list の各子要素に付与され、値はアプリ側が指定した一意キー。
/// `wasm-full`（#343/#345）はこの属性値でキー照合を行い、挿入・削除・
/// 並べ替えを最小の DOM 操作へ変換する契約値（設計書 §3.1 で凍結）。
pub const KEY_ATTR: &str = "data-key";

/// keyed list 1 件あたりに許容する最大項目数（HashDoS 対策の追加防御、
/// イシュー #1375 codex-review P1 是正）。
///
/// `crate::fx_hash` モジュール doc「ターゲット別ハッシャ選択」節が確定する
/// 設計（ネイティブ: 本物の SipHash 耐性を持つ `RandomState`、
/// wasm32-unknown-unknown: 固定初期状態の軽量 `FxHasher`）に対し、
/// codex-review は「ブラウザ側でも衝突耐性のあるハッシャを維持するか、
/// キー件数・総バイト数の厳格な上限など攻撃可能な計算量を拘束する追加
/// 防御を導入すること」を求めた（PR #1390 レビュー）。本モジュールは
/// 後者（上限による計算量拘束）を [`keyed_list`] 構築時点の fail-closed
/// 拒否として採用する。
///
/// # 全ターゲット一律で強制する理由
///
/// wasm32-unknown-unknown 限定のガードは採らない。理由は 2 点:
/// (1) `cargo test` を wasm32 target 上で実行する CI ジョブが現状存在
/// せず（`fx_hash` モジュール doc 参照）、`cfg` 分岐を持つガードは CI で
/// 実質未検証のまま出荷することになる。(2) ネイティブ（SSR/SSG）側だけ
/// 上限がない非対称構成は「サーバー側では通る同じ `items` が CSR 側での
/// み拒否される」ハイドレーション不一致を生み、`fx_hash` モジュール doc
/// が固定する「ハッシャ選択が SSR/SSG 出力バイトへ影響しない」不変条件
/// と設計思想が矛盾する。このためガードは全ターゲット共通のコード
/// パスに置き、`cfg` 分岐を持たない。
///
/// # 値の根拠（`N^2` 見積もり）
///
/// [`crate::fx_hash`] の軽量ハッシャ（wasm32-unknown-unknown で使われる
/// 実体）は固定初期状態のため、攻撃者は全項目が同一バケットへ収まる
/// キー列を事前計算できる（最悪計算量が `O(n^2)` へ劣化する HashDoS の
/// 前提）。`N = 4_096` のとき最悪計算量は `N^2 = 16_777_216` 回程度の
/// バイト単位比較に収まり、1 ブラウザタブ内の単発処理として許容できる
/// 規模に留める。
pub const MAX_KEYED_LIST_ITEMS: usize = 4_096;

/// keyed list 1 件あたりに許容するキー文字列の合計バイト数
/// （[`MAX_KEYED_LIST_ITEMS`] と対の追加防御）。
///
/// ハッシュ計算コストは走査したバイト数に比例するため、項目数の上限
/// だけでは「少数の巨大なキー文字列」による計算量膨張を防げない。総
/// バイト数を独立に拘束することで、この経路も塞ぐ。
pub const MAX_KEYED_LIST_KEY_BYTES: usize = 262_144;

/// [`MAX_KEYED_LIST_ITEMS`]/[`MAX_KEYED_LIST_KEY_BYTES`] を強制する共通
/// ゲート（PR #1390 codex-review P1 是正、イシュー #1375）。
///
/// 当初 [`keyed_list`] のみが適用していたが、[`diff_keys`]/
/// [`diff_keyed_items`] は `keyed_list` を経由しない生の `&[String]`/
/// `&[(String, Node)]` を直接受け取れる公開 API であり、`keyed_list` の
/// 上限を経由せずに攻撃者が選んだキー列を直接投入できてしまう（wasm32-
/// unknown-unknown 上の軽量 `FxHasher` は固定初期状態であるため、この経路
/// をすり抜けると `keyed_list` 側の追加防御が無意味化する。`fx_hash`
/// モジュール doc「ターゲット別ハッシャ選択」節参照）。このため両関数も
/// 呼び出し直後にこのゲートを通す（内部で `HashMap`/`HashSet` を構築する
/// 前に必ず適用する）。
fn enforce_key_limits(count: usize, total_key_bytes: usize) -> Result<(), KeyedListError> {
    if count > MAX_KEYED_LIST_ITEMS {
        return Err(KeyedListError::TooManyItems { count });
    }
    if total_key_bytes > MAX_KEYED_LIST_KEY_BYTES {
        return Err(KeyedListError::KeyBytesExceeded {
            total_bytes: total_key_bytes,
        });
    }
    Ok(())
}

/// [`keyed_list`] 構築時、および [`diff_keys`]/[`diff_keyed_items`] 実行時の
/// fail-closed エラー。
///
/// いずれの異常系も `panic!`/`unwrap()` ではなく `Err` として安全側に倒す
/// （ライブラリコードでの panic 回避規約、OWASP A05 安全でない設計への対抗）。
/// 「衝突・欠落したキーを持つ不正な HTML を出力しない」という fail-closed の
/// 目的を、`render()` 呼び出し時点ではなく**構築時点**で満たす（不正な状態を
/// そもそも表現不能にする設計。`docs/design/dom-binding-update-design.md`
/// §5.2 改訂内容を参照）。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedListError {
    /// 項目数が [`MAX_KEYED_LIST_ITEMS`] を超えている（HashDoS 対策の追加
    /// 防御、イシュー #1375 codex-review P1 是正）。[`keyed_list`]・
    /// [`diff_keys`]・[`diff_keyed_items`] のいずれからも返り得る
    /// （PR #1390 レビュー是正で `diff_keys`/`diff_keyed_items` にも同じ
    /// ゲート [`enforce_key_limits`] を適用したため）。
    TooManyItems {
        /// 実際の項目数。
        count: usize,
    },
    /// 全項目のキー文字列の合計バイト数が [`MAX_KEYED_LIST_KEY_BYTES`]
    /// を超えている（同上。同じく 3 関数いずれからも返り得る）。
    KeyBytesExceeded {
        /// 実際の合計バイト数。
        total_bytes: usize,
    },
    /// キーが空文字列（キー欠落）。`index` は `items` 内の位置。
    /// [`keyed_list`] 専用（[`diff_keys`]/[`diff_keyed_items`] は生の
    /// キー列を比較するのみで空キーを拒否しない）。
    EmptyKey {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 同一リスト内でキーが重複している（直下の子スコープのみが対象）。
    /// [`keyed_list`] 専用。
    DuplicateKey {
        /// 最初に当該キーが出現したインデックス。
        first_index: usize,
        /// 重複が検出されたインデックス。
        duplicate_index: usize,
    },
    /// 子ノードが `Node::Element` でなく、`data-key` 属性を付与できない。
    /// [`keyed_list`] 専用。
    NonElementItem {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 呼び出し側が渡した属性列に予約マーカー属性
    /// （[`BIND_LIST_ATTR`] / [`KEY_ATTR`]）が既に含まれている。
    /// [`keyed_list`] 専用。
    ReservedAttr {
        /// 衝突した予約属性名。
        attr: &'static str,
    },
}

impl std::fmt::Display for KeyedListError {
    /// エラーメッセージは英語・固定文言 + インデックスのみとし、キー値・
    /// 項目内容（アプリ状態）は含めない（ログ・エラーメッセージへの機微
    /// 情報非露出、OWASP A09 対策。設計書 §9 不変条件 7 を継承）。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyedListError::TooManyItems { count } => write!(
                f,
                "keyed_list: item count {count} exceeds the maximum of {MAX_KEYED_LIST_ITEMS}"
            ),
            KeyedListError::KeyBytesExceeded { total_bytes } => write!(
                f,
                "keyed_list: total key bytes {total_bytes} exceeds the maximum of \
                 {MAX_KEYED_LIST_KEY_BYTES}"
            ),
            KeyedListError::EmptyKey { index } => {
                write!(f, "keyed_list: empty key at item index {index}")
            }
            KeyedListError::DuplicateKey {
                first_index,
                duplicate_index,
            } => write!(
                f,
                "keyed_list: duplicate key at item index {duplicate_index} \
                 (first seen at index {first_index})"
            ),
            KeyedListError::NonElementItem { index } => {
                write!(
                    f,
                    "keyed_list: item at index {index} is not an Element node"
                )
            }
            KeyedListError::ReservedAttr { attr } => {
                write!(f, "keyed_list: attribute \"{attr}\" is reserved")
            }
        }
    }
}

impl std::error::Error for KeyedListError {}

/// keyed list を構築する。構造変化（挿入・削除・並べ替え）を表現できる
/// **唯一の経路**（設計書第 5 節）。
///
/// 呼び出し側は親要素のタグ名・属性・リスト化対象フィールド名・
/// `(キー, 子ノード)` のペア列を渡す。成功時は次の形の `Node::Element` を
/// 返す。
///
/// - 親要素: `attrs` の末尾に `data-bind-list="<field>"` を付加したもの。
/// - 各子要素: 元の属性列の末尾に `data-key="<key>"` を付加したもの。
///
/// キーの一意性検査は**直下の子のみ**が対象（子孫にネストした
/// `keyed_list` 呼び出しのキー空間とは独立）。
///
/// # Errors
///
/// - [`KeyedListError::TooManyItems`][]: 項目数が
///   [`MAX_KEYED_LIST_ITEMS`] を超えている（HashDoS 対策の追加防御）。
/// - [`KeyedListError::KeyBytesExceeded`][]: キー文字列の合計バイト数が
///   [`MAX_KEYED_LIST_KEY_BYTES`] を超えている（同上）。
/// - [`KeyedListError::EmptyKey`][]: いずれかのキーが空文字列。
/// - [`KeyedListError::DuplicateKey`][]: 同一リスト内でキーが重複。
/// - [`KeyedListError::NonElementItem`]: 子ノードが `Node::Element` でない
///   （`data-key` を付与する対象を持たないため）。
/// - [`KeyedListError::ReservedAttr`]: `attrs` または各子要素の属性列に
///   [`BIND_LIST_ATTR`] / [`KEY_ATTR`] が既に含まれている（マーカー属性の
///   重複・偽装を構造的に防止）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, text, render, keyed::keyed_list};
///
/// let list = keyed_list(
///     "ul",
///     vec![("data-testid", "item-list")],
///     "items",
///     vec![
///         ("a".to_string(), el("li", vec![], vec![text("item-a")])),
///         ("b".to_string(), el("li", vec![], vec![text("item-b")])),
///     ],
/// )
/// .expect("valid keyed list");
///
/// assert_eq!(
///     render(&list),
///     concat!(
///         r#"<ul data-testid="item-list" data-bind-list="items">"#,
///         r#"<li data-key="a">item-a</li>"#,
///         r#"<li data-key="b">item-b</li>"#,
///         "</ul>",
///     ),
/// );
/// ```
pub fn keyed_list(
    tag: &'static str,
    attrs: Vec<(&str, &str)>,
    field: &'static str,
    items: Vec<(String, Node)>,
) -> Result<Node, KeyedListError> {
    // (1) 親属性への予約属性の混入を拒否する。呼び出し側が data-bind-list を
    // 直接指定できると、実際のリスト構造と食い違う値で #343/#345 のキー照合
    // 契約を偽装できてしまうため fail-closed で遮断する。
    reject_reserved_attr(&attrs, BIND_LIST_ATTR)?;
    reject_reserved_attr(&attrs, KEY_ATTR)?;

    // (1.5) 項目数・キー総バイト数の上限チェック（HashDoS 対策の追加防御、
    // イシュー #1375 codex-review P1 是正。cfg 分岐を持たず全ターゲット
    // 共通で適用する理由は [`MAX_KEYED_LIST_ITEMS`] doc 参照）。以降の
    // HashMap 構築・走査（(2)(3)）が攻撃者の事前に選んだキー列に対しても
    // 最悪 `O(n^2)` に収まる規模であることを、この時点の fail-closed
    // 拒否で保証する。項目数チェックを先に行うことで、項目数自体が
    // 過大な場合は `first_index_of` の `with_capacity` 割り当てより前に
    // 拒否できる。[`enforce_key_limits`] は [`diff_keys`]/[`diff_keyed_items`]
    // とも共有する（PR #1390 レビュー是正、下記 doc 参照）。
    let total_key_bytes: usize = items.iter().map(|(key, _)| key.len()).sum();
    enforce_key_limits(items.len(), total_key_bytes)?;

    // (2)(3) 各子要素の検証: Element であること・キー非空・キー一意性・予約
    // 属性の非混入。パス 1（本ループ）は `items.iter()` による**参照のみ**の
    // 走査であり、`items` を消費しない。検証順序（空キー → 重複キー →
    // 非 Element → 予約属性）・エラーの index/first_index は旧実装（1 パス
    // 構成）と完全に同一（回帰テスト `duplicate_error_precedence_is_stable`
    // が固定）。直下スコープのみを対象に、初出インデックスを HashMap で記録
    // して O(n) で重複判定する（非再帰、DoS 耐性）。
    // ハッシャは `crate::fx_hash`（ネイティブ: `RandomState`/SipHash、
    // wasm32: 軽量 FxHasher とターゲット別に切り替え）を使う。選定根拠・
    // 脅威モデルは `fx_hash` モジュール doc「ターゲット別ハッシャ選択」節
    // 参照（イシュー #1375）。
    let mut first_index_of: crate::fx_hash::FxHashMap<&str, usize> =
        crate::fx_hash::map_with_capacity(items.len());

    for (index, (key, item)) in items.iter().enumerate() {
        if key.is_empty() {
            return Err(KeyedListError::EmptyKey { index });
        }
        if let Some(&first_index) = first_index_of.get(key.as_str()) {
            return Err(KeyedListError::DuplicateKey {
                first_index,
                duplicate_index: index,
            });
        }
        first_index_of.insert(key.as_str(), index);

        let Node::Element {
            attrs: item_attrs, ..
        } = item
        else {
            return Err(KeyedListError::NonElementItem { index });
        };

        // 子要素の既存属性にも同じ予約属性チェックをかける（親と同じ理由）。
        // `reject_reserved_attr_owned` は `Vec<(&str, &str)>` への中間 collect
        // を経由せず `&[(String, String)]` を直接走査する（項目数ぶんの Vec
        // アロケーションを避ける、イシュー #1326）。
        reject_reserved_attr_owned(item_attrs, KEY_ATTR)?;
        reject_reserved_attr_owned(item_attrs, BIND_LIST_ATTR)?;
    }

    // (2')(3') パス 2（構築）: 検証をすべて通過した後にのみ `items` を
    // `into_iter()` でムーブ消費し、各項目の属性 Vec・子ノード木を**一切
    // clone しない**（パス 1 時点では clone していた deep clone を全廃、
    // イシュー #1326）。パス 1 で全項目の妥当性を確認済みのため、ここでの
    // パターンマッチが `NonElementItem` に落ちることはない
    // （`unreachable!` は使わず、`Node::Text`/`Node::RawHtml` は
    // 空 children の要素として扱う縮退経路にせず、パス 1 と同じ判定を
    // 再度行い prod ビルドでも安全側に倒す。ただし到達しない前提であり
    // 二重コストは軽微）。
    let mut children = Vec::with_capacity(items.len());
    for (index, (key, item)) in items.into_iter().enumerate() {
        let Node::Element {
            tag: item_tag,
            attrs: mut item_attrs,
            children: item_children,
        } = item
        else {
            // パス 1 で検証済みのため通常到達しないが、`items` の消費順序が
            // 変わっても fail-closed であり続けるよう安全側の分岐を残す
            // （ライブラリコードでの panic 回避規約）。
            return Err(KeyedListError::NonElementItem { index });
        };

        item_attrs.push((KEY_ATTR.to_string(), key));
        children.push(Node::Element {
            tag: item_tag,
            attrs: item_attrs,
            children: item_children,
        });
    }

    // (4) 親 Node::Element を組み立てる。data-bind-list は呼び出し側 attrs の
    // 後ろへ決定的順序で付加する（出力バイトの決定性・SSR/SSG 一致の土台）。
    let mut parent_attrs: Vec<(String, String)> = attrs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    parent_attrs.push((BIND_LIST_ATTR.to_string(), field.to_string()));

    Ok(Node::Element {
        tag,
        attrs: parent_attrs,
        children,
    })
}

/// 呼び出し側属性列に予約マーカー属性が含まれていないか検査する。
///
/// HTML の属性名は大文字小文字を区別しないため、比較は
/// `eq_ignore_ascii_case` で行う。単純な完全一致（大文字小文字を区別する
/// 比較）だと `DATA-BIND-LIST` / `Data-Key` のような表記ゆれでこの検査を
/// 迂回でき、その後 `keyed_list` が正規のマーカー属性を追加することで
/// 重複・競合するリストマーカーが生成されてしまう。これは
/// `docs/design/dom-binding-update-design.md` §5.2 が要求する
/// fail-closed のなりすまし防止保証を破るため、大文字小文字を区別しない
/// 比較で確実に遮断する。
fn reject_reserved_attr(
    attrs: &[(&str, &str)],
    reserved: &'static str,
) -> Result<(), KeyedListError> {
    if attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(reserved)) {
        return Err(KeyedListError::ReservedAttr { attr: reserved });
    }
    Ok(())
}

/// [`reject_reserved_attr`] の所有属性列版。
///
/// `Vec<(String, String)>` を `Vec<(&str, &str)>` へ中間 collect すること
/// なく `&[(String, String)]` を直接走査する（`keyed_list` の各項目検証で
/// 項目数ぶんの Vec アロケーションを避けるため、イシュー #1326）。判定
/// ロジック（大文字小文字を区別しない比較・fail-closed）は
/// [`reject_reserved_attr`] と完全に同一。
fn reject_reserved_attr_owned(
    attrs: &[(String, String)],
    reserved: &'static str,
) -> Result<(), KeyedListError> {
    if attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(reserved)) {
        return Err(KeyedListError::ReservedAttr { attr: reserved });
    }
    Ok(())
}

/// keyed list へ適用する 1 操作（設計書 §3.4）。
///
/// [`fandhe_frontend_wasm_client`](https://docs.rs/fandhe-frontend-wasm-client)
/// の `keyed_diff::KeyedOp`（イシュー #345 が導入した旧実装）と同型の
/// `Remove`/`Insert`/`Move` に加え、本イシュー（#1323）で `Update` を
/// 追加した 4 variant 構成。`index` は操作適用後の「新しい並び」における
/// 位置を指す（挿入・移動先の決定的な位置決め、DOM 適用層が
/// `insert_before` の参照ノードを求める際の入力）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedOp {
    /// 当該キーの既存ノードを削除する。
    Remove {
        /// 削除対象のキー。
        key: String,
    },
    /// 当該キーに対応する新規ノードを `index` の位置へ挿入する
    /// （旧キー列に存在しなかったキー）。
    Insert {
        /// 挿入先の位置（新しい並びでのインデックス）。
        index: usize,
        /// 挿入するキー。
        key: String,
    },
    /// 当該キーに対応する既存ノードを `index` の位置へ移動する（旧キー列に
    /// 存在したが位置が変わったキー）。既存 DOM ノードを再生成せず参照を
    /// 保持したまま移動することがフォーカス・入力途中の値の保持に直結する
    /// ため、`Remove` + `Insert` へ分解しない専用の操作として区別する。
    Move {
        /// 移動先の位置（新しい並びでのインデックス）。
        index: usize,
        /// 移動するキー。
        key: String,
    },
    /// 当該キーの既存ノードの内容（属性・子要素）だけが変わったことを表す
    /// （設計書 §3.1・§4.2、イシュー #1323 で新設）。
    ///
    /// フィールドは `key` のみで、変更後の `Node` そのものは運ばない。
    /// [`diff_keyed_items`] の呼び出し側（DOM 適用側、#1324 の
    /// `apply_keyed_list_with_previous` が想定消費者）は自身の入力である
    /// 新旧 `(String, Node)` 列を必ず保持しているため、`key` から新旧
    /// `Node` を O(n) の一括 map 構築で解決できる契約とする（`Node` 複製の
    /// 埋め込みは冗長データであり避ける）。`Update` は「保持キー
    /// （新旧両方に存在するキー）のうち `Node` の内容が
    /// `PartialEq` で不一致だったもの」にのみ発行され、[`diff_keyed_items`]
    /// の `new_items` 順で並ぶ（設計書 §3.4）。
    Update {
        /// 内容が変化したキー。
        key: String,
    },
}

/// 「現在のキー列」から「新しいキー列」へ変換する最小の操作列を計画する。
///
/// アルゴリズム（O(n)、汎用 diff・仮想 DOM ではなく keyed list 専用の単純な
/// 2 パス方式、設計書 §5・§7 が確定する「唯一の経路」の実装）:
///
/// 1. `old_keys` を先頭から走査し、`new_keys` に存在しないキーは
///    [`KeyedOp::Remove`] として記録し、作業列（`working`）から除外する。
/// 2. `new_keys` を先頭から走査し、`working[i]` が期待するキーと一致しなけ
///    れば、`working` の残り（`i` 以降）から探して見つかれば
///    [`KeyedOp::Move`]、見つからなければ [`KeyedOp::Insert`] とする。
///    いずれの場合も `working` を新しい並びに合わせて更新してから次へ進む。
///
/// キー重複がある場合（本来は [`keyed_list`] が構築時点で拒否するため到達
/// しない想定だが、DOM 改ざん等で `old_keys`/`new_keys` 側に重複が混入した
/// 場合の防御）は、新旧両側とも「最初の 1 件のみを対象とし、無限ループ・
/// panic を起こさない」fail-closed を保証する。ただし新旧で「最初の 1 件」
/// が指す実体は非対称である（イシュー #1336 codex レビュー P1 是正、
/// 詳細は [`remove_pass`] 参照）: **旧側は最後の出現を保持し、それより前の
/// 出現をすべて [`KeyedOp::Remove`] として発行する**。[`KeyedOp::Remove`]
/// は `key` のみを運び、DOM 適用側は `querySelector` 相当の「現時点で最初
/// に一致するノード」を除去する契約であるため、除去対象を古い出現順に
/// 発行することで、除去が進むたびに「まだ残っている中の最初の一致」が
/// 順に消え、最終的に最後の出現だけが残る（先に最初の出現を残す設計だと、
/// 位置ベースの除去が常に「残っている最初の一致」＝保持すべきノードその
/// ものを削除してしまい、保持したかった方が消え重複が残ってしまう）。
/// 一方 **新側は最初の出現を保持し、2 件目以降を無条件にスキップ**する
/// （op を一切発行しない）。新側の重複は挿入によって新規ノードを作る
/// だけで既存ノードの物理的な同一性が問題にならないため、位置（`index`）
/// さえ正規化されていれば最初の出現のみを処理すれば十分である。
/// [`insert_or_move_pass`] はこの正規化（重複でスキップした要素だけ
/// `index` のカウントアップをスキップする）も担う。
///
/// `fandhe-frontend-wasm-client`（イシュー #345）が導入した実装をそのまま
/// 移管したもの（イシュー #1323、`docs/design/keyed-update-op-design.md`
/// §4.1）。挙動・op 発行順は移管元と完全に同一（回帰テストで固定）。
/// wasm-client 側の同名関数は #1324 で本関数への re-export へ置換予定で
/// あり、それまでの一時的な実装重複は意図的。
///
/// # HashDoS 対策の追加防御（PR #1390 レビュー是正、イシュー #1375）
///
/// 本関数は [`keyed_list`] を経由しない生の `&[String]` を直接受け取れる
/// 公開 API のため、`keyed_list` 側の項目数・キー総バイト数の上限
/// （[`MAX_KEYED_LIST_ITEMS`]/[`MAX_KEYED_LIST_KEY_BYTES`]）を経由せずに
/// 攻撃者が選んだキー列を直接投入できてしまう。内部で `HashMap`/
/// `HashSet` を構築する [`remove_pass`]/[`insert_or_move_pass`] を呼ぶ前に、
/// `old_keys`/`new_keys` それぞれへ同じ上限ゲート（[`enforce_key_limits`]）
/// を適用し、超過時は `HashMap`/`HashSet` を一切構築せずに `Err` を返す。
///
/// # Errors
///
/// - [`KeyedListError::TooManyItems`][]: `old_keys`/`new_keys` のいずれかの
///   要素数が [`MAX_KEYED_LIST_ITEMS`] を超えている。
/// - [`KeyedListError::KeyBytesExceeded`][]: `old_keys`/`new_keys`
///   いずれかのキー文字列の合計バイト数が [`MAX_KEYED_LIST_KEY_BYTES`]
///   を超えている。
pub fn diff_keys(old_keys: &[String], new_keys: &[String]) -> Result<Vec<KeyedOp>, KeyedListError> {
    enforce_key_limits(old_keys.len(), old_keys.iter().map(String::len).sum())?;
    enforce_key_limits(new_keys.len(), new_keys.iter().map(String::len).sum())?;

    let mut ops = Vec::new();
    let working = remove_pass(old_keys, new_keys, &mut ops);
    insert_or_move_pass(working, new_keys, &mut ops);
    Ok(ops)
}

/// [`diff_keys`] 第 1 パス: `new_keys` に存在しないキーを [`KeyedOp::Remove`]
/// として記録し、除外した残り（保持キーのみ）を `working` として返す。
/// [`diff_keyed_items`] も同一のパスを共有する（挙動不変の内部ヘルパー化、
/// 設計書 §4.2 の実装確定事項）。
///
/// `old_keys` 側にキー重複が混入している場合（[`keyed_list`] が構築時点で
/// 拒否するため通常到達しないが、DOM 改ざん等の防御として想定）は、
/// **最後の出現のみ**を `working` へ残し、それより前の出現はすべて
/// （`new_keys` に存在するかに関わらず）無条件に [`KeyedOp::Remove`] として
/// 発行する（イシュー #1336 codex レビュー P1 是正）。
///
/// 「最初の 1 件を残す」ではなく「最後の 1 件を残す」を選ぶ理由:
/// [`KeyedOp::Remove`] は `key` のみを運び、DOM 適用側（テスト用シミュレー
/// タは [`tests::apply_ops`]）は「現時点で `key` に最初に一致するノード」
/// を除去する契約になっている（実 DOM の `querySelector` 相当）。重複が
/// ある間はどの物理ノードも同じキーで一致してしまうため、除去対象を
/// **古い出現順**（先頭寄り）に発行すれば、1 件除去するたびに「残っている
/// 中で最初の一致」が入れ替わりながら順に消えていき、最終的に最後の出現
/// だけが手つかずのまま残る。逆に「最初の出現を残す」設計を採ると、
/// 最初に発行される Remove が常に「残っている中の最初の一致」＝保持
/// したかったノードそのものを削除してしまい、削除したかった重複の方が
/// 残ってしまう（`old=["a","b","a"], new=["a","b"]` で実際に発生し、
/// 回帰テスト `diff_keys_keeps_last_old_occurrence_of_duplicate_key` が
/// この事例を固定する）。
///
/// これにより `working` は常に「重複を含まない」列になり、後続の
/// [`insert_or_move_pass`]（`new_keys` の要素数ぶんしか走査しない）が
/// 余剰ノードを取りこぼさず、適用後の DOM に旧キーの重複ノードが残る
/// ことを防ぐ。[`diff_keyed_items`] の Update 判定（`old_by_key`）も、
/// ここで保持されるのが最後の出現であることに合わせて上書き挿入で
/// 構築する（最初の出現の内容と比較すると、保持されないノードの内容と
/// 誤って比較してしまう）。
fn remove_pass(old_keys: &[String], new_keys: &[String], ops: &mut Vec<KeyedOp>) -> Vec<String> {
    // ハッシャは軽量ハッシャ（`crate::fx_hash`）。選定根拠・脅威モデルは
    // `fx_hash` モジュール doc 参照（イシュー #1375）。
    let new_set: crate::fx_hash::FxHashSet<&str> = new_keys.iter().map(String::as_str).collect();

    // 各キーの「最後の出現インデックス」を先に求める。ループ中に
    // 「これ以降その key は二度と現れない」かを判定する必要があるため、
    // 逆順走査ではなく前方 1 パスで `key -> 最後に出現した index` を
    // 構築してから本走査に使う。
    let mut last_index_of: crate::fx_hash::FxHashMap<&str, usize> =
        crate::fx_hash::map_with_capacity(old_keys.len());
    for (index, key) in old_keys.iter().enumerate() {
        last_index_of.insert(key.as_str(), index);
    }

    let mut working: Vec<String> = Vec::with_capacity(old_keys.len());
    for (index, key) in old_keys.iter().enumerate() {
        let is_last_occurrence = last_index_of.get(key.as_str()) == Some(&index);
        if new_set.contains(key.as_str()) && is_last_occurrence {
            working.push(key.clone());
        } else {
            ops.push(KeyedOp::Remove { key: key.clone() });
        }
    }
    working
}

/// [`diff_keys`] 第 2 パス: `working`（保持キーのみ、旧順序）を `new_keys`
/// の並びへ揃えながら [`KeyedOp::Move`] / [`KeyedOp::Insert`] を記録する。
///
/// # 計算量（O(n)、イシュー #1335 codex レビュー P1 是正）
///
/// 旧実装は `working[index..].iter().position(...)` による線形探索と
/// `Vec::remove` + `Vec::insert` による O(n) シフトを `new_keys` の要素数
/// ぶん繰り返すため最悪 O(n²) だった（完全逆順・大規模シャッフルでほぼ
/// n 回発生し、設計書 §3.5・本モジュール doc 冒頭が定める O(n) 契約に
/// 違反していた）。本実装は `working` を配列添字ベースの双方向連結リスト
/// （`next`/`prev`）として表現し、各キーの「未消費ノード」を出現順に保持
/// する `queue`（`HashMap<&str, VecDeque<usize>>`）を併用することで、
/// 「現在の探索位置以降で最初に見つかる一致」という旧実装と同一の探索
/// 意味論を次の 2 操作のみで実現する:
///
/// - 連結リストからのノード取り外し（`splice`）: 前後ノードの `next`/`prev`
///   参照を張り替えるだけの O(1) 操作（`Vec::remove` の O(n) シフトが
///   不要）。
/// - 該当キーの「最も早い未消費ノード」の特定: `queue` の `pop_front` で
///   O(1) 償却（`working` の元の並び順で `queue` を構築しており、消費順に
///   `pop_front` することで `position()` の「探索位置以降で最初に見つかる
///   一致」と同じノードを常に返す。証明: 未消費ノードの相対順序は
///   `working` の元の並びから一切変化しない ── `Insert` 分岐は新規キーを
///   追加するのみで既存ノードの相対順序に影響せず、`Move` 分岐で取り外す
///   ノードは以後二度と参照されないため、残る未消費ノード同士の順序は
///   常に元の並びのまま保たれる）。
///
/// この結果、全体で O(旧キー数 + 新キー数) の時間・空間で完結し、`Move`/
/// `Insert` の発行順序・`index` 値・`key` 値は旧実装と完全に同一（回帰
/// テストで固定）。取り外したノードを旧実装のように `index` 位置へ
/// 再挿入しない点が実装上の差分だが、再挿入された要素は以後の反復で
/// 二度と参照されない（`new_keys` の走査は単調増加のインデックスのみを
/// 見る）ため、単純に連結リストから完全に取り除いても出力に影響しない。
fn insert_or_move_pass(working: Vec<String>, new_keys: &[String], ops: &mut Vec<KeyedOp>) {
    let n = working.len();

    // `new_keys` 側にキー重複が混入している場合（[`keyed_list`] が構築時点
    // で拒否するため通常到達しないが、`remove_pass` と対称の防御として
    // 想定）は、`seen_new` で最初の出現のみを処理対象とし、2 件目以降は
    // 無条件にスキップする（op を一切発行しない）。重複分にも Insert/Move
    // を発行すると、DOM 上に同一キーのノードが 2 つ生成されてしまい
    // 「最初の 1 件のみを対象とする」fail-closed 契約に反するため
    // （イシュー #1336 codex レビュー P1 是正。旧実装は `new_keys` の重複を
    // 検出せず全要素を走査していたため、2 件目以降が保持キューの枯渇後に
    // 誤って Insert として発行されていた）。
    // ハッシャは軽量ハッシャ（`crate::fx_hash`）。選定根拠・脅威モデルは
    // `fx_hash` モジュール doc 参照（イシュー #1375）。
    let mut seen_new: crate::fx_hash::FxHashSet<&str> =
        crate::fx_hash::set_with_capacity(new_keys.len());

    // `index`（`new_keys.iter().enumerate()` の生インデックス）ではなく、
    // 重複でスキップした要素を除いた「出力後の並びでの位置」を
    // Insert/Move の `index` に使う（イシュー #1336 codex レビュー P1
    // 是正）。`new_keys` に重複が混入していると、2 件目以降は上の
    // `seen_new` チェックで op を発行せずスキップするが、生インデックスを
    // そのまま使うと以降の Insert/Move の `index` に重複分の「隙間」が
    // 残ってしまい、適用結果が `new_keys` の重複除去後の並び（保持契約
    // 上の正しい結果）からずれる（例: `old=["b"], new=["a","a","c","b"]`
    // で生インデックスを使うと `c` が `index: 2` として発行され、適用結果
    // が `["a","b","c"]` になってしまうが、正しくは `["a","c","b"]`）。
    // `out_index` は「新側で実際に処理対象となった要素」（マッチ消費・
    // Move・Insert のいずれか、重複スキップを除く）ごとに 1 ずつ進める。
    let mut out_index: usize = 0;

    if n == 0 {
        for key in new_keys.iter() {
            if !seen_new.insert(key.as_str()) {
                continue;
            }
            ops.push(KeyedOp::Insert {
                index: out_index,
                key: key.clone(),
            });
            out_index += 1;
        }
        return;
    }

    // working を双方向連結リストとして表現する（ノード index = working
    // 内の添字）。`next[i]`/`prev[i]` はそれぞれ次・前ノードの添字。
    let mut next: Vec<Option<usize>> = (0..n)
        .map(|i| if i + 1 < n { Some(i + 1) } else { None })
        .collect();
    let mut prev: Vec<Option<usize>> = (0..n)
        .map(|i| if i == 0 { None } else { Some(i - 1) })
        .collect();
    let mut head: Option<usize> = Some(0);

    // キーごとに未消費ノードの添字を出現順（昇順）で保持するキュー。
    // ハッシャは軽量ハッシャ（`crate::fx_hash`）。選定根拠・脅威モデルは
    // `fx_hash` モジュール doc 参照（イシュー #1375）。
    let mut queue: crate::fx_hash::FxHashMap<&str, std::collections::VecDeque<usize>> =
        crate::fx_hash::map_with_capacity(n);
    for (i, key) in working.iter().enumerate() {
        queue.entry(key.as_str()).or_default().push_back(i);
    }

    for key in new_keys.iter() {
        if !seen_new.insert(key.as_str()) {
            // 2 件目以降の重複キーは無条件にスキップする（op を発行せず、
            // `out_index` も進めない。上記コメント参照）。
            continue;
        }

        if let Some(h) = head {
            if working[h] == *key {
                // 先頭ノードが期待キーと一致: 操作を発行せず消費するのみ。
                if let Some(q) = queue.get_mut(key.as_str()) {
                    q.pop_front();
                }
                head = next[h];
                out_index += 1;
                continue;
            }
        }

        let found = queue.get_mut(key.as_str()).and_then(|q| q.pop_front());
        if let Some(node) = found {
            // 連結リストから O(1) で取り外す（旧実装の Vec::remove +
            // Vec::insert による O(n) シフトを回避）。
            let node_prev = prev[node];
            let node_next = next[node];
            if let Some(p) = node_prev {
                next[p] = node_next;
            }
            if let Some(nx) = node_next {
                prev[nx] = node_prev;
            }
            if head == Some(node) {
                head = node_next;
            }
            ops.push(KeyedOp::Move {
                index: out_index,
                key: key.clone(),
            });
        } else {
            ops.push(KeyedOp::Insert {
                index: out_index,
                key: key.clone(),
            });
        }
        out_index += 1;
    }
}

/// 内容比較付き keyed list diff（イシュー #1323、設計書 §3.1・§3.4・§4.2）。
///
/// `old_items`/`new_items` は `(キー, Node)` のペア列（[`keyed_list`] が
/// 消費する形と同じ）。動作は次の 3 パス:
///
/// 1. [`diff_keys`] と完全同一の Remove 発行（[`remove_pass`] を共有）。
/// 2. [`diff_keys`] と完全同一の Insert/Move 発行（[`insert_or_move_pass`]
///    を共有）。
/// 3. 新旧両方に存在する保持キー**すべて**（Move の有無に関わらず）につい
///    て、新旧 `Node` を [`PartialEq`] で比較し、不一致のときのみ
///    [`KeyedOp::Update`] を `new_items` の順序で発行する。一致するキーは
///    op を一切発行しない。
///
/// 重複キー混入時は [`diff_keys`] と同じ fail-closed（最初の 1 件のみ対象、
/// panic しない）。計算量は O(キー数) + O(Σ 保持キーの部分木サイズ)
/// （設計書 §3.5。仮想 DOM 型の再帰 diff ではなく、保持キー 1 件あたり
/// `Node::eq` 1 回のみ）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, text, keyed::{diff_keyed_items, KeyedOp}};
///
/// let old_items = vec![
///     ("a".to_string(), el("li", vec![], vec![text("a")])),
///     ("b".to_string(), el("li", vec![], vec![text("b-old")])),
/// ];
/// let new_items = vec![
///     ("b".to_string(), el("li", vec![], vec![text("b-new")])),
///     ("a".to_string(), el("li", vec![], vec![text("a")])),
/// ];
///
/// let ops = diff_keyed_items(&old_items, &new_items).expect("within limits");
/// assert_eq!(
///     ops,
///     vec![
///         KeyedOp::Move { index: 0, key: "b".to_string() },
///         KeyedOp::Update { key: "b".to_string() },
///     ],
/// );
/// ```
///
/// # HashDoS 対策の追加防御（PR #1390 レビュー是正、イシュー #1375）
///
/// [`diff_keys`] と同じ理由（doc 参照）で、`old_items`/`new_items` それぞれ
/// へ [`enforce_key_limits`] を適用してから `HashMap`/`HashSet` を構築する。
///
/// # Errors
///
/// - [`KeyedListError::TooManyItems`][]: `old_items`/`new_items` のいずれか
///   の要素数が [`MAX_KEYED_LIST_ITEMS`] を超えている。
/// - [`KeyedListError::KeyBytesExceeded`][]: `old_items`/`new_items`
///   いずれかのキー文字列の合計バイト数が [`MAX_KEYED_LIST_KEY_BYTES`]
///   を超えている。
pub fn diff_keyed_items(
    old_items: &[(String, Node)],
    new_items: &[(String, Node)],
) -> Result<Vec<KeyedOp>, KeyedListError> {
    enforce_key_limits(
        old_items.len(),
        old_items.iter().map(|(k, _)| k.len()).sum(),
    )?;
    enforce_key_limits(
        new_items.len(),
        new_items.iter().map(|(k, _)| k.len()).sum(),
    )?;

    let old_keys: Vec<String> = old_items.iter().map(|(k, _)| k.clone()).collect();
    let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();

    let mut ops = Vec::new();
    let working = remove_pass(&old_keys, &new_keys, &mut ops);
    insert_or_move_pass(working, &new_keys, &mut ops);

    // 第 3 パス: 保持キー（重複混入時も `remove_pass` と同じ fail-closed
    // 防御を適用）を new_items 順に走査し、新旧 Node が不一致のときのみ
    // Update を発行する。
    // `old_by_key` は**最後に出現したキー**で上書きしながら記録する
    // （`entry().or_insert()` ではなく `insert()`）。`remove_pass` が
    // `old_items` 側の重複キーのうち保持する（`working` に残す）のは
    // 最後の出現であるため（イシュー #1336 codex レビュー P1 是正、理由は
    // `remove_pass` rustdoc 参照）、Update 判定の比較対象も同じ「最後の
    // 出現」の内容でなければならない。最初の出現の内容と比較すると、
    // 実際には保持されない（Remove される）ノードの内容と誤って比較して
    // しまい、Update の要否判定を誤る。
    // ハッシャは軽量ハッシャ（`crate::fx_hash`）。選定根拠・脅威モデルは
    // `fx_hash` モジュール doc 参照（イシュー #1375）。
    let mut old_by_key: crate::fx_hash::FxHashMap<&str, &Node> =
        crate::fx_hash::map_with_capacity(old_items.len());
    for (key, node) in old_items {
        old_by_key.insert(key.as_str(), node);
    }

    let mut seen_new_keys: crate::fx_hash::FxHashSet<&str> =
        crate::fx_hash::set_with_capacity(new_items.len());
    for (key, new_node) in new_items {
        // new_items 側の重複キーも最初の 1 件のみを対象とする（同一防御を
        // 対称に適用する）。
        if !seen_new_keys.insert(key.as_str()) {
            continue;
        }
        if let Some(old_node) = old_by_key.get(key.as_str()) {
            if *old_node != new_node {
                ops.push(KeyedOp::Update { key: key.clone() });
            }
        }
    }

    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{el, li, render, text, ul};

    /// 正常系: SSR 出力のバイトを固定する。属性の付加順序
    /// （呼び出し側 attrs の後ろに data-bind-list / data-key）を保証する。
    #[test]
    fn keyed_list_renders_expected_byte_output() {
        let list = keyed_list(
            "ul",
            vec![("data-testid", "item-list")],
            "items",
            vec![
                ("a".to_string(), el("li", vec![], vec![text("item-a")])),
                ("b".to_string(), el("li", vec![], vec![text("item-b")])),
            ],
        )
        .expect("valid keyed list");

        assert_eq!(
            render(&list),
            concat!(
                r#"<ul data-testid="item-list" data-bind-list="items">"#,
                r#"<li data-key="a">item-a</li>"#,
                r#"<li data-key="b">item-b</li>"#,
                "</ul>",
            ),
        );
    }

    /// 決定性: 同一入力を 2 回構築しても同一出力になる（SSR/SSG 出力一致の
    /// 保証・新レンダリング経路を追加しないことの回帰確認）。
    #[test]
    fn keyed_list_output_is_deterministic() {
        let build = || {
            keyed_list(
                "ul",
                vec![],
                "items",
                vec![
                    ("x".to_string(), el("li", vec![], vec![text("x")])),
                    ("y".to_string(), el("li", vec![], vec![text("y")])),
                ],
            )
            .expect("valid keyed list")
        };
        assert_eq!(render(&build()), render(&build()));
    }

    /// 空 items は正常系（空リスト状態）。親要素のみが出力される。
    #[test]
    fn keyed_list_with_empty_items_renders_parent_only() {
        let list = keyed_list("ul", vec![], "items", vec![]).expect("valid keyed list");
        assert_eq!(render(&list), r#"<ul data-bind-list="items"></ul>"#);
    }

    /// ネスト: 子孫に別の keyed_list を含んでも、一意性検査は直下スコープの
    /// みが対象であるため正常に構築できる（設計書 §3.1 直下子スコープ規約）。
    #[test]
    fn nested_keyed_list_is_allowed() {
        let inner = keyed_list(
            "ul",
            vec![],
            "children",
            vec![("c1".to_string(), el("li", vec![], vec![text("c1")]))],
        )
        .expect("valid inner keyed list");

        let outer = keyed_list(
            "div",
            vec![],
            "groups",
            vec![("g1".to_string(), el("li", vec![], vec![inner]))],
        )
        .expect("valid outer keyed list");

        let html = render(&outer);
        assert!(html.contains(r#"data-bind-list="groups""#));
        assert!(html.contains(r#"data-bind-list="children""#));
        assert!(html.contains(r#"data-key="g1""#));
        assert!(html.contains(r#"data-key="c1""#));
    }

    /// 異常系（HashDoS 追加防御、イシュー #1375）: 項目数が
    /// [`MAX_KEYED_LIST_ITEMS`] を超えると TooManyItems で拒否される。
    /// 全ターゲット共通のガード（`cfg` 分岐なし）であるため、ネイティブの
    /// `cargo test`（本テスト）でも実際に検証できる。
    #[test]
    fn too_many_items_is_rejected() {
        let items: Vec<(String, Node)> = (0..=MAX_KEYED_LIST_ITEMS)
            .map(|i| (format!("k{i}"), el("li", vec![], vec![])))
            .collect();
        let count = items.len();
        let err = keyed_list("ul", vec![], "items", items).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 正常系（境界値）: 項目数がちょうど [`MAX_KEYED_LIST_ITEMS`] 件なら
    /// 拒否されない。
    #[test]
    fn item_count_at_the_limit_is_accepted() {
        let items: Vec<(String, Node)> = (0..MAX_KEYED_LIST_ITEMS)
            .map(|i| (format!("k{i}"), el("li", vec![], vec![])))
            .collect();
        assert!(keyed_list("ul", vec![], "items", items).is_ok());
    }

    /// 異常系（HashDoS 追加防御、イシュー #1375）: 項目数は少なくても
    /// キー文字列の合計バイト数が [`MAX_KEYED_LIST_KEY_BYTES`] を超えると
    /// KeyBytesExceeded で拒否される（項目数の上限だけでは「少数の巨大な
    /// キー」による計算量膨張を防げないことの回帰確認）。
    #[test]
    fn key_bytes_exceeded_is_rejected() {
        let huge_key = "k".repeat(MAX_KEYED_LIST_KEY_BYTES + 1);
        let total_bytes = huge_key.len();
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![(huge_key, el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::KeyBytesExceeded { total_bytes });
    }

    /// 回帰: サイズ上限（項目数・キー総バイト数）は per-item 検証
    /// （空キー・重複キー・非 Element・予約属性）より先に評価される。
    /// 項目数超過かつ先頭項目が空キーというケースで TooManyItems が
    /// 優先されることを固定する（`duplicate_error_precedence_is_stable_
    /// across_mixed_violations` と対をなす追加の優先順位固定）。
    #[test]
    fn too_many_items_takes_precedence_over_empty_key() {
        let mut items: Vec<(String, Node)> = vec![(String::new(), el("li", vec![], vec![]))];
        items
            .extend((0..MAX_KEYED_LIST_ITEMS).map(|i| (format!("k{i}"), el("li", vec![], vec![]))));
        let count = items.len();
        let err = keyed_list("ul", vec![], "items", items).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 回帰: キー総バイト数超過も per-item 検証より先に評価される
    /// （項目数自体は上限以内でも、先頭項目が空キーであるより先に
    /// KeyBytesExceeded が返ることを固定する）。
    #[test]
    fn key_bytes_exceeded_takes_precedence_over_empty_key() {
        let huge_key = "k".repeat(MAX_KEYED_LIST_KEY_BYTES + 1);
        let total_bytes = huge_key.len();
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                (huge_key, el("li", vec![], vec![])),
                (String::new(), el("li", vec![], vec![])),
            ],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::KeyBytesExceeded { total_bytes });
    }

    /// 異常系: 空文字列キーは EmptyKey で拒否される。
    #[test]
    fn empty_key_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![(String::new(), el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::EmptyKey { index: 0 });
    }

    /// 異常系: 同一リスト内のキー重複は DuplicateKey で拒否される。
    #[test]
    fn duplicate_key_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                ("a".to_string(), el("li", vec![], vec![])),
                ("b".to_string(), el("li", vec![], vec![])),
                ("a".to_string(), el("li", vec![], vec![])),
            ],
        )
        .unwrap_err();
        assert_eq!(
            err,
            KeyedListError::DuplicateKey {
                first_index: 0,
                duplicate_index: 2,
            }
        );
    }

    /// 異常系: 子が Element でない（Text）場合は NonElementItem で拒否される。
    #[test]
    fn non_element_item_is_rejected() {
        let err =
            keyed_list("ul", vec![], "items", vec![("a".to_string(), text("bare"))]).unwrap_err();
        assert_eq!(err, KeyedListError::NonElementItem { index: 0 });
    }

    /// 異常系: 親属性に予約属性 data-bind-list を渡すと ReservedAttr で拒否
    /// される（マーカー属性の偽装防止）。
    #[test]
    fn reserved_attr_on_parent_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![(BIND_LIST_ATTR, "fake")],
            "items",
            vec![("a".to_string(), el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(
            err,
            KeyedListError::ReservedAttr {
                attr: BIND_LIST_ATTR
            }
        );
    }

    /// 異常系: 子要素の属性に予約属性 data-key を渡すと ReservedAttr で拒否
    /// される。
    #[test]
    fn reserved_attr_on_item_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![("a".to_string(), el("li", vec![(KEY_ATTR, "fake")], vec![]))],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::ReservedAttr { attr: KEY_ATTR });
    }

    /// 異常系: 親属性に予約属性を大文字小文字違いの表記（`DATA-BIND-LIST`）
    /// で渡しても ReservedAttr で拒否される。HTML の属性名は大文字小文字を
    /// 区別しないため、表記ゆれでの偽装防止バリデーション迂回を許さない
    /// （Bugbot 指摘、#344/PR #362）。
    #[test]
    fn reserved_attr_on_parent_is_rejected_case_insensitively() {
        let err = keyed_list(
            "ul",
            vec![("DATA-BIND-LIST", "fake")],
            "items",
            vec![("a".to_string(), el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(
            err,
            KeyedListError::ReservedAttr {
                attr: BIND_LIST_ATTR
            }
        );
    }

    /// 異常系: 子要素の属性に予約属性を大文字小文字違いの表記
    /// （`Data-Key`）で渡しても ReservedAttr で拒否される（同上）。
    #[test]
    fn reserved_attr_on_item_is_rejected_case_insensitively() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![(
                "a".to_string(),
                el("li", vec![("Data-Key", "fake")], vec![]),
            )],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::ReservedAttr { attr: KEY_ATTR });
    }

    /// 回帰: 複数項目に異種の違反が混在するとき、**先行項目のエラーが
    /// 優先される**こと（検証順序: 空キー → 重複キー → 非 Element → 予約
    /// 属性）を固定する。パス分離（検証 → ムーブ構築、イシュー #1326）を
    /// 経ても、1 パス構成だった旧実装と同一のエラー優先順位・index が
    /// 得られることの回帰確認。
    #[test]
    fn duplicate_error_precedence_is_stable_across_mixed_violations() {
        // index 0: 正常項目。index 1: 予約属性混入。index 2: 非 Element。
        // 予約属性チェックは非 Element 判定より後（Element マッチ後）に
        // 行われるため、非 Element 項目自体は NonElementItem を先に返す。
        // ここでは「先に出現した違反が優先される」ことを、同種違反が複数
        // 混在するケースで固定する（先行 = index 1 の予約属性違反が、
        // 後続 index 2 の非 Element 違反より先に検出される）。
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                ("a".to_string(), el("li", vec![], vec![])),
                ("b".to_string(), el("li", vec![(KEY_ATTR, "fake")], vec![])),
                ("c".to_string(), text("bare")),
            ],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::ReservedAttr { attr: KEY_ATTR });

        // 空キー・重複キーが予約属性違反・非 Element より先に来る場合も
        // 検証順序どおり空キーが優先される。
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                (String::new(), el("li", vec![(KEY_ATTR, "fake")], vec![])),
                ("dup".to_string(), text("bare")),
            ],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::EmptyKey { index: 0 });
    }

    /// PoC-5 相当デモ: `interactive` クレートの list_section（項目 + 削除
    /// ボタン、`data-action="remove_item"`/`data-payload`）と同型の構造を
    /// `keyed_list` で構築できることを固定する（受け入れ条件 3 の証跡）。
    #[test]
    fn poc5_style_dynamic_list_is_expressible_as_keyed_list() {
        let raw_items = ["牛乳を買う".to_string(), "掃除する".to_string()];
        let items: Vec<(String, Node)> = raw_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let key = i.to_string();
                (
                    key.clone(),
                    li(
                        vec![],
                        vec![
                            text(item.clone()),
                            el(
                                "button",
                                vec![
                                    ("data-action", "remove_item"),
                                    ("data-payload", &key),
                                    ("data-testid", "remove-btn"),
                                ],
                                vec![text("削除")],
                            ),
                        ],
                    ),
                )
            })
            .collect();

        let list = keyed_list("ul", vec![("data-testid", "item-list")], "items", items).unwrap();
        let html = render(&list);

        assert!(html.contains(r#"data-bind-list="items""#));
        assert!(html.contains(r#"data-key="0""#));
        assert!(html.contains(r#"data-key="1""#));
        assert!(html.contains(r#"data-action="remove_item""#));
        assert!(html.contains("牛乳を買う"));
    }

    /// 非影響回帰: `keyed_list` を使わない既存ノード構築の `render()` 出力が
    /// バイト不変であることを固定する（#342 の同旨テストと対をなす）。
    #[test]
    fn existing_node_construction_output_is_unaffected() {
        let tree = ul(
            vec![],
            vec![
                li(vec![], vec![text("item1")]),
                li(vec![], vec![text("item2")]),
            ],
        );
        assert_eq!(render(&tree), "<ul><li>item1</li><li>item2</li></ul>");
    }

    // --- diff_keys（イシュー #1323 で fandhe-frontend-wasm-client から移管。
    // テスト内容は移管元 `crates/wasm-client/src/keyed_diff.rs` と完全同一。
    // 挙動不変・op 発行順不変であることの回帰確認、受け入れ条件 2）。

    fn keys(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// 変化なし: 操作列は空。
    #[test]
    fn diff_keys_returns_empty_ops_when_unchanged() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(diff_keys(&old, &new).unwrap(), Vec::new());
    }

    /// 末尾への追加は Insert 1 件のみ。
    #[test]
    fn diff_keys_detects_append_at_tail() {
        let old = keys(&["a", "b"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(
            diff_keys(&old, &new).unwrap(),
            vec![KeyedOp::Insert {
                index: 2,
                key: "c".to_string()
            }]
        );
    }

    /// 先頭への追加は Insert 1 件のみ。
    #[test]
    fn diff_keys_detects_prepend_at_head() {
        let old = keys(&["b", "c"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(
            diff_keys(&old, &new).unwrap(),
            vec![KeyedOp::Insert {
                index: 0,
                key: "a".to_string()
            }]
        );
    }

    /// 中間削除は Remove 1 件のみ（受け入れ条件: 無関係ノードへの操作なし）。
    #[test]
    fn diff_keys_detects_middle_removal() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["a", "c"]);
        assert_eq!(
            diff_keys(&old, &new).unwrap(),
            vec![KeyedOp::Remove {
                key: "b".to_string()
            }]
        );
    }

    /// 全削除は各キーの Remove のみ。
    #[test]
    fn diff_keys_detects_removal_to_empty() {
        let old = keys(&["a", "b"]);
        let new: Vec<String> = Vec::new();
        assert_eq!(
            diff_keys(&old, &new).unwrap(),
            vec![
                KeyedOp::Remove {
                    key: "a".to_string()
                },
                KeyedOp::Remove {
                    key: "b".to_string()
                },
            ]
        );
    }

    /// 空から全追加は各キーの Insert のみ。
    #[test]
    fn diff_keys_detects_insertion_from_empty() {
        let old: Vec<String> = Vec::new();
        let new = keys(&["a", "b"]);
        assert_eq!(
            diff_keys(&old, &new).unwrap(),
            vec![
                KeyedOp::Insert {
                    index: 0,
                    key: "a".to_string()
                },
                KeyedOp::Insert {
                    index: 1,
                    key: "b".to_string()
                },
            ]
        );
    }

    /// 隣接 2 件の入れ替えは Move 1 件で表現される（既存ノード参照を保持し
    /// 再生成しないことがフォーカス保持の土台、設計書 §5.3）。
    #[test]
    fn diff_keys_detects_adjacent_swap_as_single_move() {
        let old = keys(&["a", "b"]);
        let new = keys(&["b", "a"]);
        let ops = diff_keys(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![KeyedOp::Move {
                index: 0,
                key: "b".to_string()
            }]
        );
    }

    /// 末尾要素を先頭へ移動。
    #[test]
    fn diff_keys_detects_move_from_tail_to_head() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["c", "a", "b"]);
        let ops = diff_keys(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![KeyedOp::Move {
                index: 0,
                key: "c".to_string()
            }]
        );
    }

    /// 削除・挿入・移動が同時に起きる複合ケースでも、無関係キーの操作は
    /// 生成されない。
    #[test]
    fn diff_keys_handles_mixed_remove_insert_move() {
        let old = keys(&["a", "b", "c", "d"]);
        let new = keys(&["d", "a", "e", "c"]);
        let ops = diff_keys(&old, &new).unwrap();
        // "b" は new に存在しないため削除される。
        assert!(ops.contains(&KeyedOp::Remove {
            key: "b".to_string()
        }));
        // "e" は old に存在しない新規キーのため挿入される。
        assert!(ops
            .iter()
            .any(|op| matches!(op, KeyedOp::Insert { key, .. } if key == "e")));
        // "d" は末尾から先頭へ移動する。
        assert!(ops
            .iter()
            .any(|op| matches!(op, KeyedOp::Move { key, .. } if key == "d")));
    }

    /// 重複キーが混入していても panic しない（DOM 改ざん等の異常系に対する
    /// fail-closed 防御、本モジュール doc 参照。旧側は最後の出現のみを
    /// 対象とする非対称な扱いになる理由は [`remove_pass`] の doc 参照）。
    #[test]
    fn diff_keys_does_not_panic_on_duplicate_keys_in_old() {
        let old = keys(&["a", "a", "b"]);
        let new = keys(&["a", "b"]);
        // panic しないことのみを確認する。
        let _ = diff_keys(&old, &new).unwrap();
    }

    /// `old_keys` に発行された [`KeyedOp`] 列を素朴に適用し、結果のキー列
    /// を返す（実 DOM 適用の代わりに `Vec<String>` 上でシミュレートする
    /// テスト専用ヘルパー）。`Remove`/`Move` の `key` ベースの検索は
    /// 「現時点で最初に一致する要素」を対象とする（実 DOM の
    /// `querySelector` 相当の位置ベース照合を模す。`key` だけでは重複時に
    /// 物理ノードを一意に特定できないため、`remove_pass` は除去対象を
    /// 古い出現順に発行し最後の出現だけが残るよう op 列側で辻褄を合わせて
    /// いる。詳細は [`remove_pass`] の doc 参照）。
    fn apply_ops(old_keys: &[String], ops: &[KeyedOp]) -> Vec<String> {
        let mut result: Vec<String> = old_keys.to_vec();
        for op in ops {
            match op {
                KeyedOp::Remove { key } => {
                    if let Some(pos) = result.iter().position(|k| k == key) {
                        result.remove(pos);
                    }
                }
                KeyedOp::Insert { index, key } => {
                    result.insert((*index).min(result.len()), key.clone());
                }
                KeyedOp::Move { index, key } => {
                    if let Some(pos) = result.iter().position(|k| k == key) {
                        result.remove(pos);
                    }
                    result.insert((*index).min(result.len()), key.clone());
                }
                KeyedOp::Update { .. } => {}
            }
        }
        result
    }

    /// 回帰（イシュー #1336 codex レビュー P1 是正）: `old_keys` 側に重複
    /// キーが混入していても、発行された操作列を適用した結果が
    /// `new_keys` と厳密に一致する（旧側の重複ノードが残らない）ことを
    /// 固定する。旧実装は `remove_pass` が `new_keys` の集合会員性のみで
    /// 判定していたため、`old_keys = ["a", "a", "b"]` → `new_keys =
    /// ["a", "b"]` で 2 個目の `"a"` に対する Remove が発行されず、適用後
    /// も重複ノードが残っていた。
    #[test]
    fn diff_keys_duplicate_old_key_is_fully_removed_after_applying_ops() {
        let old = keys(&["a", "a", "b"]);
        let new = keys(&["a", "b"]);

        let ops = diff_keys(&old, &new).unwrap();
        let applied = apply_ops(&old, &ops);

        assert_eq!(
            applied, new,
            "重複キー混入時も操作列適用後は new_keys と厳密に一致するはず（余剰ノードが残ってはならない）"
        );
    }

    /// 回帰: 重複キーが 3 件以上・複数キーにまたがって混入していても、
    /// 操作列適用後は `new_keys` と一致する。
    #[test]
    fn diff_keys_multiple_duplicate_old_keys_are_fully_removed_after_applying_ops() {
        let old = keys(&["a", "a", "a", "b", "b", "c"]);
        let new = keys(&["c", "a", "b"]);

        let ops = diff_keys(&old, &new).unwrap();
        let applied = apply_ops(&old, &ops);

        assert_eq!(applied, new);
    }

    /// 回帰（PR #1336 codex レビュー P1 是正）: 重複キーが `working` の
    /// 途中・末尾に挟まる（先頭以外の位置に最後の出現がある）場合でも、
    /// 「最初の出現を残す」設計だと Remove が `position()` で常に「残って
    /// いる中の最初の一致」＝保持したかったノードそのものを削除してしまい、
    /// 適用結果が `new_keys` と一致しなくなる（`key` のみが `apply_ops` の
    /// 入力である `Vec<String>` 上でも観測できる、実 DOM の node identity
    /// 問題を必要としない再現）。`old=["a","b","a"], new=["a","b"]` は
    /// 「最初の出現を残す」旧設計だと `ops=[Remove{a}]` を適用して
    /// `["a","b","a"]` → `["b","a"]` となり `new` と一致しない。最後の
    /// 出現を残す現設計では `ops=[Remove{a}, Move{index:0,key:"a"}]` を
    /// 適用して `["a","b","a"]` → (Remove) `["b","a"]` → (Move) `["a","b"]`
    /// となり一致する。
    #[test]
    fn diff_keys_keeps_last_old_occurrence_of_duplicate_key() {
        let old = keys(&["a", "b", "a"]);
        let new = keys(&["a", "b"]);

        let ops = diff_keys(&old, &new).unwrap();
        let applied = apply_ops(&old, &ops);

        assert_eq!(
            applied, new,
            "重複キーの保持対象は最後の出現でなければならない: ops={ops:?}"
        );
    }

    /// 回帰（PR #1336 codex レビュー P1 是正）: `new_keys` 側に重複キーが
    /// 混入していても、2 件目以降に対して [`KeyedOp::Insert`] を発行しない
    /// （最初の 1 件のみを対象とする fail-closed 契約）。
    /// 旧実装は `new_keys` の重複を検出せず全要素を走査していたため、
    /// `old_keys = ["a"]` → `new_keys = ["a", "a"]` で保持キューが枯渇した
    /// 2 個目の `"a"` が誤って `Insert { index: 1, key: "a" }` として発行
    /// され、操作列適用後の DOM に重複キーのノードが残っていた。
    #[test]
    fn diff_keys_duplicate_new_key_does_not_emit_insert_for_second_occurrence() {
        let old = keys(&["a"]);
        let new = keys(&["a", "a"]);

        let ops = diff_keys(&old, &new).unwrap();

        assert!(
            !ops.iter()
                .any(|op| matches!(op, KeyedOp::Insert { key, .. } if key == "a")),
            "new_keys 側の重複キーに対して Insert を発行してはならない: {ops:?}"
        );
        let applied = apply_ops(&old, &ops);
        assert_eq!(
            applied,
            vec!["a".to_string()],
            "重複キー混入時は最初の 1 件のみを対象とするため、適用後の結果は重複を含まない"
        );
    }

    /// 回帰: `working` が空（`old_keys` が空）の状態でも `new_keys` 側の
    /// 重複キーは最初の 1 件のみ [`KeyedOp::Insert`] される
    /// （`insert_or_move_pass` の `n == 0` 分岐も同一防御を共有する）。
    #[test]
    fn diff_keys_duplicate_new_key_from_empty_old_inserts_only_once() {
        let old: Vec<String> = vec![];
        let new = keys(&["a", "a", "b"]);

        let ops = diff_keys(&old, &new).unwrap();

        let insert_count_for_a = ops
            .iter()
            .filter(|op| matches!(op, KeyedOp::Insert { key, .. } if key == "a"))
            .count();
        assert_eq!(
            insert_count_for_a, 1,
            "重複キー \"a\" に対する Insert は 1 件のみのはず: {ops:?}"
        );
    }

    /// 回帰（PR #1336 codex レビュー P1 是正、`n == 0` 分岐の index 補正
    /// 漏れ）: `old_keys` が空のまま `new_keys` 側に重複キーが挟まると、
    /// スキップした重複分だけ後続 Insert の `index` に「隙間」が残って
    /// はならない。`insert_or_move_pass_from_empty_old_normalizes_index`
    /// と同じ契約を `n == 0` 分岐（`old_keys` が空）側で固定する。
    /// `apply_ops` は `index` を `result.len()` へ `.min()` でクランプする
    /// ため、この index ずれは `Insert` の個数・キーだけを見るテストでは
    /// 検知できず op の `index` フィールドを直接検証する必要がある。
    #[test]
    fn diff_keys_duplicate_new_key_from_empty_old_normalizes_index() {
        let old: Vec<String> = vec![];
        let new = keys(&["a", "a", "c", "b"]);

        let ops = diff_keys(&old, &new).unwrap();

        assert_eq!(
            ops,
            vec![
                KeyedOp::Insert {
                    index: 0,
                    key: "a".to_string(),
                },
                KeyedOp::Insert {
                    index: 1,
                    key: "c".to_string(),
                },
                KeyedOp::Insert {
                    index: 2,
                    key: "b".to_string(),
                },
            ],
            "重複でスキップした要素の分だけ後続 Insert の index を詰めるはず: {ops:?}"
        );

        let applied = apply_ops(&old, &ops);
        assert_eq!(applied, keys(&["a", "c", "b"]));
    }

    /// 回帰（PR #1336 codex レビュー P1 是正）: `new_keys` 側の重複キーを
    /// スキップした際に後続 Insert/Move の `index` がずれない（重複でない
    /// 通常経路、`n > 0` 側）ことを固定する。`old=["b"], new=["a","a","c",
    /// "b"]` は、`index` を生の走査位置のまま使う旧実装だと `c` が
    /// `index: 2` として発行され、適用結果が `["a","b","c"]` になって
    /// しまう（正しくは `["a","c","b"]`）。
    #[test]
    fn diff_keys_duplicate_new_key_normalizes_index_of_later_inserts() {
        let old = keys(&["b"]);
        let new = keys(&["a", "a", "c", "b"]);

        let ops = diff_keys(&old, &new).unwrap();
        let applied = apply_ops(&old, &ops);

        // new_keys 側の重複は最初の 1 件のみを対象とする fail-closed
        // 契約のため、適用結果は new から 2 個目の "a" を除いた重複除去後
        // の並びと一致する（"a" の 2 個目は Insert 自体が発行されない）。
        assert_eq!(
            applied,
            keys(&["a", "c", "b"]),
            "重複でスキップした要素の分だけ後続 Insert/Move の index を詰めるはず: ops={ops:?}"
        );
    }

    /// 回帰: `new_keys` 側に重複キーが 3 件以上混入していても panic せず、
    /// 最初の 1 件のみが処理対象となる。
    #[test]
    fn diff_keys_does_not_panic_on_duplicate_keys_in_new() {
        let old = keys(&["a", "b"]);
        let new = keys(&["a", "a", "a", "b"]);
        // panic しないことに加え、重複分に Insert が発行されないことも確認する。
        let ops = diff_keys(&old, &new).unwrap();
        let insert_count_for_a = ops
            .iter()
            .filter(|op| matches!(op, KeyedOp::Insert { key, .. } if key == "a"))
            .count();
        assert_eq!(
            insert_count_for_a, 0,
            "\"a\" は old にも存在するため Insert 自体が不要: {ops:?}"
        );
    }

    // --- イシュー #1318: O(n²) 再発検知の一環として、大規模ケースの op 数
    // 自体を契約化する（ルート issue #1313 が特定した CSR create の
    // O(n²) コストは `nth_element_child`/`next_element_sibling` の sibling
    // 走査回数に起因し `diff_keys` 自体は O(n) だが、`diff_keys` が発行する
    // op 数が想定外に膨らむと DOM 適用側の DOM 操作コストも連動して膨らむ
    // ため、まず purely な diff 層で op 数の上限を固定しておく）。

    /// 空 → N 行（N=1,000）: Insert がちょうど N 件、index は 0..N の昇順で
    /// 1 つも欠けない（余分な Remove/Move が発生しないこと）。
    #[test]
    fn diff_keys_from_empty_to_n_rows_emits_exactly_n_inserts_in_order() {
        const N: usize = 1_000;
        let old: Vec<String> = Vec::new();
        let new: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();

        let ops = diff_keys(&old, &new).unwrap();

        assert_eq!(ops.len(), N, "空 → N 行は Insert ちょうど N 件のみのはず");
        for (i, op) in ops.iter().enumerate() {
            assert_eq!(
                op,
                &KeyedOp::Insert {
                    index: i,
                    key: format!("k{i}"),
                },
                "Insert の index は 0..N の昇順で欠けないはず"
            );
        }
    }

    /// 既存 N 行の先頭へ 1 件挿入: op はちょうど 1 件（Insert index=0）の
    /// みで、既存 N 件への無関係な Remove/Move は発生しない。
    #[test]
    fn diff_keys_prepend_one_to_n_rows_emits_exactly_one_insert() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new: Vec<String> = vec!["new".to_string()];
        new.extend(old.iter().cloned());

        let ops = diff_keys(&old, &new).unwrap();

        assert_eq!(
            ops,
            vec![KeyedOp::Insert {
                index: 0,
                key: "new".to_string(),
            }]
        );
    }

    /// 既存 N 行の末尾へ 1 件挿入: op はちょうど 1 件（Insert index=N）の
    /// みで、既存 N 件への無関係な Remove/Move は発生しない。
    #[test]
    fn diff_keys_append_one_to_n_rows_emits_exactly_one_insert() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new: Vec<String> = old.clone();
        new.push("new".to_string());

        let ops = diff_keys(&old, &new).unwrap();

        assert_eq!(
            ops,
            vec![KeyedOp::Insert {
                index: N,
                key: "new".to_string(),
            }]
        );
    }

    /// 完全逆順（reverse）: 全件が同じ集合のまま並びだけが反転するケース。
    /// Remove/Insert は 1 件も発生せず、Move が高々 N-1 件で収まる
    /// （全 N 件が Move になるとは限らない実装だが、上限として N-1 を
    /// 固定し「N 件超の異常な op 数」の再発を検知する）。
    #[test]
    fn diff_keys_full_reverse_emits_only_moves_within_n_minus_one() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let new: Vec<String> = old.iter().rev().cloned().collect();

        let ops = diff_keys(&old, &new).unwrap();

        assert!(
            ops.iter().all(|op| matches!(op, KeyedOp::Move { .. })),
            "完全逆順は Remove/Insert を発生させず Move のみのはず"
        );
        assert!(
            ops.len() < N,
            "Move の件数は N-1 件以内のはず（実測: {}）",
            ops.len()
        );
    }

    /// 固定シード LCG（線形合同法）による決定的シャッフル
    /// （Fisher–Yates）。乱数・時刻・環境に依存する `rand` クレート等は
    /// 使わず標準ライブラリのみで完結させる（REQ-3: 外部依存追加ゼロ）。
    fn lcg_shuffle(items: &mut [String], seed: u64) {
        let mut state = seed;
        let mut next = move || {
            // Numerical Recipes の定数（決定的であれば具体的な定数の由来は
            // 問わない用途。テストの再現性のみが目的）。
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        };
        for i in (1..items.len()).rev() {
            let j = (next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }

    /// 固定シード LCG によるシャッフル（N=1,000）: 集合は変化しないため
    /// Remove/Insert は発生せず、Move の件数は N-1 件を超えない
    /// （O(n²) 再発検知: op 数自体が線形の上限に収まることを固定する）。
    #[test]
    fn diff_keys_deterministic_shuffle_emits_only_moves_within_n_minus_one() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new = old.clone();
        lcg_shuffle(&mut new, 0x1318_1318_1318_1318);

        let ops = diff_keys(&old, &new).unwrap();

        assert!(
            ops.iter().all(|op| matches!(op, KeyedOp::Move { .. })),
            "同一集合のシャッフルは Remove/Insert を発生させず Move のみのはず"
        );
        assert!(
            ops.len() < N,
            "Move の件数は N-1 件以内のはず（実測: {}）",
            ops.len()
        );
    }

    // --- diff_keyed_items（イシュー #1323 新設: 内容比較付き diff）。

    fn item(key: &str, text_content: &str) -> (String, Node) {
        (key.to_string(), el("li", vec![], vec![text(text_content)]))
    }

    /// 同一キー内容変更のみ: Update がちょうど 1 件発行される。
    #[test]
    fn diff_keyed_items_detects_content_only_change_as_single_update() {
        let old = vec![item("a", "old")];
        let new = vec![item("a", "new")];

        assert_eq!(
            diff_keyed_items(&old, &new).unwrap(),
            vec![KeyedOp::Update {
                key: "a".to_string()
            }]
        );
    }

    /// 新旧完全一致: op ゼロ件（大規模 N=1,000 でも成立）。
    #[test]
    fn diff_keyed_items_emits_no_ops_when_fully_unchanged() {
        let old: Vec<(String, Node)> = (0..1_000).map(|i| item(&format!("k{i}"), "v")).collect();
        let new = old.clone();

        assert_eq!(diff_keyed_items(&old, &new).unwrap(), Vec::new());
    }

    /// 混在ケース: 削除 + 新規挿入 + 移動かつ内容変更。無関係キーへの余分な
    /// op が発生せず、パス順（Remove 群 → Insert/Move 群 → Update 群）が
    /// 固定される。
    #[test]
    fn diff_keyed_items_handles_mixed_remove_insert_move_and_update() {
        let old = vec![item("a", "a-v"), item("b", "b-v"), item("c", "c-v")];
        // b は削除される。c は先頭へ移動しつつ内容変更。d は新規挿入。
        let new = vec![item("c", "c-v2"), item("a", "a-v"), item("d", "d-v")];

        let ops = diff_keyed_items(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![
                KeyedOp::Remove {
                    key: "b".to_string()
                },
                KeyedOp::Move {
                    index: 0,
                    key: "c".to_string()
                },
                KeyedOp::Insert {
                    index: 2,
                    key: "d".to_string()
                },
                KeyedOp::Update {
                    key: "c".to_string()
                },
            ]
        );
    }

    /// 設計書 §3.4 正準例: a は位置のみ変更・b は位置と内容変更。
    #[test]
    fn diff_keyed_items_canonical_example_from_design_doc() {
        let old = vec![item("a", "a-v"), item("b", "b-old")];
        let new = vec![item("b", "b-new"), item("a", "a-v")];

        let ops = diff_keyed_items(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![
                KeyedOp::Move {
                    index: 0,
                    key: "b".to_string()
                },
                KeyedOp::Update {
                    key: "b".to_string()
                },
            ]
        );
    }

    /// Move 対象なしで両キーとも内容変更: Update が new_items 順で 2 件。
    #[test]
    fn diff_keyed_items_emits_updates_in_new_items_order_without_move() {
        let old = vec![item("a", "a-old"), item("b", "b-old")];
        let new = vec![item("a", "a-new"), item("b", "b-new")];

        let ops = diff_keyed_items(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![
                KeyedOp::Update {
                    key: "a".to_string()
                },
                KeyedOp::Update {
                    key: "b".to_string()
                },
            ]
        );
    }

    /// 保持キーが Move のみ（内容一致）: Update は発行されない。
    #[test]
    fn diff_keyed_items_does_not_emit_update_for_move_only_change() {
        let old = vec![item("a", "v"), item("b", "v")];
        let new = vec![item("b", "v"), item("a", "v")];

        let ops = diff_keyed_items(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![KeyedOp::Move {
                index: 0,
                key: "b".to_string()
            }]
        );
    }

    /// Insert された新規キーには Update が発行されない（old 側に存在しない
    /// ため比較対象そのものがない）。
    #[test]
    fn diff_keyed_items_does_not_emit_update_for_newly_inserted_key() {
        let old = vec![item("a", "v")];
        let new = vec![item("a", "v"), item("b", "v")];

        let ops = diff_keyed_items(&old, &new).unwrap();
        assert_eq!(
            ops,
            vec![KeyedOp::Insert {
                index: 1,
                key: "b".to_string()
            }]
        );
    }

    /// 重複キー混入（old 側）でも panic しない（fail-closed 防御の継承）。
    #[test]
    fn diff_keyed_items_does_not_panic_on_duplicate_keys_in_old() {
        let old = vec![item("a", "v1"), item("a", "v2"), item("b", "v")];
        let new = vec![item("a", "v3"), item("b", "v")];

        // panic しないことのみを確認する（重複時の正確な操作列は未規定）。
        let _ = diff_keyed_items(&old, &new).unwrap();
    }

    /// 回帰（PR #1336 codex レビュー P1 是正）: `old_items` 側にキーが
    /// 重複していても、保持される（`working` に残る）のは最後の出現
    /// であるため、Update 判定の比較対象（`old_by_key`）も同じ「最後の
    /// 出現」の内容でなければならない。`old=[("a",v1),("a",v2)],
    /// new=[("a",v1)]` は、最初の出現（v1）と比較する旧設計だと新旧が
    /// 一致していると誤判定して `Update` を発行せず、`Remove` で v1 が
    /// 消えて実際には残る v2 が古い内容のまま放置される。最後の出現
    /// （v2）と正しく比較する現設計では、`Remove` で v1 が消えたあと
    /// 残る v2 の内容が `new` の要求する v1 と食い違うため `Update` が
    /// 発行される。
    #[test]
    fn diff_keyed_items_compares_last_old_occurrence_for_update_detection() {
        let old = vec![item("a", "v1"), item("a", "v2")];
        let new = vec![item("a", "v1")];

        let ops = diff_keyed_items(&old, &new).unwrap();

        assert_eq!(
            ops,
            vec![
                KeyedOp::Remove {
                    key: "a".to_string()
                },
                KeyedOp::Update {
                    key: "a".to_string()
                },
            ],
            "保持される最後の出現（v2）が new の要求する内容（v1）と \
             食い違うため Update が必要: {ops:?}"
        );
    }

    /// 重複キー混入（new 側）でも panic しない（対称的な fail-closed 防御）。
    #[test]
    fn diff_keyed_items_does_not_panic_on_duplicate_keys_in_new() {
        let old = vec![item("a", "v")];
        let new = vec![item("a", "v1"), item("a", "v2")];

        let _ = diff_keyed_items(&old, &new).unwrap();
    }

    /// 構造的固定: 内容全一致時、`diff_keys`（キー列のみ）と
    /// `diff_keyed_items`（内容比較つき）の op 列は一致する（第 1・2 パス
    /// 共有の回帰確認）。
    #[test]
    fn diff_keys_and_diff_keyed_items_agree_when_content_is_unchanged() {
        let old_items = vec![item("a", "v"), item("b", "v"), item("c", "v")];
        let new_items = vec![item("c", "v"), item("a", "v"), item("d", "v")];

        let old_keys: Vec<String> = old_items.iter().map(|(k, _)| k.clone()).collect();
        let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();

        assert_eq!(
            diff_keys(&old_keys, &new_keys).unwrap(),
            diff_keyed_items(&old_items, &new_items).unwrap(),
        );
    }

    /// 異常系（PR #1390 レビュー是正、イシュー #1375）: `diff_keys` は
    /// `keyed_list` を経由しない生のキー列を直接受け取れる公開 API のため、
    /// `old_keys`/`new_keys` いずれかの要素数が [`MAX_KEYED_LIST_ITEMS`] を
    /// 超えると `HashMap`/`HashSet` を構築せず TooManyItems で拒否される
    /// （`keyed_list` 側の上限を経由しない直接投入をここで塞ぐ）。
    #[test]
    fn diff_keys_rejects_too_many_new_keys() {
        let old: Vec<String> = Vec::new();
        let new: Vec<String> = (0..=MAX_KEYED_LIST_ITEMS)
            .map(|i| format!("k{i}"))
            .collect();
        let count = new.len();
        let err = diff_keys(&old, &new).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 異常系（同上）: `old_keys` 側の要素数超過も同じ上限で拒否される
    /// （`new_keys` 側のみを検査する非対称な実装になっていないことの固定）。
    #[test]
    fn diff_keys_rejects_too_many_old_keys() {
        let old: Vec<String> = (0..=MAX_KEYED_LIST_ITEMS)
            .map(|i| format!("k{i}"))
            .collect();
        let new: Vec<String> = Vec::new();
        let count = old.len();
        let err = diff_keys(&old, &new).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 異常系（同上）: 項目数は少なくてもキー文字列の合計バイト数が
    /// [`MAX_KEYED_LIST_KEY_BYTES`] を超えると KeyBytesExceeded で拒否
    /// される（`keyed_list` の `key_bytes_exceeded_is_rejected` と対称）。
    #[test]
    fn diff_keys_rejects_key_bytes_exceeded() {
        let huge_key = "k".repeat(MAX_KEYED_LIST_KEY_BYTES + 1);
        let total_bytes = huge_key.len();
        let old: Vec<String> = Vec::new();
        let new: Vec<String> = vec![huge_key];
        let err = diff_keys(&old, &new).unwrap_err();
        assert_eq!(err, KeyedListError::KeyBytesExceeded { total_bytes });
    }

    /// 異常系（PR #1390 レビュー是正、イシュー #1375）: `diff_keyed_items`
    /// も同じ上限ゲートを `old_items`/`new_items` それぞれへ適用する
    /// （`diff_keys` と対称の回帰確認）。
    #[test]
    fn diff_keyed_items_rejects_too_many_new_items() {
        let old: Vec<(String, Node)> = Vec::new();
        let new: Vec<(String, Node)> = (0..=MAX_KEYED_LIST_ITEMS)
            .map(|i| (format!("k{i}"), el("li", vec![], vec![])))
            .collect();
        let count = new.len();
        let err = diff_keyed_items(&old, &new).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 異常系（同上）: `old_items` 側の要素数超過も同じ上限で拒否される。
    #[test]
    fn diff_keyed_items_rejects_too_many_old_items() {
        let old: Vec<(String, Node)> = (0..=MAX_KEYED_LIST_ITEMS)
            .map(|i| (format!("k{i}"), el("li", vec![], vec![])))
            .collect();
        let new: Vec<(String, Node)> = Vec::new();
        let count = old.len();
        let err = diff_keyed_items(&old, &new).unwrap_err();
        assert_eq!(err, KeyedListError::TooManyItems { count });
    }

    /// 正常系（境界値）: `old_keys`/`new_keys` ともちょうど
    /// [`MAX_KEYED_LIST_ITEMS`] 件なら拒否されない（`diff_keys` 版の
    /// `item_count_at_the_limit_is_accepted` 相当）。
    #[test]
    fn diff_keys_item_count_at_the_limit_is_accepted() {
        let old: Vec<String> = (0..MAX_KEYED_LIST_ITEMS).map(|i| format!("k{i}")).collect();
        let new: Vec<String> = old.clone();
        assert!(diff_keys(&old, &new).is_ok());
    }
}
