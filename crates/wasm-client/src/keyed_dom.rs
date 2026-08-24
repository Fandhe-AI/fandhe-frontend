//! keyed list の DOM 適用: wasm32 配線層（イシュー #345）。
//!
//! [`crate::keyed_diff`]（DOM 非依存の純粋 diff 層）が計画した操作列
//! （[`crate::keyed_diff::KeyedOp`]）を実 DOM（`web-sys`）へ適用する。
//! `set_inner_html` / `insert_adjacent_html` / `raw_html` を**一切呼ばない**
//! （`crate::binding_dom` と同じ不変条件 1・2・4）。挿入ノードは
//! `fandhe_frontend_core::Node` 木から `create_element`/`set_text_content`/`append_child`
//! でプログラム的に構築し、移動は既存ノード参照を保持したまま
//! `insert_before` のみで行う（既存 DOM ノードを再生成しないことが
//! フォーカス・入力途中の値の保持に直結する、設計書 §5.3）。
//!
//! 連続する `Insert`（新規構築ノード）は `DocumentFragment` へ集約して
//! 1 回の `insert_before` で挿入する（イシュー #1320。`WebSysKeyedDom::insert_before_batch`
//! 参照）。fragment は**新規構築ノード専用**であり、既存 DOM ノードは
//! 決して fragment を経由しない: `DocumentFragment` へ `append_child` した
//! 時点でその子は元のドキュメントツリーから切り離される（DOM 標準仕様）
//! ため、既存ノードを fragment 経由で移動すると現在の親から一旦除去され
//! フォーカス・入力途中の値が失われる。既存ノードの移動は
//! `WebSysKeyedDom::move_before` が個別に `insert_before` するのみで
//! fragment を一切使わない設計を維持する（フォーカス保持の不変条件、
//! 設計書 §5.3）。
//!
//! `fandhe_frontend_core::Node::RawHtml` を含む部分木は `web_sys::console` へ
//! 英語固定文言の警告を出したうえで、その `RawHtml` ノードを含む部分木
//! **全体**を構築失敗（`None`）として呼び出し元へ伝播する（本経路に
//! エスケープ迂回を組み込まない、設計書 §9 不変条件 4）。祖先ノードの
//! `build_dom_node_with_namespace` は子 1 件の `None` を無言で読み飛ばして
//! 残りだけを DOM へ反映することはせず、`?` で自身も `None` を返す
//! （`crate::subtree::replace_subtree` が要求する「`RawHtml` が部分木の
//! どこに現れても DOM を一切変更せず `Err` を返す」fail-closed 契約、
//! `lib.rs` クレート冒頭不変条件 7）。`apply_keyed_list` から見ると、この
//! 伝播の結果 `KeyedOp::Insert` の対象アイテムが丸ごと未適用のまま残る
//! （個別ノード単位ではなくアイテム単位の skip、イシュー #1121）。

use crate::keyed_apply_result::KeyedListApplyResult;
use fandhe_frontend_core::keyed::KEY_ATTR;
use fandhe_frontend_core::Node;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

/// `list_field` の keyed list 親要素（`data-bind-list="<field>"`）を新しい
/// `Node` 木から探す（純粋ヘルパー、DOM 非依存）。
///
/// `component.view()` の出力木全体から深さ優先で探索する。見つからない
/// 場合は `None`（呼び出し側は当該フィールドの更新を no-op とする、
/// fail-closed）。
pub fn find_keyed_list_node<'a>(node: &'a Node, list_field: &str) -> Option<&'a Node> {
    if let Node::Element { attrs, .. } = node {
        if attrs
            .iter()
            .any(|(k, v)| k == fandhe_frontend_core::keyed::BIND_LIST_ATTR && v == list_field)
        {
            return Some(node);
        }
    }
    if let Node::Element { children, .. } = node {
        for child in children {
            if let Some(found) = find_keyed_list_node(child, list_field) {
                return Some(found);
            }
        }
    }
    None
}

/// `node` 配下を走査し、`data-bind-list` を持つ**全ての** keyed list 親
/// ノードを `(field, Node)` 列として収集する（イシュー #1324）。
///
/// [`find_keyed_list_node`] が単一 field の引き当てなのに対し、本関数は
/// `fandhe-frontend-wasm-full` の `Runtime::mount`/`Runtime::hydrate` が
/// マウント直後の `component.view()` から `keyed_list_cache`（`Update` op
/// 適用の内容比較基準、`Runtime::keyed_list_cache` doc 参照）を一括で
/// 種付けするために使う。マウント直後は `dirty_fields()` が空
/// （まだ 1 度も `update()` を呼んでいない）であり、`&'static str` の
/// field 名集合を dirty 経由で知る手段がないため、戻り値のキーは属性値
/// から復元した所有 `String` とする（`fandhe_frontend_core::keyed::keyed_list`
/// が `field: &'static str` を `to_string()` して `data-bind-list` へ書き込む
/// ため、ここでの復元はラウンドトリップになる）。
///
/// ネストした keyed list（親アイテムの子孫として別の keyed list が現れる
/// 構成）も想定し、マッチした要素の子孫へも再帰を続ける（マッチで打ち
/// 切らない）。
pub fn collect_keyed_list_nodes(node: &Node) -> Vec<(String, Node)> {
    let mut out = Vec::new();
    collect_keyed_list_nodes_into(node, &mut out);
    out
}

/// [`collect_keyed_list_nodes`] が返す各 keyed list ノードを
/// [`crate::keyed_apply::sanitize_node_for_achieved`] へ通し、「実際に
/// マウント時 DOM へ書き込まれた内容」へ正規化する（イシュー #1340
/// codex-review 全面棚卸し対応、`Runtime::mount`/`Runtime::hydrate` の
/// `keyed_list_cache` 初期 baseline 種付け専用）。
///
/// # マウント時のキャッシュ種付けにも同じ不変条件が必要な理由
///
/// `Runtime::mount`/`Runtime::hydrate` は `dom::mount_initial`
/// （`root.set_inner_html(render_component_html(component))`）で
/// [`fandhe_frontend_core::render`] の出力から実 DOM を構築する。`render`
/// は本モジュールの `sync_attrs`/`build_dom_node_with_namespace` と同じ
/// 述語（危険 URL スキーム・イベントハンドラ属性・不正 `srcset` の書き込み
/// skip、`fandhe_frontend_core::lib.rs` の `render` 実装参照）を適用する
/// ため、`component.view()` に検証拒否対象の属性が含まれていれば実 DOM
/// には最初から書き込まれない。`keyed_list_cache` を素の `view()` 出力で
/// 種付けすると、この「実際には書き込まれなかった属性」がキャッシュ上は
/// 存在する扱いになり、以後の `apply_keyed_list_with_previous` の diff
/// 基準がマウント時点から既に実 DOM と乖離した状態で始まってしまう
/// （本モジュール冒頭・`keyed_apply` モジュール冒頭 doc「cache-miss
/// フォールバックの達成契約」と同根の不変条件）。`sanitize_node_for_achieved`
/// は新規構築経路（`Insert`・タグ変更を伴う `replace_root`）の「達成
/// Node」合成が使う述語と同一のものを使うため、`render` の skip 判定と
/// 一致する。
pub fn sanitize_keyed_list_node_for_achieved(node: &Node) -> Node {
    crate::keyed_apply::sanitize_node_for_achieved(node)
}

fn collect_keyed_list_nodes_into(node: &Node, out: &mut Vec<(String, Node)>) {
    let Node::Element {
        attrs, children, ..
    } = node
    else {
        return;
    };
    if let Some((_, field)) = attrs
        .iter()
        .find(|(k, _)| k == fandhe_frontend_core::keyed::BIND_LIST_ATTR)
    {
        out.push((field.clone(), node.clone()));
    }
    for child in children {
        collect_keyed_list_nodes_into(child, out);
    }
}

/// 実 DOM 上で `field` に対応する keyed list 親要素
/// （`[data-bind-list="<field>"]`）を探す。`root` 配下を 1 度だけ
/// `query_selector` で探索する（`crate::binding_dom::BindingTable::scan` と
/// 同様、DOM 改ザン等による予期しない複数一致は最初の 1 件を採用する）。
///
/// # Errors
///
/// `query_selector` 自体が失敗した場合（不正なセレクタ文字列等、通常は
/// 到達しない）に `Err` を返す。要素が見つからない場合は `Ok(None)`。
pub fn find_list_element(
    root: &Element,
    field: &str,
) -> Result<Option<Element>, wasm_bindgen::JsValue> {
    let selector = format!(
        "[{}=\"{field}\"]",
        fandhe_frontend_core::keyed::BIND_LIST_ATTR
    );
    root.query_selector(&selector)
        .map_err(|_| wasm_bindgen::JsValue::from_str("query_selector failed for keyed list"))
}

/// keyed list 親要素（`Node::Element`）の直下の子から `(key, Node)` の
/// **所有**列を取り出す（`data-key` 属性を持たない子・非 Element 子は
/// fail-closed で skip する）。
///
/// `fandhe_frontend_core::Node` は `Clone` を実装するため、ここでクローン
/// する（当初の借用版 `list_item_nodes` から、イシュー #1324 で
/// [`apply_keyed_list_with_previous`] が要求する所有権へ拡張した）。
/// [`fandhe_frontend_core::keyed::diff_keyed_items`] の呼び出しには所有
/// `Node` の `&[(String, Node)]` が必要（内容比較付き diff・「達成 Node」
/// 合成の双方が要求する）。構造変化のみを扱う [`apply_keyed_list`]（本関数
/// を使う既存経路）にとってもコストは無視できる程度（リスト 1 件あたりの
/// アイテム部分木クローン、O(n) の追加作業でありアルゴリズムの計算量
/// クラスは変わらない）。
/// [`WebSysKeyedDom::replace_item_children`] のロールバック手順自体
/// （`insert_before`/`remove_child`/`append_child` の逆操作）が失敗した
/// 場合に出す固定英語文言の警告（設計書 §6 不変条件 6「残る有限の
/// リスク」・不変条件 7〔キー値・アイテム内容を含めない〕）。`unwrap()`/
/// `panic!` は使わず、当該アイテム 1 件が不定状態になりうることを警告
/// ログのみで示し処理を継続する。文言は rodata 削減のため短縮している
/// （不変条件 6・7 は保ったまま、イシュー #1388）。
fn warn_replace_item_children_rollback_failed() {
    web_sys::console::warn_1(
        &"fandhe-frontend-wasm-client: child replacement rollback failed (item left in an inconsistent state)"
            .into(),
    );
}

/// [`WebSysKeyedDom::replace_root`] のロールバック手順自体（挿入済みの
/// 新要素を取り除く `remove_child`）が失敗した場合に出す固定英語文言の
/// 警告（設計書 §6 不変条件 6「残る有限のリスク」と同種、不変条件 7
/// 〔キー値・アイテム内容を含めない〕、イシュー #1340 codex-review P1
/// 〔3 巡目〕対応）。`unwrap()`/`panic!` は使わず、当該アイテム 1 件が
/// 不定状態（旧要素・新要素が同時に存在しうる）になりうることを警告ログ
/// のみで示し処理を継続する。文言は rodata 削減のため短縮している
/// （不変条件 6・7 は保ったまま、イシュー #1388）。
fn warn_replace_root_rollback_failed() {
    web_sys::console::warn_1(
        &"fandhe-frontend-wasm-client: root replacement rollback failed (item left in an inconsistent state)"
            .into(),
    );
}

fn owned_list_item_nodes(list_node: &Node) -> Vec<(String, Node)> {
    let Node::Element { children, .. } = list_node else {
        return Vec::new();
    };
    children
        .iter()
        .filter_map(|child| {
            let Node::Element { attrs, .. } = child else {
                return None;
            };
            attrs
                .iter()
                .find(|(k, _)| k == KEY_ATTR)
                .map(|(_, key)| (key.clone(), child.clone()))
        })
        .collect()
}

/// `fandhe_frontend_core::Node` から実 DOM 要素をプログラム的に構築する
/// （`create_element`/`set_text_content`/`append_child` の再帰、
/// `innerHTML`/`insert_adjacent_html` 不使用）。
///
/// `Node::RawHtml` を含む部分木は `console` へ警告したうえで部分木全体を
/// 構築失敗として呼び出し元へ伝播する fail-closed（不変条件 4）。属性名・
/// タグ名は `fandhe_frontend_interactive::AppState::view()` 等
/// 呼び出し側の `&'static str`/コンパイル時に固定された文字列であることを
/// 前提とするが、`set_attribute` 自体は DOM 標準 API であり `on*` 属性名を
/// 渡した場合でも `setAttribute` はイベントハンドラを実行コード化しない
/// （`element.onclick = ...` とは異なる。属性値は `escape_html` を経由しない
/// 生文字列だが、`setAttribute`/`set_text_content` は HTML パースを行わない
/// ため breakout 系 XSS 経路にならない）。
///
/// ただし URL スキーム経由の XSS（`href="javascript:..."` 等）は breakout を
/// 伴わないため上記の理由では防げない。`render_into`（fandhe-frontend-core）・
/// `binding_dom.rs` と同一の URL 検証（`srcset` のカンマ区切り候補分割検証
/// を含む）・イベントハンドラ属性ブロックを本経路にも適用する
/// （イシュー #373。`docs/policy/attribute-output-policy.md`）。
///
/// [`apply_keyed_list`]（本モジュール内、keyed list 挿入）に加え、
/// `fandhe-frontend-wasm-full` の遷移描画（`nav.rs`、イシュー #374）からも
/// `fandhe_frontend_wasm_client::build_dom_node` として呼ばれる公開 API（`lib.rs` の
/// `pub use keyed_dom::build_dom_node` 経由）。挿入先で `RawHtml` を `None`
/// として fail-closed に拒否する契約は呼び出し元を問わず不変（不変条件 4
/// を遷移経路にも継承）。
///
/// 祖先の名前空間を持たない単独呼び出し（HTML 名前空間が既定）向けの薄い
/// ラッパー。`svg`/`path` 等 SVG 要素を含む挿入は [`build_dom_node_with_namespace`]
/// （本モジュール内部の [`apply_keyed_list`] が使う）が名前空間を明示的に
/// 引き継ぐ。
pub fn build_dom_node(document: &Document, node: &Node) -> Option<web_sys::Node> {
    build_dom_node_with_namespace(document, node, None)
}

/// SVG 要素の名前空間 URI
/// （[MDN: Element.namespaceURI](https://developer.mozilla.org/en-US/docs/Web/API/Element/namespaceURI)）。
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// [`build_dom_node`] の名前空間対応版（内部実装、イシュー #843 Bugbot
/// 指摘「Runtime skips stroke DOM updates」是正の一部）。
///
/// `Document::create_element` は常に HTML 名前空間の要素を生成する。HTML
/// パーサは `<svg>` 配下を自動的に SVG 名前空間へ切り替えるが、
/// `create_element` によるプログラム的構築にはその挙動がないため、素朴に
/// `create_element("path")` すると HTML 名前空間の（ブラウザが SVG として
/// 描画しない）要素になってしまう。`crates/headless-ui/src/signature_pad.rs`
/// の SignaturePad が keyed list（`data-bind-list="strokes"`、親は
/// `<svg>`）でストロークを追加する経路がこの不具合の初出であり、
/// [`apply_keyed_list`] の `KeyedOp::Insert` は挿入先 `list_element` の
/// 実際の名前空間（`Element::namespace_uri()`）を `namespace` 引数へ渡す。
///
/// `tag` 自体が `"svg"` の場合は、渡された `namespace`（祖先から引き継いだ
/// 値）に関わらず SVG 名前空間へ切り替える。これにより `<svg>` 要素それ
/// 自体を含むノード木をまるごと新規構築するケース（`fandhe-frontend-wasm-full`
/// の遷移描画 `nav.rs` 等、`namespace` が `None` から始まる呼び出し）でも
/// HTML パーサと同じ挙動を再現する。決定した名前空間は子孫へそのまま
/// 引き継ぐ（`foreignObject` 等での名前空間の再切り替えは本経路のスコープ
/// 外、SignaturePad の SVG 構造には現れない）。
fn build_dom_node_with_namespace(
    document: &Document,
    node: &Node,
    namespace: Option<&str>,
) -> Option<web_sys::Node> {
    match node {
        Node::Text(text) => Some(document.create_text_node(text).into()),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let element_namespace = if *tag == "svg" {
                Some(SVG_NAMESPACE)
            } else {
                namespace
            };
            // `.ok()?`（イシュー #1340 codex-review P1〔3 巡目〕全走査で
            // 正当性を再確認）: `create_element`/`create_element_ns` の
            // 失敗（不正なタグ名等）は関数冒頭であり、まだ何も構築して
            // いないためロールバック対象は存在しない。失敗を `?` で即座に
            // 呼び出し元へ伝播する（本関数「fail-closed」契約と同型）。
            let element = match element_namespace {
                Some(ns) => document.create_element_ns(Some(ns), tag).ok()?,
                None => document.create_element(tag).ok()?,
            };
            for (name, value) in attrs {
                if fandhe_frontend_core::is_event_handler_attr(name) {
                    // イベントハンドラ属性は一律出力しない（不変条件 9 と同一）。
                    continue;
                }
                if fandhe_frontend_core::is_url_attr(name)
                    && !fandhe_frontend_core::is_safe_url(value)
                {
                    // 危険スキームの URL 属性は書き込まない（fail-closed）。
                    continue;
                }
                // `srcset` はカンマ区切りの URL 候補を持つ特殊構文のため
                // `is_url_attr` の対象外。`render_into`/`binding_dom.rs` と
                // 同一の `is_safe_srcset` で候補分割検証する
                // （イシュー #373 レビュー指摘対応）。
                if name.eq_ignore_ascii_case("srcset")
                    && !fandhe_frontend_core::is_safe_srcset(value)
                {
                    continue;
                }
                // 属性書き込み失敗（不正な属性名等の `InvalidCharacterError`
                // 相当）を無視すると、当該属性が欠落した不完全な要素が
                // あたかも完全に構築できたかのように呼び出し元へ返り、
                // `insert_before_batch`/`replace_root`/`replace_item_children`
                // 等の後続コミット処理がそれを「達成」として確定してしまう
                // （イシュー #1340 codex-review P1〔3 巡目〕全走査対応）。
                // `element` は本関数外にまだ一切共有されていない detached
                // ノードのため、ここで `None` を返すだけで実 DOM への
                // 副作用は残らない（fail-closed、本関数の「部分木全体を
                // 構築失敗として伝播する」契約と同型）。
                element.set_attribute(name, value).ok()?;
            }
            // 子孫のいずれかが `RawHtml`（あるいは要素生成失敗）で `None` を
            // 返した場合、その子だけを無言で読み飛ばさず部分木全体を
            // 構築失敗として呼び出し元へ伝播する（fail-closed）。
            // `crate::subtree::replace_subtree` はこの関数を「`RawHtml` が
            // 部分木のどこに現れても DOM を一切変更せず `Err` を返す」契約
            // （不変条件 7）で呼ぶため、ここで子 1 件だけをスキップして
            // `Some` を返すと、`div` の子が `raw_html()` のケースのように
            // 危険なノードだけを除いた「一部だけ反映された」DOM が挿入され
            // てしまい契約違反になる。
            let mut built_children = Vec::with_capacity(children.len());
            for child in children {
                let child_node = build_dom_node_with_namespace(document, child, element_namespace)?;
                built_children.push(child_node);
            }
            for child_node in &built_children {
                // `element`（親）はここでもまだ detached のため、
                // `append_child` 失敗時も `None` を返すだけで実 DOM への
                // 副作用は残らない（上記 `set_attribute` と同じ理由、
                // イシュー #1340 codex-review P1〔3 巡目〕全走査対応）。
                element.append_child(child_node).ok()?;
            }
            Some(element.into())
        }
        Node::RawHtml(_) => {
            // keyed list 経由の挿入ノードに raw_html を混入させる経路を
            // 構造的に持たない（設計書 §9 不変条件 4）。内容は含めない
            // 固定英語文言でログのみ残す（不変条件 6。文言は rodata 削減の
            // ため短縮している、イシュー #1388）。
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-client: skipped an unsupported RawHtml node".into(),
            );
            None
        }
    }
}

