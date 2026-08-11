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
//!   （1,000 行）で `nth_element_child`/`next_element_sibling` の sibling
//!   走査が累積 O(n²) になる」問題を、実ブラウザ計測（不安定・低速）では
//!   なく `cargo test` レベルで決定的に再発検知する（本イシュー #1318 の
//!   目的そのもの）。
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

    /// `key` に対応する新規ノードを構築する。構築失敗（`RawHtml` 混入等）
    /// は `None`（呼び出し元は当該 `Insert` 1 件を丸ごと skip する、
    /// 本モジュール doc「セキュリティ不変条件の引き継ぎ」参照）。
    fn create_item(&mut self, key: &str) -> Option<Self::NewNode>;

    /// `node` を `reference`（`None` なら末尾）の直前へ挿入する
    /// （`Element::insert_before` の Insert 用途）。
    fn insert_before(&mut self, node: Self::NewNode, reference: Option<&Self::Handle>);

    /// 既存の `child` を `reference`（`None` なら末尾）の直前へ移動する
    /// （`Element::insert_before` の Move 用途。既存ノード参照を保持した
    /// まま移動することがフォーカス・入力途中の値の保持に直結する、
    /// `keyed_diff` モジュール doc §5.3 参照）。
    fn move_before(&mut self, child: &Self::Handle, reference: Option<&Self::Handle>);

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

