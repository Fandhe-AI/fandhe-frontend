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

use crate::keyed_diff::{diff_keys, KeyedOp};

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
pub(crate) fn apply_ops<D: KeyedListDom>(dom: &mut D, new_keys: &[String]) {
    let old_keys = dom_item_keys(dom);
    let ops = diff_keys(&old_keys, new_keys);
    let mut i = 0;
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
                    }
                    // RawHtml 混入等の構築失敗（`None`）は当該アイテムのみ
                    // `items` から除外する fail-closed skip（本関数 doc
                    // 参照）。`expected_index` は診断済みの `diff_keys`
                    // 計画どおり進める（失敗の有無に関わらず区間検出の
                    // 基準は変えない）。
                    expected_index += 1;
                    j += 1;
                }
                if !items.is_empty() {
                    let reference = dom.child_at(start_index);
                    dom.insert_before_batch(start_index, items, reference.as_ref());
                }
                i = j;
            }
            KeyedOp::Move { index, key } => {
                if let Some(existing) = find_child_by_key(dom, key) {
                    let reference = dom.child_at(*index);
                    dom.move_before(*index, key, &existing, reference.as_ref());
                }
                i += 1;
            }
        }
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
    }

    impl CallCounts {
        /// 実 DOM 呼び出しに数える全メソッドの合計
        /// （`item_key` は `get_attribute` 相当で実 DOM 呼び出しを伴うため
        /// 合計に含める。`insert_before_batch` はイシュー #1320 で連続
        /// Insert 区間 1 件につき 1 回に集約されるため、旧
        /// `insert_before`（アイテム 1 件ごとに 1 回）より少ない回数になる。
        /// 1,000 行 create の上限値コメントの内訳と対応する）。
        fn total(&self) -> usize {
            self.first_element_child
                + self.next_element_sibling
                + self.item_key
                + self.create_item
                + self.insert_before_batch
                + self.move_before
                + self.remove_child
                + self.child_at
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
            calls: CallCounts::default(),
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
            calls: CallCounts::default(),
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
            calls: CallCounts::default(),
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
            calls: CallCounts::default(),
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
    }

    #[test]
    fn apply_ops_skips_only_the_item_whose_create_item_fails_within_a_run() {
        let mut dom = PoisonedCreateDom {
            inner: CountingDom {
                items: vec!["a".to_string()],
                calls: CallCounts::default(),
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

    // --- 意味的一致の確認（モックとアルゴリズムが噛み合っていることの
    // 担保、上記コスト値の信頼性の裏付け） ---

    /// 削除: 対象キーのみが取り除かれ、他のキー列の順序は保たれる。
    #[test]
    fn apply_ops_removes_only_target_key() {
        let mut dom = CountingDom {
            items: keys_n(3),
            calls: CallCounts::default(),
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
            calls: CallCounts::default(),
        };
        let new_keys = vec!["k2".to_string(), "k0".to_string(), "k1".to_string()];

        apply_ops(&mut dom, &new_keys);

        assert_eq!(dom.items, new_keys);
    }
}