/// [`crate::keyed_apply::KeyedListDom`] の `web-sys` 実装アダプタ
/// （イシュー #1318）。
///
/// 走査アルゴリズム自体（旧 `dom_item_keys`/`find_child_by_key`/
/// `nth_element_child`/op 適用ループ）は [`crate::keyed_apply::apply_ops`]
/// へ等価移植済みであり、本 struct は「`web-sys` の実 DOM 呼び出し」を
/// トレイトメソッドへ 1:1 で委譲するだけの薄いアダプタに徹する
/// （本モジュール冒頭 doc の 2 層構成、`keyed_apply` モジュール doc 参照）。
///
/// [`crate::keyed_apply::KeyedListDom::child_at`] のみ 1:1 委譲ではなく、
/// `children` フィールド（本 struct 内で保持する
/// [`crate::keyed_children_cache::KeyedChildrenCache`]、`(key, Element)` の
/// 順序付きキャッシュ）への添字アクセスで解決する（イシュー #1319。
/// codex-review 指摘: `Element::children()` + `HtmlCollection::item(index)`
/// を都度呼ぶ実装は、`HTMLCollection` が live collection であり
/// `item(index)` の計算量が WHATWG 仕様上保証されないため、ブラウザ側の
/// 実装次第で二乗コストへ退行しうる。`children` キャッシュは
/// `first_element_child`/`next_element_sibling`（ブラウザが隣接ポインタで
/// 実装する真の O(1) 操作）による 1 度きりの sibling 走査で構築し、以降は
/// 実 DOM を一切問い合わせない純粋な `Vec` 操作のみで
/// `insert_before`/`move_before` の追随更新を行う。これによりブラウザの
/// `item()` 実装がどのような計算量であっても本アダプタの `child_at` は
/// ブラウザ API のその計算量に依存しない）。全削除ワークロードでの
/// `Vec::remove` 連続呼び出しによる O(N²) 退行（PR #1392 codex-review P1
/// 指摘）は [`crate::keyed_children_cache::KeyedChildrenCache`] が
/// tombstone 化 + 遅延 compact で解消する（同モジュール doc「計算量保証」
/// 参照）。
struct WebSysKeyedDom<'a> {
    document: &'a Document,
    list_element: &'a Element,
    /// `new_list_node`（`component.view()` 側の `Node` 木）から抽出した
    /// `(key, Node)` 列。`create_item`/`KeyedOp::Update` 適用の双方が
    /// キー引きでノードを探す（イシュー #1324 で所有版
    /// [`owned_list_item_nodes`] へ切り替え、`&Node` 借用ではなくなった）。
    new_items: &'a [(String, Node)],
    /// 挿入先 `list_element` の実際の名前空間（[`build_dom_node_with_namespace`]
    /// rustdoc 参照。SVG keyed list への挿入で HTML 名前空間の要素が生成
    /// されてしまう不具合の是正を維持する）。
    namespace: Option<&'a str>,
    /// `child_at` が返す「現在の子要素列」のキャッシュ
    /// （[`crate::keyed_children_cache::KeyedChildrenCache`]、`(data-key,
    /// Element)` の順序付きハンドルキャッシュ）。`None` は未構築（初回
    /// `child_at` 呼び出しで実 DOM を 1 度だけ sibling 走査して埋める）を
    /// 表す。
    ///
    /// [`crate::keyed_apply::apply_ops`] は [`crate::keyed_diff::diff_keys`]
    /// が生成した操作列（`Remove` が必ず先頭にまとまり、続く `Move`/
    /// `Insert` は昇順 `index` で並ぶ、`keyed_diff` モジュール doc・
    /// `diff_keys` 実装参照）を順に適用する。イシュー #1374 以降は `Remove`
    /// の対象解決も [`crate::keyed_apply::KeyedListDom::find_by_key`]
    /// （`ensure_children_cache` を内部で呼ぶ）を経由するため、キャッシュの
    /// 初回構築は「最初の `Move`/`Insert` の直前」ではなく「最初の
    /// `Remove`/`Move`/`Update` op の直前」（＝操作列中の最初の op の直前、
    /// 通常は先頭にまとまる `Remove` の 1 件目）に前倒しされる。この時点は
    /// まだどの `Remove` も実 DOM へ適用されていないため、sibling 走査して
    /// 得る並びは「削除・挿入・移動のいずれも未適用」の基準状態と一致する
    /// （旧 doc が述べていた「削除後・挿入/移動適用前」ではない）。以降
    /// `insert_before`/`move_before`/`remove_child` が実 DOM への適用と同時に
    /// このキャッシュへも追随更新するため、実 DOM の並びと同期したまま
    /// 保たれる（イシュー #1374 で `remove_child` も `key` 引数を受けて
    /// インプレース追随更新するよう変更した。旧実装は「Remove はキャッシュ
    /// 構築前にのみ呼ばれる」前提で成功・失敗を問わず丸ごと `None`
    /// 無効化する fail-safe を持っていたが、
    /// [`crate::keyed_apply::KeyedListDom::find_by_key`] が
    /// `Remove`/`Move`/`Update` 共通の対象解決を担うようになったことで
    /// この前提は崩れ、丸ごと無効化のままだと全削除ワークロードで O(N²)
    /// を再導入するため置き換えた。PR #1392 codex-review P1 是正で
    /// `Vec::remove(pos)` を直接呼ぶ実装自体も全削除時に O(N²) の要素
    /// シフトを引き起こすと判明し、
    /// [`crate::keyed_children_cache::KeyedChildrenCache`]（tombstone 化 +
    /// 遅延 compact）へ置き換えた。`remove_child` の doc 参照）。
    children: Option<crate::keyed_children_cache::KeyedChildrenCache<Element>>,
}

impl WebSysKeyedDom<'_> {
    /// `children` キャッシュが未構築なら、実 DOM を 1 度だけ sibling 走査
    /// して埋める（[`crate::keyed_apply::KeyedListDom::child_at`] doc 参照）。
    /// 構築済みなら no-op（実 DOM に一切触れない）。
    ///
    /// [`Self::child_at`]（`Insert`/`Move` の参照ノード決定）に加え、
    /// [`crate::keyed_apply::KeyedListDom::find_by_key`]（`Update` 対象の
    /// 既存要素解決、イシュー #1324）からも呼ばれる共有経路。後者を独自に
    /// `first_element_child`/`next_element_sibling` の sibling 走査で実装
    /// すると、`Update` 件数 × リスト長 に比例する実 DOM 呼び出し
    /// （O(n²) 相当）へ退行し、#1318/#1319 が固定した O(n) 相当の契約を
    /// `Update` 経路だけ破ってしまう（レビュー指摘で判明）。本メソッドを
    /// 経由することで、`Update` のみが発生する構成（構造変化なしの純粋な
    /// 内容変更、実運用上最も典型的な keyed list 更新パターン）でも実 DOM
    /// 走査は初回 1 回に抑えられ、以降は `children` への添字/線形走査
    /// （実 DOM 呼び出しを伴わない純粋なメモリ操作）のみで完結する。
    fn ensure_children_cache(&mut self) {
        if self.children.is_some() {
            return;
        }
        let list_element = self.list_element;
        let mut items = Vec::new();
        let mut maybe_child = list_element.first_element_child();
        while let Some(child) = maybe_child {
            maybe_child = child.next_element_sibling();
            if let Some(key) = child.get_attribute(KEY_ATTR) {
                items.push((key, child));
            }
        }
        self.children = Some(crate::keyed_children_cache::KeyedChildrenCache::from_items(
            items,
        ));
    }
}

