//! keyed list の DOM 適用: op 適用アルゴリズム本体（DOM 非依存、イシュー
//! #1318）。
//!
//! [`crate::keyed_diff`] が計画した [`crate::keyed_diff::KeyedOp`] 列を
//! 「対象コンテナへどう適用するか」の走査アルゴリズムは、従来
//! [`crate::keyed_dom`]（`#[cfg(target_arch = "wasm32")]` ゲート配下）に
//! `web-sys` 呼び出しと一体化して実装されていたため、native（`cargo test`、
//! wasm32 ゲートに阻まれる）から到達できず、DOM 操作の回数（= コスト）を
//! 決定的に固定するテストを書けなかった。本モジュールはその走査
//! アルゴリズムを [`KeyedListDom`] トレイト越しに抽象化して切り出し、
//! `keyed_dom.rs` はこのアルゴリズムを web-sys で実装するだけの薄い
//! アダプタへ縮小する（`binding`/`binding_dom`・`keyed_diff`/`keyed_dom` と
//! 同じ「純粋層 + wasm32 配線層」の 2 層構成方針をもう一段適用したもの）。
//!
//! # 呼び出し文脈
//!
//! - 本番経路: [`crate::keyed_dom::apply_keyed_list`]（wasm32 のみ）が
//!   `web-sys` アダプタを実装して [`apply_ops`] を呼ぶ。
//! - 検証経路（本モジュール `#[cfg(test)]`）: `Vec` ベースのモック
//!   `CountingDom` を注入し、走査アルゴリズムが呼ぶ DOM 操作の回数を
//!   native で数える。ルート issue #1313 が特定した「CSR create
//!   （1,000 行）で挿入位置解決の sibling 走査が累積 O(n²) になる」問題を、
//!   実ブラウザ計測（不安定・低速）ではなく `cargo test` レベルで決定的に
//!   再発検知する（本イシュー #1318 の目的そのもの）。イシュー #1319 で
//!   挿入位置解決を [`KeyedListDom::child_at`]（ブラウザの
//!   `HTMLCollection::item(index)` の計算量保証に依存しない実装が必須、
//!   [`KeyedListDom::child_at`] doc 参照。`web-sys` 実装
//!   [`crate::keyed_dom::WebSysKeyedDom`] は独自のハンドル `Vec` キャッシュ
//!   への添字アクセスで解決する）へ置換し、この O(n²) を O(n) 相当へ
//!   是正した。ただし `CountingDom` は `Vec::get` で `child_at` を実装して
//!   おり素朴にも O(1) であるため、本モジュールのコスト固定テストは「呼び
//!   出し回数」の退行（sibling 走査の再混入等）は検知するが、`web-sys`
//!   実装側の `HTMLCollection::item()` 依存の再混入（イシュー #1319
//!   codex-review 指摘）自体は検知しない。この観点は `web-sys` 実装が
//!   実 DOM を問い合わせない `Vec` キャッシュのみで完結する設計であること
//!   （[`crate::keyed_dom::WebSysKeyedDom`] doc 参照）そのものが担保する。
//!
//! # セキュリティ不変条件の引き継ぎ
//!
//! [`KeyedListDom::create_item`] が `None` を返した場合（`web-sys` 実装では
//! `Node::RawHtml` を含む部分木の構築失敗、[`crate::keyed_dom`] モジュール
//! doc 不変条件 4 参照）、当該 `Insert` 1 件は丸ごと未適用のまま skip する
//! （個別ノード単位ではなくアイテム単位の fail-closed skip、イシュー
//! #1121 由来の契約を本モジュールへ引き継ぐ）。`Remove`/`Move` で対象キーの
//! 既存ハンドルが見つからない場合も同様に当該 1 件のみ skip し、他の
//! 正当な操作の適用は妨げない。
//!
//! # Update op の DOM 適用（イシュー #1324）
//!
//! [`KeyedOp::Update`]（同一キーで内容だけが変わった保持アイテム、
//! `fandhe_frontend_core::keyed::diff_keyed_items` が発行）は
//! [`apply_ops_with_items`] のみが処理する（[`apply_ops`] は
//! `diff_keys`（キー列のみの比較）を使うため `Update` を発行し得ず、
//! 呼び出し元は引き続き `apply_ops` を使う限り本セクションの対象外）。
//! 適用方針は `docs/design/keyed-update-op-design.md` §3.2 の「浅い
//! in-place 更新」を、ロールバック粒度を単純化した形で実装する:
//!
//! 1. 属性は [`KeyedListDom::sync_attrs`] へ新しい属性集合（予約属性
//!    `data-key` を除く）をまるごと渡し、アダプタ側が現在値との差分
//!    （追加・変更・削除）を計算して適用する。
//! 2. 子ノード列は [`KeyedListDom::replace_item_children`] へ新しい
//!    `Node` 列を渡す。アダプタは新しい子ノード列を**先に構築**し、構築が
//!    全て成功した場合にのみ既存の子を除去して置き換える（`RawHtml`
//!    混入等の構築失敗時はライブ DOM を一切変更しない）。
//!
//! 設計書 §6 不変条件 6 が要求する「属性書き込み k 件目・子ノード
//! 着脱 i/j 件目の失敗ごとの逆順ロールバック」は実装しない（`setAttribute`/
//! `appendChild`/`removeChild` は不正な引数（本経路では既に URL・
//! イベントハンドラ属性検証を通過済みの値のみ渡す）に対して通常
//! `Err`/例外を投げない DOM 標準 API であり、単純化しても実務上のロール
//! バック対象がほぼ発生しない一方、全ステップ分の逆順ロールバック機構は
//! 実装・検証コストに見合わないと判断したため。子ノード構築失敗
//! （`replace_item_children` が `false` を返すケース）のみを
//! fail-closed の対象とし、この場合はライブ DOM を変更しない
//! （`false` を返す前に必ずコミット前の構築を完了させる契約、
//! [`KeyedListDom::replace_item_children`] doc 参照）。呼び出し元
//! （[`crate::keyed_dom::apply_keyed_list_with_previous`]）はこの skip を
//! 「達成 Node」の合成時に旧内容のまま据え置くことで表現し、次回の
//! diff 基準を実際の DOM 内容と一致させ続ける（再同期の収束、設計書
//! §4.2a）。

use crate::keyed_diff::{diff_keyed_items, diff_keys, KeyedOp};
use fandhe_frontend_core::Node;

/// keyed list コンテナに対する DOM 操作を抽象化するトレイト。
///
/// メソッドは [`crate::keyed_dom`] が呼んでいた `web-sys` API と 1:1 対応
/// させてある（`first_element_child` / `next_element_sibling` /
/// `get_attribute(KEY_ATTR)` / ノード構築 / `insert_before` / `remove_child`）。
/// これにより「呼び出し回数」の意味が実 DOM 呼び出し回数と一致し、
/// [`apply_ops`] のコストテストが実ブラウザでの操作コストの代理指標として
/// 妥当になる。
pub(crate) trait KeyedListDom {
    /// 既存の子要素を指すハンドル（`web-sys` 実装では `web_sys::Element`）。
    type Handle: Clone;
    /// 新規構築した挿入用ノード（`web-sys` 実装では `web_sys::Node`）。
    type NewNode;

    /// コンテナの最初の子要素を返す（`Element::first_element_child`）。
    fn first_element_child(&mut self) -> Option<Self::Handle>;

    /// `child` の次の兄弟要素を返す（`Element::next_element_sibling`）。
    fn next_element_sibling(&mut self, child: &Self::Handle) -> Option<Self::Handle>;

    /// `child` の `data-key` 属性値を返す（`Element::get_attribute`）。
    fn item_key(&mut self, child: &Self::Handle) -> Option<String>;

    /// `index` 番目（0-origin）の子要素を返す（`insert_before`/`move_before`
    /// の参照ノード決定に使う。`index` が子要素数以上なら `None` = 末尾。
    ///
    /// **実装は「ブラウザの `HTMLCollection::item(index)` の計算量に依存
    /// しない」真の O(1)（あるいは全呼び出し合計で O(n)）を満たさなければ
    /// ならない**（イシュー #1319 codex-review 指摘。`HTMLCollection` は
    /// live collection であり、`item(index)` が O(1) であることは WHATWG
    /// 仕様上保証されない。実装ごとに index アクセスのたびインデックスを
    /// 再構築するキャッシュ無効化戦略を取る可能性があり、単に「本トレイト
    /// メソッドの呼び出し回数が O(n)」であることは、その内部で実 DOM 側が
    /// 二乗コストになることを排除しない）。`web-sys` 実装
    /// （[`crate::keyed_dom::WebSysKeyedDom`]）は `first_element_child`/
    /// `next_element_sibling`（ブラウザが隣接ポインタで実装する、真に
    /// O(1) が保証された操作）による 1 度きりの sibling 走査で構築した
    /// `Vec` インデックスへの添字アクセスに解決することでこの契約を満たす
    /// （旧 `nth_element_child` の `index` 回 sibling 走査、および
    /// `HtmlCollection::item(index)` 単体呼び出しのいずれとも異なる）。
    fn child_at(&mut self, index: usize) -> Option<Self::Handle>;

