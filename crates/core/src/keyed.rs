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

/// [`keyed_list`] 構築時の fail-closed エラー。
///
/// いずれの異常系も `panic!`/`unwrap()` ではなく `Err` として安全側に倒す
/// （ライブラリコードでの panic 回避規約、OWASP A05 安全でない設計への対抗）。
/// 「衝突・欠落したキーを持つ不正な HTML を出力しない」という fail-closed の
/// 目的を、`render()` 呼び出し時点ではなく**構築時点**で満たす（不正な状態を
/// そもそも表現不能にする設計。`docs/design/dom-binding-update-design.md`
/// §5.2 改訂内容を参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedListError {
    /// キーが空文字列（キー欠落）。`index` は `items` 内の位置。
    EmptyKey {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 同一リスト内でキーが重複している（直下の子スコープのみが対象）。
    DuplicateKey {
        /// 最初に当該キーが出現したインデックス。
        first_index: usize,
        /// 重複が検出されたインデックス。
        duplicate_index: usize,
    },
    /// 子ノードが `Node::Element` でなく、`data-key` 属性を付与できない。
    NonElementItem {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 呼び出し側が渡した属性列に予約マーカー属性
    /// （[`BIND_LIST_ATTR`] / [`KEY_ATTR`]）が既に含まれている。
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

    // (2)(3) 各子要素の検証: Element であること・キー非空・キー一意性・予約
    // 属性の非混入。パス 1（本ループ）は `items.iter()` による**参照のみ**の
    // 走査であり、`items` を消費しない。検証順序（空キー → 重複キー →
    // 非 Element → 予約属性）・エラーの index/first_index は旧実装（1 パス
    // 構成）と完全に同一（回帰テスト `duplicate_error_precedence_is_stable`
    // が固定）。直下スコープのみを対象に、初出インデックスを HashMap で記録
    // して O(n) で重複判定する（非再帰、DoS 耐性）。
    let mut first_index_of: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::with_capacity(items.len());

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
/// しない想定だが、DOM 改ざん等で `old_keys` 側に重複が混入した場合の防御）
/// は、`working` 側の探索を「未処理の先頭要素」に限定することで最初の 1 件
/// のみを対象とし、無限ループ・panic を起こさない（fail-closed）。
///
/// `fandhe-frontend-wasm-client`（イシュー #345）が導入した実装をそのまま
/// 移管したもの（イシュー #1323、`docs/design/keyed-update-op-design.md`
/// §4.1）。挙動・op 発行順は移管元と完全に同一（回帰テストで固定）。
/// wasm-client 側の同名関数は #1324 で本関数への re-export へ置換予定で
/// あり、それまでの一時的な実装重複は意図的。
pub fn diff_keys(old_keys: &[String], new_keys: &[String]) -> Vec<KeyedOp> {
    let mut ops = Vec::new();
    let working = remove_pass(old_keys, new_keys, &mut ops);
    insert_or_move_pass(working, new_keys, &mut ops);
    ops
}

/// [`diff_keys`] 第 1 パス: `new_keys` に存在しないキーを [`KeyedOp::Remove`]
/// として記録し、除外した残り（保持キーのみ）を `working` として返す。
/// [`diff_keyed_items`] も同一のパスを共有する（挙動不変の内部ヘルパー化、
/// 設計書 §4.2 の実装確定事項）。
fn remove_pass(old_keys: &[String], new_keys: &[String], ops: &mut Vec<KeyedOp>) -> Vec<String> {
    let new_set: std::collections::HashSet<&str> = new_keys.iter().map(String::as_str).collect();

    let mut working: Vec<String> = Vec::with_capacity(old_keys.len());
    for key in old_keys {
        if new_set.contains(key.as_str()) {
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
    if n == 0 {
        for (index, key) in new_keys.iter().enumerate() {
            ops.push(KeyedOp::Insert {
                index,
                key: key.clone(),
            });
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
    let mut queue: std::collections::HashMap<&str, std::collections::VecDeque<usize>> =
        std::collections::HashMap::with_capacity(n);
    for (i, key) in working.iter().enumerate() {
        queue.entry(key.as_str()).or_default().push_back(i);
    }

    for (index, key) in new_keys.iter().enumerate() {
        if let Some(h) = head {
            if working[h] == *key {
                // 先頭ノードが期待キーと一致: 操作を発行せず消費するのみ。
                if let Some(q) = queue.get_mut(key.as_str()) {
                    q.pop_front();
                }
                head = next[h];
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
                index,
                key: key.clone(),
            });
        } else {
            ops.push(KeyedOp::Insert {
                index,
                key: key.clone(),
            });
        }
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
/// let ops = diff_keyed_items(&old_items, &new_items);
/// assert_eq!(
///     ops,
///     vec![
///         KeyedOp::Move { index: 0, key: "b".to_string() },
///         KeyedOp::Update { key: "b".to_string() },
///     ],
/// );
/// ```
pub fn diff_keyed_items(
    old_items: &[(String, Node)],
    new_items: &[(String, Node)],
) -> Vec<KeyedOp> {
    let old_keys: Vec<String> = old_items.iter().map(|(k, _)| k.clone()).collect();
    let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();

    let mut ops = Vec::new();
    let working = remove_pass(&old_keys, &new_keys, &mut ops);
    insert_or_move_pass(working, &new_keys, &mut ops);

    // 第 3 パス: 保持キー（重複混入時も old_items 側の最初の 1 件のみを
    // 対象とする fail-closed、diff_keys と同じ防御）を new_items 順に
    // 走査し、新旧 Node が不一致のときのみ Update を発行する。
    // `old_by_key` は最初に出現したキーのみを記録する（重複混入時に後続の
    // 同名キーで上書きしないことで「最初の 1 件のみを対象」を維持する）。
    let mut old_by_key: std::collections::HashMap<&str, &Node> =
        std::collections::HashMap::with_capacity(old_items.len());
    for (key, node) in old_items {
        old_by_key.entry(key.as_str()).or_insert(node);
    }

    let mut seen_new_keys: std::collections::HashSet<&str> =
        std::collections::HashSet::with_capacity(new_items.len());
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

    ops
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
        assert_eq!(diff_keys(&old, &new), Vec::new());
    }

    /// 末尾への追加は Insert 1 件のみ。
    #[test]
    fn diff_keys_detects_append_at_tail() {
        let old = keys(&["a", "b"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(
            diff_keys(&old, &new),
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
            diff_keys(&old, &new),
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
            diff_keys(&old, &new),
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
            diff_keys(&old, &new),
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
            diff_keys(&old, &new),
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
        let ops = diff_keys(&old, &new);
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
        let ops = diff_keys(&old, &new);
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
        let ops = diff_keys(&old, &new);
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

    /// 重複キーが混入していても panic せず、先頭の 1 件のみを対象に処理する
    /// （DOM 改ざん等の異常系に対する fail-closed 防御、本モジュール doc 参照）。
    #[test]
    fn diff_keys_does_not_panic_on_duplicate_keys_in_old() {
        let old = keys(&["a", "a", "b"]);
        let new = keys(&["a", "b"]);
        // panic しないことのみを確認する（重複時の正確な操作列は未規定）。
        let _ = diff_keys(&old, &new);
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

        let ops = diff_keys(&old, &new);

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

        let ops = diff_keys(&old, &new);

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

        let ops = diff_keys(&old, &new);

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

        let ops = diff_keys(&old, &new);

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

        let ops = diff_keys(&old, &new);

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
            diff_keyed_items(&old, &new),
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

        assert_eq!(diff_keyed_items(&old, &new), Vec::new());
    }

    /// 混在ケース: 削除 + 新規挿入 + 移動かつ内容変更。無関係キーへの余分な
    /// op が発生せず、パス順（Remove 群 → Insert/Move 群 → Update 群）が
    /// 固定される。
    #[test]
    fn diff_keyed_items_handles_mixed_remove_insert_move_and_update() {
        let old = vec![item("a", "a-v"), item("b", "b-v"), item("c", "c-v")];
        // b は削除される。c は先頭へ移動しつつ内容変更。d は新規挿入。
        let new = vec![item("c", "c-v2"), item("a", "a-v"), item("d", "d-v")];

        let ops = diff_keyed_items(&old, &new);
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

        let ops = diff_keyed_items(&old, &new);
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

        let ops = diff_keyed_items(&old, &new);
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

        let ops = diff_keyed_items(&old, &new);
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

        let ops = diff_keyed_items(&old, &new);
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
        let _ = diff_keyed_items(&old, &new);
    }

    /// 重複キー混入（new 側）でも panic しない（対称的な fail-closed 防御）。
    #[test]
    fn diff_keyed_items_does_not_panic_on_duplicate_keys_in_new() {
        let old = vec![item("a", "v")];
        let new = vec![item("a", "v1"), item("a", "v2")];

        let _ = diff_keyed_items(&old, &new);
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
            diff_keys(&old_keys, &new_keys),
            diff_keyed_items(&old_items, &new_items),
        );
    }
}