/// `index` 番目（0-origin）の子要素を返す（`insert_before`/`move_before` の
/// 参照ノード決定に使う。`index` が子要素数以上なら `None` = 末尾、
/// [`crate::keyed_dom::nth_element_child`] の等価移植）。
///
/// この関数の 1 呼び出しが `first_element_child` 1 回 + `index` 回の
/// `next_element_sibling` を要する（= O(index)）ことが、`apply_ops` を
/// `Insert`/`Move` の度に呼ぶ現行実装が累積 O(n²) になる直接の原因
/// （ルート issue #1313・本イシュー #1318 が固定するコストの本体）。
fn nth_element_child<D: KeyedListDom>(dom: &mut D, index: usize) -> Option<D::Handle> {
    let mut maybe_child = dom.first_element_child();
    let mut i = 0;
    while let Some(child) = maybe_child {
        if i == index {
            return Some(child);
        }
        i += 1;
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
pub(crate) fn apply_ops<D: KeyedListDom>(dom: &mut D, new_keys: &[String]) {
    let old_keys = dom_item_keys(dom);
    let ops = diff_keys(&old_keys, new_keys);
    for op in ops {
        match op {
            KeyedOp::Remove { key } => {
                if let Some(child) = find_child_by_key(dom, &key) {
                    dom.remove_child(&child);
                }
            }
            KeyedOp::Insert { index, key } => {
                let Some(new_node) = dom.create_item(&key) else {
                    // RawHtml 混入等の構築失敗: 当該アイテムのみ丸ごと skip
                    // する fail-closed（本モジュール doc 参照）。
                    continue;
                };
                let reference = nth_element_child(dom, index);
                dom.insert_before(new_node, reference.as_ref());
            }
            KeyedOp::Move { index, key } => {
                let Some(existing) = find_child_by_key(dom, &key) else {
                    continue;
                };
                let reference = nth_element_child(dom, index);
                dom.move_before(&existing, reference.as_ref());
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
        insert_before: usize,
        move_before: usize,
        remove_child: usize,
    }

    impl CallCounts {
        /// 実 DOM 呼び出しに数える全メソッドの合計
        /// （`item_key` は `get_attribute` 相当で実 DOM 呼び出しを伴うため
        /// 合計に含める。1,000 行 create の上限値コメントの内訳と対応する）。
        fn total(&self) -> usize {
            self.first_element_child
                + self.next_element_sibling
                + self.item_key
                + self.create_item
                + self.insert_before
                + self.move_before
                + self.remove_child
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

        fn create_item(&mut self, key: &str) -> Option<Self::NewNode> {
            self.calls.create_item += 1;
            Some(key.to_string())
        }

        fn insert_before(&mut self, node: Self::NewNode, reference: Option<&Self::Handle>) {
            self.calls.insert_before += 1;
            let pos = match reference {
                Some(r) => self
                    .items
                    .iter()
                    .position(|k| k == r)
                    .unwrap_or(self.items.len()),
                None => self.items.len(),
            };
            self.items.insert(pos, node);
        }

        fn move_before(&mut self, child: &Self::Handle, reference: Option<&Self::Handle>) {
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

    // --- コスト固定テスト（イシュー #1318 本体） ---
    //
    // 上限値は「現状実測値（O(n²) 相当）+ 小さな余裕」で固定し、O(n²) →
    // O(n³) や定数倍の悪化（sibling 走査の重複化等）が起きた場合にのみ
    // FAIL するようにする。Phase 2-1（#1319、`children().item()` による
    // O(1) 化）が実装された時点で、この上限は O(n) 相当（例: 数千回）へ
    // 絞り直す（`#[ignore]` は使わず、常に green を維持したまま複雑度
    // クラスの回帰を検知し続けることが本イシューの目的）。

    /// 空 → 1,000 行の create: DOM 操作の総呼び出し回数が現状値
    /// （実測 502,501 回。内訳: `first_element_child` 1,001 回（初期の
    /// `dom_item_keys` 読み 1 回 + `Insert` 1,000 件それぞれの
    /// `nth_element_child` 内 1 回）+ `next_element_sibling`
    /// Σ(i=0..999) i = 499,500 回 + `item_key` 0 回（旧キー列が空のため
    /// `dom_item_keys` のループ本体が 1 度も回らない）+
    /// `create_item`/`insert_before` 各 1,000 回）を大きく超えないこと。
    /// `next_element_sibling` が支配項であり、これが
    /// `nth_element_child`（O(index) の sibling 走査）を `Insert` の度に
    /// 呼ぶことに起因する O(n²) の本体（[`nth_element_child`] rustdoc
    /// 参照）。上限 600,000 は現状値の約 1.19 倍で、O(n²) → O(n³) や
    /// 定数倍の悪化を検知できる余裕として設定した。
    #[test]
    fn apply_ops_create_1000_rows_from_empty_stays_within_current_o_n_squared_cost() {
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
            total <= 600_000,
            "1,000 行 create の DOM 操作総数は 600,000 回以内のはず \
             （実測: {total}、内訳: {:?}）。O(n²) の悪化さらなる複雑度クラス \
             悪化（O(n³) 等）の再発を検知する上限であり、Phase 2-1（#1319）\
             完了後は 20,000 以下へ絞る計画",
            dom.calls
        );
    }

    /// 既存 1,000 行の先頭へ 1 件挿入: 単発挿入は O(n) （`nth_element_child`
    /// が高々 1 回、`dom_item_keys` の初期走査が 1,000 行分）であり、
    /// 実測 ≈ 2,004 回（`first_element_child` 2 回 + `next_element_sibling`
    /// 1,000 回 + `item_key` 1,000 回 + `create_item`/`insert_before` 各 1
    /// 回）に対して +2 割弱のタイトな上限（2,500 回）で固定する
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
    }

    /// 既存 1,000 行の末尾へ 1 件挿入: `nth_element_child(index=1000)` が
    /// 子要素数を超えるため全 sibling を走査する分だけ先頭挿入より重いが、
    /// それでも O(n) の範囲（実測 3,004 回: `first_element_child` 2 回 +
    /// `next_element_sibling` 2,000 回（`dom_item_keys` の初期走査 1,000 回 +
    /// `nth_element_child` の走査 1,000 回）+ `item_key` 1,000 回 +
    /// `create_item`/`insert_before` 各 1 回）。+2 割弱のタイトな上限
    /// （3,500 回）で固定する。
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
            total <= 3_500,
            "末尾 1 件挿入の DOM 操作総数は 3,500 回以内のはず（実測: {total}、\
             内訳: {:?}）",
            dom.calls
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