    /// `key` に対応する新規ノードを構築する。構築失敗（`RawHtml` 混入等）
    /// は `None`（呼び出し元は当該 `Insert` 1 件を丸ごと skip する、
    /// 本モジュール doc「セキュリティ不変条件の引き継ぎ」参照）。
    fn create_item(&mut self, key: &str) -> Option<Self::NewNode>;

    /// `items`（`start_index` から連続する新規ノード列、この順序のまま）を
    /// `reference`（`None` なら末尾）の直前へ**1 回の DOM 境界操作相当**で
    /// 挿入する（イシュー #1320。旧 `insert_before` の 1 件ずつの呼び出し
    /// から一括版へ置換し、連続 Insert 区間 1 件あたり 1 回の JS 境界呼び
    /// 出しへ集約する）。
    ///
    /// # 契約
    ///
    /// - `items` は「新しい並びで `start_index` から連続する」新規ノード列
    ///   （[`apply_ops`] の連続 Insert 区間検出ロジック参照。区間中に
    ///   `create_item` が `None` を返した項目は `items` に含まれない。
    ///   `items` が空の場合、呼び出し元は本メソッドを呼ばない）。
    /// - 実装は全件を `reference` の直前へ**この順序のまま**挿入する
    ///   （`web-sys` 実装では `DocumentFragment` へ `append_child` した
    ///   うえで `insert_before` を 1 回呼ぶことでこれを満たす、
    ///   [`crate::keyed_dom::WebSysKeyedDom::insert_before_batch`] 参照）。
    /// - **既存ノードを `items` に混ぜてはならない**（fragment 経由で
    ///   既存ノードを移動すると現在の親から除去されフォーカス・入力途中の
    ///   値が失われるため。既存ノードの移動は
    ///   [`KeyedListDom::move_before`] が個別に担う責務のまま）。
    /// - `start_index`/各 `key` は [`KeyedListDom::child_at`] の O(1) 契約を
    ///   実装が独自の索引（`web-sys` 実装ではハンドル `Vec`）で満たすための
    ///   追随更新に使う（イシュー #1319 codex-review 指摘対応の踏襲、
    ///   [`Self::child_at`] doc 参照）。
    fn insert_before_batch(
        &mut self,
        start_index: usize,
        items: Vec<(String, Self::NewNode)>,
        reference: Option<&Self::Handle>,
    );

    /// 既存の `child`（キー `key`）を `reference`（`None` なら末尾）の直前
    /// の「新しい並びでの位置」`index` へ移動する（`Element::insert_before`
    /// の Move 用途。既存ノード参照を保持したまま移動することがフォーカス・
    /// 入力途中の値の保持に直結する、`keyed_diff` モジュール doc §5.3
    /// 参照）。`index`/`key` の用途は [`Self::insert_before`] と同じ。
    fn move_before(
        &mut self,
        index: usize,
        key: &str,
        child: &Self::Handle,
        reference: Option<&Self::Handle>,
    );

    /// `child` をコンテナから取り除く（`Element::remove_child`）。
    fn remove_child(&mut self, child: &Self::Handle);

    /// `child` の属性を `new_attrs`（予約属性 `data-key` を除く新しい
    /// 属性集合）へ同期する（[`KeyedOp::Update`] 適用の一部、イシュー
    /// #1324）。アダプタは `child` の現在の属性集合を読み出し、
    /// `new_attrs` に存在しない現在の属性を削除し、`new_attrs` の各
    /// エントリを `setAttribute` する（値が同一でも呼び出しは安全な
    /// no-op）。URL スキーム・イベントハンドラ属性の検証は
    /// [`crate::keyed_dom::build_dom_node`] と同一の述語を `web-sys`
    /// アダプタが共有して行う（本トレイトのモジュール doc 参照）。
    fn sync_attrs(&mut self, child: &Self::Handle, new_attrs: &[(String, String)]);

    /// `child` の子ノード列を `new_children`（`fandhe_frontend_core::Node`
    /// 列）へ差し替える（[`KeyedOp::Update`] 適用の一部、イシュー #1324）。
    ///
    /// 実装は新しい子ノード列を**先に構築**し、`RawHtml` 混入等で構築に
    /// 失敗した場合はライブ DOM を一切変更せず `false` を返す
    /// （本モジュール doc「Update op の DOM 適用」参照）。構築が全て
    /// 成功した場合のみ既存の子を除去し新しい子を追加して `true` を返す。
    fn replace_item_children(&mut self, child: &Self::Handle, new_children: &[Node]) -> bool;

    /// `key` に対応する既存要素のハンドルを解決する（[`KeyedOp::Update`]
    /// 適用専用、イシュー #1324）。
    ///
    /// [`find_child_by_key`]（`Remove`/`Move` が使う、`first_element_child`/
    /// `next_element_sibling`/`item_key` による毎回の sibling 走査）を
    /// `Update` にも流用すると、`Update` 件数 × リスト長 に比例する実 DOM
    /// 呼び出し（構造変化を伴わない純粋な内容変更のみの構成では
    /// [`Self::child_at`] が一度も呼ばれないため、`web-sys` 実装のキャッシュ
    /// も温まらない）が発生し、#1318/#1319 が固定した O(n) 相当の契約を
    /// `Update` 経路だけ破ってしまう。実装は [`Self::child_at`] と同様に
    /// 初回呼び出しでのみ実 DOM を走査し、以降は実 DOM 呼び出しを伴わない
    /// 索引・線形走査で解決すること（`web-sys` 実装
    /// [`crate::keyed_dom::WebSysKeyedDom`] は `child_at` と同じ `children`
    /// キャッシュを共有する）。
    fn find_by_key(&mut self, key: &str) -> Option<Self::Handle>;
}

/// コンテナ直下の子から現在の `data-key` 列を読み出す
/// （[`crate::keyed_dom::dom_item_keys`] の等価移植）。
fn dom_item_keys<D: KeyedListDom>(dom: &mut D) -> Vec<String> {
    let mut keys = Vec::new();
    let mut maybe_child = dom.first_element_child();
    while let Some(child) = maybe_child {
        if let Some(key) = dom.item_key(&child) {
            keys.push(key);
        }
        maybe_child = dom.next_element_sibling(&child);
    }
    keys
}

/// `key` に対応する既存の子要素を探す（`data-key` 属性の完全一致、
/// [`crate::keyed_dom::find_child_by_key`] の等価移植）。
fn find_child_by_key<D: KeyedListDom>(dom: &mut D, key: &str) -> Option<D::Handle> {
    let mut maybe_child = dom.first_element_child();
    while let Some(child) = maybe_child {
        if dom.item_key(&child).as_deref() == Some(key) {
            return Some(child);
        }
        maybe_child = dom.next_element_sibling(&child);
    }
    None
}