impl crate::keyed_apply::KeyedListDom for WebSysKeyedDom<'_> {
    type Handle = Element;
    type NewNode = web_sys::Node;

    fn first_element_child(&mut self) -> Option<Element> {
        self.list_element.first_element_child()
    }

    fn next_element_sibling(&mut self, child: &Element) -> Option<Element> {
        child.next_element_sibling()
    }

    fn item_key(&mut self, child: &Element) -> Option<String> {
        child.get_attribute(KEY_ATTR)
    }

    /// `children` キャッシュへの添字アクセスで `index` 番目の子要素を返す
    /// （イシュー #1319 codex-review 指摘対応、本 struct doc・
    /// [`crate::keyed_apply::KeyedListDom::child_at`] doc 参照）。未構築なら
    /// ここで 1 度だけ実 DOM を sibling 走査してキャッシュを埋める（この
    /// 走査自体は `first_element_child`/`next_element_sibling` の O(1) 操作
    /// を子要素数分だけ行う真の O(n) であり、以降の `child_at` 呼び出しは
    /// 実 DOM に触れない）。
    fn child_at(&mut self, index: usize) -> Option<Element> {
        self.ensure_children_cache();
        self.children.as_mut()?.get(index)
    }

    fn create_item(&mut self, key: &str) -> Option<web_sys::Node> {
        let (_, node) = self.new_items.iter().find(|(k, _)| k == key)?;
        build_dom_node_with_namespace(self.document, node, self.namespace)
    }

    /// 連続 Insert 区間（[`crate::keyed_apply::apply_ops`] が検出した
    /// `start_index` から連続する新規ノード列）を実 DOM へ適用する
    /// （イシュー #1320）。
    ///
    /// `items.len() == 1` の場合は従来どおり `list_element.insert_before`
    /// を直接 1 回呼ぶ（`DocumentFragment` 生成の分だけオーバーヘッドが
    /// 増える単発挿入・keyed list 更新シナリオでの退行を避けるための分岐、
    /// `KeyedListDom::insert_before_batch` doc の旧 `insert_before` と
    /// 同一挙動）。`items.len() >= 2` の場合は `Document::create_document_fragment`
    /// で構築した `DocumentFragment` へ各ノードを `append_child` し、
    /// `list_element.insert_before(&fragment, reference)` を 1 回だけ呼ぶ
    /// ことで、連続 Insert 区間 1 件あたりの JS 境界呼び出しを件数分から
    /// 1 回へ集約する（性能改善の本体、`_/bench` 変種実測で fragment 方式が
    /// 有意に高速だったことが動機、イシュー #1313 ベンチ起点）。
    /// `DocumentFragment` へ `append_child` した時点でその子はドキュメント
    /// ツリーから切り離される（DOM 標準仕様）ため、`items` に**新規構築
    /// ノードのみ**を渡す契約（[`crate::keyed_apply::KeyedListDom::insert_before_batch`]
    /// doc 参照）が破られると既存ノードのフォーカス・入力途中の値が失われる。
    /// 本メソッドは `create_item` が返した新規ノードのみを受け取る
    /// `apply_ops` からしか呼ばれないためこの契約は構造的に保たれる
    /// （既存ノードの移動は [`Self::move_before`] が個別に `list_element`
    /// 上で直接 `insert_before` するのみで、fragment を一切経由しない）。
    /// `set_inner_html`/`insert_adjacent_html` はここでも一切使わない
    /// （モジュール冒頭 doc 不変条件 1・2）。
    fn insert_before_batch(
        &mut self,
        start_index: usize,
        items: Vec<(String, web_sys::Node)>,
        reference: Option<&Element>,
    ) -> bool {
        let reference_web_node: Option<web_sys::Node> =
            reference.cloned().map(|el| el.unchecked_into());

        if items.len() == 1 {
            // `items.len() == 1` を確認済みのため `next()` は必ず `Some` だが、
            // panic 整形機構（`expect_failed`）を wasm へ引き込まないよう
            // 到達不能分岐は「DOM・キャッシュ無変更で false を返す」fail-closed
            // に倒す（イシュー #1388）。
            let Some((key, node)) = items.into_iter().next() else {
                return false;
            };
            if self
                .list_element
                .insert_before(&node, reference_web_node.as_ref())
                .is_err()
            {
                // 実 DOM への挿入自体が失敗（参照要素が既に親から外れて
                // いた等）。`node` は未挿入のまま破棄されるだけなので DOM
                // 側の後始末は不要だが、`children` キャッシュは一切更新
                // しない（トレイト契約「失敗時は DOM・内部キャッシュとも
                // 無変更」、イシュー #1340 codex-review P1〔3 巡目〕全走査
                // 対応）。
                return false;
            }
            self.cache_inserted_nodes(start_index, vec![(key, node)]);
            return true;
        }

        // `document.create_document_fragment()` は `set_inner_html` を
        // 経由しない構造的な DOM ノード集約手段であり、既定エスケープ
        // 迂回経路にはならない（挿入内容はいずれも `create_item` が
        // `create_element`/`create_text_node` で構築済みのノード）。
        let fragment = self.document.create_document_fragment();
        for (_, node) in &items {
            if fragment.append_child(node).is_err() {
                // fragment はまだ `list_element` に未接続（デタッチ状態）
                // のため、ここで失敗しても実 DOM（list_element 配下）は
                // 一切変更されていない。fragment ごと破棄して安全に
                // fail-closed できる（`children` キャッシュも更新しない）。
                return false;
            }
        }
        if self
            .list_element
            .insert_before(&fragment, reference_web_node.as_ref())
            .is_err()
        {
            // fragment の子ノードは仕様上まとめて `list_element` へ移動
            // する（`insert_before` は成功時に fragment を空にする）ため、
            // 失敗時は fragment・実 DOM のいずれにもノードが残らず消失
            // する（未挿入のまま破棄される点は単一要素パスと同じ）。
            // `children` キャッシュは一切更新しない。
            return false;
        }
        self.cache_inserted_nodes(start_index, items);
        true
    }

    fn move_before(
        &mut self,
        index: usize,
        key: &str,
        child: &Element,
        reference: Option<&Element>,
    ) -> bool {
        // 移動元と移動先参照が同一要素の場合、`insert_before` は no-op
        // （DOM 標準仕様: 挿入前に自身を除去してから挿入するため同一ノード
        // 指定は何も動かさない）。
        let reference_web_node: Option<web_sys::Node> =
            reference.cloned().map(|el| el.unchecked_into());
        let existing_web_node: web_sys::Node = child.clone().unchecked_into();
        if self
            .list_element
            .insert_before(&existing_web_node, reference_web_node.as_ref())
            .is_err()
        {
            // 実 DOM への移動自体が失敗。`insert_before` は単一の DOM
            // 境界操作であり、失敗時は呼び出し前の状態（`child` は移動前の
            // 位置のまま）から一切変わらない（DOM 標準仕様）ためロール
            // バックは不要。`children` キャッシュも一切更新しない（イシュー
            // #1340 codex-review P1〔3 巡目〕全走査対応）。
            return false;
        }
        if let Some(cache) = self.children.as_mut() {
            // キャッシュ内の旧位置を `key` の文字列比較で特定し、新しい
            // 位置へ挿入し直す（実 DOM 呼び出しを一切伴わない純粋なメモリ
            // 操作。ブラウザ API の計算量に依存しないという `child_at` の
            // 契約を、この追随更新側でも維持するための設計。
            // [`crate::keyed_children_cache::KeyedChildrenCache::move_to`]
            // doc 参照）。
            cache.move_to(key, index, child.clone());
        }
        true
    }

    /// `key` 一致エントリを `children` キャッシュから実 DOM 再問い合わせ
    /// なしに除去する（イシュー #1374。旧実装は成功・失敗いずれでも
    /// キャッシュを丸ごと `None` へ無効化していたが、これは「Remove は
    /// キャッシュ構築前にのみ呼ばれる」前提に依存しており、
    /// [`KeyedListDom::find_by_key`]（`Remove`/`Move`/`Update` の対象解決を
    /// 共通で担う、`keyed_apply::KeyedListDom::find_by_key` doc 参照）の
    /// 導入でこの前提は崩れた。丸ごと無効化のまま維持すると、全削除
    /// ワークロードで「Remove 1 件ごとにキャッシュを O(n) 再構築」を
    /// 繰り返し O(N²) を再導入してしまう（親イシュー #1371 実測起点）。
    ///
    /// PR #1392 codex-review P1 是正: `children.remove(pos)`（`Vec` への
    /// 直接除去）に置き換えた版も、全削除ワークロードでは対象が常に
    /// 先頭（`pos == 0`）になるため `Vec::remove(0)` が残り全要素をシフト
    /// し、合計の要素移動量が O(N²) へ退行していた（呼び出し回数自体は
    /// O(N) のままのため、呼び出し回数だけを数える固定テストでは検知
    /// できない）。[`crate::keyed_children_cache::KeyedChildrenCache::remove`]
    /// （tombstone 化 + 遅延 compact、モジュール doc「計算量保証」参照）へ
    /// 委譲することで、この経路の要素移動量を amortized O(N) へ是正した。
    fn remove_child(&mut self, key: &str, child: &Element) -> bool {
        if self.list_element.remove_child(child).is_err() {
            // 実 DOM への削除自体が失敗（`child` が既に `list_element` の
            // 子でない等）。DOM 標準上 `removeChild` 失敗時は no-op であり
            // `child` は実 DOM 上に残ったままのため、キャッシュも無変更の
            // まま `false` を返す（トレイト契約「失敗時は DOM・内部
            // キャッシュとも無変更」、イシュー #1340 codex-review P1
            // 〔3 巡目〕全走査対応を `remove_child` にも一貫適用）。
            return false;
        }
        if let Some(cache) = self.children.as_mut() {
            cache.remove(key);
        }
        true
    }

    /// `Node::set_text_content(None)` 1 回で `list_element` の全子ノードを
    /// 取り除く（イシュー #1373。keyed list 全キー削除の一括 clear 経路、
    /// [`crate::keyed_apply::KeyedListDom::clear_children`] doc 参照）。
    ///
    /// `textContent` への `None`（DOM 標準では空文字列と等価、
    /// [MDN](https://developer.mozilla.org/docs/Web/API/Node/textContent)
    /// 参照）代入は、要素・テキストを問わず全子ノードを除去し新規テキスト
    /// ノードも追加しない仕様であるため、`list_element.first_element_child()`
    /// を回しながら `remove_child` を N 回呼ぶ per-item フォールバック
    /// （`crate::keyed_apply::keyed_apply_tests::DefaultClearDom` 参照。
    /// このフォールバックは要素のみを走査するため、`KeyedListDom::clear_children`
    /// は既定実装を持たず全実装に完全な全ノード除去を要求する契約へ
    /// 変更済み〔イシュー #1373 codex-review P2〕）と等価な結果を、
    /// N 回の JS 境界呼び出しではなく 1 回で
    /// 達成できる（lit / vue / js-framework-benchmark 上位実装と同型の
    /// 定石、`textContent = ""`）。`Element::insert_before`/`remove_child`
    /// を個々に組み合わせる `replace_children`（可変長引数の分割メソッド
    /// になり煩雑）よりも意味が明確で `web-sys` の型付けとも相性が良い
    /// ため採用した。
    ///
    /// `set_text_content` は `web-sys` 上 infallible（`Result` を返さない）
    /// のため常に成功として `true` を返し、内部索引キャッシュ（`children`）
    /// を空 `Vec` へ確定する。他メソッド（`remove_child`/`insert_before_batch`
    /// 等）が守る「キャッシュ更新は完全成功時のみ」契約と矛盾しない
    /// （本メソッドに失敗しうる分岐が存在しないため）。
    fn clear_children(&mut self) -> bool {
        self.list_element.set_text_content(None);
        match self.children.as_mut() {
            Some(cache) => cache.clear(),
            None => {
                self.children = Some(crate::keyed_children_cache::KeyedChildrenCache::from_items(
                    Vec::new(),
                ))
            }
        }
        true
    }

    /// `Element::tag_name()`（DOM 標準 API、常に大文字化された値を返す
    /// 仕様）を ASCII 小文字化して返す（イシュー #1340 codex-review
    /// P1/Bugbot〔10 巡目〕対応、`KeyedListDom::tag_name` doc 参照）。
    fn tag_name(&mut self, child: &Element) -> String {
        child.tag_name().to_ascii_lowercase()
    }

    /// `child` の属性を `old_attrs`（呼び出し元 `keyed_apply::apply_ops_with_items`
    /// が渡す、直前に反映済みの `reserved_attr` 除外済み属性集合）から
    /// `new_attrs`（同じく `reserved_attr` 除外済みの新しい属性集合）へ
    /// 同期する（イシュー #1324）。属性の追加・更新は
    /// [`build_dom_node_with_namespace`] と同一の URL スキーム・
    /// イベントハンドラ・`srcset` 検証を経由する（不変条件 1〜4 の Update
    /// 経路への継承）。
    ///
    /// # Result 破棄の正当化と読み戻しによる ground truth 取得
    /// （イシュー #1340 codex-review P1〔3 巡目〕・〔5 巡目〕対応）
    ///
    /// 内部の `remove_attribute`/`set_attribute` 呼び出しは個々の戻り値
    /// （`Result`）に基づく分岐（逐次ロールバック）を行わない。これは
    /// [`crate::keyed_apply::KeyedListDom::sync_attrs`] のトレイト doc・
    /// 本クレート `keyed_apply` モジュール doc「Update op の DOM 適用」に
    /// 明記された設計判断の実装側の反映であり見落としではない:
    /// `setAttribute`/`removeAttribute` は不正な引数に対して通常
    /// `Err`/例外を投げない DOM 標準 API であるため、属性 1 件ごとの
    /// 逆順ロールバック機構は実装・検証コストに見合わないと判断した
    /// （設計書 §6 不変条件 6 が要求する完全なロールバックの対象外として
    /// 明示的に許容された残余リスク）。
    ///
    /// ただし `Node::Element` はタグ名と異なり属性名を構築時に検証しない
    /// ため、`setAttribute` が実行時に `InvalidCharacterError` 相当の
    /// `Err` を返す余地は残る。この失敗（および `removeAttribute` の
    /// 失敗）を無視して「新属性へ更新済み」と扱うと、`sync_attrs` の
    /// 呼び出し元（`crate::keyed_apply::apply_ops_with_items` 経由で
    /// `crate::keyed_apply::compose_achieved_children` が合成する「達成
    /// Node」）が実 DOM の実際の状態と乖離したままキャッシュされてしまう
    /// （イシュー #1340 codex-review P1〔5 巡目〕指摘）。この失敗は
    /// `(属性名, 属性値)` のみからは決定できない実行時の事実であり、
    /// 呼び出し元でポリシー判断のように再計算することができない。
    ///
    /// # 削除判定の基準（イシュー #1340 codex-review Bugbot〔8 巡目〕対応）
    ///
    /// 削除対象は `old_attrs`（キャッシュされた直前の内容）ではなく
    /// **ライブ要素の実属性列挙**（`child.attributes()`/`NamedNodeMap`）
    /// から決定する。SSR hydrate 直後はサーバー側 HTML に含まれる属性が
    /// ライブ DOM 上に存在する一方、クライアント側の `old_attrs`
    /// キャッシュにはその属性が含まれない構成があり得るため（hydrate 由来
    /// のライブ属性ドリフト）、`old_attrs` のみを基準にすると `new_attrs`
    /// にも無いその属性が Update を何度適用しても永遠に除去されない
    /// （codex-review 指摘、旧実装のバグ）。属性名の一致判定は ASCII
    /// 大小文字を区別しない（HTML 文書の属性名はライブ DOM 上で小文字化
    /// されて列挙されるため）。
    ///
    /// # 読み戻しの決定的正規化（イシュー #1340 codex-review Bugbot〔6 巡目〕対応）
    ///
    /// 戻り値の**合成**（値の取得・格納）は `child.attributes()` の生の
    /// 列挙順・大小文字表記を一切使わず、[`crate::keyed_apply::KeyedListDom::sync_attrs`]
    /// doc「決定的な正規化契約」の手順どおり `new_attrs`/`old_attrs` にある
    /// 属性名だけを `get_attribute` で個別照会して合成する（削除**判定**に
    /// ライブ列挙を使うことと矛盾しない。詳細は同 doc 参照）。全操作成功時
    /// は戻り値が `new_attrs` とバイト等価になり、失敗時のみ実際の DOM
    /// 状態が反映される。
    ///
    /// # 同値スキップ（イシュー #1382）
    ///
    /// set ループでは、`new_attrs` の各エントリが `old_attrs`（達成 Node
    /// キャッシュ由来、直前 tick の実 DOM 読み戻し値 = ground truth）と
    /// 名前（ASCII 大小文字非区別）・値（バイト厳密）とも同値の場合、
    /// `set_attribute` の wasm→JS 境界呼び出しを省略する
    /// （[`crate::keyed_apply::attr_value_unchanged`] が判定を担う）。
    /// 判定は予約属性チェックの直後・イベントハンドラ / URL / `srcset`
    /// の書き込み検証より前に置く: スキップは書き込みを一切行わない
    /// 判定であり、実際に書き込む経路は従来どおり全検証を通るため
    /// 検証バイパスにはならない。
    ///
    /// **適用対象からのイベントハンドラ / URL / `srcset` 属性の除外**
    /// （イシュー #1382 codex-review P0 対応）: `old_attrs` キャッシュは
    /// あくまで直前 tick の読み戻し値であり、ライブ DOM の**現在**の値を
    /// 保証しない。外部スクリプト等がキャッシュ・新 view の値を変えずに
    /// ライブ属性だけを `javascript:` 等の危険値へ直接書き換えた場合、
    /// 同値スキップは「変わっていない」という誤った前提で危険なライブ値
    /// をそのまま放置してしまう（REQ-1 既定エスケープの弱体化）。この
    /// リスクを避けるため、イベントハンドラ属性・URL 属性・`srcset` の
    /// 3 カテゴリは同値スキップの対象外とし、毎 tick 従来どおり検証・
    /// 書き込み経路を通す（安全なら安全値を書き戻して同一 tick で自己
    /// 修復する）。同値スキップは `class` 等それ以外の属性にのみ適用される。
    ///
    /// スキップ後も読み戻し（下記の決定的正規化）は変更しないため、
    /// 外部コードによるライブ値ドリフト（テストや他スクリプトが直接
    /// `setAttribute` した場合等）があっても achieved には実 DOM の値が
    /// 反映され、次 tick の diff で自己修復される（ground truth 契約は
    /// 崩さない）。`old_attrs` が空（cache-miss フォールバック経路、
    /// hydrate 直後の属性ドリフト是正のための初回適用）ではスキップは
    /// 一切発動しない。
    fn sync_attrs(
        &mut self,
        child: &Element,
        reserved_attr: &str,
        old_attrs: &[(String, String)],
        new_attrs: &[(String, String)],
    ) -> Vec<(String, String)> {
        // 削除判定: ライブ要素の実属性列挙から `reserved_attr`・
        // `new_attrs` 側の名前を除いたものを削除候補とする（`old_attrs` は
        // 削除判定に使わない、上記 doc「削除判定の基準」参照）。
        let attributes = child.attributes();
        let len = attributes.length();
        let mut removal_candidates: Vec<String> = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(attr) = attributes.item(i) {
                let name = attr.name();
                if name.eq_ignore_ascii_case(reserved_attr) {
                    continue;
                }
                if new_attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(&name)) {
                    continue;
                }
                removal_candidates.push(name);
            }
        }
        for name in &removal_candidates {
            let _ = child.remove_attribute(name);
        }

        for (name, value) in new_attrs {
            if name.eq_ignore_ascii_case(reserved_attr) {
                // 多層防御: 呼び出し元が既に除外済みの前提だが、万一
                // 予約属性が紛れ込んでも書き込まない。
                continue;
            }
            // セキュリティ上重要な属性カテゴリ（イベントハンドラ / URL /
            // `srcset`）は同値スキップの対象から除外する（イシュー #1382
            // codex-review P0 対応、上記 doc「同値スキップ」参照）。
            // `old_attrs` キャッシュは直前 tick の読み戻し値であり、外部
            // スクリプト等がライブ DOM を直接 `setAttribute` で書き換えた
            // 場合（キャッシュ・新 view の値は変わらないままライブ値だけが
            // ドリフトするケース）を検知できない。危険なライブ値
            // （`javascript:` スキーム等）がキャッシュと新 view の値の一致
            // だけを根拠にスキップされると、同一 tick での自己修復が失われ
            // 少なくとも次 tick まで残存してしまう（REQ-1 既定エスケープの
            // 弱体化）。このためこれら 3 カテゴリは常に検証・書き込み経路を
            // 通し、安全な場合は毎 tick 安全値を書き戻して即時修復する
            // 従来の挙動を維持する。同値スキップはそれ以外の属性
            // （`class` 等）にのみ適用される。
            let is_security_sensitive = fandhe_frontend_core::is_event_handler_attr(name)
                || fandhe_frontend_core::is_url_attr(name)
                || name.eq_ignore_ascii_case("srcset");
            if !is_security_sensitive
                && crate::keyed_apply::attr_value_unchanged(old_attrs, name, value)
            {
                // 同値スキップ（イシュー #1382、上記 doc「同値スキップ」
                // 参照）: 書き込みを一切行わない判定であり、検証
                // バイパスにはならない。
                continue;
            }
            if fandhe_frontend_core::is_event_handler_attr(name) {
                continue;
            }
            if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value)
            {
                continue;
            }
            if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
                continue;
            }
            let _ = child.set_attribute(name, value);
        }

        // 決定的な正規化契約（上記 doc 参照）: `new_attrs` にある属性名だけ
        // を個別照会して合成する。
        let mut achieved: Vec<(String, String)> = Vec::with_capacity(new_attrs.len());
        for (name, value) in new_attrs {
            if name.eq_ignore_ascii_case(reserved_attr) {
                continue;
            }
            if let Some(actual) = child.get_attribute(name) {
                achieved.push((
                    name.clone(),
                    if &actual == value {
                        value.clone()
                    } else {
                        actual
                    },
                ));
            }
        }
        // 残存（削除失敗）属性: `old_attrs` に同名エントリがあるものを
        // `old_attrs` の順序で先に並べ、それ以外（hydrate 由来のライブ
        // 専用属性）は `removal_candidates` の列挙順で末尾に追加する
        // （`KeyedListDom::sync_attrs` doc「決定的な正規化契約」手順 2）。
        let mut residual_order: Vec<String> = Vec::new();
        for (name, _) in old_attrs {
            if removal_candidates.contains(name) && !residual_order.contains(name) {
                residual_order.push(name.clone());
            }
        }
        for name in &removal_candidates {
            if !residual_order.contains(name) {
                residual_order.push(name.clone());
            }
        }
        for name in &residual_order {
            if let Some(actual) = child.get_attribute(name) {
                achieved.push((name.clone(), actual));
            }
        }
        achieved
    }

    /// `child` の子ノード列を `new_children`（`fandhe_frontend_core::Node`
    /// 列）へ差し替える（イシュー #1324、設計書 §3.2 c 案の簡略実装。
    /// コミットフェーズの失敗時ロールバックはイシュー #1340 codex-review
    /// P1（2 巡目）指摘対応、`docs/design/keyed-update-op-design.md` §6
    /// 不変条件 6 の「子ノード交換」規定を実装する）。
    ///
    /// # 構築フェーズ
    ///
    /// 新しい子ノードを**先に構築**（`document.create*` のみで detached、
    /// まだ `child` へ append しない）し、1 件でも構築に失敗（`RawHtml`
    /// 混入等、[`build_dom_node_with_namespace`] が `None` を返すケース）
    /// した場合は構築済みの detached ノードを（DOM へ一切未挿入のまま）
    /// 破棄し、ライブ DOM を変更せず `false` を返す（fail-closed、旧稿から
    /// 不変）。
    ///
    /// # コミットフェーズ（構造的原子性、設計書 §6 不変条件 6）
    ///
    /// `remove_child`/`append_child` の戻り値 `Result` を検査し、途中で
    /// 失敗した場合は設計書 §6 不変条件 6 が規定する手順で**ライブ root
    /// 要素の子ノード列の構造**を Update 適用開始前の状態へ復元する
    /// （旧実装は戻り値を `let _ =` で握りつぶし、部分適用〔一部だけ削除・
    /// 追加された DOM〕を `true` として呼び出し元へ返してしまっていた
    /// codex-review 指摘）:
    ///
    /// - 旧子ノード `i` 件目の `remove_child` が失敗した場合、既に取り外し
    ///   済みの `0..i` 件を、ルート要素に残っている未取り外し suffix の
    ///   先頭（`i` 件目のノード、`remove_child` 失敗により付いたまま）の
    ///   直前へ `insert_before` で元の順序のまま再度取り付ける
    ///   （`append_child`〔末尾追加〕では suffix の後ろへ回り込み元の順序が
    ///   壊れるため使わない）。
    /// - 旧子ノードの取り外しをすべて終えた後、新子ノード `j` 件目の
    ///   `append_child` が失敗した場合、追加済み新子ノード `0..j` を取り除き、
    ///   保持しておいた旧子ノード列を元の順序で再度取り付ける。
    ///
    /// 属性適用（`sync_attrs`）は本メソッドの**後**にのみ呼ばれる契約
    /// （呼び出し元 `apply_ops_with_items` の `KeyedOp::Update` 処理順序、
    /// イシュー #1340 codex-review P1〔1 巡目〕対応）のため、本メソッド
    /// 実行時点で属性はまだ Update 前の値のままであり、設計書 §6 不変条件 6
    /// が個別に規定する「属性適用のロールバック」は本メソッドの守備範囲
    /// ではない（呼び出し元がそもそも属性へ触れる前に本メソッドの結果で
    /// 分岐するため）。
    ///
    /// ロールバック自体（`insert_before`/`remove_child`/`append_child` の
    /// 逆操作）が失敗する残余リスクは設計書 §6 不変条件 6「残る有限の
    /// リスク」として明示的に許容し、`unwrap()`/`panic!` は使わず固定英語
    /// 文言の警告ログ（不変条件 7、キー値・アイテム内容を含めない）を出して
    /// 処理を継続する（ベストエフォートの復元）。
    fn replace_item_children(&mut self, child: &Element, new_children: &[Node]) -> bool {
        let mut built: Vec<web_sys::Node> = Vec::with_capacity(new_children.len());
        for new_child in new_children {
            match build_dom_node_with_namespace(self.document, new_child, self.namespace) {
                Some(node) => built.push(node),
                None => return false,
            }
        }

        // コミットフェーズの走査本体（構造的原子性のロールバック含む）は
        // `crate::keyed_apply::exchange_children` へ切り出し済み
        // （`ChildExchangeDom` trait doc・native テスト
        // `crate::keyed_apply::tests` 参照）。本メソッドはそれを `child`
        // へ適用する薄いアダプタに徹する。
        let mut exchange = ElementChildExchange { parent: child };
        crate::keyed_apply::exchange_children(&mut exchange, &built)
    }

    /// `children` キャッシュ（[`Self::ensure_children_cache`]、`child_at`
    /// と共有）から `key` の既存要素の「現在位置とハンドル」を解決する
    /// （イシュー #1324/#1374、
    /// [`crate::keyed_apply::KeyedListDom::find_by_key`] doc 参照）。実体は
    /// [`crate::keyed_children_cache::KeyedChildrenCache::find`] へ委譲する
    /// （tombstone を跨いだ前方走査 + フォールバックの compact、同モジュール
    /// doc「計算量保証」参照。PR #1392 codex-review P1 是正で単純な
    /// `Vec::position` 全走査から置き換えた）。キャッシュ未構築時はここで
    /// 初めて実 DOM を 1 度だけ sibling 走査する（`Update`/`Remove`/`Move`
    /// のみが発生する構成、すなわち `Insert` が 1 件も無く `child_at` が
    /// 未呼び出しのケースでも、実 DOM 走査は高々 1 回に抑えられる契約を
    /// ここで担保する）。イシュー #1374 で `Remove`/`Move` の対象解決（旧
    /// `find_child_by_key` の sibling 走査）もここへ統合され、全削除
    /// ワークロードの O(N²) 退行を解消した。
    fn find_by_key(&mut self, key: &str) -> Option<(usize, Element)> {
        self.ensure_children_cache();
        self.children.as_mut()?.find(key)
    }

    /// `new`（[`crate::keyed_apply::KeyedListDom::create_item`] が構築済みの
    /// 新規ノード）を `old` の直前へ挿入したうえで `old` を取り除く（イシュー
    /// #1340 codex-review P1〔2 巡目〕対応、`replace_root` トレイト doc
    /// 参照）。`old` 自身を参照ノードとして使うため `child_at`（index 解決）
    /// は不要 —— `list_element.insert_before(new, Some(old))` は「`old` の
    /// 直前」を意味し、続けて `old` を `remove_child` することで両者の
    /// 相対位置を保ったまま置き換わる（DOM 標準の `insert_before`/
    /// `remove_child` 呼び出し 2 回、`set_inner_html`/`insert_adjacent_html`
    /// は使わない、モジュール冒頭 doc 不変条件 1・2）。
    ///
    /// `children` キャッシュ（構築済みの場合）は完全成功時のみ `old` の
    /// エントリを `key` で特定し `new` へ差し替える（[`Self::move_before`]
    /// と同様、実 DOM を再度問い合わせない純粋な `Vec` 走査で追随更新
    /// する）。`new` は [`crate::keyed_apply::KeyedListDom::create_item`] が
    /// `build_dom_node_with_namespace` で `Node::Element` から構築した要素
    /// ノードである契約（`create_item` doc 参照）のため `Element` への
    /// ダウンキャストは安全。
    ///
    /// `insert_before`/`remove_child` の `Result` を検査し、部分適用時は
    /// [`crate::keyed_apply::KeyedListDom::replace_root`] doc「戻り値と
    /// 部分失敗時の契約」（イシュー #1340 codex-review P1〔3 巡目〕対応）
    /// が規定する手順でロールバックする（codex-review 指摘: 旧実装は
    /// 両呼び出しの `Result` を `let _ =` で握りつぶし、挿入失敗後も
    /// `old` を削除してキーが消滅する・挿入成功後の削除失敗で同一キー
    /// 要素が重複する、のいずれの部分適用でも無条件に「達成」として
    /// `children` キャッシュを更新してしまっていた）。
    fn replace_root(&mut self, old: &Element, key: &str, new: web_sys::Node) -> bool {
        let old_as_node: web_sys::Node = old.clone().unchecked_into();
        // コミット手順の走査本体（部分失敗時のロールバック含む）は
        // `crate::keyed_apply::replace_root_node` へ切り出し済み
        // （`RootReplaceDom` trait doc・native テスト
        // `crate::keyed_apply::tests` 参照）。本メソッドはそれを
        // `list_element` へ適用する薄いアダプタに徹する。
        let mut adapter = ListElementRootReplace {
            list_element: self.list_element,
        };
        if !crate::keyed_apply::replace_root_node(&mut adapter, &old_as_node, &new) {
            return false;
        }
        if let Some(cache) = self.children.as_mut() {
            let new_element: Element = new.unchecked_into();
            cache.replace(key, new_element);
        }
        true
    }
}

