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
/// ログのみで示し処理を継続する。
fn warn_replace_item_children_rollback_failed() {
    web_sys::console::warn_1(
        &"fandhe-frontend-wasm-client: keyed_dom failed to roll back a partially applied \
          child node replacement (structural restoration incomplete for this item)"
            .into(),
    );
}

/// [`WebSysKeyedDom::replace_root`] のロールバック手順自体（挿入済みの
/// 新要素を取り除く `remove_child`）が失敗した場合に出す固定英語文言の
/// 警告（設計書 §6 不変条件 6「残る有限のリスク」と同種、不変条件 7
/// 〔キー値・アイテム内容を含めない〕、イシュー #1340 codex-review P1
/// 〔3 巡目〕対応）。`unwrap()`/`panic!` は使わず、当該アイテム 1 件が
/// 不定状態（旧要素・新要素が同時に存在しうる）になりうることを警告ログ
/// のみで示し処理を継続する。
fn warn_replace_root_rollback_failed() {
    web_sys::console::warn_1(
        &"fandhe-frontend-wasm-client: keyed_dom failed to roll back a partially applied \
          root element replacement (structural restoration incomplete for this item)"
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
            // 固定英語文言でログのみ残す（不変条件 6）。
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-client: keyed_dom skipped a RawHtml node (unsupported in keyed list insertion)".into(),
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
/// `children` フィールド（本 struct 内で保持する `(key, Element)` の
/// `Vec` キャッシュ）への添字アクセスで解決する（イシュー #1319。
/// codex-review 指摘: `Element::children()` + `HtmlCollection::item(index)`
/// を都度呼ぶ実装は、`HTMLCollection` が live collection であり
/// `item(index)` の計算量が WHATWG 仕様上保証されないため、ブラウザ側の
/// 実装次第で二乗コストへ退行しうる。`children` キャッシュは
/// `first_element_child`/`next_element_sibling`（ブラウザが隣接ポインタで
/// 実装する真の O(1) 操作）による 1 度きりの sibling 走査で構築し、以降は
/// 実 DOM を一切問い合わせない純粋な `Vec` 操作のみで
/// `insert_before`/`move_before` の追随更新を行う。これによりブラウザの
/// `item()` 実装がどのような計算量であっても本アダプタの `child_at` は
/// ブラウザ API のその計算量に依存しない）。
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
    /// `child_at` が返す「現在の子要素列」のキャッシュ（`(data-key, Element)`
    /// 順序付き `Vec`）。`None` は未構築（初回 `child_at` 呼び出しで
    /// 実 DOM を 1 度だけ sibling 走査して埋める）を表す。
    ///
    /// [`crate::keyed_apply::apply_ops`] は [`crate::keyed_diff::diff_keys`]
    /// が生成した操作列（`Remove` が必ず先頭にまとまり、続く `Move`/
    /// `Insert` は昇順 `index` で並ぶ、`keyed_diff` モジュール doc・
    /// `diff_keys` 実装参照）を順に適用するため、最初の `child_at` 呼び出し
    /// （最初の `Move`/`Insert` の直前）時点で全 `Remove` は実 DOM へ適用
    /// 済みであり、ここで sibling 走査して得る並びは「削除後・挿入/移動
    /// 適用前」の基準状態と一致する。以降 `insert_before`/`move_before` が
    /// 実 DOM への適用と同時にこの `Vec` へも追随更新するため、キャッシュは
    /// 常に実 DOM の並びと同期したまま保たれる。`remove_child` は
    /// キャッシュ構築前にのみ呼ばれる契約だが、将来の呼び出し順変更に
    /// 備えてキャッシュ構築後に呼ばれた場合は無効化（`None` へリセット、
    /// 次回 `child_at` で再構築）する fail-safe を持つ。
    children: Option<Vec<(String, Element)>>,
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
    /// （実 DOM 呼び出しを伴わない `Vec` 操作）のみで完結する。
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
        self.children = Some(items);
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
        self.children
            .as_ref()
            .and_then(|children| children.get(index))
            .map(|(_, el)| el.clone())
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
            let (key, node) = items.into_iter().next().expect("len == 1 で確認済み");
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
        if let Some(children) = self.children.as_mut() {
            // キャッシュ内の旧位置を `key` の文字列比較で特定する（実 DOM
            // 呼び出しを一切伴わない純粋な `Vec` 走査。ブラウザ API の
            // 計算量に依存しないという `child_at` の契約を、この追随更新
            // 側でも維持するための設計）。
            if let Some(pos) = children.iter().position(|(k, _)| k == key) {
                children.remove(pos);
            }
            let pos = index.min(children.len());
            children.insert(pos, (key.to_string(), child.clone()));
        }
        true
    }

    fn remove_child(&mut self, child: &Element) -> bool {
        if self.list_element.remove_child(child).is_err() {
            // 実 DOM への削除自体が失敗（`child` が既に `list_element` の
            // 子でない等）。`child` は実 DOM 上に残ったままのため、
            // `children` キャッシュを無効化して「削除済み」と誤って扱わ
            // ないようにする（次回 `child_at` で実 DOM から再構築させる
            // fail-safe。イシュー #1340 codex-review P1〔3 巡目〕全走査
            // 対応）。
            self.children = None;
            return false;
        }
        // `diff_keys` は Remove を Move/Insert より必ず先に列挙するため
        // （`keyed_diff` doc 参照）、通常は `children` キャッシュが構築
        // される（最初の `child_at` 呼び出しが起きる）前にここへ到達する。
        // 仮に将来アルゴリズムが変わりキャッシュ構築後に Remove が来ても、
        // キャッシュを丸ごと無効化して次回 `child_at` で再構築させることで
        // （コストは O(n) の再走査 1 回に留まる）誤ったキャッシュを使い
        // 続けて誤挿入位置を返す不整合を防ぐ（fail-safe）。
        self.children = None;
        true
    }

    /// `child` の属性を `new_attrs`（呼び出し元 `keyed_apply::apply_ops_with_items`
    /// が既に `data-key` を除外済みの集合）へ同期する（イシュー #1324）。
    ///
    /// `Element::attributes()`（`NamedNodeMap`）で現在の属性を列挙し、
    /// `new_attrs` に存在しない属性のみ `remove_attribute` する（`data-key`
    /// は呼び出し元が渡す集合から既に除外されているため、たとえ現在の属性
    /// 列挙にヒットしても `new_attrs` 側チェックだけでは保護されない点に
    /// 注意し、ここでも明示的に除外する: 予約属性を Update 経路から改変
    /// できないようにする不変条件を、呼び出し元の 1 箇所だけに依存させない
    /// 多層防御）。属性の追加・更新は [`build_dom_node_with_namespace`] と
    /// 同一の URL スキーム・イベントハンドラ・`srcset` 検証を経由する
    /// （不変条件 1〜4 の Update 経路への継承）。
    ///
    /// # Result 破棄の正当化（イシュー #1340 codex-review P1〔3 巡目〕全走査対応）
    ///
    /// 内部の `remove_attribute`/`set_attribute` 呼び出しは戻り値
    /// （`Result`）を破棄する。これは [`crate::keyed_apply::KeyedListDom::sync_attrs`]
    /// のトレイト doc・本クレート `keyed_apply` モジュール doc「Update op
    /// の DOM 適用」に明記された設計判断の実装側の反映であり見落としでは
    /// ない: 本メソッドは既に URL スキーム・イベントハンドラ・`srcset`
    /// 検証を通過済みの値のみを渡す構成であり、`setAttribute`/
    /// `removeAttribute` は不正な引数に対して通常 `Err`/例外を投げない
    /// DOM 標準 API であるため、属性 1 件ごとの失敗検出・逆順ロールバック
    /// 機構は実装・検証コストに見合わないと判断した（設計書 §6 不変条件 6
    /// が要求する完全なロールバックの対象外として明示的に許容された残余
    /// リスク）。`replace_root`/`insert_before_batch`/`move_before`/
    /// `remove_child` 等、構造（ノードの存在・親子関係）を変える操作の
    /// 失敗が「達成状態」の恒久的な乖離を招くのとは異なり、属性の
    /// 部分失敗は次回の `Update` diff で自然に再試行され得る（対象ノード
    /// 自体は変わらず存在し続けるため、次回 view 適用時に同じ
    /// `sync_attrs` 呼び出しが再度差分を検出して収束を試みる）。
    fn sync_attrs(&mut self, child: &Element, new_attrs: &[(String, String)]) {
        let attributes = child.attributes();
        let mut current_names: Vec<String> = Vec::new();
        let len = attributes.length();
        for i in 0..len {
            if let Some(attr) = attributes.item(i) {
                current_names.push(attr.name());
            }
        }
        for name in current_names {
            if name == KEY_ATTR {
                continue;
            }
            if !new_attrs.iter().any(|(k, _)| k == &name) {
                let _ = child.remove_attribute(&name);
            }
        }
        for (name, value) in new_attrs {
            if name == KEY_ATTR {
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
    /// と共有）への線形走査で `key` の既存要素を解決する（イシュー #1324、
    /// [`crate::keyed_apply::KeyedListDom::find_by_key`] doc 参照）。
    /// キャッシュ未構築時はここで初めて実 DOM を 1 度だけ sibling 走査する
    /// （`Update` のみが発生する構成、すなわち `Insert`/`Move` が 1 件も
    /// 無く `child_at` が未呼び出しのケースでも、実 DOM 走査は高々 1 回に
    /// 抑えられる契約をここで担保する）。
    fn find_by_key(&mut self, key: &str) -> Option<Element> {
        self.ensure_children_cache();
        self.children
            .as_ref()
            .and_then(|children| children.iter().find(|(k, _)| k == key))
            .map(|(_, el)| el.clone())
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
        if let Some(children) = self.children.as_mut() {
            if let Some(pos) = children.iter().position(|(k, _)| k == key) {
                let new_element: Element = new.unchecked_into();
                children[pos] = (key.to_string(), new_element);
            }
        }
        true
    }
}

impl WebSysKeyedDom<'_> {
    /// [`crate::keyed_apply::KeyedListDom::insert_before_batch`] 実装が
    /// 実 DOM へ適用した挿入結果を `children` キャッシュへ追随させる
    /// （キャッシュ未構築なら no-op、[`Self::child_at`] doc 参照。トレイト
    /// 非公開のヘルパーのため本 struct の inherent メソッドとして持つ）。
    fn cache_inserted_nodes(&mut self, start_index: usize, items: Vec<(String, web_sys::Node)>) {
        let Some(children) = self.children.as_mut() else {
            return;
        };
        let start = start_index.min(children.len());
        for (pos, (key, node)) in (start..).zip(items) {
            // `node` は `build_dom_node_with_namespace` が `Node::Element`
            // から構築した要素ノード（`create_item` の契約、`keyed_dom`
            // モジュール doc 不変条件 4 参照）であり `Element` へのダウン
            // キャストは安全。以降は実 DOM を問い合わせない純粋な `Vec`
            // 操作のみでキャッシュを追随させる。
            let element: Element = node.unchecked_into();
            children.insert(pos, (key, element));
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

/// [`crate::keyed_diff::diff_keys`] が計画した操作列を `list_element` へ
/// 適用する（本モジュールの公開エントリポイント）。
///
/// `new_list_node` は `component.view()` が返す木のうち、
/// [`find_keyed_list_node`] で特定した `field` の keyed list 親ノード
/// （呼び出し側で特定済みのものを渡す設計。`wasm-full::Runtime` が
/// `dirty_fields()` に含まれる keyed list 対象 field ごとに本関数を呼ぶ
/// 想定）。`list_element` は実 DOM 上の対応する親要素（`data-bind-list`
/// で走査済み）。
///
/// キー照合に失敗する要素（`Insert` で `build_dom_node` が `None` を返す
/// ケース = `RawHtml` 子や不正タグ名）は skip し、当該 1 件のみ未適用のまま
/// 残す（fail-closed。他の正当な操作の適用を妨げない）。走査アルゴリズム
/// 本体は [`crate::keyed_apply::apply_ops`]（イシュー #1318 で DOM 非依存へ
/// 切り出し済み、native `cargo test` で DOM 操作コストを決定的に検証する
/// 土台）。
///
/// 戻り値は `new_list_node` が指す目標状態へ**完全に**到達できたか
/// （`true` = ライブ DOM は `new_list_node` と一致する。`false` = 上記の
/// skip が 1 件でも発生し未達成のまま終わった）を表す（イシュー #1340
/// Bugbot 指摘対応）。呼び出し元（`fandhe-frontend-wasm-full` の
/// `Runtime::apply_update_for_dirty`）は、この戻り値が `false` の回に
/// `new_list_node` を「直前に DOM へ反映した内容」のキャッシュ
/// （`keyed_list_cache`）へ確定させてはならない。未達成のまま
/// `new_list_node`（望ましい view であって実 DOM の達成状態ではない）を
/// キャッシュしてしまうと、[`KeyedListApplyResult::ResyncRequired`] が
/// `apply_keyed_list_with_previous` 経路で防いでいるのと同種の「実 DOM と
/// キャッシュの乖離が 1 tick 後に再シードされ、以降解消されない」不具合が
/// 本関数の呼び出し元（`previous` キャッシュが無い field の経路）でも
/// 再現する（`KeyedListApplyResult::ResyncRequired` doc 参照）。
pub fn apply_keyed_list(document: &Document, list_element: &Element, new_list_node: &Node) -> bool {
    let new_items = owned_list_item_nodes(new_list_node);
    let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();
    let namespace = list_element.namespace_uri();

    let mut dom = WebSysKeyedDom {
        document,
        list_element,
        new_items: &new_items,
        namespace: namespace.as_deref(),
        children: None,
    };
    crate::keyed_apply::apply_ops(&mut dom, &new_keys)
}

/// [`apply_keyed_list_with_previous`] の適用結果（イシュー #1324）。
///
/// 呼び出し元（`fandhe-frontend-wasm-full` の `Runtime`）が「直前に DOM へ
/// 反映した内容」のキャッシュを次回呼び出しの `previous_list_node` として
/// 使い続けるための状態遷移を表す（設計書 §4.2/§4.2a）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedListApplyResult {
    /// ライブ DOM が実際に表している「達成 Node」（設計書 §4.2）。
    /// `Update` が全件成功していれば `new_list_node` そのものと等価だが、
    /// 子ノード構築に失敗して据え置かれたアイテムがあれば当該アイテムのみ
    /// 旧内容のまま含む。呼び出し元はこの `Node` を次回呼び出しの
    /// `previous_list_node` として保持し続けることで、以降の diff 基準を
    /// 実際の DOM 内容と一致させ続ける（キャッシュの再同期）。
    Achieved(Node),
    /// 「要再同期」（設計書 §4.2a）。
    ///
    /// `Update` の子ノード構築失敗（`Node::RawHtml` 混入等）は当該アイテムが
    /// 旧内容のまま DOM 上に残り続けるだけなので `stale_update_keys` 経由で
    /// 「達成 Node」へ正しく表現でき、本 variant の対象にはならない
    /// （`Achieved` が返る）。一方、`Insert` の構築失敗・`Move`/`Update` の
    /// 対象キーがライブ DOM 上に見つからない等「op が計画どおりに適用され
    /// なかった」ケース（[`crate::keyed_apply::ApplyOutcome::resync_required`]
    /// doc 参照、イシュー #1340 codex-review P1 対応）では本 variant が返る:
    /// `diff_keyed_items` が計画した `index` は「全 op が成功した前提の
    /// 最終並び」上の位置であり、一部が未達成のまま「達成 Node」を確定させ
    /// キャッシュしてしまうと、次回呼び出しの diff 基準がライブ DOM の実際
    /// の内容と乖離したまま固定され、以降いくら同じ view を再適用しても
    /// 乖離が解消されない（本 variant 導入前の実際の不具合、PR #1340
    /// codex-review 指摘）。呼び出し元はこの `field` の保持 Node を破棄し、
    /// 次回は [`apply_keyed_list`] のフォールバック経路（ライブ DOM を直接
    /// 読み出す構造変化のみの適用、`Update` を発行しないため diff 基準が
    /// 常に実際の DOM と一致する）へ委ねること。
    ResyncRequired,
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
pub fn apply_keyed_list_with_previous(
    document: &Document,
    list_element: &Element,
    previous_list_node: &Node,
    new_list_node: &Node,
) -> KeyedListApplyResult {
    if !matches!(previous_list_node, Node::Element { .. })
        || !matches!(new_list_node, Node::Element { .. })
    {
        return KeyedListApplyResult::ResyncRequired;
    }

    let old_items = owned_list_item_nodes(previous_list_node);
    let new_items = owned_list_item_nodes(new_list_node);
    let namespace = list_element.namespace_uri();

    let mut dom = WebSysKeyedDom {
        document,
        list_element,
        new_items: &new_items,
        namespace: namespace.as_deref(),
        children: None,
    };
    let outcome = crate::keyed_apply::apply_ops_with_items(&mut dom, &old_items, &new_items);

    if outcome.resync_required {
        // 1 件でも op が計画どおりに適用できなかった（`ApplyOutcome::
        // resync_required` doc 参照）。`final_keys`/`stale_update_keys` から
        // 「達成 Node」を合成してキャッシュへ確定させると、ライブ DOM の
        // 実際の内容と乖離した diff 基準が固定されてしまう
        // （`KeyedListApplyResult::ResyncRequired` doc・イシュー #1340
        // codex-review P1 対応）ため、達成 Node の合成自体を行わず
        // 呼び出し元へ再同期を要求する。
        return KeyedListApplyResult::ResyncRequired;
    }

    // 「達成 Node」を合成する: final_keys の順序で、stale（子ノード構築
    // 失敗で据え置かれた）キーは旧内容、それ以外は新内容を使う。
    // `new_list_node` が `Node::Element` であることは関数冒頭の契約検証で
    // 既に保証済みのため、この `else` 分岐は到達しない想定だが
    // `unwrap()`/`panic!` は使わず（`.claude/rules/coding-rust.md`）、
    // 冒頭と同じ fail-closed（`ResyncRequired`）で安全側に倒す
    // （codex-review 指摘: DOM 操作後に契約外ノードをそのまま `Achieved`
    // として返すと、実 DOM と一致しないノードがキャッシュへ確定して
    // しまう）。
    let Node::Element {
        tag: parent_tag,
        attrs: parent_attrs,
        ..
    } = new_list_node
    else {
        return KeyedListApplyResult::ResyncRequired;
    };

    let old_by_key: std::collections::HashMap<&str, &Node> =
        old_items.iter().map(|(k, n)| (k.as_str(), n)).collect();
    let new_by_key: std::collections::HashMap<&str, &Node> =
        new_items.iter().map(|(k, n)| (k.as_str(), n)).collect();

    let achieved_children: Vec<Node> = outcome
        .final_keys
        .iter()
        .filter_map(|key| {
            if outcome.stale_update_keys.contains(key) {
                old_by_key.get(key.as_str()).map(|n| (*n).clone())
            } else {
                new_by_key.get(key.as_str()).map(|n| (*n).clone())
            }
        })
        .collect();

    KeyedListApplyResult::Achieved(Node::Element {
        tag: parent_tag,
        attrs: parent_attrs.clone(),
        children: achieved_children,
    })
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
    #[wasm_bindgen_test]
    fn apply_keyed_list_preserves_focus_across_fragment_batched_insert() {
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

        // "a" の直後（中間）へ連続 2 件挿入する: フォーカス中の要素は
        // 移動対象ではなく再構築対象でもないため、fragment 集約経路に
        // 一切関与しないはず。
        let new_tree = keyed_items(&["a", "x", "y", "b"]);
        apply_keyed_list(&document, &list_element, &new_tree);

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
             （fragment 集約は既存ノードへ触れない不変条件の回帰固定）"
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
        assert!(matches!(result, KeyedListApplyResult::Achieved(_)));
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
            KeyedListApplyResult::Achieved(node) => node,
            KeyedListApplyResult::ResyncRequired => {
                panic!("構築成功時は Achieved が返るはず")
            }
        };

        // 収束確認: 達成 Node を previous として同じ view を再適用しても
        // 安定している（冪等性、以後の再適用で差分が出ず収束しないという
        // codex-review 指摘の再発がないことの確認）。
        let result2 = apply_keyed_list_with_previous(&document, &list_element, &achieved, &updated);
        assert!(matches!(result2, KeyedListApplyResult::Achieved(_)));
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
            matches!(result, KeyedListApplyResult::ResyncRequired),
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

        assert!(matches!(result, KeyedListApplyResult::ResyncRequired));
        assert_eq!(
            list_element.children().length(),
            2,
            "DOM には一切触れておらず、旧アイテムがそのまま残っているはず"
        );
    }
}