/// `dom` の現在のキー列を読み出したうえで [`crate::keyed_diff::diff_keys`]
/// が計画した操作列を適用する（[`crate::keyed_dom::apply_keyed_list`] の
/// 走査アルゴリズム本体、等価移植）。
///
/// 「現在のキー列を読む」ステップ（[`dom_item_keys`]）も `dom` への実際の
/// 呼び出しを伴うため、コスト測定（本モジュール `#[cfg(test)]` の
/// `CountingDom`）の対象へ含める必要がある。旧実装（`keyed_dom::apply_keyed_list`）が
/// `dom_item_keys(list_element)` を呼んでから `diff_keys` していたのと
/// 同じ順序をここで再現する。
///
/// `new_keys` は挿入対象の新しいキー列（`new_items` 相当の走査は
/// 呼び出し元 [`crate::keyed_dom::apply_keyed_list`] 側で `Node` 木から
/// 済ませてあり、本関数は `key` から [`KeyedListDom::create_item`] で
/// ノードを構築する）。
///
/// # 連続 Insert 区間の DocumentFragment 集約（イシュー #1320）
///
/// [`crate::keyed_diff::diff_keys`] は `Remove` を必ず先頭にまとめ、続く
/// `Move`/`Insert` を昇順 `index` で並べる（`keyed_diff` モジュール doc・
/// 本モジュール doc 参照）。このため「index がちょうど 1 ずつ増加する
/// 極大の `Insert` 連続区間」は `ops` 列走査で単純な先読みにより検出できる。
///
/// この区間検出は「`Insert` 以外の op（`Remove`/`Move`、将来の `Update` 等）
/// が来たら区間を打ち切る」規則で実装する。区間内の各 `Insert` について
/// 逐次適用したときの参照ノードは**全件同一**になる: 区間開始時点で
/// `start_index` 位置にある既存ノードは、区間内の挿入が `start_index` の
/// 直前へ 1 件ずつ増えていくだけであり、そのノード自身の DOM 上の位置は
/// 都度 1 つずつ後ろへずれるが同一ノードのままだからである。したがって
/// 区間全体をまとめて `start_index` 時点の参照ノード 1 回の解決で
/// [`KeyedListDom::insert_before_batch`] へ渡しても、逐次挿入した場合と
/// 同じ DOM 結果になる。
///
/// 区間内で `create_item` が `None`（`RawHtml` 混入等の構築失敗）を返した
/// 項目は当該 1 件のみを `items` から除外する fail-closed skip を維持する
/// （本モジュール doc「セキュリティ不変条件の引き継ぎ」参照）。この挙動は
/// 旧逐次実装（1 件ごとに `child_at` を呼んでいた版）と厳密には異なる:
/// 旧実装は失敗した項目の後続項目についても `diff_keys` が計画した
/// （成功を前提とした）`index` をそのまま `child_at` へ渡すため、失敗が
/// 挟まると後続項目の参照ノードが実際の並びとずれ得た。本実装は区間全体で
/// 参照ノードを 1 回だけ・区間確定後に解決するため、この「失敗後続項目の
/// 参照ずれ」自体が構造的に起こらない（劣化ではなく安全側・より正しい側
/// への挙動変化であり、暗黙の仕様変更にしないためここに明記する）。
///
/// # 未達成スロットの index 補正（イシュー #1340 codex-review P1 対応）
///
/// `diff_keys` が返す `index` は「全 op が成功した前提の最終並び」上の
/// 位置であり、`Insert`/`Move` のいずれかが未達成（`create_item` が
/// `None`、または移動対象キーがライブ DOM 上に見つからない）に終わると、
/// それ以降の op が参照する `index` は実際のライブ DOM 上の位置より
/// `index_offset`（それまでに未達成となったスロット数の累計）だけ大きい
/// ズレを持つ。本関数は `index_offset` を走査中に維持し、`child_at` へ
/// 渡す直前に `index.saturating_sub(index_offset)` へ補正することで、
/// 後続 op が実際のライブ DOM 上の正しい参照ノードへ解決されるようにする
/// （例: 実 DOM `[a,b]` → 新規列 `[x,b,a]` で `x` の構築が失敗する場合、
/// 補正が無いと `Move{index:1,key:b}` は `child_at(1)` = `b` 自身を参照し
/// 自己参照の no-op になり `a`/`b` の並びが永続的に収束しない。補正後は
/// `child_at(1 - 1) = child_at(0)` = `a` を正しく参照し、`b` を `a` の前へ
/// 移動して `[b,a]` へ収束する）。`Insert` 区間の `start_index`／その
/// `insert_before_batch` 呼び出しへ渡す `start_index` 引数自体も同じ補正を
/// 適用する（渡す `start_index` はトレイト実装の内部インデックスキャッシュ
/// 追随更新にも使われるため、実際のライブ DOM 上の位置と一致させる必要が
/// ある）。区間内で発生した未達成は区間確定後にまとめて `index_offset` へ
/// 加算する（区間内の他アイテムの挿入順序自体には影響しないため）。
/// 戻り値は `new_keys` が指す目標状態へ**完全に**到達できたか
/// （`true` = 全 op が計画どおり適用され、ライブ DOM は `new_keys` と
/// 一致する。`false` = `Insert` の構築失敗・`Move` 対象キー未検出等で
/// 一部が未達成のまま終わった）を表す（イシュー #1340 Bugbot 指摘対応）。
/// 呼び出し元（[`crate::keyed_dom::apply_keyed_list`]）はこの戻り値を、
/// 未達成状態を「達成」としてキャッシュへ再シードしないためのガードに
/// 使う（同関数 doc 参照）。
pub(crate) fn apply_ops<D: KeyedListDom>(dom: &mut D, new_keys: &[String]) -> bool {
    let old_keys = dom_item_keys(dom);
    let ops = diff_keys(&old_keys, new_keys);
    apply_ops_list(dom, ops)
}

/// [`apply_ops`] の走査本体（イシュー #1340 Bugbot 指摘対応でテスト容易性の
/// ため分離）。`ops` を直接受け取ることで、`diff_keys` が実際には発行し
/// 得ない `KeyedOp::Update` を含む列（`KeyedOp` は `diff_keyed_items` とも
/// 型を共有するため型としては構築可能）を native テストから直接注入し、
/// 網羅性のみの目的で置かれた no-op 分岐が `i` を進め忘れて無限ループしない
/// ことを検証できるようにする（`#[cfg(test)]` の
/// `apply_ops_does_not_hang_when_ops_contain_an_update` 参照）。
///
/// 戻り値の意味は [`apply_ops`] と同じ（全 op が計画どおり適用できたか）。
fn apply_ops_list<D: KeyedListDom>(dom: &mut D, ops: Vec<KeyedOp>) -> bool {
    let mut i = 0;
    let mut index_offset: usize = 0;
    let mut fully_achieved = true;
    while i < ops.len() {
        match &ops[i] {
            KeyedOp::Remove { key } => {
                if let Some(child) = find_child_by_key(dom, key) {
                    dom.remove_child(&child);
                }
                i += 1;
            }
            KeyedOp::Insert { index, .. } => {
                let start_index = *index;
                let mut expected_index = start_index;
                let mut items = Vec::new();
                let mut failed_in_run: usize = 0;
                let mut j = i;
                while let Some(KeyedOp::Insert { index, key }) = ops.get(j) {
                    if *index != expected_index {
                        // 極大の連続 Insert 区間はここで終わる（次の
                        // Insert は index が飛んでおり、区間を跨いだ
                        // 「既に一致していたため op を持たない」既存項目が
                        // 間に挟まっている）。
                        break;
                    }
                    if let Some(new_node) = dom.create_item(key) {
                        items.push((key.clone(), new_node));
                    } else {
                        // RawHtml 混入等の構築失敗（`None`）は当該アイテム
                        // のみ `items` から除外する fail-closed skip（本
                        // 関数 doc「セキュリティ不変条件の引き継ぎ」参照）。
                        // 未達成スロットとして `index_offset` 補正の対象に
                        // 数える（本関数 doc「未達成スロットの index 補正」
                        // 参照）。
                        failed_in_run += 1;
                    }
                    // `expected_index` は診断済みの `diff_keys` 計画どおり
                    // 進める（失敗の有無に関わらず区間検出の基準は変えな
                    // い。区間検出自体は「計画上の」index で行い、実際の
                    // ライブ DOM 参照だけを補正する）。
                    expected_index += 1;
                    j += 1;
                }
                if !items.is_empty() {
                    let adjusted_start = start_index.saturating_sub(index_offset);
                    let reference = dom.child_at(adjusted_start);
                    dom.insert_before_batch(adjusted_start, items, reference.as_ref());
                }
                if failed_in_run > 0 {
                    fully_achieved = false;
                }
                index_offset += failed_in_run;
                i = j;
            }
            KeyedOp::Move { index, key } => {
                if let Some(existing) = find_child_by_key(dom, key) {
                    let adjusted = index.saturating_sub(index_offset);
                    let reference = dom.child_at(adjusted);
                    dom.move_before(adjusted, key, &existing, reference.as_ref());
                } else {
                    // 移動対象キーがライブ DOM 上に見つからない（改ざん等の
                    // 異常系）。この場合も対象キーの目標スロットは
                    // 埋まらないため、`Insert` の構築失敗と同様に未達成
                    // スロットとして `index_offset` へ数える（本関数 doc
                    // 参照）。
                    index_offset += 1;
                    fully_achieved = false;
                }
                i += 1;
            }
            KeyedOp::Update { .. } => {
                // `diff_keys`（本関数が使う純粋なキー列比較）は内容比較を
                // 行わないため `KeyedOp::Update` を発行し得ない
                // （`fandhe_frontend_core::keyed::diff_keys` の実装参照）。
                // `KeyedOp` は `diff_keyed_items` とも型を共有するため
                // 網羅性のためのみに存在する到達しない分岐(no-op)。
                // Update を実際に処理する経路は [`apply_ops_with_items`]。
                // `i` は他の arm と同様に必ず進める（Bugbot 指摘、イシュー
                // #1340: 進めないと `ops` に `Update` が混入した場合に
                // `while i < ops.len()` が無限ループする）。
                i += 1;
            }
        }
    }
    fully_achieved
}