/// [`crate::keyed_apply::ChildNodeDom`] の `web-sys` 実装（イシュー #1381、
/// `KeyedOp::Update` 適用の子ノード最小差分化）。
///
/// アイテムルート要素だけでなく [`crate::keyed_apply::diff_children`] が
/// 降りる任意深さの Element ノードへも同じ `Handle`（`Element`）で
/// 再入力される（`ChildNodeDom: KeyedListDom` の supertrait 契約、トレイト
/// doc 参照）。`children` キャッシュ（`KeyedListDom` の他メソッドが使う
/// アイテムルート直下専用の索引）はここでは一切使わない
/// （`child_handles` は毎回 `Node::childNodes` を直接問い合わせる。
/// `diff_children` 自身が「書き込み単位ごとに再検証する」契約
/// （設計書 §3.2a）を満たすためにこのメソッドを何度も呼び直すため、
/// アイテムルート専用キャッシュを流用すると子孫スコープの解決と整合しない）。
impl crate::keyed_apply::ChildNodeDom for WebSysKeyedDom<'_> {
    type ChildHandle = web_sys::Node;

    fn child_handles(&mut self, parent: &Element) -> Vec<web_sys::Node> {
        let list = parent.child_nodes();
        let len = list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = list.get(i) {
                out.push(node);
            }
        }
        out
    }

    /// `Node::nodeType`（DOM 標準の数値定数、`web_sys::Node::TEXT_NODE`/
    /// `ELEMENT_NODE` 等）で種別判定する。`Text`/`Element` のいずれにも
    /// 該当しない値（コメントノード `COMMENT_NODE` 等）は `Other` を返し、
    /// 誤分類・パニックのいずれも行わない（[`ChildNodeKind`] doc 参照）。
    fn child_kind(&mut self, node: &web_sys::Node) -> crate::keyed_apply::ChildNodeKind {
        match node.node_type() {
            web_sys::Node::TEXT_NODE => crate::keyed_apply::ChildNodeKind::Text,
            web_sys::Node::ELEMENT_NODE => crate::keyed_apply::ChildNodeKind::Element,
            _ => crate::keyed_apply::ChildNodeKind::Other,
        }
    }

    fn as_element(&mut self, node: &web_sys::Node) -> Option<Element> {
        node.clone().dyn_into::<Element>().ok()
    }

    /// `CharacterData.data` への文字列代入（`set_data`、REQ-1 既定
    /// エスケープの不変条件参照: HTML パースを一切経由しない代入であり
    /// `innerHTML`/`insertAdjacentHTML` の新設・使用は行わない）。DOM 標準
    /// 上この代入自体は失敗しないが、`node` が `CharacterData` へ変換
    /// できない（`child_kind` が `Text` を返したにもかかわらず実際には
    /// 異なる、あり得ない乖離）場合のみ `false` を返す。
    fn set_text_data(&mut self, node: &web_sys::Node, value: &str) -> bool {
        match node.dyn_ref::<web_sys::CharacterData>() {
            Some(cd) => {
                cd.set_data(value);
                true
            }
            None => false,
        }
    }

    fn text_data(&mut self, node: &web_sys::Node) -> String {
        node.dyn_ref::<web_sys::CharacterData>()
            .map(|cd| cd.data())
            .unwrap_or_default()
    }
}

impl WebSysKeyedDom<'_> {
    /// [`crate::keyed_apply::KeyedListDom::insert_before_batch`] 実装が
    /// 実 DOM へ適用した挿入結果を `children` キャッシュへ追随させる
    /// （キャッシュ未構築なら no-op、[`Self::child_at`] doc 参照。トレイト
    /// 非公開のヘルパーのため本 struct の inherent メソッドとして持つ）。
    fn cache_inserted_nodes(&mut self, start_index: usize, items: Vec<(String, web_sys::Node)>) {
        let Some(cache) = self.children.as_mut() else {
            return;
        };
        for (pos, (key, node)) in (start_index..).zip(items) {
            // `node` は `build_dom_node_with_namespace` が `Node::Element`
            // から構築した要素ノード（`create_item` の契約、`keyed_dom`
            // モジュール doc 不変条件 4 参照）であり `Element` へのダウン
            // キャストは安全。以降は実 DOM を問い合わせない純粋なメモリ
            // 操作のみでキャッシュを追随させる
            // （[`crate::keyed_children_cache::KeyedChildrenCache::insert`]
            // が挿入先位置を現在の生存件数でクランプするため、ここでの
            // 事前クランプは不要）。
            let element: Element = node.unchecked_into();
            cache.insert(pos, key, element);
        }
    }
}

/// [`crate::keyed_apply::ChildExchangeDom`] の `web-sys` 実装アダプタ
/// （イシュー #1340 codex-review P1〔2 巡目〕対応）。`parent`（keyed list
/// アイテムのルート要素）配下の子ノード列に対する `remove_child`/
/// `insert_before` を、走査本体（[`crate::keyed_apply::exchange_children`]）
/// から呼べる薄い形へ委譲するだけの構造体（`WebSysKeyedDom` と同じ「実
/// DOM 呼び出しをトレイトメソッドへ 1:1 で委譲するだけ」の方針）。
struct ElementChildExchange<'a> {
    parent: &'a Element,
}

impl crate::keyed_apply::ChildExchangeDom for ElementChildExchange<'_> {
    type Node = web_sys::Node;

    fn current_children(&mut self) -> Vec<web_sys::Node> {
        let mut out = Vec::new();
        let mut cursor = self.parent.first_child();
        while let Some(node) = cursor {
            cursor = node.next_sibling();
            out.push(node);
        }
        out
    }

    fn remove_child(&mut self, node: &web_sys::Node) -> bool {
        self.parent.remove_child(node).is_ok()
    }

    fn insert_before(&mut self, node: &web_sys::Node, reference: Option<&web_sys::Node>) -> bool {
        self.parent.insert_before(node, reference).is_ok()
    }

    fn on_rollback_failed(&mut self) {
        warn_replace_item_children_rollback_failed();
    }
}

/// [`crate::keyed_apply::RootReplaceDom`] の `web-sys` 実装アダプタ
/// （イシュー #1340 codex-review P1〔3 巡目〕対応）。`list_element`
/// （keyed list の親要素）に対する `insert_before`/`remove_child` を、
/// 走査本体（[`crate::keyed_apply::replace_root_node`]）から呼べる薄い形へ
/// 委譲するだけの構造体（`ElementChildExchange`/`WebSysKeyedDom` と同じ
/// 「実 DOM 呼び出しをトレイトメソッドへ 1:1 で委譲するだけ」の方針）。
struct ListElementRootReplace<'a> {
    list_element: &'a Element,
}

impl crate::keyed_apply::RootReplaceDom for ListElementRootReplace<'_> {
    type Node = web_sys::Node;

    fn insert_before(&mut self, new: &web_sys::Node, old: &web_sys::Node) -> bool {
        self.list_element.insert_before(new, Some(old)).is_ok()
    }

    fn remove(&mut self, node: &web_sys::Node) -> bool {
        self.list_element.remove_child(node).is_ok()
    }

    fn on_rollback_failed(&mut self) {
        warn_replace_root_rollback_failed();
    }
}

/// [`crate::keyed_apply::RootReplaceDom`] の `web-sys` 実装アダプタ
/// （親要素自身のタグが変わる更新専用、[`replace_list_element_for_tag_change`]
/// から呼ばれる。イシュー #1340 codex-review P1〔9 巡目〕対応）。
///
/// [`ListElementRootReplace`] と同型だが、コンテナが `list_element`
/// 自身ではなく**その親**（`Element::parent_node()` の戻り値）である点が
/// 異なる: `list_element` 自身のタグは置換できないため、置換操作は
/// `list_element` の親から見た「子の入れ替え」として行う必要がある。
/// `insert_before`/`remove_child` はいずれも `Node` 自体のメソッド
/// （`Element` はこれを継承する）であるため、コンテナを `Element` へ
/// ダウンキャストする必要はない。
struct ParentNodeRootReplace<'a> {
    parent: &'a web_sys::Node,
}

impl crate::keyed_apply::RootReplaceDom for ParentNodeRootReplace<'_> {
    type Node = web_sys::Node;

    fn insert_before(&mut self, new: &web_sys::Node, old: &web_sys::Node) -> bool {
        self.parent.insert_before(new, Some(old)).is_ok()
    }

    fn remove(&mut self, node: &web_sys::Node) -> bool {
        self.parent.remove_child(node).is_ok()
    }

    fn on_rollback_failed(&mut self) {
        warn_replace_root_rollback_failed();
    }
}

/// `list_element` 自身のタグが変わる更新（[`apply_keyed_list_with_previous`]
/// の親タグ不一致検出）を、`list_element` を丸ごと新規要素へ置き換える
/// ことで表現する（イシュー #1340 codex-review P1〔9 巡目〕対応）。
///
/// `Element.tagName` は DOM 標準仕様上不変であり、`list_element` 自身の
/// in-place 更新ではタグを変更できない。子アイテムの `KeyedOp::Update`
/// タグ変更時に [`WebSysKeyedDom::replace_root`] が行う「新要素を
/// detached で構築 → 旧要素の直前へ挿入 → 旧要素を削除」と同じ流儀を、
/// `list_element` 自身とその親（`parent_node()`）に対して適用する。
///
/// - `list_element` が既にライブツリーから外れている（`parent_node()` が
///   `None`）場合は置換できないため、DOM に一切触れず `ResyncRequired` を
///   返す（fail-closed）。
/// - `new_list_node`（親要素・全子アイテムを含む部分木全体）の構築に
///   失敗した場合（`RawHtml` 混入等、[`build_dom_node_with_namespace`] が
///   `None` を返すケース）も、旧 `list_element` には一切触れず
///   `ResyncRequired` を返す（`create_item`/`replace_root` と同じ既存
///   契約）。
/// - 挿入・削除のいずれかが失敗した場合（[`crate::keyed_apply::replace_root_node`]
///   がベストエフォートでロールバックを試みる）も `ResyncRequired` を
///   返す（部分適用状態を「達成」としてキャッシュしない）。
/// - 完全に成功した場合のみ、新しく構築した部分木全体を
///   [`crate::keyed_apply::sanitize_node_for_achieved`]（検証拒否属性を
///   丸ごと除外する新規構築経路のポリシー再計算、本クレート `keyed_apply`
///   モジュール冒頭 doc「属性検証拒否と「達成 Node」の整合」参照）へ通した
///   ものを「達成 Node」として返す。子アイテムは `apply_ops_with_items`/
///   `sync_attrs` を一切経由しない（丸ごと新規構築のため
///   `stale_update_keys`/`achieved_attrs` は無関係）。
fn replace_list_element_for_tag_change(
    document: &Document,
    list_element: &Element,
    new_list_node: &Node,
) -> KeyedListApplyResult {
    let Some(parent) = list_element.parent_node() else {
        // DOM に一切触れていない（`parent_node()` の読み出しのみ）。
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    };

    let namespace = list_element.namespace_uri();
    let Some(new_container) =
        build_dom_node_with_namespace(document, new_list_node, namespace.as_deref())
    else {
        // detached 構築の失敗のみ（ライブ DOM への書き込みは未試行）。
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    };

    let old_as_node: web_sys::Node = list_element.clone().unchecked_into();
    let mut adapter = ParentNodeRootReplace { parent: &parent };
    if !crate::keyed_apply::replace_root_node(&mut adapter, &old_as_node, &new_container) {
        // `replace_root_node` はライブ DOM への `insert_before`/`remove` を
        // 試行済み（成否は不問、`dom_mutated` は試行基準）。
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: true,
        };
    }

    // 独立敵対レビュー指摘 A（イシュー #1340）対応: `new_list_node` の
    // アイテム（子）の子孫に別 field の keyed list マーカーがあれば、その
    // ライブ DOM もこの丸ごと構築で新しい状態になっている
    // （`KeyedListApplyResult::Achieved::invalidated_nested_fields` doc
    // 参照）。`new_list_node` 自身（現在処理中の field 自身のマーカーを
    // 持つ）は対象に含めない（現在処理中の field は呼び出し元がこの
    // 戻り値の `node` で直接キャッシュ更新するため自己無効化は不要かつ
    // 有害）ため、子要素（アイテム）から走査する。
    let mut invalidated_nested_fields = std::collections::HashSet::new();
    if let Node::Element { children, .. } = new_list_node {
        for child in children {
            invalidated_nested_fields
                .extend(crate::keyed_apply::collect_nested_bind_list_fields(child));
        }
    }

    KeyedListApplyResult::Achieved {
        node: crate::keyed_apply::sanitize_node_for_achieved(new_list_node),
        invalidated_nested_fields,
    }
}