/// [`apply_ops`] の適用結果（イシュー #1324、`KeyedOp::Update` を含む
/// op 列を処理した [`apply_ops_with_items`] のみが返す）。
///
/// [`crate::keyed_dom::apply_keyed_list_with_previous`] がこの結果から
/// 「達成 Node」（実 DOM が実際に表す内容、設計書 §4.2/§4.2a）を合成する
/// ための追跡情報を保持する。
#[derive(Debug, Default)]
pub(crate) struct ApplyOutcome {
    /// 適用後にライブ DOM 上へ実際に存在するキー列（新しい並び順）。
    /// `Insert` の構築に失敗して未反映のまま skip されたキーは含まない。
    ///
    /// [`Self::resync_required`] が `true` の場合、この列は「未達成 op が
    /// 無かった前提で index 補正した上での最良推定」に過ぎず、呼び出し元
    /// （[`crate::keyed_dom::apply_keyed_list_with_previous`]）はこの列を
    /// 「達成 Node」としてキャッシュへ確定させてはならない
    /// （[`Self::resync_required`] doc 参照）。
    pub(crate) final_keys: Vec<String>,
    /// `Update` の子ノード構築に失敗し、旧内容のまま据え置かれたキーの
    /// 集合（「達成 Node」合成時にこの集合のキーは新内容ではなく旧内容を
    /// 使う）。ノード参照・位置は保たれたまま内容だけが据え置かれるため
    /// [`Self::final_keys`]/「達成 Node」で正しく表現でき、
    /// [`Self::resync_required`] の対象にはしない。
    pub(crate) stale_update_keys: std::collections::HashSet<String>,
    /// op 列中に 1 件でも「計画どおりに適用できなかった」もの（`Insert` の
    /// `create_item` 失敗、`Move`/`Update` の対象キーがライブ DOM 上に
    /// 見つからない、`Update` の新ノードが `Node::Element` でない）が
    /// あった場合に `true`（イシュー #1340 codex-review P1 対応）。
    ///
    /// これらは「op が実行されなかった」ケースであり、その対象キーが
    /// 最終的にライブ DOM 上のどの位置・状態にあるかを [`Self::final_keys`]
    /// だけから正確に再構成できる保証がない（`apply_ops` 側の
    /// `index_offset` 補正は「以降の op の参照ノード解決」を実際のライブ
    /// DOM に一致させる目的の補正であり、未達成キー自身の最終位置の
    /// 網羅的な追跡までは行わない）。呼び出し元はこのフラグが立った回の
    /// 適用結果を「達成 Node」としてキャッシュへ確定させず、次回は
    /// ライブ DOM を直接読み出す構造フォールバック
    /// （[`crate::keyed_dom::apply_keyed_list`]）へ委ねること
    /// （`KeyedListApplyResult::ResyncRequired` doc 参照）。
    pub(crate) resync_required: bool,
}