/// [`apply_keyed_list`]/[`apply_keyed_list_with_previous`] の共通実装本体
/// （イシュー #1340 codex-review P1/Bugbot〔10 巡目〕対応）。
///
/// 親タグ判定を**ライブ実タグ**（`list_element.tag_name()`）基準で行う
/// （`old_parent_attrs`/`old_items` の由来がキャッシュ〔with-previous〕か
/// ライブ由来のプレースホルダ〔cache-miss フォールバック〕かに関わらず
/// 常に同じ判定基準になる）。これにより:
///
/// - with-previous 経路: 直前のキャッシュに古いタグが残っていても
///   （例えば cache-miss フォールバックが親タグの置換を試みずに
///   キャッシュだけ新タグへ進めてしまった場合でも）、ライブ実タグが
///   実際に不一致である限り置換が必ず再試行される（codex-review 指摘の
///   「一度きりしか再試行されない」問題の解消）。
/// - cache-miss フォールバック経路: `previous_list_node` が存在しない
///   ため以前はタグ判定自体を行っていなかったが、ライブ実タグを直接
///   問い合わせられるため同じ判定を適用できる。
///
/// タグが一致する場合は [`crate::keyed_apply::apply_ops_with_items`]
/// （子アイテムの Insert/Remove/Move/Update）→
/// [`crate::keyed_apply::compose_achieved_children`]（達成 Node の子ノード
/// 合成）→ [`crate::keyed_apply::sync_parent_attrs`]（親要素自身の属性
/// 同期）という with-previous 経路と共通の処理を行う。
fn apply_keyed_list_core(
    document: &Document,
    list_element: &Element,
    old_parent_attrs: &[(String, String)],
    old_items: &[(String, Node)],
    old_items_are_placeholders: bool,
    new_list_node: &Node,
) -> KeyedListApplyResult {
    let Node::Element {
        tag: new_tag,
        attrs: new_parent_attrs,
        ..
    } = new_list_node
    else {
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    };

    let live_tag = list_element.tag_name().to_ascii_lowercase();
    if !live_tag.eq_ignore_ascii_case(new_tag) {
        return replace_list_element_for_tag_change(document, list_element, new_list_node);
    }

    let new_items = owned_list_item_nodes(new_list_node);
    let namespace = list_element.namespace_uri();

    let mut dom = WebSysKeyedDom {
        document,
        list_element,
        new_items: &new_items,
        namespace: namespace.as_deref(),
        children: None,
    };
    let outcome = crate::keyed_apply::apply_ops_with_items(&mut dom, old_items, &new_items);

    if outcome.resync_required {
        // 1 件でも op が計画どおりに適用できなかった（`ApplyOutcome::
        // resync_required` doc 参照）。`final_keys`/`stale_update_keys` から
        // 「達成 Node」を合成してキャッシュへ確定させると、ライブ DOM の
        // 実際の内容と乖離した diff 基準が固定されてしまう
        // （`KeyedListApplyResult::ResyncRequired` doc・イシュー #1340
        // codex-review P1 対応）ため、達成 Node の合成自体を行わず
        // 呼び出し元へ再同期を要求する。
        //
        // 最終確認レビュー指摘 1（イシュー #1340）対応: `resync_required`
        // が立つより**前**に成功していた op（例: 保持キー a の Update が
        // 丸ごと新規構築で成功した直後、別の保持キー b の Update 対象が
        // ライブ DOM 上に見つからず本分岐に到達するケース）は、既に
        // ライブ DOM を変更済みであり、その部分木に含まれるネストした
        // 別 field のキャッシュも同様に無効化する必要がある
        // （`ApplyOutcome::invalidated_nested_fields` は成功 op のみを
        // 収集済みのためここでそのまま伝播しても偽陽性は生じない）。
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: outcome.invalidated_nested_fields,
            dom_mutated: outcome.dom_mutated,
        };
    }

    if old_items_are_placeholders && !outcome.stale_update_keys.is_empty() {
        // cache-miss フォールバック専用のブロッキング検証（イシュー #1340
        // codex-review P1/Bugbot〔10 巡目〕対応の追加是正）。
        //
        // `compose_achieved_children` は `stale_update_keys` に含まれる
        // キーについて「子ノード構築に失敗し旧内容のまま据え置かれた」
        // ことを表すため `old_by_key.get(key)`（＝ `old_items` の該当
        // エントリ）をそのまま「達成 Node」へ採用する（with-previous
        // 経路ではこれが正しい: `old_items` はライブ DOM へ実際に反映
        // 済みだった直前の内容そのものであり、旧内容のまま据え置かれた
        // 実態と一致する）。
        //
        // しかし cache-miss フォールバック（`old_items_are_placeholders
        // == true`）では `old_items` の各エントリは
        // `synthesize_live_placeholder_items` が割り当てた
        // `Node::Text(String::new())` プレースホルダであり、実際の直前
        // 内容を表していない。`stale_update_keys` が空でなければこの
        // プレースホルダがそのまま「達成 Node」へ紛れ込み、実際には旧
        // 要素（例: `Node::Element`）のまま残っているライブ DOM を空の
        // テキストノードとしてキャッシュしてしまう（この PR が解消しよう
        // としている「キャッシュが達成状態と乖離する」不具合を一段深い
        // 場所で再導入することになる）。したがって、この場合は達成 Node
        // の合成を行わず再同期を要求する。1 件でも子ノード構築が恒久的に
        // 失敗し続ける場合はフォールバックが毎 tick 再試行されキャッシュ
        // が確定しないままになるが、これは既存の `Runtime` 側
        // 自己修復ループ（`None` 分岐の `ResyncRequired` 処理）と同じ
        // 「収束しないが誤ったキャッシュも作らない」設計であり安全側。
        //
        // 最終確認レビュー指摘 1（イシュー #1340）対応: この分岐も
        // `resync_required` 分岐と同じ理由でネスト field 無効化を伝播する
        // （プレースホルダ経路であっても、stale でない他の保持キーは
        // 実際に丸ごと新規構築されライブ DOM が変化している）。
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: outcome.invalidated_nested_fields,
            dom_mutated: outcome.dom_mutated,
        };
    }

    // 「達成 Node」の子ノード列合成本体は
    // `crate::keyed_apply::compose_achieved_children` へ切り出し済み
    // （DOM 非依存の純粋関数、native `cargo test` から到達可能。イシュー
    // #1340 codex-review P1〔4 巡目〕対応、`keyed_apply` モジュール冒頭
    // doc「属性検証拒否と「達成 Node」の整合」参照）。検証を拒否された
    // 属性（危険 URL スキーム・イベントハンドラ・不正 `srcset`）は実際に
    // DOM へ書き込まれた値（in-place 更新なら旧値、新規構築なら不在）へ
    // 正規化されるため、`new_list_node` をそのまま使う旧実装のように
    // 拒否済みの危険値がキャッシュへ紛れ込むことはない。
    let achieved_children =
        crate::keyed_apply::compose_achieved_children(old_items, &new_items, &outcome);

    // 親要素自身（`list_element`）の属性同期（イシュー #1340 codex-review
    // P1〔7 巡目〕対応）: 子アイテムの `KeyedOp::Update` と同じ `sync_attrs`
    // （実 DOM 読み戻し + 決定的正規化契約、`KeyedListDom::sync_attrs` doc
    // 参照）を `list_element` 自身へも適用する
    // `crate::keyed_apply::sync_parent_attrs` へ委譲する（DOM 非依存・
    // native `cargo test` から到達可能な合成本体、`Vec<(String, String)>`
    // で表す `list_element` のハンドルを渡すため `Element::clone`
    // （`web-sys` のハンドルは参照カウント方式であり安価）で `dom`
    // フィールド由来の借用と分離する）。予約属性 `data-bind-list`
    // （`fandhe_frontend_core::keyed::BIND_LIST_ATTR`）は子アイテムの
    // `data-key` と同じ理由で同期対象から除外され、`new_parent_attrs`
    // 側の表記のまま末尾に保持される（`sync_parent_attrs` doc 参照）。
    let list_element_handle = list_element.clone();
    let achieved_parent_attrs = crate::keyed_apply::sync_parent_attrs(
        &mut dom,
        &list_element_handle,
        old_parent_attrs,
        new_parent_attrs,
    );

    KeyedListApplyResult::Achieved {
        node: Node::Element {
            tag: new_tag,
            attrs: achieved_parent_attrs,
            children: achieved_children,
        },
        // 独立敵対レビュー指摘 A（イシュー #1340）対応:
        // `apply_ops_with_items` が丸ごと新規構築した部分木（`Insert`・
        // タグ変更を伴う `Update`・内容変更の `Update`）の子孫に現れた
        // 別 field の keyed list マーカーをそのまま伝播する
        // （`ApplyOutcome::invalidated_nested_fields` doc 参照）。
        invalidated_nested_fields: outcome.invalidated_nested_fields,
    }
}

/// [`crate::keyed_diff::diff_keys`] が計画した操作列を `list_element` へ
/// 適用する（本モジュールの公開エントリポイント、cache-miss フォール
/// バック専用。`fandhe-frontend-wasm-full` の `Runtime` が「直前に反映
/// 済みの `Node`」キャッシュをまだ持たない field へ使う）。
///
/// `new_list_node` は `component.view()` が返す木のうち、
/// [`find_keyed_list_node`] で特定した `field` の keyed list 親ノード
/// （呼び出し側で特定済みのものを渡す設計。`wasm-full::Runtime` が
/// `dirty_fields()` に含まれる keyed list 対象 field ごとに本関数を呼ぶ
/// 想定）。`list_element` は実 DOM 上の対応する親要素（`data-bind-list`
/// で走査済み）。
///
/// # cache-miss フォールバックの達成契約（イシュー #1340 codex-review
/// P1/Bugbot〔10 巡目〕対応）
///
/// 旧実装はキー列のみの [`crate::keyed_apply::apply_ops`]（内容比較を
/// 一切行わない）を実行し、その `bool`（構造変化が計画どおり適用できた
/// か）を戻り値としていた。呼び出し元はこの `bool` が `true` の回に
/// **望ましい view 全体**をそのままキャッシュへ確定させていたため、
/// 既存アイテムの内容・親要素のタグ/属性が実際には一切同期されていない
/// にもかかわらず「達成済み」としてキャッシュされてしまい、以後差分が
/// 出ず未反映のまま恒久的に収束しなかった（codex-review P1 指摘）。
///
/// 本関数は [`apply_keyed_list_with_previous`] と共通の
/// [`apply_keyed_list_core`] へ処理を委譲し、戻り値も `bool` から
/// [`KeyedListApplyResult`] へ変更した。`old_items` にはライブ DOM から
/// 読み出した現在のキー列（[`crate::keyed_apply::synthesize_live_placeholder_items`]）
/// を渡す（`keyed_apply` モジュール冒頭 doc「cache-miss フォールバックの
/// 達成契約」参照。本物の直前内容が無いため、内容比較を必ず不一致にする
/// プレースホルダを使い、保持キー全件に `Update` を強制発行させる。
/// タグ一致判定〔in-place 更新か `replace_root` か〕はプレースホルダに
/// 依存せずライブ問い合わせで行われるため、`Move` のみで内容が変わらない
/// 典型ケースでもルート要素自身のノード同一性は保たれる（要素そのものは
/// 再生成されない）。親要素自身の属性同期に使う `old_parent_attrs` は
/// 空スライスを渡す（`sync_attrs`/`sync_parent_attrs` の削除判定はライブ
/// 属性列挙が基準〔`KeyedListDom::sync_attrs` doc「削除判定の基準」参照〕
/// のため、`old_attrs` が空でも安全に完全な同期ができる。`old_attrs` は
/// 達成 attrs 合成時の残存〔削除失敗〕エントリの順序決定にのみ使われる
/// 補助情報であり、空でも正しさには影響しない）。
///
/// # フォーカス・入力途中の値の保持は保証しない（イシュー #1340 最終確認
/// レビュー対応）
///
/// プレースホルダは `Node::Text` と `Node::Element` の enum variant の
/// 違いにより内容比較を必ず不一致にするため、保持キー全件へ `Update` が
/// 強制発行される（上記「達成契約」参照）。内容差分がなくても
/// `replace_item_children` によりアイテムの**子ノード列**が丸ごと
/// 再構築されるため、`new_list_node`（望ましい view）が明示的に持たない
/// 子孫（例: 動的に追加された入力欄でフォーカス中の `<input>`）は消える。
/// これはルート要素自身のノード同一性（上記段落）とは別の話であり、
/// 「保持アイテムに触れない」ことを前提にしたフォーカス保持は
/// [`apply_keyed_list_with_previous`]（通常運用の主経路、内容が変化して
/// いない保持アイテムには `Update` を一切発行しない）でのみ成立する。
/// 本関数（cache-miss フォールバック）は `Runtime::mount`/`Runtime::hydrate`
/// が常にキャッシュを種付けする実運用では `ResyncRequired`・ネスト
/// field 無効化（`docs/design/keyed-update-op-design.md` §6 不変条件 9・10
/// 参照）の後にのみ到達するリカバリ経路であり、収束保証を最優先する
/// トレードオフとしてこの一時状態の喪失を許容する。
pub fn apply_keyed_list(
    document: &Document,
    list_element: &Element,
    new_list_node: &Node,
) -> KeyedListApplyResult {
    if !matches!(new_list_node, Node::Element { .. }) {
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    }
    let namespace = list_element.namespace_uri();
    let mut probe = WebSysKeyedDom {
        document,
        list_element,
        new_items: &[],
        namespace: namespace.as_deref(),
        children: None,
    };
    let old_items = crate::keyed_apply::synthesize_live_placeholder_items(&mut probe);
    apply_keyed_list_core(document, list_element, &[], &old_items, true, new_list_node)
}

/// [`fandhe_frontend_core::keyed::diff_keyed_items`] が計画した操作列
/// （`Remove`/`Insert`/`Move`/`Update`）を `list_element` へ適用する
/// （イシュー #1324、`Update` 対応の新規公開エントリポイント）。
///
/// [`apply_keyed_list`] と異なり、呼び出し元は直前に DOM へ反映した内容
/// （`previous_list_node`）を保持して渡す必要がある
/// （`fandhe-frontend-wasm-full` の `Runtime` が field ごとにキャッシュする
/// 想定、設計書 §4.1）。初回呼び出し（保持 Node がまだ無い）の場合は
/// 呼び出し元は本関数ではなく [`apply_keyed_list`]（DOM 読み出しベースの
/// 構造変化のみの適用）を使うこと（`Update` は内容比較を要するため、比較
/// 対象となる直前の `Node` が無い初回には原理的に適用できない）。
///
/// 戻り値は [`KeyedListApplyResult`]（達成 Node の合成規則は同 enum doc
/// 参照）。
///
/// # 契約検証（イシュー #1340 codex-review P1〔3 巡目〕対応）
///
/// `previous_list_node`/`new_list_node` はいずれも `keyed_list()` が生成
/// する `Node::Element`（親要素）である契約（`fandhe_frontend_core::keyed::keyed_list`
/// 参照）。この契約は DOM 操作を一切開始する**前**に検証し、契約外の形状
/// （呼び出し元の実装誤りで独自に組み立てた非 `Element` ノード等）が
/// 渡された場合は DOM に一切触れず [`KeyedListApplyResult::ResyncRequired`]
/// を返す（fail-closed）。旧実装は検証を「達成 Node」合成時（DOM 操作が
/// 完了した後）まで遅延させていたため、`new_list_node` が非 `Element` の
/// 場合に `owned_list_item_nodes` が空列を返し、旧アイテムの `Remove` を
/// 実際にライブ DOM へ適用したうえで、実 DOM の親要素と一致しない
/// `new_list_node` をそのまま `Achieved` として返してしまっていた
/// （codex-review 指摘: 呼び出し元がこれをそのままキャッシュするため
/// 実 DOM とキャッシュが恒久的に乖離する）。
/// # 親タグ判定はライブ実タグ基準（イシュー #1340 codex-review P1/Bugbot
/// 〔10 巡目〕対応）
///
/// 親（`list_element` 自身）のタグ変更検出は `previous_list_node` に
/// キャッシュされた `tag` フィールドではなく、[`apply_keyed_list_core`]
/// が問い合わせる**ライブ実タグ**（`list_element.tag_name()`）を基準に
/// 行う。`list_element` は呼び出し元（`fandhe-frontend-wasm-full` の
/// `Runtime`）が `data-bind-list` 属性値で解決した既存ライブ要素そのもの
/// であり、`Element.tagName` は DOM 標準仕様上不変のため、タグ変更を
/// 伴う更新を「浅い in-place 更新」で表現することは原理的に不可能（子
/// アイテムの `KeyedOp::Update` がタグ不一致時に `replace_root` へ切り
/// 替える判断と同型）。
///
/// 旧実装（イシュー #1340 codex-review P1〔7 巡目〕時点）は `previous_list_node`
/// にキャッシュされた `tag` を基準に判定していたため、`fandhe-frontend-wasm-full`
/// の `Runtime::apply_update_for_dirty` の cache-miss フォールバック経路
/// （[`apply_keyed_list`]）が親タグの置換を試みずキャッシュだけ新タグへ
/// 進めてしまうと、以後この関数がキャッシュ上は「タグ一致」と誤判定し
/// 置換が二度と再試行されなかった（codex-review P1〔9 巡目〕指摘）。ライブ
/// 実タグを基準にすることで、キャッシュの精度に関わらず実 DOM が実際に
/// 不一致である限り置換が必ず再試行される（自己修復）。
///
/// タグが不一致の場合は [`replace_list_element_for_tag_change`] が
/// `list_element` 自身を丸ごと安全に置換する（子アイテムの
/// `KeyedOp::Update` タグ変更時の `replace_root`/[`RootReplaceDom`] と
/// 同じ「新要素を detached で構築 → insert → 旧削除、部分失敗は
/// ロールバック」流儀）。置換後の新コンテナは `new_list_node` から丸ごと
/// 構築されるため `data-bind-list` 予約属性も新しい子アイテムもすべて
/// 引き継がれる（[`build_dom_node_with_namespace`] はイベントハンドラ・
/// 危険 URL・不正 `srcset` 以外の属性を無条件で書き込む、予約属性のみを
/// 狙って除外する処理はしていない）。`fandhe-frontend-wasm-full` の
/// `Runtime` は `list_element` のハンドルをキャッシュせず
/// `find_list_element`（`data-bind-list` 属性値によるライブ DOM 再
/// クエリ）で毎 tick 再解決するため、置換後の新コンテナは次回呼び出しで
/// 自然に見つかる（wasm-full 側の変更は不要）。
pub fn apply_keyed_list_with_previous(
    document: &Document,
    list_element: &Element,
    previous_list_node: &Node,
    new_list_node: &Node,
) -> KeyedListApplyResult {
    let Node::Element {
        attrs: old_parent_attrs,
        ..
    } = previous_list_node
    else {
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    };
    if !matches!(new_list_node, Node::Element { .. }) {
        return KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated: false,
        };
    }

    let old_items = owned_list_item_nodes(previous_list_node);
    apply_keyed_list_core(
        document,
        list_element,
        old_parent_attrs,
        &old_items,
        false,
        new_list_node,
    )
}