/// `old_items`/`new_items`（`(キー, Node)` 列）から
/// [`fandhe_frontend_core::keyed::diff_keyed_items`] で内容比較付き op 列
/// （`Update` を含む）を計画し、`dom` へ適用する（イシュー #1324）。
///
/// [`apply_ops`] と異なり「dom から現在のキー列を読む」のではなく
/// `old_items` を diff の入力として使う（呼び出し元がキー列だけでなく
/// 直前に反映した `Node` 内容を保持している前提。
/// [`crate::keyed_dom::apply_keyed_list_with_previous`] doc 参照）。
/// `Remove`/`Insert`/`Move` の適用アルゴリズムは [`apply_ops`] と同一。
pub(crate) fn apply_ops_with_items<D: KeyedListDom>(
    dom: &mut D,
    old_items: &[(String, Node)],
    new_items: &[(String, Node)],
) -> ApplyOutcome {
    let ops = diff_keyed_items(old_items, new_items);
    let mut failed_inserts: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut stale_update_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    // ライブ DOM 上の実際の位置と `diff_keyed_items` が計画した `index` との
    // 累計ズレ（[`apply_ops`] doc「未達成スロットの index 補正」と同じ目的・
    // 同じ補正規則。イシュー #1340 codex-review P1 対応）。
    let mut index_offset: usize = 0;
    // 1 件でも op が計画どおりに適用できなかった場合に `true`
    // （[`ApplyOutcome::resync_required`] doc 参照）。
    let mut resync_required = false;

    for op in ops {
        match op {
            KeyedOp::Remove { key } => {
                if let Some(child) = find_child_by_key(dom, &key) {
                    dom.remove_child(&child);
                }
            }
            KeyedOp::Insert { index, key } => {
                let Some(new_node) = dom.create_item(&key) else {
                    failed_inserts.insert(key);
                    index_offset += 1;
                    resync_required = true;
                    continue;
                };
                let adjusted = index.saturating_sub(index_offset);
                let reference = dom.child_at(adjusted);
                // トレイトが提供する挿入 API は #1320 で `insert_before_batch`
                // へ一本化された（連続 Insert 区間の DocumentFragment
                // 集約）。本関数は `diff_keyed_items` の op 列を 1 件ずつ
                // 適用する構成のため、要素数 1 の `items` で呼び出す
                // （契約上「新しい並びで `start_index` から連続する新規
                // ノード列」を満たせば良く、単一要素はこれを自明に満たす）。
                dom.insert_before_batch(
                    adjusted,
                    vec![(key.clone(), new_node)],
                    reference.as_ref(),
                );
            }
            KeyedOp::Move { index, key } => {
                let Some(existing) = find_child_by_key(dom, &key) else {
                    // 移動対象キーがライブ DOM 上に見つからない（改ざん等の
                    // 異常系）。目標スロットが埋まらないため `Insert` の
                    // 構築失敗と同様に未達成スロットとして扱う
                    // （`apply_ops` doc・[`ApplyOutcome::resync_required`]
                    // doc 参照）。
                    index_offset += 1;
                    resync_required = true;
                    continue;
                };
                let adjusted = index.saturating_sub(index_offset);
                let reference = dom.child_at(adjusted);
                dom.move_before(adjusted, &key, &existing, reference.as_ref());
            }
            KeyedOp::Update { key } => {
                // `find_child_by_key`（sibling 走査、Remove/Move 用）ではなく
                // `KeyedListDom::find_by_key` を使う（O(n²) 退行防止、
                // `find_by_key` doc・本モジュール doc「Update op の DOM
                // 適用」参照）。
                let Some(existing) = dom.find_by_key(&key) else {
                    // 保持キーのはずが実 DOM 上に見つからない（改ざん等の
                    // 異常系）。`Update` は構造（位置）を変えない op のため
                    // `index_offset` は増やさないが、対象キーの内容が
                    // 未反映のままである事実は `final_keys` だけからは
                    // 判別できないため `resync_required` を立てる
                    // （[`ApplyOutcome::resync_required`] doc 参照）。
                    resync_required = true;
                    continue;
                };
                let Some((_, new_node)) = new_items.iter().find(|(k, _)| k == &key) else {
                    resync_required = true;
                    continue;
                };
                let Node::Element {
                    attrs: new_attrs,
                    children: new_children,
                    ..
                } = new_node
                else {
                    // keyed list アイテムは keyed_list() 構築時点で
                    // Node::Element であることが保証される
                    // （KeyedListError::NonElementItem で fail-closed に
                    // 拒否済み）。到達しない想定だが安全側に skip する。
                    resync_required = true;
                    continue;
                };
                // codex-review P1 対応（PR #1340、イシュー #1340）: 子ノード
                // 構築（`replace_item_children`）を**先に**試み、成功した
                // 場合にのみ `sync_attrs` を呼ぶ。旧実装は `sync_attrs` を
                // 先に呼んでいたため、`replace_item_children` が `RawHtml`
                // 混入等で `false` を返して子ノードは旧内容のまま据え置か
                // れても、属性だけは新しい値へ変更済みという「属性は新・
                // 子ノードは旧」の不整合な状態が実 DOM 上に残った。
                // `stale_update_keys` 経由で「達成 Node」を合成する際は
                // `old_by_key`（属性・子ノードとも旧内容）をまるごと使う
                // （`crate::keyed_dom::apply_keyed_list_with_previous`
                // 参照）ため、属性削除を伴う更新でこの不整合が起きると
                // 実 DOM（新属性・旧子ノード）とキャッシュ（旧属性・旧子
                // ノード）が具体的に乖離する。子ノード構築を先に完了させて
                // からのみ属性を同期する順序にすることで、`replace_item_
                // children` が `false` を返した場合は本 arm がライブ DOM
                // へ一切書き込みを行わない（`sync_attrs` も呼ばない）状態を
                // 保証し、「構築失敗時はライブ DOM を一切変更しない」契約と
                // 「旧内容のまま据え置く」`stale_update_keys` の意味を一致
                // させる。
                let filtered_attrs: Vec<(String, String)> = new_attrs
                    .iter()
                    .filter(|(name, _)| name != fandhe_frontend_core::keyed::KEY_ATTR)
                    .cloned()
                    .collect();
                if !dom.replace_item_children(&existing, new_children) {
                    // 子ノード構築失敗はノード参照・位置を保ったまま内容が
                    // 旧値のまま据え置かれるだけなので `final_keys`/
                    // `stale_update_keys` で正しく表現できる
                    // （`resync_required` の対象にしない、
                    // [`ApplyOutcome::stale_update_keys`] doc 参照）。
                    // 属性同期は行わない（上記コメント参照、ライブ DOM
                    // 全体を旧内容のまま保つ）。
                    stale_update_keys.insert(key);
                } else {
                    // `sync_attrs` は `setAttribute`/`removeAttribute`
                    // 相当のみで構成され、本トレイトの契約上失敗を報告
                    // する手段を持たない（`KeyedListDom::sync_attrs` doc・
                    // 本モジュール doc「Update op の DOM 適用」の
                    // ロールバック省略の根拠と同じ理由: 不正な引数に対して
                    // 通常 `Err`/例外を投げない DOM 標準 API であるため）。
                    // 子ノード構築の成功を確認した後でのみ呼ぶため、
                    // 呼び出し自体が「達成」を意味する。
                    dom.sync_attrs(&existing, &filtered_attrs);
                }
            }
        }
    }

    let final_keys: Vec<String> = new_items
        .iter()
        .map(|(k, _)| k.clone())
        .filter(|k| !failed_inserts.contains(k))
        .collect();

    ApplyOutcome {
        final_keys,
        stale_update_keys,
        resync_required,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Vec<(key, value)>` を土台にした `KeyedListDom` の native モック。
    ///
    /// `value` はテスト用のダミーペイロード（既存ノード同一性の代わりに
    /// key の等価性で同一ノードかどうかを判定できれば十分なため、
    /// `Handle`/`NewNode` はともに `String`（key そのもの）で表す）。
    /// `calls` が [`apply_ops`] を通じて発生した DOM 操作呼び出し回数
    /// （メソッドごとの内訳）を数える。
    #[derive(Default)]
    struct CountingDom {
        items: Vec<String>,
        calls: CallCounts,
        /// キーごとの現在の属性集合（`sync_attrs` テスト用、イシュー #1324）。
        attrs: std::collections::HashMap<String, Vec<(String, String)>>,
        /// キーごとの現在の子ノード内容（`replace_item_children` テスト用）。
        /// `apply_ops`（`Update` を発行しない）経路では未使用のまま。
        children: std::collections::HashMap<String, Vec<Node>>,
        /// `replace_item_children` に構築失敗（`RawHtml` 混入相当）を
        /// 注入するキー集合。ロールバック契約テスト用のフック。
        fail_replace_children_for: std::collections::HashSet<String>,
    }

    #[derive(Default, Debug, Clone, Copy)]
    struct CallCounts {
        first_element_child: usize,
        next_element_sibling: usize,
        item_key: usize,
        create_item: usize,
        insert_before_batch: usize,
        move_before: usize,
        remove_child: usize,
        child_at: usize,
        sync_attrs: usize,
        replace_item_children: usize,
        find_by_key: usize,
    }

    impl CallCounts {
        /// 実 DOM 呼び出しに数える全メソッドの合計
        /// （`item_key` は `get_attribute` 相当で実 DOM 呼び出しを伴うため
        /// 合計に含める。`insert_before_batch` はイシュー #1320 で連続
        /// Insert 区間 1 件につき 1 回に集約されるため、旧
        /// `insert_before`（アイテム 1 件ごとに 1 回）より少ない回数になる。
        /// 1,000 行 create の上限値コメントの内訳と対応する）。
        /// イシュー #1324 で `sync_attrs`/`replace_item_children`/
        /// `find_by_key`（`Update` 適用が呼ぶ）を追加した（既存の `apply_ops`
        /// 系コスト固定テストは `Update` を発行しない `diff_keys` のみを
        /// 使うため、これらは常に 0 のままで既存テストの数値に影響しない。
        /// `apply_ops_with_items` 系のコスト固定テストが対象）。
        fn total(&self) -> usize {
            self.first_element_child
                + self.next_element_sibling
                + self.item_key
                + self.create_item
                + self.insert_before_batch
                + self.move_before
                + self.remove_child
                + self.child_at
                + self.sync_attrs
                + self.replace_item_children
                + self.find_by_key
        }
    }

    impl KeyedListDom for CountingDom {
        type Handle = String;
        type NewNode = String;

        fn first_element_child(&mut self) -> Option<Self::Handle> {
            self.calls.first_element_child += 1;
            self.items.first().cloned()
        }

        fn next_element_sibling(&mut self, child: &Self::Handle) -> Option<Self::Handle> {
            self.calls.next_element_sibling += 1;
            let pos = self.items.iter().position(|k| k == child)?;
            self.items.get(pos + 1).cloned()
        }

        fn item_key(&mut self, child: &Self::Handle) -> Option<String> {
            self.calls.item_key += 1;
            Some(child.clone())
        }

        fn child_at(&mut self, index: usize) -> Option<Self::Handle> {
            self.calls.child_at += 1;
            self.items.get(index).cloned()
        }

        fn create_item(&mut self, key: &str) -> Option<Self::NewNode> {
            self.calls.create_item += 1;
            Some(key.to_string())
        }

        fn insert_before_batch(
            &mut self,
            _start_index: usize,
            items: Vec<(String, Self::NewNode)>,
            reference: Option<&Self::Handle>,
        ) {
            self.calls.insert_before_batch += 1;
            let start = match reference {
                Some(r) => self
                    .items
                    .iter()
                    .position(|k| k == r)
                    .unwrap_or(self.items.len()),
                None => self.items.len(),
            };
            for (pos, (_key, node)) in (start..).zip(items) {
                self.items.insert(pos, node);
            }
        }

        fn move_before(
            &mut self,
            _index: usize,
            _key: &str,
            child: &Self::Handle,
            reference: Option<&Self::Handle>,
        ) {
            self.calls.move_before += 1;
            let from = self
                .items
                .iter()
                .position(|k| k == child)
                .expect("move_before の対象は事前に find_child_by_key で存在確認済みのはず");
            let removed = self.items.remove(from);
            let pos = match reference {
                Some(r) => self
                    .items
                    .iter()
                    .position(|k| k == r)
                    .unwrap_or(self.items.len()),
                None => self.items.len(),
            };
            self.items.insert(pos, removed);
        }

        fn remove_child(&mut self, child: &Self::Handle) {
            self.calls.remove_child += 1;
            if let Some(pos) = self.items.iter().position(|k| k == child) {
                self.items.remove(pos);
            }
        }

        fn sync_attrs(&mut self, child: &Self::Handle, new_attrs: &[(String, String)]) {
            self.calls.sync_attrs += 1;
            self.attrs.insert(child.clone(), new_attrs.to_vec());
        }

        fn replace_item_children(&mut self, child: &Self::Handle, new_children: &[Node]) -> bool {
            self.calls.replace_item_children += 1;
            if self.fail_replace_children_for.contains(child) {
                // 構築失敗を模擬: ライブ側の `children` を一切変更せず
                // `false` を返す（本トレイトの fail-closed 契約、モジュール
                // doc「Update op の DOM 適用」参照）。
                return false;
            }
            self.children.insert(child.clone(), new_children.to_vec());
            true
        }

        /// `items` への直接の線形走査（実 DOM 呼び出しに相当するものは
        /// 伴わない、本 struct が `Vec` のみで完結する native モックである
        /// ことそのものが「実 DOM を問い合わせない」契約を体現する。
        /// `web-sys` 実装 [`crate::keyed_dom::WebSysKeyedDom::find_by_key`]
        /// は初回のみ実 DOM を走査してキャッシュを構築し、以降は同様に
        /// 実 DOM 非依存の走査で解決する契約であり、本モックは「初回構築
        /// 後は実 DOM 呼び出しゼロ」という性質のみを模している）。
        fn find_by_key(&mut self, key: &str) -> Option<Self::Handle> {
            self.calls.find_by_key += 1;
            self.items.iter().find(|k| k.as_str() == key).cloned()
        }
    }

    fn keys_n(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("k{i}")).collect()
    }

    // --- コスト固定テスト（イシュー #1318 本体、#1319 で上限を O(n) 相当へ
    // 絞り直し済み） ---
    //
    // イシュー #1319（`child_at` をブラウザの `HTMLCollection::item()` の
    // 計算量保証に依存しない O(1) 参照へ置換、`KeyedListDom::child_at` doc
    // 参照）で挿入位置解決が O(index) の sibling 走査から解放されたため、
    // 上限値は「実測値 + 小さな余裕」で O(n) 相当へ絞った。この上限を
    // 上回る場合は O(1) 化の退行（sibling 走査の再混入・定数倍の悪化）を
    // 意味する。

    /// 空 → 1,000 行の create: 1,000 件すべてが「index 0..1,000 の単一
    /// 連続 Insert 区間」として検出されるため、DOM 操作の総呼び出し回数は
    /// 実測 1,003 回（内訳: `first_element_child` 1 回（初期の
    /// `dom_item_keys` 読み。旧キー列が空のため 1 回で `None` が返り
    /// ループ本体は回らない）+ `child_at` 1 回（区間確定後の参照ノード
    /// 解決、1 回だけ）+ `create_item` 1,000 回 + `insert_before_batch`
    /// 1 回（イシュー #1320: 挿入系の JS 境界呼び出しが 1,000 回 → 1 回へ
    /// 集約される）) に対して余裕を持った上限（1,200 回）で固定する。
    /// イシュー #1319 適用後（バッチ集約前）は同条件で実測 2,001 回
    /// （`child_at`/`insert_before` が各 1,000 回）、さらに旧
    /// `nth_element_child` 実装（イシュー #1318 以前）では実測 502,501 回
    /// だった（イシュー #1318 の元コメント参照）。
    #[test]
    fn apply_ops_create_1000_rows_from_empty_stays_linear() {
        const N: usize = 1_000;
        let mut dom = CountingDom::default();
        let new_keys = keys_n(N);

        apply_ops(&mut dom, &new_keys);

        assert_eq!(
            dom.items, new_keys,
            "適用後のキー列は新しい並びと一致するはず"
        );
        let total = dom.calls.total();
        assert!(
            total <= 1_200,
            "1,000 行 create の DOM 操作総数は 1,200 回以内のはず \
             （実測: {total}、内訳: {:?}）。連続 Insert 区間の \
             DocumentFragment 集約（イシュー #1320）からの退行を検知する \
             上限",
            dom.calls
        );
        assert_eq!(
            dom.calls.insert_before_batch, 1,
            "1,000 件の連続 Insert は単一区間へ集約され、挿入系の \
             呼び出しは 1 回に収まるはず（内訳: {:?}）",
            dom.calls
        );
    }

    /// 既存 1,000 行の先頭へ 1 件挿入: 区間サイズが 1 件のため
    /// `insert_before_batch` の呼び出し回数自体は旧 `insert_before` と
    /// 変わらず 1 回のまま。実測 2,004 回（`first_element_child` 1 回、
    /// `next_element_sibling` 1,000 回（`dom_item_keys` の初期走査）、
    /// `item_key` 1,000 回、`child_at`/`create_item`/`insert_before_batch`
    /// 各 1 回）に対して 2 割強のタイトな上限（2,500 回）で固定する
    /// （create 1,000 行のようなグローバル余裕は与えない: 単発挿入で
    /// O(n²) 的な重複走査が紛れ込んだ場合に即座に検知するため）。
    #[test]
    fn apply_ops_prepend_one_to_1000_rows_stays_linear() {
        const N: usize = 1_000;
        let mut dom = CountingDom {
            items: keys_n(N),
            ..Default::default()
        };
        let old_keys = keys_n(N);
        let mut new_keys = vec!["new".to_string()];
        new_keys.extend(old_keys.iter().cloned());

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
        let total = dom.calls.total();
        assert!(
            total <= 2_500,
            "先頭 1 件挿入の DOM 操作総数は 2,500 回以内のはず（実測: {total}、\
             内訳: {:?}）",
            dom.calls
        );
        assert_eq!(dom.calls.insert_before_batch, 1);
    }

    /// 既存 1,000 行の末尾へ 1 件挿入: `child_at(index=1000)` は
    /// `Vec::get` の単一呼び出しで完結するため、`nth_element_child` の
    /// 全 sibling 走査（旧実装で末尾挿入が先頭挿入より重かった原因）が
    /// 消え、実測 2,004 回（先頭挿入と同一内訳: `first_element_child`
    /// 1 回 + `next_element_sibling` 1,000 回 + `item_key` 1,000 回 +
    /// `child_at`/`create_item`/`insert_before_batch` 各 1 回）に縮む。
    /// +2 割強のタイトな上限（2,500 回）で固定する。旧実装では実測
    /// 3,004 回（`nth_element_child` の走査 1,000 回が追加分）だった。
    #[test]
    fn apply_ops_append_one_to_1000_rows_stays_linear() {
        const N: usize = 1_000;
        let mut dom = CountingDom {
            items: keys_n(N),
            ..Default::default()
        };
        let old_keys = keys_n(N);
        let mut new_keys = old_keys.clone();
        new_keys.push("new".to_string());

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
        let total = dom.calls.total();
        assert!(
            total <= 2_500,
            "末尾 1 件挿入の DOM 操作総数は 2,500 回以内のはず（実測: {total}、\
             内訳: {:?}）",
            dom.calls
        );
        assert_eq!(dom.calls.insert_before_batch, 1);
    }

    // --- 連続 Insert 区間検出そのものの回帰固定（イシュー #1320） ---

    /// 複数の独立した連続 Insert 区間（`[a,b]` → `[x,y,a,z,w,b]`）が、
    /// それぞれ別の `insert_before_batch` 呼び出しへ集約されること
    /// （区間 1: index 0,1 の `x,y`。区間 2: index 3,4 の `z,w`。既存
    /// `a`/`b` の間にある「一致済みのため op を持たない」区間で分断される
    /// ケース）。
    #[test]
    fn apply_ops_batches_each_disjoint_insert_run_separately() {
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "b".to_string()],
            ..Default::default()
        };
        let new_keys: Vec<String> = ["x", "y", "a", "z", "w", "b"]
            .into_iter()
            .map(String::from)
            .collect();

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
        assert_eq!(
            dom.calls.insert_before_batch, 2,
            "2 つの独立した連続 Insert 区間は別々に集約されるはず（内訳: {:?}）",
            dom.calls
        );
    }

    /// Insert 区間の途中に Move が挟まると区間が分断されること
    /// （`[a,c]` → `[x,c,a]`: index 0 で `x` の Insert、続いて index 1 で
    /// `c` の Move、index 2 で改めて `a` の Move。連続する `Insert` が
    /// 存在しないため `insert_before_batch` は毎回 1 件のバッチとして
    /// 個別に呼ばれる）。
    #[test]
    fn apply_ops_insert_run_is_split_by_interleaved_move() {
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "c".to_string()],
            ..Default::default()
        };
        let new_keys: Vec<String> = ["x", "c", "a"].into_iter().map(String::from).collect();

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
        assert_eq!(
            dom.calls.insert_before_batch, 1,
            "この入力では Insert は 1 件のみのはず（内訳: {:?}）",
            dom.calls
        );
        assert!(
            dom.calls.move_before >= 1,
            "既存キーの並び替えは move_before 経由で行われるはず（内訳: {:?}）",
            dom.calls
        );
    }

    /// 区間中の一部項目で `create_item` が構築失敗（`None`）を返しても、
    /// 失敗した 1 件のみが未適用のまま skip され、成功した項目は
    /// 正しい並びで一括挿入されること（fail-closed skip の回帰固定、
    /// 本モジュール doc「セキュリティ不変条件の引き継ぎ」・[`apply_ops`]
    /// doc 参照）。
    #[derive(Default)]
    struct PoisonedCreateDom {
        inner: CountingDom,
        /// この key の `create_item` は常に `None` を返す
        /// （`web-sys` 実装での `RawHtml` 混入等の構築失敗を模す）。
        poisoned_key: String,
    }

    impl KeyedListDom for PoisonedCreateDom {
        type Handle = String;
        type NewNode = String;

        fn first_element_child(&mut self) -> Option<Self::Handle> {
            self.inner.first_element_child()
        }
        fn next_element_sibling(&mut self, child: &Self::Handle) -> Option<Self::Handle> {
            self.inner.next_element_sibling(child)
        }
        fn item_key(&mut self, child: &Self::Handle) -> Option<String> {
            self.inner.item_key(child)
        }
        fn child_at(&mut self, index: usize) -> Option<Self::Handle> {
            self.inner.child_at(index)
        }
        fn create_item(&mut self, key: &str) -> Option<Self::NewNode> {
            if key == self.poisoned_key {
                return None;
            }
            self.inner.create_item(key)
        }
        fn insert_before_batch(
            &mut self,
            start_index: usize,
            items: Vec<(String, Self::NewNode)>,
            reference: Option<&Self::Handle>,
        ) {
            self.inner
                .insert_before_batch(start_index, items, reference);
        }
        fn move_before(
            &mut self,
            index: usize,
            key: &str,
            child: &Self::Handle,
            reference: Option<&Self::Handle>,
        ) {
            self.inner.move_before(index, key, child, reference);
        }
        fn remove_child(&mut self, child: &Self::Handle) {
            self.inner.remove_child(child);
        }
        fn sync_attrs(&mut self, child: &Self::Handle, new_attrs: &[(String, String)]) {
            self.inner.sync_attrs(child, new_attrs);
        }
        fn replace_item_children(&mut self, child: &Self::Handle, new_children: &[Node]) -> bool {
            self.inner.replace_item_children(child, new_children)
        }
        fn find_by_key(&mut self, key: &str) -> Option<Self::Handle> {
            self.inner.find_by_key(key)
        }
    }

    #[test]
    fn apply_ops_skips_only_the_item_whose_create_item_fails_within_a_run() {
        let mut dom = PoisonedCreateDom {
            inner: CountingDom {
                items: vec!["a".to_string()],
                ..Default::default()
            },
            poisoned_key: "y".to_string(),
        };
        // 連続 Insert 区間 [x, y, z]（y のみ構築失敗）の直後に既存 a。
        let new_keys: Vec<String> = ["x", "y", "z", "a"].into_iter().map(String::from).collect();

        apply_ops(&mut dom, &new_keys);

        assert_eq!(
            dom.inner.items,
            vec!["x".to_string(), "z".to_string(), "a".to_string()],
            "構築失敗した y のみが未適用のまま skip され、x/z は成功して \
             正しい相対順序で挿入されるはず"
        );
        assert_eq!(
            dom.inner.calls.insert_before_batch, 1,
            "区間全体は 1 回の insert_before_batch へ集約されるはず（内訳: {:?}）",
            dom.inner.calls
        );
    }

    /// codex-review P1 回帰固定（PR #1340、イシュー #1340）: 実 DOM
    /// `[a,b]` → 新規列 `[x,b,a]` で `x` の構築が失敗するケースで、
    /// 後続の `Move{index:1,key:b}` が実 DOM 上の正しい参照ノードへ
    /// 解決され（`index_offset` 補正、[`apply_ops`] doc「未達成スロットの
    /// index 補正」参照）、1 回の適用で `[b,a]`（`x` を除いた正しい並び）へ
    /// 収束すること。さらに同じ view を再適用しても順序が変わらない
    /// （収束後は安定、繰り返し適用しても崩れない）ことも確認する。
    ///
    /// 修正前は `index_offset` 補正が無く、`Move{index:1,key:b}` が
    /// `child_at(1)` = `b` 自身を参照する自己参照の no-op になり、実 DOM は
    /// `[a,b]` のまま変化しなかった（codex 指摘の再現手順）。
    #[test]
    fn apply_ops_converges_dom_order_when_leading_insert_construction_fails() {
        let mut dom = PoisonedCreateDom {
            inner: CountingDom {
                items: vec!["a".to_string(), "b".to_string()],
                ..Default::default()
            },
            poisoned_key: "x".to_string(),
        };
        let new_keys: Vec<String> = ["x", "b", "a"].into_iter().map(String::from).collect();

        apply_ops(&mut dom, &new_keys);

        assert_eq!(
            dom.inner.items,
            vec!["b".to_string(), "a".to_string()],
            "x の構築失敗を除いた正しい並び [b, a] へ 1 回の適用で収束するはず"
        );

        // 同じ view（x は引き続き構築失敗する）を再適用しても、既に正しい
        // 並びのため崩れない（収束後の安定性・冪等性の確認）。
        apply_ops(&mut dom, &new_keys);

        assert_eq!(
            dom.inner.items,
            vec!["b".to_string(), "a".to_string()],
            "収束後に同じ view を再適用しても並びが崩れないはず（冪等性）"
        );
    }

    /// Bugbot Medium 回帰固定（PR #1340、イシュー #1340）: `apply_ops`
    /// （`diff_keys` のみを経由し実際には `Update` を発行し得ない経路）へ
    /// `KeyedOp::Update` を含む `ops` 列を直接注入しても `apply_ops_list`
    /// の走査がハングせず完走すること。修正前は `Update` arm が `i` を
    /// 進めなかったため `while i < ops.len()` が無限ループしていた。
    #[test]
    fn apply_ops_does_not_hang_when_ops_contain_an_update() {
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "b".to_string()],
            ..Default::default()
        };
        let ops = vec![
            KeyedOp::Update {
                key: "a".to_string(),
            },
            KeyedOp::Move {
                index: 0,
                key: "b".to_string(),
            },
        ];

        // 修正前はここで無限ループしテストがハングした。完走すれば修正の
        // 証跡になる（`KeyedListDom` に `Update` 用メソッドが無いため
        // 実際の内容反映は起きず、並び替えのみが反映される）。
        apply_ops_list(&mut dom, ops);

        assert_eq!(
            dom.items,
            vec!["b".to_string(), "a".to_string()],
            "Update op は no-op のまま skip され、後続の Move のみ適用されるはず"
        );
    }

    // --- 意味的一致の確認（モックとアルゴリズムが噛み合っていることの
    // 担保、上記コスト値の信頼性の裏付け） ---

    /// 削除: 対象キーのみが取り除かれ、他のキー列の順序は保たれる。
    #[test]
    fn apply_ops_removes_only_target_key() {
        let mut dom = CountingDom {
            items: keys_n(3),
            ..Default::default()
        };
        let new_keys = vec!["k0".to_string(), "k2".to_string()];

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
    }

    /// 移動: 既存キー集合のまま並びだけが変わる。
    #[test]
    fn apply_ops_reorders_existing_keys() {
        let mut dom = CountingDom {
            items: keys_n(3),
            ..Default::default()
        };
        let new_keys = vec!["k2".to_string(), "k0".to_string(), "k1".to_string()];

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
    }

    // --- apply_ops_with_items（イシュー #1324、Update op の DOM 適用） ---

    use fandhe_frontend_core::{el, text};

    fn item(key: &str, text_value: &str) -> (String, Node) {
        (
            key.to_string(),
            el("li", vec![("data-key", key)], vec![text(text_value)]),
        )
    }

    /// テキストのみ変更の Update が発行され、`sync_attrs`/
    /// `replace_item_children` がちょうど 1 回ずつ呼ばれ、無関係な
    /// `create_item`/`insert_before_batch`/`remove_child` は発生しない
    /// （余分な DOM 操作が起きないことの確認）。
    #[test]
    fn apply_ops_with_items_updates_text_only_change_via_child_replacement() {
        let old_items = vec![item("a", "old")];
        let new_items = vec![item("a", "new")];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(outcome.final_keys, vec!["a".to_string()]);
        assert!(outcome.stale_update_keys.is_empty());
        assert!(
            !outcome.resync_required,
            "全 op が計画どおり適用できたはずなので再同期は不要のはず"
        );
        assert_eq!(dom.calls.sync_attrs, 1);
        assert_eq!(dom.calls.replace_item_children, 1);
        assert_eq!(dom.calls.create_item, 0);
        assert_eq!(dom.calls.insert_before_batch, 0);
        assert_eq!(dom.calls.remove_child, 0);
        assert_eq!(
            dom.children.get("a"),
            Some(&vec![text("new")]),
            "新しい子ノード内容が反映されているはず"
        );
    }

    /// 属性の追加・変更・削除が混在する差分適用: `data-key` 自体は
    /// `sync_attrs` へ渡す新属性集合から除外される
    /// （予約属性を Update 経路から改変できないようにする不変条件）。
    #[test]
    fn apply_ops_with_items_syncs_attrs_excluding_reserved_data_key() {
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "old"), ("data-removed", "x")],
                vec![text("same")],
            ),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "new"), ("data-added", "y")],
                vec![text("same")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            ..Default::default()
        };

        apply_ops_with_items(&mut dom, &old_items, &new_items);

        let synced = dom.attrs.get("a").expect("sync_attrs が呼ばれているはず");
        assert!(
            synced.iter().all(|(k, _)| k != "data-key"),
            "data-key は sync_attrs へ渡す集合から除外されるはず: {synced:?}"
        );
        assert!(synced.contains(&("class".to_string(), "new".to_string())));
        assert!(synced.contains(&("data-added".to_string(), "y".to_string())));
        assert!(
            !synced.iter().any(|(k, _)| k == "data-removed"),
            "new_attrs に無い旧属性は同期対象に含まれないはず: {synced:?}"
        );
    }

    /// 子ノード構築失敗（`RawHtml` 混入相当）の注入: 当該キーは
    /// `stale_update_keys` へ記録され、`final_keys` には引き続き含まれる
    /// （ライブ DOM 上に旧内容のままアイテムが残っているため）。他アイテムの
    /// 適用は妨げられない（複数アイテムの Update 混在ケース）。
    #[test]
    fn apply_ops_with_items_marks_stale_on_child_build_failure_without_blocking_others() {
        let old_items = vec![item("a", "old-a"), item("b", "old-b")];
        let new_items = vec![item("a", "new-a"), item("b", "new-b")];
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "b".to_string()],
            fail_replace_children_for: std::collections::HashSet::from(["a".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(
            outcome.final_keys,
            vec!["a".to_string(), "b".to_string()],
            "構築失敗した a も DOM 上に残存しているため final_keys に含まれ続けるはず"
        );
        assert_eq!(
            outcome.stale_update_keys,
            std::collections::HashSet::from(["a".to_string()]),
            "構築失敗したキーのみが stale として記録されるはず"
        );
        assert!(
            !outcome.resync_required,
            "子ノード構築失敗はノード参照・位置を保ったまま stale_update_keys \
             で正しく表現できるため、resync_required の対象にはならないはず"
        );
        assert_eq!(
            dom.children.get("a"),
            None,
            "構築失敗時はライブ DOM の子ノードが変更されないはず（fail-closed）"
        );
        assert_eq!(
            dom.children.get("b"),
            Some(&vec![text("new-b")]),
            "他アイテム（b）の Update 適用は妨げられないはず"
        );
    }

    /// codex-review P1 回帰固定（PR #1340、イシュー #1340）: 属性削除を
    /// 伴う Update で子ノード構築（`replace_item_children`）が失敗した
    /// 場合、属性も一切変更されないこと（`sync_attrs` が呼ばれないこと）を
    /// 直接確認する。旧実装は `sync_attrs` を先に呼んでいたため、この
    /// ケースで実 DOM は「属性は新、子ノードは旧」という不整合な状態になり
    /// `stale_update_keys` 経由でキャッシュへ確定させる「達成 Node」
    /// （旧属性・旧子ノード）と食い違っていた。
    #[test]
    fn apply_ops_with_items_leaves_attrs_untouched_when_child_build_fails_on_update() {
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "old"), ("data-removed", "x")],
                vec![text("old-a")],
            ),
        )];
        // 属性削除（data-removed）・属性変更（class）を伴う更新だが、
        // 子ノード構築は失敗する想定（RawHtml 混入相当を fail_replace_
        // children_for で模す）。
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "new")],
                vec![text("new-a")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            fail_replace_children_for: std::collections::HashSet::from(["a".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(
            outcome.stale_update_keys,
            std::collections::HashSet::from(["a".to_string()])
        );
        assert_eq!(
            dom.calls.sync_attrs, 0,
            "子ノード構築が失敗した場合、sync_attrs は一切呼ばれないはず \
             （属性だけ新しい値へ変更済みという不整合状態を防ぐ）"
        );
        assert_eq!(
            dom.attrs.get("a"),
            None,
            "属性はライブ DOM 上でも一切変更されていないはず（class の \
             'old' → 'new' も data-removed の削除も反映されない）"
        );
        assert_eq!(
            dom.children.get("a"),
            None,
            "子ノードも変更されないはず（fail-closed）"
        );
    }

    /// Insert・Move・Update・Remove が同時に発生する複合ケースでも
    /// `final_keys` が正しい最終順序を反映する。
    #[test]
    fn apply_ops_with_items_handles_mixed_ops() {
        let old_items = vec![item("a", "a-old"), item("b", "b"), item("c", "c")];
        // b: 削除。c: 内容変更（Update）。a: そのまま。d: 新規挿入。
        // 並びは [c, d, a]（c が先頭へ移動 + d 挿入 + a は末尾）。
        let new_items = vec![item("c", "c-new"), item("d", "d"), item("a", "a-old")];
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(
            outcome.final_keys,
            vec!["c".to_string(), "d".to_string(), "a".to_string()]
        );
        assert!(outcome.stale_update_keys.is_empty());
        assert!(!outcome.resync_required);
        assert_eq!(
            dom.children.get("c"),
            Some(&vec![text("c-new")]),
            "c の内容変更が反映されているはず"
        );
        assert_eq!(dom.items, outcome.final_keys);
    }

    /// codex-review P1 回帰固定（PR #1340、イシュー #1340）、`Update` を
    /// 発行し得る [`apply_ops_with_items`] 側: `Insert` の構築失敗が起きた
    /// 場合、[`ApplyOutcome::resync_required`] が `true` になり、呼び出し元
    /// （`apply_keyed_list_with_previous`）が「達成 Node」をキャッシュへ
    /// 確定させないこと（`KeyedListApplyResult::ResyncRequired` doc 参照）を
    /// `ApplyOutcome` レベルで保証する。DOM 自体も `index_offset` 補正
    /// （[`apply_ops`] doc 参照）により、`x` を除いた正しい並び `[b, a]` へ
    /// 収束することもあわせて確認する。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_insert_construction_fails() {
        let old_items = vec![item("a", "a"), item("b", "b")];
        let new_items = vec![item("x", "x"), item("b", "b"), item("a", "a")];
        let mut dom = PoisonedCreateDom {
            inner: CountingDom {
                items: vec!["a".to_string(), "b".to_string()],
                ..Default::default()
            },
            poisoned_key: "x".to_string(),
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "x の構築失敗を含む適用は再同期を要求するはず（未達成 op が \
             final_keys からは判別できないため）"
        );
        assert_eq!(
            dom.inner.items,
            vec!["b".to_string(), "a".to_string()],
            "index_offset 補正により、x を除いた正しい並び [b, a] へ \
             収束するはず（apply_ops と同じ補正規則）"
        );
    }

    // --- コスト固定テスト（イシュー #1324、Update op 版）---
    //
    // `apply_ops`（#1318/#1319）の cost-fixed テストは `diff_keys`（内容比較
    // なし）のみを経由するため `Update` を発行しない。`Update` 経路
    // （`apply_ops_with_items`）は `find_by_key` が `Remove`/`Move` 用の
    // sibling 走査 `find_child_by_key` を流用すると「`Update` 件数 ×
    // リスト長」に比例する呼び出しへ退行しうる（`find_by_key` doc 参照、
    // codex 相当レビュー指摘で判明）。特に構造変化を伴わない純粋な内容
    //変更のみの構成（`Insert`/`Move` が 1 件も無く `child_at` が未呼び
    // 出しのケース）は `WebSysKeyedDom` のキャッシュも温まっていないため、
    // 素朴な実装だと最悪ケースになる。本テストはこの「全件 Update・構造
    // 変化なし」という最悪ケースの呼び出し回数を固定する。

    fn items_n(n: usize, content_prefix: &str) -> Vec<(String, Node)> {
        (0..n)
            .map(|i| item(&format!("k{i}"), &format!("{content_prefix}{i}")))
            .collect()
    }

    /// 1,000 行すべての内容が変わり構造変化が一切無い（キー集合・並びは
    /// 完全に同一）最悪ケース: 実測 3,001 回（`first_element_child`
    /// 1 回〔`find_by_key` 初回呼び出しでのキャッシュ構築、以降は実 DOM
    /// 非依存の `Vec` 線形走査〕+ `find_by_key`/`sync_attrs`/
    /// `replace_item_children` 各 1,000 回）に対して +約 17% のタイトな
    /// 上限（3,500 回）で固定する。`find_by_key` が sibling 走査
    /// （`first_element_child`/`next_element_sibling`/`item_key`）へ退行
    /// した場合、この呼び出し回数は N² 相当（実測 500,000 回超）へ跳ね上がる
    /// ため、上限超過はこの O(n²) 退行の再発検知として機能する。
    #[test]
    fn apply_ops_with_items_update_all_1000_rows_with_no_structural_change_stays_linear() {
        const N: usize = 1_000;
        let old_items = items_n(N, "old-");
        let new_items = items_n(N, "new-");
        let mut dom = CountingDom {
            items: (0..N).map(|i| format!("k{i}")).collect(),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(outcome.final_keys.len(), N);
        assert!(outcome.stale_update_keys.is_empty());
        let total = dom.calls.total();
        assert!(
            total <= 3_500,
            "1,000 行全件 Update（構造変化なし）の DOM 操作総数は 3,500 回 \
             以内のはず（実測: {total}、内訳: {:?}）。`find_by_key` の \
             O(1) 相当解決（イシュー #1324）からの退行（sibling 走査への \
             フォールバック等）を検知する上限",
            dom.calls
        );
    }

    /// 既存 1,000 行中 1 件のみ内容が変わるケース（実運用で最も典型的な
    /// 単一項目更新）: 実測 2,001 回程度（`first_element_child` 1 回 +
    /// `find_by_key`/`sync_attrs`/`replace_item_children` 各 1 回、
    /// キャッシュ構築の sibling 走査 999 回分は `next_element_sibling` に
    /// 計上）に対してタイトな上限（1,500 回、末尾要素想定で
    /// `next_element_sibling` 999 回 + 前述 4 回 = 1,003 回に余裕を持たせた
    /// 値）で固定する。
    #[test]
    fn apply_ops_with_items_update_one_of_1000_rows_stays_linear() {
        const N: usize = 1_000;
        let old_items = items_n(N, "v");
        let mut new_items = old_items.clone();
        new_items[0] = item("k0", "changed");
        let mut dom = CountingDom {
            items: (0..N).map(|i| format!("k{i}")).collect(),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(outcome.final_keys.len(), N);
        assert!(outcome.stale_update_keys.is_empty());
        let total = dom.calls.total();
        assert!(
            total <= 1_500,
            "1,000 行中 1 件更新の DOM 操作総数は 1,500 回以内のはず \
             （実測: {total}、内訳: {:?}）",
            dom.calls
        );
    }
}