/// `list_element`（keyed list コンテナ）の全子ノードを一括除去する
/// （イシュー #1381 設計 §6.1/§6.2 段 3「クリア終端」の実装本体）。
///
/// `fandhe-frontend-wasm-full` の `Runtime::commit_keyed_list_result` が、
/// `KeyedListApplyResult::ResyncRequired` を受けた同一更新サイクル内で
/// 即時再同期（[`apply_keyed_list`]）を試みてもなお収束できず、かつ
/// 最初の適用試行・即時再同期試行のいずれかがライブ DOM への書き込みを
/// 試行済み（`dom_mutated` の論理和が `true`）の場合にのみ呼ぶ。
///
/// 実装は [`crate::keyed_apply::KeyedListDom::clear_children`] の
/// `web-sys` 実装（`Node::textContent` への `None` 代入 1 回）と同一手段を
/// 公開エントリポイントとして再利用する（PR #1391 でマージ済みの全キー
/// 削除一括 clear 経路プリミティブの再利用であり、新しい DOM 操作
/// プリミティブを追加しない）。`set_text_content` は DOM 標準上失敗しない
/// ため戻り値は常に `true`（呼び出し元は失敗時のベストエフォート継続
/// ロジックを持つが、本関数自体が `false` を返すことは実質ない）。
pub fn clear_keyed_list_container(list_element: &Element) -> bool {
    list_element.set_text_content(None);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, keyed::keyed_list, li, text};
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn doc() -> Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn make_list_element(doc: &Document, keys: &[&str]) -> Element {
        let ul = doc.create_element("ul").unwrap();
        for key in keys {
            let li = doc.create_element("li").unwrap();
            li.set_attribute(KEY_ATTR, key).unwrap();
            li.set_text_content(Some(key));
            ul.append_child(&li).unwrap();
        }
        ul
    }

    fn keyed_items(keys: &[&str]) -> Node {
        let items: Vec<(String, Node)> = keys
            .iter()
            .map(|k| (k.to_string(), li(vec![], vec![text(*k)])))
            .collect();
        keyed_list("ul", vec![], "items", items).expect("valid keyed list")
    }

    /// 挿入: 末尾追加で既存ノードの同一性が保たれる（フォーカス保持の土台）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_appends_new_item_without_touching_existing_nodes() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let existing_first = list_element.first_element_child().unwrap();

        let new_tree = keyed_items(&["a", "b", "c"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 3);
        assert!(existing_first.is_same_node(Some(&list_element.first_element_child().unwrap())));
        let last = list_element.children().item(2).unwrap();
        assert_eq!(last.get_attribute(KEY_ATTR).as_deref(), Some("c"));
    }

    /// 削除: 中間項目のみが除去され、他ノードは同一参照のまま残る。
    #[wasm_bindgen_test]
    fn apply_keyed_list_removes_middle_item_only() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b", "c"]);

        let new_tree = keyed_items(&["a", "c"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 2);
        let keys: Vec<Option<String>> = (0..2)
            .map(|i| {
                list_element
                    .children()
                    .item(i)
                    .unwrap()
                    .get_attribute(KEY_ATTR)
            })
            .collect();
        assert_eq!(keys, vec![Some("a".to_string()), Some("c".to_string())]);
    }

    /// 移動: 既存ノード参照を保持したまま並び替わる（再生成されないことを
    /// `is_same_node` で確認）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_moves_item_preserving_node_identity() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let node_b = list_element.children().item(1).unwrap();

        let new_tree = keyed_items(&["b", "a"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        let first = list_element.first_element_child().unwrap();
        assert!(first.is_same_node(Some(&node_b)));
        assert_eq!(first.get_attribute(KEY_ATTR).as_deref(), Some("b"));
    }

    /// XSS 構造的排除: script 文字列を含む項目を挿入しても script 要素は
    /// 生成されず、テキストとして安全に格納される（innerHTML 不使用の
    /// 回帰固定）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_script_like_text_as_plain_text_not_element() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let malicious = "<script>alert(1)</script>";
        let items: Vec<(String, Node)> = vec![("x".to_string(), li(vec![], vec![text(malicious)]))];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 1);
        assert_eq!(list_element.query_selector("script").unwrap(), None);
        let li_el = list_element.first_element_child().unwrap();
        assert_eq!(li_el.text_content().as_deref(), Some(malicious));
    }

    /// `srcset` はカンマ区切りの URL 候補を持つ特殊構文のため `URL_ATTRS`
    /// （単一 URL 属性の正リスト）に非該当。候補の 1 件でも危険スキームを
    /// 含む場合、`build_dom_node` が `is_safe_srcset` による候補分割検証を
    /// 経由して `srcset` 属性そのものを書き込まないこと（イシュー #373
    /// レビュー指摘対応: keyed list 経由のプログラム的ノード構築でも
    /// `render_into`/`binding_dom.rs` と同一の保証を持たせる契約の実ブラウザ
    /// 証跡）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_drops_srcset_when_a_candidate_has_a_dangerous_scheme() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let items: Vec<(String, Node)> = vec![(
            "x".to_string(),
            li(
                vec![],
                vec![el(
                    "img",
                    vec![("srcset", "/safe.png 1x, javascript:alert(1) 2x")],
                    vec![],
                )],
            ),
        )];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let img = list_element.query_selector("img").unwrap().unwrap();
        assert!(
            img.get_attribute("srcset").is_none(),
            "srcset 候補の 1 件でも危険スキームを含む場合、属性全体が \
             書き込まれないこと（fail-closed）"
        );
    }

    /// 全候補が安全な URL である `srcset` は従来どおり反映されること
    /// （過剰ブロックでないことの確認）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_keeps_srcset_when_all_candidates_are_safe() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let items: Vec<(String, Node)> = vec![(
            "x".to_string(),
            li(
                vec![],
                vec![el("img", vec![("srcset", "/a.png 1x, /b.png 2x")], vec![])],
            ),
        )];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let img = list_element.query_selector("img").unwrap().unwrap();
        assert_eq!(
            img.get_attribute("srcset").as_deref(),
            Some("/a.png 1x, /b.png 2x"),
            "全候補が安全な URL の srcset は反映されること"
        );
    }

    // --- SVG 名前空間（イシュー #843 Bugbot 指摘「Runtime skips stroke
    // DOM updates」の根本原因の 1 つ、`SignaturePad` の keyed list ストローク
    // 挿入回帰固定） ---

    /// SVG 名前空間の `<svg data-bind-list="strokes">` 親要素へ keyed list
    /// 挿入した `<path>` 子要素が、`document.create_element` 由来の HTML
    /// 名前空間ではなく SVG 名前空間で生成されること。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_svg_children_in_svg_namespace() {
        let document = doc();
        let list_element = document
            .create_element_ns(Some(SVG_NAMESPACE), "svg")
            .unwrap();
        list_element
            .set_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "strokes")
            .unwrap();

        let items: Vec<(String, Node)> = vec![(
            "0".to_string(),
            el("path", vec![("d", "M0.00,0.00 L1.00,1.00")], vec![]),
        )];
        let new_tree = keyed_list("svg", vec![], "strokes", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let path = list_element.query_selector("path").unwrap().unwrap();
        assert_eq!(
            path.namespace_uri().as_deref(),
            Some(SVG_NAMESPACE),
            "SVG keyed list へ挿入された <path> は SVG 名前空間で生成される \
             こと（HTML 名前空間だとブラウザが SVG として描画しない）"
        );
    }

    /// [`build_dom_node`]（公開 API）が `<svg>` 自体をルートとするノード木を
    /// 新規構築する場合も、`<svg>`・その子孫（`<path>`）の双方が SVG 名前
    /// 空間で生成されること（`fandhe-frontend-wasm-full` の遷移描画
    /// `nav.rs` のようにノード木をまるごと構築する呼び出し経路の回帰固定）。
    #[wasm_bindgen_test]
    fn build_dom_node_creates_svg_subtree_in_svg_namespace() {
        let document = doc();
        let svg_node = el(
            "svg",
            vec![("viewBox", "0 0 300 150")],
            vec![el("path", vec![("d", "M0.00,0.00 L1.00,1.00")], vec![])],
        );
        let built = build_dom_node(&document, &svg_node).unwrap();
        let svg_element: Element = built.unchecked_into();
        assert_eq!(svg_element.namespace_uri().as_deref(), Some(SVG_NAMESPACE));

        let path = svg_element.query_selector("path").unwrap().unwrap();
        assert_eq!(path.namespace_uri().as_deref(), Some(SVG_NAMESPACE));
    }

    /// イシュー #1340 codex-review 全面棚卸し対応（`Runtime::mount`/
    /// `Runtime::hydrate` の `keyed_list_cache` 種付け専用ヘルパー）:
    /// `sanitize_keyed_list_node_for_achieved` が危険 URL スキームの属性を
    /// 除外することの直接確認（`fandhe_frontend_core::render` の書き込み
    /// skip 判定と同一述語であることのポリシーレベルでの回帰固定。DOM
    /// 非依存の純粋関数だが、公開 API 契約として wasm32 ターゲットの
    /// モジュール内に置く）。
    #[wasm_bindgen_test]
    fn sanitize_keyed_list_node_for_achieved_strips_rejected_url_attr() {
        let raw = keyed_list(
            "ul",
            vec![],
            "items",
            vec![(
                "a".to_string(),
                el(
                    "li",
                    vec![("href", "javascript:alert(1)"), ("class", "safe")],
                    vec![text("a")],
                ),
            )],
        )
        .expect("valid keyed list");

        let sanitized = sanitize_keyed_list_node_for_achieved(&raw);

        let Node::Element { children, .. } = &sanitized else {
            panic!("親はタグを保つはず");
        };
        let Node::Element { attrs, .. } = &children[0] else {
            panic!("子アイテムはタグを保つはず");
        };
        assert!(
            !attrs.iter().any(|(k, _)| k == "href"),
            "javascript: スキームの href は render() が書き込みを skip する \
             ため達成 Node からも除外されるはず: {attrs:?}"
        );
        assert!(
            attrs.contains(&("class".to_string(), "safe".to_string())),
            "安全な属性はそのまま保たれるはず: {attrs:?}"
        );
    }

    /// security-auditor P1〔可用性〕指摘対応（イシュー #1340、
    /// `docs/design/keyed-update-op-design.md` §6 不変条件 10 参照）:
    /// アイテム子孫に `Node::RawHtml` が混入した keyed list を cache-miss
    /// フォールバック（`apply_keyed_list`）へ適用すると、当該アイテムの
    /// 子ノード構築（`build_dom_node_with_namespace` 経由）が fail-closed
    /// に失敗し続けるため `ResyncRequired` が返ること、かつ壊れていない
    /// 他アイテムはライブ DOM 上で実際に新しい内容へ書き込まれる
    /// （「他アイテムの更新までブロックする」わけではないこと）ことを
    /// 実 DOM で確認する。
    #[wasm_bindgen_test]
    fn apply_keyed_list_resync_required_on_raw_html_item_still_updates_healthy_sibling() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);

        let new_tree = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                (
                    "a".to_string(),
                    el(
                        "li",
                        vec![],
                        vec![Node::RawHtml("<b>injected</b>".to_string())],
                    ),
                ),
                ("b".to_string(), li(vec![], vec![text("new-b")])),
            ],
        )
        .expect("valid keyed list");

        let result = apply_keyed_list(&document, &list_element, &new_tree);

        assert!(
            matches!(result, KeyedListApplyResult::ResyncRequired { .. }),
            "RawHtml 混入アイテムの構築は恒久的に失敗するため \
             ResyncRequired を返すはず: {result:?}"
        );

        let children = list_element.children();
        assert_eq!(children.length(), 2, "アイテム数自体は変化しないはず");
        assert_eq!(
            children.item(0).unwrap().text_content().as_deref(),
            Some("a"),
            "構築失敗したアイテムは旧内容のまま据え置かれるはず \
             （fail-closed、ライブ DOM に一切書き込まれない）"
        );
        assert_eq!(
            children.item(1).unwrap().text_content().as_deref(),
            Some("new-b"),
            "壊れていない他アイテムは実際に新しい内容へ書き込まれて \
             いるはず（resync_required は他アイテムの更新をブロックしない）"
        );
    }

    // --- 連続 Insert の DocumentFragment 集約（イシュー #1320） ---

    /// 既存 `[a,b]` の中間へ連続 3 件を挿入すると、`DocumentFragment` 経由
    /// でも並びが正当であり、既存ノードは同一参照のまま保たれること
    /// （fragment 集約が既存ノードへ触れないことの実ブラウザ回帰固定）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_multiple_consecutive_items_via_fragment() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let existing_a = list_element.first_element_child().unwrap();
        let existing_b = list_element.children().item(1).unwrap();

        let new_tree = keyed_items(&["a", "x", "y", "z", "b"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 5);
        let keys: Vec<Option<String>> = (0..5)
            .map(|i| {
                list_element
                    .children()
                    .item(i)
                    .unwrap()
                    .get_attribute(KEY_ATTR)
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                Some("a".to_string()),
                Some("x".to_string()),
                Some("y".to_string()),
                Some("z".to_string()),
                Some("b".to_string()),
            ],
            "fragment 集約後も新旧ノードの並びは逐次挿入と同じであるはず"
        );
        assert!(
            existing_a.is_same_node(Some(&list_element.first_element_child().unwrap())),
            "既存ノード a は fragment 経由の挿入で再生成されず同一参照のまま \
             のはず"
        );
        assert!(
            existing_b.is_same_node(Some(&list_element.children().item(4).unwrap())),
            "既存ノード b も同一参照のまま保たれるはず"
        );
    }

    /// フォーカス保持の直接証跡: 既存アイテム内の `input` へフォーカスした
    /// 状態で連続 Insert（`DocumentFragment` 集約経路）を適用しても、
    /// フォーカスは同一要素に残ったままであること（既存ノードは fragment
    /// を経由しないという設計上の不変条件、`keyed_dom` モジュール doc
    /// 参照、の実ブラウザ回帰固定）。
    ///
    /// # #1339 由来の回帰固定の帰属先は with-previous 経路（イシュー #1340
    /// 最終確認レビュー対応）
    ///
    /// 本テストは元々 [`apply_keyed_list`]（cache-miss フォールバック、
    /// one-shot）を使っていたが、イシュー #1340 codex-review P1/Bugbot
    /// 〔10 巡目〕対応で cache-miss フォールバックはプレースホルダ
    /// `old_items` により保持キー全件へ `Update`（`replace_item_children`
    /// による子ノード丸ごと再構築）を強制発行する契約へ変更された
    /// （`apply_keyed_list` doc「cache-miss フォールバックの達成契約」
    /// 参照）。この新契約下では "a" 項目内の `input`（`new_tree` の一部
    /// ではない、テストが動的に追加した子孫）は "a" 自身が Update
    /// 対象になった時点で子ノード列ごと破棄されてしまい、フォーカスが
    /// 失われる（fragment 集約〔#1320〕自体は既存ノードに一切触れない
    /// という不変条件は変わらず健在だが、それとは別の経路〔one-shot の
    /// 強制 Update〕でフォーカスが失われるため、one-shot 経路でこの
    /// テストの意図〔フォーカス保持〕を検証すること自体が契約上不可能に
    /// なった）。
    ///
    /// #1339 が固定したかった不変条件（fragment 集約は既存ノードへ触れ
    /// ない）の本来の観測対象は、内容が変化していない保持アイテムには
    /// 一切触れない [`apply_keyed_list_with_previous`]（通常運用の主経路、
    /// `Runtime::mount`/`Runtime::hydrate` が常にキャッシュを種付けする
    /// ため定常状態はこちらを通る）であるため、本テストをこちらへ
    /// 書き換える。`one_shot_rebuilds_item_children_and_can_lose_child_focus_as_cache_miss_recovery_cost`
    /// （本テストの直後）が one-shot 側の新契約（保持アイテムの内容再
    /// 構築によりフォーカス等の一時状態が失われうる、cache-miss リカバリ
    /// のコストとして許容）を別途固定する。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_preserves_focus_across_fragment_batched_insert() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        // `Element::focus()` は要素がドキュメントツリーに接続されていない
        // と効かない（ブラウザ仕様）ため、テスト対象の `list_element` を
        // 一時的に `document.body` へ接続する。
        let body = document.body().unwrap();
        body.append_child(&list_element).unwrap();

        // "a" 項目内に input を追加してフォーカス対象にする。
        let existing_a = list_element.first_element_child().unwrap();
        let input = document.create_element("input").unwrap();
        existing_a.append_child(&input).unwrap();
        let input_element: web_sys::HtmlElement = input.clone().unchecked_into();
        input_element.focus().unwrap();
        assert_eq!(
            document.active_element().as_ref(),
            Some(&input.clone().unchecked_into::<Element>()),
            "テスト前提: input へフォーカスできていること"
        );

        // "a"/"b" とも `previous`（直前に反映済みだった内容）と
        // `new_tree` とで内容が完全に同一のため、`diff_keyed_items` は
        // いずれにも `Update` を発行しない（"a"/"b" とも一切触れられない、
        // "x"/"y" の Insert のみが起きる）。
        let previous = keyed_items(&["a", "b"]);
        let new_tree = keyed_items(&["a", "x", "y", "b"]);
        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &new_tree);
        assert!(
            matches!(result, KeyedListApplyResult::Achieved { .. }),
            "全 op が計画どおり適用できたはず: {result:?}"
        );

        let focus_preserved =
            document.active_element().as_ref() == Some(&input.clone().unchecked_into::<Element>());

        // 他テストへの影響を残さないよう後片付けしてから assert する。
        // テスト専用の best-effort クリーンアップであり
        // `WebSysKeyedDom::remove_child`（達成状態契約の対象）とは無関係
        // （イシュー #1340 codex-review P1〔3 巡目〕全走査で確認）。失敗
        // してもテスト結果の正しさには影響しない（次テストの `document`
        // 生成・要素配置は独立している前提）。
        let _ = body.remove_child(&list_element);

        assert!(
            focus_preserved,
            "連続 Insert 適用後もフォーカスは同一要素に残ったままのはず \
             （fragment 集約は既存ノードへ触れない不変条件の回帰固定。\
             with-previous 経路では内容が変化していない保持アイテムに \
             Update が発行されないため、フォーカス保持のため 2 重の保証が \
             成立する）"
        );
    }

    /// イシュー #1340 最終確認レビュー対応: cache-miss フォールバック
    /// （one-shot、[`apply_keyed_list`]）の新契約（保持キー全件へ
    /// `Update` を強制発行し、内容を実際にライブ DOM から読み出せる状態
    /// へ完全収束させる、`apply_keyed_list` doc「cache-miss フォール
    /// バックの達成契約」参照）を明示的に固定する: `new_tree` の一部で
    /// ない子孫（この場合 "a" 項目内へ動的に追加した `input`、フォーカス
    /// 中の要素で表す一時状態の代表例）は "a" 自身の内容再構築によって
    /// 失われる。これは cache-miss フォールバックが「収束保証優先」の
    /// リカバリ経路（`Runtime::mount`/`Runtime::hydrate` が常時キャッシュ
    /// を種付けするため、通常運用では `ResyncRequired`・ネスト field
    /// 無効化の後にのみ到達する、`docs/design/keyed-update-op-design.md`
    /// §6 不変条件 10 参照）である以上の意図的なトレードオフであり、
    /// バグではない。
    #[wasm_bindgen_test]
    fn apply_keyed_list_one_shot_rebuilds_item_children_and_can_lose_child_focus_as_cache_miss_recovery_cost(
    ) {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let body = document.body().unwrap();
        body.append_child(&list_element).unwrap();

        let existing_a = list_element.first_element_child().unwrap();
        let input = document.create_element("input").unwrap();
        existing_a.append_child(&input).unwrap();
        let input_element: web_sys::HtmlElement = input.clone().unchecked_into();
        input_element.focus().unwrap();
        assert_eq!(
            document.active_element().as_ref(),
            Some(&input.clone().unchecked_into::<Element>()),
            "テスト前提: input へフォーカスできていること"
        );

        let new_tree = keyed_items(&["a", "x", "y", "b"]);
        let result = apply_keyed_list(&document, &list_element, &new_tree);
        assert!(
            matches!(result, KeyedListApplyResult::Achieved { .. }),
            "全 op が計画どおり適用できたはず（内容再構築自体は成功する、\
             フォーカスのみ失われる）: {result:?}"
        );

        let focus_lost =
            document.active_element().as_ref() != Some(&input.clone().unchecked_into::<Element>());
        let _ = body.remove_child(&list_element);

        assert!(
            focus_lost,
            "one-shot（cache-miss フォールバック）は保持キー全件へ \
             Update を強制発行するため、new_tree に含まれない子孫 \
             （動的に追加した input）は消え、フォーカスは失われるはず \
             （収束保証優先のリカバリ経路の意図的なコスト）"
        );
    }

    /// SVG keyed list への連続複数件挿入でも、fragment 経由の挿入で
    /// 各要素が SVG 名前空間のまま生成されること（`build_dom_node_with_namespace`
    /// の名前空間引き継ぎが fragment 経路でも壊れていないことの回帰固定）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_multiple_svg_children_via_fragment_in_svg_namespace() {
        let document = doc();
        let list_element = document
            .create_element_ns(Some(SVG_NAMESPACE), "svg")
            .unwrap();
        list_element
            .set_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "strokes")
            .unwrap();

        let items: Vec<(String, Node)> = vec![
            (
                "0".to_string(),
                el("path", vec![("d", "M0.00,0.00 L1.00,1.00")], vec![]),
            ),
            (
                "1".to_string(),
                el("path", vec![("d", "M1.00,1.00 L2.00,2.00")], vec![]),
            ),
        ];
        let new_tree = keyed_list("svg", vec![], "strokes", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let paths = list_element.query_selector_all("path").unwrap();
        assert_eq!(paths.length(), 2, "2 件の <path> がいずれも挿入されるはず");
        for i in 0..paths.length() {
            let path: Element = paths.get(i).unwrap().unchecked_into();
            assert_eq!(
                path.namespace_uri().as_deref(),
                Some(SVG_NAMESPACE),
                "fragment 経由で挿入された <path> も SVG 名前空間のままの \
                 はず"
            );
        }
    }

    // --- apply_keyed_list_with_previous（イシュー #1324、KeyedOp::Update の
    // DOM 適用・受け入れ条件 1・2） ---

    /// 受け入れ条件 1: 同一キー・新ラベルの再適用で DOM テキストが更新
    /// される。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_updates_text_for_same_key() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);

        let previous = keyed_items(&["a"]);
        let updated_items: Vec<(String, Node)> =
            vec![("a".to_string(), li(vec![], vec![text("new-label")]))];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        let li_el = list_element.first_element_child().unwrap();
        assert_eq!(li_el.text_content().as_deref(), Some("new-label"));
        assert!(matches!(result, KeyedListApplyResult::Achieved { .. }));
    }

    // --- 全キー削除の一括 clear 経路（イシュー #1373） ---

    /// 全キー削除は `list_element` の子ノードをすべて除去し、
    /// `KeyedListApplyResult::Achieved`（`final_keys` 空）を返す
    /// （`clear_children` の `web-sys` 実装 = `set_text_content(None)` が
    /// 呼ばれたことを間接的に検証する: `child_element_count`/`text_content`
    /// がともに空になる）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_clears_all_items_via_text_content_reset() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b", "c"]);

        let previous = keyed_items(&["a", "b", "c"]);
        let updated = keyed_list("ul", vec![], "items", vec![]).unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(
            list_element.child_element_count(),
            0,
            "全キー削除後、コンテナ配下の子要素は 0 件のはず"
        );
        assert_eq!(
            list_element.text_content().as_deref(),
            Some(""),
            "textContent クリアにより残留テキストも無いはず"
        );
        match result {
            KeyedListApplyResult::Achieved { node, .. } => {
                let Node::Element { children, .. } = node else {
                    panic!("keyed list ノードは Element のはず");
                };
                assert!(children.is_empty(), "達成後のキー列は空のはず");
            }
            other => panic!("Achieved を期待したが {other:?} だった"),
        }
    }

    /// 全キー削除と同時に発生する親要素（`list_element` 自身）の属性変更が
    /// 正しく同期される（`apply_ops_with_items` の一括 clear 経路は早期
    /// `return` を持つが、それを呼ぶ `apply_keyed_list_core`（`keyed_dom.rs`）
    /// 側の `compose_achieved_children`/`sync_parent_attrs` 呼び出しは
    /// clear 経路の有無に関わらず通常どおり実行されることの確認、イシュー
    /// #1373）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_clears_all_items_and_syncs_parent_attrs_together() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        list_element.set_attribute("class", "old").unwrap();
        // `apply_keyed_list_with_previous_syncs_parent_attrs` と同じ理由
        // （SSR/マウント直後の予約属性込み状態を模す）で明示的に種付ける。
        list_element
            .set_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "items")
            .unwrap();

        let previous = keyed_list(
            "ul",
            vec![("class", "old")],
            "items",
            vec![
                ("a".to_string(), li(vec![], vec![text("a")])),
                ("b".to_string(), li(vec![], vec![text("b")])),
            ],
        )
        .unwrap();
        let updated = keyed_list("ul", vec![("class", "new")], "items", vec![]).unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(
            list_element.child_element_count(),
            0,
            "全キー削除は親属性変更と同時でも正しく反映されるはず"
        );
        assert_eq!(
            list_element.get_attribute("class").as_deref(),
            Some("new"),
            "全キー削除と同時の親属性変更（class）も反映されるはず"
        );
        assert!(matches!(result, KeyedListApplyResult::Achieved { .. }));
    }

    /// 全キー削除の直後に別アイテムを再挿入する連続適用でもキャッシュ
    /// （`WebSysKeyedDom::children`）が実 DOM と整合したまま正しい並びへ
    /// 収束する（`clear_children` 後の `children = Some(Vec::new())` 確定が
    /// 次回適用の `child_at`/`insert_before_batch` 追随更新と矛盾しないこと
    /// の確認）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_reinserts_correctly_after_clear() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);

        let previous = keyed_items(&["a", "b"]);
        let cleared = keyed_list("ul", vec![], "items", vec![]).unwrap();
        apply_keyed_list_with_previous(&document, &list_element, &previous, &cleared);
        assert_eq!(list_element.child_element_count(), 0);

        let reinserted = keyed_items(&["x", "y"]);
        let result =
            apply_keyed_list_with_previous(&document, &list_element, &cleared, &reinserted);

        assert_eq!(list_element.child_element_count(), 2);
        let keys: Vec<Option<String>> = (0..2)
            .map(|i| {
                list_element
                    .children()
                    .item(i)
                    .unwrap()
                    .get_attribute(KEY_ATTR)
            })
            .collect();
        assert_eq!(keys, vec![Some("x".to_string()), Some("y".to_string())]);
        assert!(matches!(result, KeyedListApplyResult::Achieved { .. }));
    }

    /// 受け入れ条件: Update 対象アイテムのルート要素は同一 DOM ノードの
    /// まま保たれる（`is_same_node`、既存ノード参照保持＝フォーカス保持の
    /// 土台）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_preserves_node_identity_on_update() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let node_a = list_element.first_element_child().unwrap();

        let previous = keyed_items(&["a", "b"]);
        let updated_items: Vec<(String, Node)> = vec![
            ("a".to_string(), li(vec![], vec![text("a-new")])),
            ("b".to_string(), li(vec![], vec![text("b")])),
        ];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        let current_first = list_element.first_element_child().unwrap();
        assert!(
            current_first.is_same_node(Some(&node_a)),
            "Update 対象のルート要素は再生成されず同一ノードのままのはず"
        );
        assert_eq!(current_first.text_content().as_deref(), Some("a-new"));
    }

    /// 受け入れ条件 2（XSS 回帰）: 更新値に script 相当のペイロードを含めて
    /// も script 要素が生成されず、テキストとして安全に格納される。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_keeps_updated_script_like_text_as_plain_text() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);

        let previous = keyed_items(&["a"]);
        let malicious = "<script>alert(1)</script>";
        let updated_items: Vec<(String, Node)> =
            vec![("a".to_string(), li(vec![], vec![text(malicious)]))];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(list_element.query_selector("script").unwrap(), None);
        let li_el = list_element.first_element_child().unwrap();
        assert_eq!(li_el.text_content().as_deref(), Some(malicious));
    }

    /// 属性更新経路の XSS 回帰: Update で `href="javascript:..."` のような
    /// 危険スキームへ変える試みが書き込まれない（fail-closed）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_drops_dangerous_href_on_update() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let previous_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            el("a", vec![("href", "/safe")], vec![text("link")]),
        )];
        let previous = keyed_list("ul", vec![], "items", previous_items).unwrap();
        // 初回反映（apply_keyed_list、Insert のみ）でライブ DOM を previous
        // 内容へ揃えてから Update を試みる。
        apply_keyed_list(&document, &list_element, &previous);

        let updated_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            el(
                "a",
                vec![("href", "javascript:alert(1)")],
                vec![text("link")],
            ),
        )];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        let a_el = list_element.query_selector("a").unwrap().unwrap();
        assert_eq!(
            a_el.get_attribute("href").as_deref(),
            Some("/safe"),
            "危険スキームへの href 変更は書き込まれず、旧値のまま残るはず \
             （sync_attrs は new_attrs に無い属性のみ削除し、危険な新値は \
             そもそも new_attrs へ書き込まれない）"
        );
    }

    /// Move と Update の併発（`[Move{b}, Update{b}]` 相当）が DOM で正しく
    /// 適用される: 並び順・内容の双方が新しい状態に一致する。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_handles_move_and_update_together() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);

        let previous = keyed_items(&["a", "b"]);
        let updated_items: Vec<(String, Node)> = vec![
            ("b".to_string(), li(vec![], vec![text("b-new")])),
            ("a".to_string(), li(vec![], vec![text("a")])),
        ];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(list_element.children().length(), 2);
        let first = list_element.first_element_child().unwrap();
        assert_eq!(first.get_attribute(KEY_ATTR).as_deref(), Some("b"));
        assert_eq!(first.text_content().as_deref(), Some("b-new"));
        let second = list_element.children().item(1).unwrap();
        assert_eq!(second.get_attribute(KEY_ATTR).as_deref(), Some("a"));
        assert_eq!(second.text_content().as_deref(), Some("a"));
    }

    /// codex-review P1 回帰固定（PR #1340 push 後の再レビュー、イシュー
    /// #1340）: 同一キーでルート要素のタグが `li` → `div` へ変わる
    /// `Update` が実 DOM 上でも正しくタグを置き換えること（`setAttribute`
    /// ではタグ名を変更できないため、旧実装のまま「浅い in-place 更新」
    /// 経路を使い続けると更新が反映されず旧タグのまま Achieved としてキャッシュ
    /// され、以後同じ view を再適用しても収束しなかった）。タグ変更後も
    /// 他アイテムの位置は保たれ、達成 Node を previous として同じ view を
    /// 再適用しても安定していることまで確認する。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_replaces_root_element_when_tag_changes() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let old_a = list_element.first_element_child().unwrap();
        assert_eq!(
            old_a.tag_name().to_lowercase(),
            "li",
            "テスト前提: 旧タグは li"
        );

        let previous = keyed_items(&["a", "b"]);
        let updated_items: Vec<(String, Node)> = vec![
            ("a".to_string(), el("div", vec![], vec![text("a-as-div")])),
            ("b".to_string(), li(vec![], vec![text("b")])),
        ];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(list_element.children().length(), 2);
        let first = list_element.first_element_child().unwrap();
        assert_eq!(
            first.tag_name().to_lowercase(),
            "div",
            "タグが li → div へ実 DOM 上でも正しく置き換わっているはず"
        );
        assert_eq!(first.text_content().as_deref(), Some("a-as-div"));
        assert_eq!(first.get_attribute(KEY_ATTR).as_deref(), Some("a"));
        assert!(
            !first.is_same_node(Some(&old_a)),
            "DOM 標準上タグは不変のため、タグ変更では旧ノードと別ノードに \
             なるのが正しい（ルート要素同一性を維持する「浅い in-place \
             更新」の対象外）"
        );
        let second = list_element.children().item(1).unwrap();
        assert_eq!(
            second.get_attribute(KEY_ATTR).as_deref(),
            Some("b"),
            "タグ変更の影響を受けない他アイテムの位置は保たれるはず"
        );
        let achieved = match result {
            KeyedListApplyResult::Achieved { node, .. } => node,
            KeyedListApplyResult::ResyncRequired { .. } => {
                panic!("構築成功時は Achieved が返るはず")
            }
        };

        // 収束確認: 達成 Node を previous として同じ view を再適用しても
        // 安定している（冪等性、以後の再適用で差分が出ず収束しないという
        // codex-review 指摘の再発がないことの確認）。
        let result2 = apply_keyed_list_with_previous(&document, &list_element, &achieved, &updated);
        assert!(matches!(result2, KeyedListApplyResult::Achieved { .. }));
        assert_eq!(list_element.children().length(), 2);
        let first_after_reapply = list_element.first_element_child().unwrap();
        assert_eq!(first_after_reapply.tag_name().to_lowercase(), "div");
        assert_eq!(
            first_after_reapply.text_content().as_deref(),
            Some("a-as-div")
        );
    }

    /// codex-review P1 回帰固定（PR #1340 push 後の再レビュー、イシュー
    /// #1340）: `new_list_node` が契約外の形状（`Node::Element` でない）の
    /// 場合、DOM に一切触れず（旧アイテムの `Remove` も実行しない）
    /// `KeyedListApplyResult::ResyncRequired` を返すこと。修正前は
    /// `owned_list_item_nodes` が空列を生成し旧アイテムの `Remove` を実際
    /// に適用したうえで、実 DOM の親要素と一致しない `new_list_node` を
    /// そのまま `Achieved` として返してしまっていた。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_returns_resync_required_for_non_element_new_list_node() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);

        let previous = keyed_items(&["a", "b"]);
        // 契約外の形状（`keyed_list()` を経由しない、Text ノードを
        // トップレベルとして渡す誤用を模す）。
        let malformed_new = text("not-a-keyed-list-element");

        let result =
            apply_keyed_list_with_previous(&document, &list_element, &previous, &malformed_new);

        assert!(
            matches!(result, KeyedListApplyResult::ResyncRequired { .. }),
            "契約外の new_list_node は ResyncRequired を返すはず（Achieved \
             として誤ってキャッシュされないようにする）"
        );
        assert_eq!(
            list_element.children().length(),
            2,
            "DOM には一切触れておらず、旧アイテムがそのまま残っているはず \
             （fail-closed。Remove を実行してから ResyncRequired を返す \
             のではなく、DOM 操作の前に検証する）"
        );
    }

    /// 上記と対称のケース: `previous_list_node` 側が契約外の形状の場合も
    /// 同様に DOM に一切触れず `ResyncRequired` を返すこと。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_returns_resync_required_for_non_element_previous_list_node() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);

        let malformed_previous = text("not-a-keyed-list-element");
        let updated_items: Vec<(String, Node)> = vec![
            ("a".to_string(), li(vec![], vec![text("a")])),
            ("b".to_string(), li(vec![], vec![text("b")])),
        ];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let result =
            apply_keyed_list_with_previous(&document, &list_element, &malformed_previous, &updated);

        assert!(matches!(
            result,
            KeyedListApplyResult::ResyncRequired { .. }
        ));
        assert_eq!(
            list_element.children().length(),
            2,
            "DOM には一切触れておらず、旧アイテムがそのまま残っているはず"
        );
    }

    /// codex-review P1〔9 巡目〕回帰固定（イシュー #1340）: 親要素
    /// （`list_element` 自身）のタグが変わる更新は、`list_element` を
    /// 丸ごと新しいタグの要素へ置き換えて完全収束させること（`ResyncRequired`
    /// を返すだけの旧実装〔P1〔7 巡目〕時点〕は、`Runtime` の cache-miss
    /// フォールバック経路〔`apply_keyed_list`〕も親タグを検証・置換しない
    /// ため恒久的に収束しなかった、codex-review P1〔9 巡目〕指摘）。
    ///
    /// `list_element` は親要素（wrapper）へ実際に接続した状態で検証する
    /// （`make_list_element` は `list_element` 自身をどこにも接続しないため、
    /// 置換対象の親〔`parent_node()`〕が存在する現実的な構成を明示的に
    /// 用意する）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_replaces_container_when_parent_tag_changes() {
        let document = doc();
        let wrapper = document.create_element("div").unwrap();
        let list_element = make_list_element(&document, &["a", "b"]);
        wrapper.append_child(&list_element).unwrap();

        let previous = keyed_items(&["a", "b"]);
        let updated_items: Vec<(String, Node)> = vec![
            ("a".to_string(), li(vec![], vec![text("a")])),
            ("b".to_string(), li(vec![], vec![text("b")])),
        ];
        // 親タグを "ul" から "ol" へ変える更新（子アイテムのキー・内容は
        // 前回と同一）。予約属性 data-bind-list も含めて構築する。
        let updated_with_new_parent_tag =
            keyed_list("ol", vec![("class", "list")], "items", updated_items).unwrap();

        let result = apply_keyed_list_with_previous(
            &document,
            &list_element,
            &previous,
            &updated_with_new_parent_tag,
        );

        let new_container = wrapper.first_element_child().expect("新コンテナがあるはず");
        assert_eq!(
            new_container.tag_name().to_lowercase(),
            "ol",
            "wrapper の子要素は新しいタグ（\"ol\"）へ置き換わっているはず"
        );
        assert_eq!(
            new_container.get_attribute("class").as_deref(),
            Some("list"),
            "新コンテナは new_list_node の属性で構築されるはず"
        );
        assert_eq!(
            new_container
                .get_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR)
                .as_deref(),
            Some("items"),
            "新コンテナにも予約属性 data-bind-list が引き継がれるはず \
             （`build_dom_node_with_namespace` は予約属性を特別扱いせず \
             無条件で書き込むため）"
        );
        assert_eq!(
            new_container.children().length(),
            2,
            "子アイテムも新コンテナへ丸ごと構築されるはず"
        );
        assert!(
            !wrapper
                .children()
                .item(0)
                .unwrap()
                .is_same_node(Some(&list_element.clone().unchecked_into())),
            "旧 list_element（\"ul\"）は wrapper から取り除かれているはず"
        );

        let KeyedListApplyResult::Achieved { node: achieved, .. } = result else {
            panic!("完全成功時は Achieved が返るはず: {result:?}");
        };
        assert_eq!(
            achieved, updated_with_new_parent_tag,
            "達成 Node（新 baseline）は new_list_node とバイト等価である \
             はず（新 baseline 確定）"
        );

        // 収束確認: 達成 Node を previous として同じ view を再適用しても
        // 安定している（親タグ変更後も以後の適用で崩れない）。
        let result2 = apply_keyed_list_with_previous(
            &document,
            &new_container,
            &achieved,
            &updated_with_new_parent_tag,
        );
        assert!(matches!(result2, KeyedListApplyResult::Achieved { .. }));
    }

    /// codex-review P1〔9 巡目〕回帰固定（イシュー #1340）: `list_element`
    /// がライブツリーから外れている（`parent_node()` が `None`）場合、
    /// 親タグ変更を伴う更新は置換できないため DOM に一切触れず
    /// `ResyncRequired` を返すこと（fail-closed）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_returns_resync_required_when_detached_parent_tag_changes() {
        let document = doc();
        // `make_list_element` は `list_element` 自身をどの親にも接続しない
        // ため、この時点で `list_element.parent_node()` は `None`。
        let list_element = make_list_element(&document, &["a", "b"]);

        let previous = keyed_items(&["a", "b"]);
        let updated_items: Vec<(String, Node)> = vec![
            ("a".to_string(), li(vec![], vec![text("a")])),
            ("b".to_string(), li(vec![], vec![text("b")])),
        ];
        let updated_with_new_parent_tag = keyed_list("ol", vec![], "items", updated_items).unwrap();

        let result = apply_keyed_list_with_previous(
            &document,
            &list_element,
            &previous,
            &updated_with_new_parent_tag,
        );

        assert!(
            matches!(result, KeyedListApplyResult::ResyncRequired { .. }),
            "detached な list_element は置換できないため ResyncRequired を \
             返すはず"
        );
        assert_eq!(
            list_element.tag_name().to_lowercase(),
            "ul",
            "置換を試みていないため list_element 自身のタグは旧のまま \
             （\"ul\"）残っているはず"
        );
        assert_eq!(
            list_element.children().length(),
            2,
            "DOM には一切触れておらず、旧アイテムがそのまま残っているはず"
        );
    }

    // --- cache-miss フォールバックの収束（イシュー #1340 codex-review
    // P1/Bugbot〔10 巡目〕対応） ---

    /// codex P1〔10 巡目〕回帰固定: `apply_keyed_list_with_previous` が
    /// `ResyncRequired` を返した（＝ `previous` キャッシュがライブ DOM と
    /// 乖離していた）次の適用は、`Runtime` が同フィールドのキャッシュを
    /// 破棄して呼ぶ [`apply_keyed_list`]（cache-miss フォールバック）へ
    /// 委ねられる。本テストはこの「with-previous が ResyncRequired →
    /// フォールバックへ切替 → 実 DOM が新しい view へ完全収束する」一連の
    /// 流れを直接検証する。
    ///
    /// 乖離の作り方: `previous` はキー `["a", "b"]` を主張するが、ライブ
    /// DOM（`list_element`）は実際には `"a"` しか持たない（`hydrate`
    /// 失敗・以前の未達成適用の取りこぼし等で cache と実 DOM が乖離した
    /// 状態を模した最小の再現）。この状態で `"b"` を含む `Update` 済み
    /// view を適用すると、`"b"` に対する [`KeyedListDom::find_by_key`] が
    /// `None` を返し（`apply_ops_with_items` の `KeyedOp::Update` 分岐）
    /// `resync_required` が立つ。
    #[wasm_bindgen_test]
    fn with_previous_resync_required_then_fallback_converges_to_new_view() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);

        // ライブ DOM には無い "b" を含む、乖離した previous キャッシュ。
        let previous = keyed_items(&["a", "b"]);
        let stale_update_view = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                ("a".to_string(), li(vec![], vec![text("a-updated")])),
                ("b".to_string(), li(vec![], vec![text("b-updated")])),
            ],
        )
        .expect("valid keyed list");

        let result1 =
            apply_keyed_list_with_previous(&document, &list_element, &previous, &stale_update_view);
        assert!(
            matches!(result1, KeyedListApplyResult::ResyncRequired { .. }),
            "ライブ DOM に存在しない \"b\" への Update は解決できないため \
             ResyncRequired を返すはず: {result1:?}"
        );

        // `Runtime` はここで当該フィールドのキャッシュ（`previous`）を
        // 破棄する（`crates/wasm-full` の `Runtime::apply_update_for_dirty`
        // `ResyncRequired` 分岐）。次回の再描画で実際に生成された新しい
        // view（"b" を含まない、乖離が解消された状態）をフォールバック
        // 経由で適用する。
        let converged_view = keyed_items(&["a"]);
        let result2 = apply_keyed_list(&document, &list_element, &converged_view);

        let KeyedListApplyResult::Achieved { node: achieved, .. } = result2 else {
            panic!("フォールバック経路は完全達成するはず: {result2:?}");
        };
        assert_eq!(
            achieved, converged_view,
            "フォールバック適用後の「達成 Node」は新しい view とバイト \
             等価になるはず（with-previous の失敗を引きずらない）"
        );
        assert_eq!(
            list_element.children().length(),
            1,
            "ライブ DOM も新しい view と一致し 1 件のみになるはず"
        );
        let only_child = list_element.first_element_child().unwrap();
        assert_eq!(only_child.get_attribute(KEY_ATTR).as_deref(), Some("a"));
        assert_eq!(only_child.text_content().as_deref(), Some("a"));
    }

    /// Bugbot Medium〔10 巡目〕回帰固定: 親要素のタグ置換が「detached で
    /// 置換できず `ResyncRequired`」→「キャッシュ破棄」→「実際に接続され
    /// 直した後の再試行」で、キャッシュではなくライブ DOM の実タグを基準に
    /// 再判定され、置換が正しく再試行されること。旧実装（キャッシュされた
    /// `previous` の `tag` を基準に判定）だと、フォールバック
    /// （[`apply_keyed_list`]）はそもそも `previous` を持たないため
    /// 常にプレースホルダ発行だが、仮に何らかの理由で「タグ一致」という
    /// 誤った判定が固定化されると、以後 `list_element` が実際には旧タグの
    /// ままでも `replace_list_element_for_tag_change` が二度と呼ばれなく
    /// なる（Bugbot 指摘のシナリオ）。本実装はライブ問い合わせ基準
    /// （`list_element.tag_name()`）のため、この固定化が起こり得ないこと
    /// を直接確認する。
    #[wasm_bindgen_test]
    fn tag_replace_retries_via_live_tag_after_resync_required_then_reattach() {
        let document = doc();
        // `list_element` をどの親にも接続しない（`replace_list_element_for_tag_change`
        // が `parent_node()` 不在で失敗する状況を作る）。
        let list_element = make_list_element(&document, &["a"]);

        let previous = keyed_items(&["a"]);
        let new_tag_view = keyed_list(
            "ol",
            vec![],
            "items",
            vec![("a".to_string(), li(vec![], vec![text("a")]))],
        )
        .expect("valid keyed list");

        let result1 =
            apply_keyed_list_with_previous(&document, &list_element, &previous, &new_tag_view);
        assert!(
            matches!(result1, KeyedListApplyResult::ResyncRequired { .. }),
            "detached な親タグ変更は置換できないため ResyncRequired のはず: \
             {result1:?}"
        );
        assert_eq!(
            list_element.tag_name().to_lowercase(),
            "ul",
            "置換を試みていないため list_element 自身のタグは旧のまま \
             （\"ul\"）残っているはず"
        );

        // `Runtime` がキャッシュを破棄した後の「再試行」を模す:
        // `list_element` を wrapper へ接続してから同じフォールバック経路
        // （`apply_keyed_list`、`previous` を持たない cache-miss 想定）を
        // 再度呼ぶ。キャッシュされた旧タグ判定に依存していれば
        // （Bugbot 指摘のシナリオ）この再試行でも置換が起きないはずだが、
        // ライブ問い合わせ基準のため正しく置換が再実行される。
        let wrapper = document.create_element("div").unwrap();
        wrapper.append_child(&list_element).unwrap();

        let result2 = apply_keyed_list(&document, &list_element, &new_tag_view);

        let new_container = wrapper.first_element_child().expect("新コンテナがあるはず");
        assert_eq!(
            new_container.tag_name().to_lowercase(),
            "ol",
            "再接続後の再試行で、ライブ実タグ（\"ul\"）と new view のタグ \
             （\"ol\"）の不一致が正しく再検出され、置換が実行されるはず"
        );
        let KeyedListApplyResult::Achieved { node: achieved, .. } = result2 else {
            panic!("再接続後は置換に成功し Achieved が返るはず: {result2:?}");
        };
        assert_eq!(achieved, new_tag_view);
    }

    /// codex-review P1〔7 巡目〕回帰固定（イシュー #1340）: 親要素自身の
    /// 属性更新が実際にライブ DOM（`list_element`）へ適用され、「達成
    /// Node」の親属性が `new_list_node` の親属性とバイト等価になること。
    /// 修正前は `list_element` 自身の属性を一切同期せず
    /// `new_list_node.attrs` をそのまま `Achieved` へ格納していたため、
    /// 親属性が変わる更新では実 DOM が旧状態のままキャッシュだけ新値へ
    /// 進み、以後差分を検出できなくなっていた。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_syncs_parent_attrs() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);
        list_element.set_attribute("class", "old").unwrap();
        // `make_list_element` は `data-bind-list` を設定しないため、実際の
        // SSR/マウント直後の状態（`build_dom_node_with_namespace`/SSR 出力が
        // 予約属性込みで構築済み）を模して明示的に種付けする（イシュー
        // #1340 CI 実失敗の原因: 種付けを怠っていたため `get_attribute` が
        // 常に `None` を返し、Update 経路が改変していないことの確認には
        // なっていなかった。sync_attrs 側の実装不備ではなくテスト側の不備
        // だった）。
        list_element
            .set_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "items")
            .unwrap();

        let previous_items: Vec<(String, Node)> =
            vec![("a".to_string(), li(vec![], vec![text("a")]))];
        let previous = keyed_list("ul", vec![("class", "old")], "items", previous_items).unwrap();

        let updated_items: Vec<(String, Node)> =
            vec![("a".to_string(), li(vec![], vec![text("a")]))];
        let updated = keyed_list(
            "ul",
            vec![("class", "new"), ("id", "list")],
            "items",
            updated_items,
        )
        .unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(
            list_element.get_attribute("class").as_deref(),
            Some("new"),
            "list_element 自身の class 属性が実際に新値へ更新されるはず"
        );
        assert_eq!(
            list_element.get_attribute("id").as_deref(),
            Some("list"),
            "list_element 自身に新規属性 id が実際に追加されるはず"
        );
        assert_eq!(
            list_element.get_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR),
            Some("items".to_string()),
            "予約属性 data-bind-list は Update 経路から改変されず維持される \
             はず"
        );

        let KeyedListApplyResult::Achieved {
            node:
                Node::Element {
                    attrs: achieved_parent_attrs,
                    ..
                },
            ..
        } = result
        else {
            panic!("全操作成功時は Achieved が返るはず");
        };
        let Node::Element {
            attrs: expected_parent_attrs,
            ..
        } = &updated
        else {
            panic!("updated は Node::Element のはず");
        };
        assert_eq!(
            &achieved_parent_attrs, expected_parent_attrs,
            "達成 Node の親属性は new_list_node の親属性とバイト等価\
             （順序・大小文字とも一致）であるはず"
        );
    }

    // --- 同値属性スキップ（イシュー #1382）--------------------------------
    //
    // `MutationObserver`（`attributes: true`）で「値が不変の属性へ
    // `set_attribute` の境界呼び出しが発生しないこと」（受け入れ条件 1）を
    // 直接証明する。`takeRecords()` はコールバック発火（マイクロタスク）を
    // 待たず、キューに積まれた `MutationRecord` を同期的に返す DOM 仕様
    // （`crates/wasm-full` の `headless_avatar_browser.rs` で同型の
    // 「同値 setAttribute でも mutation record が発火する」仕様の利用実績
    // あり）のため、固定 `sleep`/`await` に頼らず決定的に検証できる。

    /// `target`（`Element`）への属性変異を記録する `MutationObserver` を
    /// 開始する。コールバック本体はテストでは使わず（`takeRecords()` の
    /// 同期呼び出しのみで十分）、`Closure` はテスト実行中生存すればよい
    /// ため `forget` してリークを許容する（テスト専用ヘルパー、製品コード
    /// 経路には現れない）。
    fn observe_attribute_mutations(target: &Element) -> web_sys::MutationObserver {
        let callback = wasm_bindgen::closure::Closure::<
            dyn FnMut(js_sys::Array, web_sys::MutationObserver),
        >::new(
            |_mutations: js_sys::Array, _observer: web_sys::MutationObserver| {}
        );
        let observer = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref())
            .expect("MutationObserver construction must not fail");
        callback.forget();
        let init = web_sys::MutationObserverInit::new();
        init.set_attributes(true);
        observer
            .observe_with_options(target, &init)
            .expect("observe must not fail");
        observer
    }

    /// `observer.take_records()` が返す `MutationRecord` 列から
    /// `attributeName`（変異した属性名）だけを取り出す。
    fn taken_attribute_names(observer: &web_sys::MutationObserver) -> Vec<String> {
        observer
            .take_records()
            .to_vec()
            .into_iter()
            .map(|record| {
                record
                    .unchecked_into::<web_sys::MutationRecord>()
                    .attribute_name()
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 受け入れ条件 1: 不変属性 + 変更属性が混在する `sync_attrs` 呼び出しで
    /// `MutationRecord.attributeName` の集合が変更属性のみになる（不変属性
    /// への `set_attribute` 呼び出しが発生していないことの直接証明）。
    /// あわせて achieved が `new_attrs` とバイト等価のままであること
    /// （決定的正規化契約の維持、既存 `apply_keyed_list_with_previous_syncs_parent_attrs`
    /// と同型の確認）も固定する。
    #[wasm_bindgen_test]
    fn sync_attrs_skips_set_attribute_for_unchanged_attr_but_writes_changed_attr() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);
        let item_element = list_element.first_element_child().unwrap();
        item_element.set_attribute("class", "old").unwrap();
        item_element.set_attribute("data-fixed", "same").unwrap();

        let previous_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            li(
                vec![("class", "old"), ("data-fixed", "same")],
                vec![text("a")],
            ),
        )];
        let previous = keyed_list("ul", vec![], "items", previous_items).unwrap();

        let updated_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            li(
                vec![("class", "new"), ("data-fixed", "same")],
                vec![text("a")],
            ),
        )];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let observer = observe_attribute_mutations(&item_element);

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        let attribute_names = taken_attribute_names(&observer);
        assert_eq!(
            attribute_names,
            vec!["class".to_string()],
            "値が不変の data-fixed への set_attribute は発生せず、値が \
             変わった class のみ mutation record が記録されるはず（内訳: \
             {attribute_names:?}）"
        );
        assert_eq!(item_element.get_attribute("class").as_deref(), Some("new"));
        assert_eq!(
            item_element.get_attribute("data-fixed").as_deref(),
            Some("same")
        );

        let KeyedListApplyResult::Achieved { node: achieved, .. } = result else {
            panic!("全操作成功時は Achieved が返るはず");
        };
        assert_eq!(
            achieved, updated,
            "同値スキップがあっても achieved は new view とバイト等価の \
             ままであるはず（決定的正規化契約は不変）"
        );
    }

    /// 受け入れ条件 1: 全属性が不変の Update では attribute mutation
    /// record が 0 件であること（不変属性のみのケースでも同値スキップが
    /// 確実に発動することの固定）。
    #[wasm_bindgen_test]
    fn sync_attrs_emits_no_attribute_mutation_when_all_attrs_unchanged() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);
        let item_element = list_element.first_element_child().unwrap();
        item_element.set_attribute("class", "same").unwrap();

        let previous_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            li(vec![("class", "same")], vec![text("a")]),
        )];
        let previous = keyed_list("ul", vec![], "items", previous_items).unwrap();

        // テキストのみ変更し、属性は完全に同一の view を Update として
        // 適用する（Update op 自体は発火するが属性同期は全件スキップ
        // される想定）。
        let updated_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            li(vec![("class", "same")], vec![text("a-updated")]),
        )];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let observer = observe_attribute_mutations(&item_element);

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        let attribute_names = taken_attribute_names(&observer);
        assert!(
            attribute_names.is_empty(),
            "全属性が不変の場合 attribute mutation record は 0 件のはず \
             （内訳: {attribute_names:?}）"
        );
    }

    /// 受け入れ条件 2 の関連確認（ground truth 契約の回帰固定）: `old_attrs`
    /// との同値でスキップが発動しても、ライブ DOM が
    /// `old_attrs`/`new_attrs` の値と異なる値へ外部からドリフトしていた
    /// 場合は achieved にそのライブ実値が反映されること。スキップは
    /// 「新しい書き込みを省略する」判定に過ぎず、読み戻し（決定的正規化）
    /// は変更しないため、この自己修復性は同値スキップ導入後も崩れない。
    #[wasm_bindgen_test]
    fn sync_attrs_skip_does_not_break_live_drift_self_healing_via_readback() {
        let document = doc();
        let list_element = make_list_element(&document, &["a"]);
        let item_element = list_element.first_element_child().unwrap();
        item_element.set_attribute("class", "old").unwrap();

        let previous_items: Vec<(String, Node)> =
            vec![("a".to_string(), li(vec![("class", "old")], vec![text("a")]))];
        let previous = keyed_list("ul", vec![], "items", previous_items).unwrap();

        // old_attrs（previous）と new_attrs（updated）が同値の Update を
        // 用意しつつ、適用前にライブ DOM だけを外部ドリフトさせておく。
        let updated_items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            li(vec![("class", "old")], vec![text("a-updated")]),
        )];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        // 外部コード（他スクリプト等）によるライブ値ドリフトを模す。
        item_element.set_attribute("class", "drifted").unwrap();

        let result = apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert_eq!(
            item_element.get_attribute("class").as_deref(),
            Some("drifted"),
            "old_attrs/new_attrs 同値によるスキップはライブ実値を書き換え \
             ないため、外部ドリフト後の値がそのまま残るはず"
        );
        let KeyedListApplyResult::Achieved { node: achieved, .. } = result else {
            panic!("全操作成功時は Achieved が返るはず");
        };
        let Node::Element {
            children: achieved_children,
            ..
        } = &achieved
        else {
            panic!("achieved は Node::Element のはず");
        };
        let Node::Element {
            attrs: achieved_item_attrs,
            ..
        } = &achieved_children[0]
        else {
            panic!("achieved の子要素は Node::Element のはず");
        };
        assert_eq!(
            achieved_item_attrs,
            &vec![
                ("class".to_string(), "drifted".to_string()),
                ("data-key".to_string(), "a".to_string()),
            ],
            "achieved は読み戻し（決定的正規化）によりライブ実値 \
             \"drifted\" を反映するはず（ground truth 契約は同値スキップ \
             導入後も不変。data-key は keyed list 項目要素の予約属性として \
             compose_achieved_children が別途付与する）"
        );
    }

    /// 受け入れ条件 1（エンドツーエンド）: `apply_keyed_list_with_previous`
    /// 経由の通常経路でも、不変属性のみの子要素への Update で attribute
    /// mutation record が発生しないこと。
    #[wasm_bindgen_test]
    fn apply_keyed_list_with_previous_emits_no_attribute_mutation_for_unchanged_item_attrs() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let item_a = list_element.first_element_child().unwrap();
        let item_b = list_element.children().item(1).unwrap();
        item_a.set_attribute("class", "keep").unwrap();
        item_b.set_attribute("class", "keep").unwrap();

        let previous_items: Vec<(String, Node)> = vec![
            (
                "a".to_string(),
                li(vec![("class", "keep")], vec![text("a")]),
            ),
            (
                "b".to_string(),
                li(vec![("class", "keep")], vec![text("b")]),
            ),
        ];
        let previous = keyed_list("ul", vec![], "items", previous_items).unwrap();

        let updated_items: Vec<(String, Node)> = vec![
            (
                "a".to_string(),
                li(vec![("class", "keep")], vec![text("a")]),
            ),
            (
                "b".to_string(),
                li(vec![("class", "keep")], vec![text("b-changed")]),
            ),
        ];
        let updated = keyed_list("ul", vec![], "items", updated_items).unwrap();

        let observer_a = observe_attribute_mutations(&item_a);
        let observer_b = observe_attribute_mutations(&item_b);

        apply_keyed_list_with_previous(&document, &list_element, &previous, &updated);

        assert!(
            taken_attribute_names(&observer_a).is_empty(),
            "変更されない項目 a の class への set_attribute は発生しない \
             はず"
        );
        assert!(
            taken_attribute_names(&observer_b).is_empty(),
            "テキストのみ変わる項目 b でも class が不変なら set_attribute \
             は発生しないはず"
        );
    }
}
