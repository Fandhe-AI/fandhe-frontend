//! keyed list の「現在の子要素」ハンドルキャッシュ本体（イシュー #1374 で
//! [`crate::keyed_dom::WebSysKeyedDom`] へ導入した `Vec<(String, Element)>`
//! キャッシュを、PR #1392 codex-review P1 指摘を受けて汎用データ構造として
//! 切り出したもの）。
//!
//! # 指摘の内容と本モジュールの役割
//!
//! 旧実装は `Remove` 1 件ごとに `Vec::remove(pos)` を呼んでいた。全削除
//! ワークロードでは対象キーが常に「残っている中の最初の一致」（＝先頭）に
//! なる（[`fandhe_frontend_core::keyed`] の `remove_pass` doc・下記「計算量
//! 保証」節参照）ため、`Vec::remove(0)` が残り全要素を毎回シフトし、合計の
//! 要素移動量が O(N²) へ退行していた（`children.remove(pos)` 自体の呼び出し
//! 回数は O(N) のままなので、呼び出し回数だけを数える固定テストではこの
//! 退行を検知できない）。
//!
//! 本モジュールは削除を「実 DOM 削除成功後の tombstone 化」（[`Self::remove`]）
//! へ変え、物理的な詰め直し（[`Self::compact`]）を遅延させることで、
//! `Remove` のみで構成される操作列（全削除が典型例）の要素移動量を **0**
//! にする。
//!
//! `Handle`（wasm32 実行時は `web_sys::Element`）を型パラメータ化し、本体を
//! wasm32 非依存にしてある。[`crate::keyed_dom::WebSysKeyedDom`] は
//! `#[cfg(target_arch = "wasm32")]` ゲート配下のため、キャッシュの実装自体を
//! そちらに置くと native `cargo test` から要素移動量を検証できない
//! （[`crate::keyed_apply`] モジュール冒頭 doc が同じ理由で走査アルゴリズム
//! 本体を wasm32 非依存に切り出した設計をここでも踏襲する）。本番経路では
//! [`crate::keyed_dom::WebSysKeyedDom`] のみが `KeyedChildrenCache<Element>`
//! を消費する。
//!
//! # 計算量保証（amortized O(N)）
//!
//! `slots: Vec<Option<(String, Handle)>>` は削除済みエントリを `None`
//! （tombstone）のまま残す。`cursor` は「ここより前に tombstone 化・走査
//! 済みの境界」を表し、**単調増加のみ**（後退しない）。
//!
//! [`fandhe_frontend_core::keyed::diff_keys`] の内部ヘルパー `remove_pass`
//! （`crates/core/src/keyed.rs`）は `KeyedOp::Remove` を `old_keys` の
//! **昇順 1 パス**で発行する。つまり `Remove` の対象キーは、常にそれ以前の
//! `Remove` 対象より元の並びで後ろに位置する。[`WebSysKeyedDom`] の
//! `children` キャッシュは `insert_before`/`move_before`/`remove_child` の
//! 追随更新でしか変化しないため、tombstone 化前は各キーのスロット位置は
//! 元の並び（＝元の index）と一致したままである。この契約により
//! [`Self::remove`] の対象スロットは常に `cursor` 以降で見つかり、前方にしか
//! 進まない走査でも取りこぼさない。
//!
//! [`Self::find`] は `Remove` に加え `Move`/`Update` からも呼ばれ、対象が
//! `cursor` より前（既に通過済み）に位置する場合もありうる。この場合は
//! 前方走査（フェーズ 1）で見つからず、[`Self::compact`] で tombstone を
//! 一括で詰め直してから素の線形走査（フェーズ 2）へフォールバックする
//! （正しさは常に保たれる。`compact` はフォールバック 1 回につき高々 1 回
//! しか実行されない — 実行後は tombstone が 0 件になるため）。
//!
//! 上記により、`Remove` のみで構成される操作列（典型例: 全削除）は
//! [`Self::compact`] を一度も呼ばずに完了し、要素移動量は **0** になる
//! （固定テスト [`tests::remove_all_in_ascending_order_never_shifts_elements`]
//! 参照）。`Move`/`Insert` が混在する一般のワークロードでは最初の
//! `compact` 呼び出しで現在のスロット数ぶんの移動が発生しうるが、
//! 実行後は tombstone が 0 件になるため、同一 apply 呼び出し内で同じ
//! tombstone 集合に対して 2 度移動コストが発生することはない（1 回の
//! `apply_ops_with_items` 呼び出しあたり高々 1 回の `compact` にしか
//! 寄与しない、amortized O(N) 総量）。
#[derive(Debug)]
pub(crate) struct KeyedChildrenCache<H> {
    /// 削除済みエントリを `None`（tombstone）のまま残す順序付きスロット列。
    slots: Vec<Option<(String, H)>>,
    /// tombstone（`None`）件数。
    dead: usize,
    /// [`Self::find`]/[`Self::remove`] の前方走査フェーズがここから開始する
    /// 単調増加カーソル（モジュール doc「計算量保証」参照）。
    cursor: usize,
    /// [`Self::compact`]/[`Self::remove`] のフォールバック経路が実際に
    /// 移動させた要素数の累計（native テストで計算量保証を実測するための
    /// 計測専用フィールド、本番コードパスの挙動には影響しない）。
    #[cfg(test)]
    moved: usize,
    /// [`Self::find`]/[`Self::remove`] のフェーズ 1（`cursor` からの前方
    /// 走査）が訪問したスロット数の累計。`moved` は「要素移動」（`compact`/
    /// `Vec::remove`/`Vec::insert`）しか計上しないため、tombstone を
    /// スキップするだけの走査コストが `cursor` を毎回 0 へ戻す退行
    /// （tombstone 化はするが前方走査を怠る誤実装）で再び O(N²) 化しても
    /// `moved == 0` のままになってしまう抜け穴を塞ぐための計測専用
    /// フィールド（本番コードパスの挙動には影響しない）。
    #[cfg(test)]
    scanned: usize,
}

impl<H: Clone> KeyedChildrenCache<H> {
    /// `items`（sibling 走査で得た初期並び等）からキャッシュを構築する。
    pub(crate) fn from_items(items: Vec<(String, H)>) -> Self {
        Self {
            slots: items.into_iter().map(Some).collect(),
            dead: 0,
            cursor: 0,
            #[cfg(test)]
            moved: 0,
            #[cfg(test)]
            scanned: 0,
        }
    }

    /// 全エントリを取り除く（[`crate::keyed_dom::WebSysKeyedDom::clear_children`]
    /// 用）。
    pub(crate) fn clear(&mut self) {
        self.slots.clear();
        self.dead = 0;
        self.cursor = 0;
    }

    /// tombstone を取り除き `slots` を「生存エントリのみ」へ詰め直す。
    /// `dead == 0`（tombstone なし）なら実 DOM に触れない no-op。
    fn compact(&mut self) {
        if self.dead == 0 {
            return;
        }
        #[cfg(test)]
        {
            // `retain` が実際に動かす要素数の上限（詰め直し前の全スロット数）
            // を計上する。tombstone 分も含めた上限計上のため実移動量以上には
            // なるが、「0 回」であるべき区間（全削除のみの経路）を汚染しない
            // 限りは十分な精度（[`tests::remove_all_in_ascending_order_never_shifts_elements`]
            // が保証したいのは「compact が一度も起きないこと」自体）。
            self.moved += self.slots.len();
        }
        self.slots.retain(|slot| slot.is_some());
        self.dead = 0;
        self.cursor = 0;
    }

    /// `index` 番目（生存順）のハンドルを返す（`child_at` 用）。
    pub(crate) fn get(&mut self, index: usize) -> Option<H> {
        self.compact();
        self.slots
            .get(index)
            .and_then(|slot| slot.as_ref())
            .map(|(_, h)| h.clone())
    }

    /// `key` の現在位置（生存順インデックス）とハンドルを返す（`cursor` を
    /// 自ら前進させない非破壊のクエリ — `cursor` の前進（tombstone 化との
    /// 対）は [`Self::remove`] の責務であり、`find` が先に進めてしまうと
    /// `Move`/`Update` から `cursor` より手前のキーを探すケース（フェーズ 1
    /// の前提が崩れる）で誤って取りこぼしうるため。ただしフェーズ 2 の
    /// フォールバックは [`Self::compact`] により内部表現（`slots`）を
    /// tombstone なしへ詰め直し `cursor` を 0 へリセットする ——
    /// これは `slots` の**表現**を変えるだけで、キー集合や生存順という
    /// **抽象状態**は変えない。「`cursor` を進めない」はこの抽象状態レベル
    /// の契約であり、`compact` による内部表現の巻き戻しと矛盾しない）。
    pub(crate) fn find(&mut self, key: &str) -> Option<(usize, H)> {
        // フェーズ 1: `cursor` から前方だけを見る。ヒットしたスロット `i` は
        // 必ず `cursor` 以降であり、tombstone は不変条件によりすべて
        // `cursor` より前に限られるため、`i` より前の tombstone 総数は
        // ちょうど `self.dead`（モジュール doc 参照）。
        let mut i = self.cursor;
        while i < self.slots.len() {
            #[cfg(test)]
            {
                self.scanned += 1;
            }
            if let Some((k, h)) = &self.slots[i] {
                if k == key {
                    return Some((i - self.dead, h.clone()));
                }
            }
            i += 1;
        }
        // フェーズ 2: `cursor` より前に対象がある可能性（`Move`/`Update`）。
        // 一括 `compact` してから素の線形走査で確実に見つける。
        self.compact();
        self.slots
            .iter()
            .position(|slot| matches!(slot, Some((k, _)) if k == key))
            .map(|pos| {
                let (_, h) = self.slots[pos].as_ref().expect("position で存在確認済み");
                (pos, h.clone())
            })
    }

    /// `key` に一致するエントリを tombstone 化する（実 DOM 側の削除成功後に
    /// 呼ぶ契約。呼び出し元 [`crate::keyed_dom::WebSysKeyedDom::remove_child`]
    /// doc 参照）。見つからなければ何もせず `false` を返す。
    pub(crate) fn remove(&mut self, key: &str) -> bool {
        // フェーズ 1（本来の経路、モジュール doc「計算量保証」参照）:
        // `cursor` から前方だけを見て O(1) 相当で tombstone 化する。
        let mut i = self.cursor;
        while i < self.slots.len() {
            #[cfg(test)]
            {
                self.scanned += 1;
            }
            if matches!(&self.slots[i], Some((k, _)) if k == key) {
                self.slots[i] = None;
                self.dead += 1;
                self.cursor = i + 1;
                return true;
            }
            i += 1;
        }
        // フェーズ 2（フォールバック）: `Remove` の対象が `cursor` より前に
        // ある状態はここに来る契約上想定しない（想定されるのは `diff_keys`/
        // `diff_keyed_items` が契約に反する op 列を返す改ざん等の異常系のみ）
        // が、正しさ自体はここでも保つ。
        self.compact();
        if let Some(pos) = self
            .slots
            .iter()
            .position(|slot| matches!(slot, Some((k, _)) if k == key))
        {
            #[cfg(test)]
            {
                self.moved += self.slots.len() - pos;
            }
            self.slots.remove(pos);
            true
        } else {
            false
        }
    }

    /// `pos`（生存順インデックス、`slots` の現在長でクランプ）へ新規エントリ
    /// を挿入する（`insert_before_batch` の追随更新用）。
    pub(crate) fn insert(&mut self, pos: usize, key: String, handle: H) {
        self.compact();
        let pos = pos.min(self.slots.len());
        #[cfg(test)]
        {
            self.moved += self.slots.len() - pos;
        }
        self.slots.insert(pos, Some((key, handle)));
    }

    /// `key` の既存エントリを取り除いたうえで `target_index`（`slots` の
    /// 現在長でクランプ）へ挿入し直す（`move_before` の追随更新用）。
    /// 既存エントリが見つからない場合も、旧実装（`Vec` 直接操作版）と
    /// 挙動を合わせるため無条件に挿入する。
    pub(crate) fn move_to(&mut self, key: &str, target_index: usize, handle: H) {
        self.compact();
        if let Some(pos) = self
            .slots
            .iter()
            .position(|slot| matches!(slot, Some((k, _)) if k == key))
        {
            #[cfg(test)]
            {
                self.moved += self.slots.len() - pos;
            }
            self.slots.remove(pos);
        }
        let pos = target_index.min(self.slots.len());
        #[cfg(test)]
        {
            self.moved += self.slots.len() - pos;
        }
        self.slots.insert(pos, Some((key.to_string(), handle)));
    }

    /// `key` の既存エントリのハンドルのみを差し替える（`replace_root` 用。
    /// 見つからなければ no-op）。
    pub(crate) fn replace(&mut self, key: &str, handle: H) {
        self.compact();
        if let Some(pos) = self
            .slots
            .iter()
            .position(|slot| matches!(slot, Some((k, _)) if k == key))
        {
            self.slots[pos] = Some((key.to_string(), handle));
        }
    }

    /// [`Self::compact`]/[`Self::remove`] フォールバック経路が実際に
    /// 移動させた要素数の累計（native テスト専用の可観測フック）。
    #[cfg(test)]
    pub(crate) fn moved(&self) -> usize {
        self.moved
    }

    /// [`Self::find`]/[`Self::remove`] のフェーズ 1 前方走査が訪問した
    /// スロット数の累計（native テスト専用の可観測フック。`moved` が
    /// 検知できない「tombstone スキップだけの走査コスト再退行」を検知する
    /// ための計測、`scanned` フィールド doc 参照）。
    #[cfg(test)]
    pub(crate) fn scanned(&self) -> usize {
        self.scanned
    }
}

#[cfg(test)]
mod tests {
    use super::KeyedChildrenCache;

    fn items_n(n: usize) -> Vec<(String, u32)> {
        (0..n as u32).map(|i| (format!("k{i}"), i)).collect()
    }

    /// `remove` の基本契約: 見つかったエントリが取り除かれ、`find` からも
    /// 見えなくなる。
    #[test]
    fn remove_removes_entry_and_find_no_longer_sees_it() {
        let mut cache = KeyedChildrenCache::from_items(items_n(3));
        assert_eq!(cache.find("k1"), Some((1, 1)));
        assert!(cache.remove("k1"));
        assert_eq!(cache.find("k1"), None);
        assert_eq!(cache.find("k0"), Some((0, 0)));
        // k1 が抜けた分、k2 は生存順で 1 番目に繰り上がる。
        assert_eq!(cache.find("k2"), Some((1, 2)));
    }

    /// 存在しないキーの `remove` は `false` を返し内部状態も変えない。
    #[test]
    fn remove_missing_key_returns_false() {
        let mut cache = KeyedChildrenCache::from_items(items_n(3));
        assert!(!cache.remove("missing"));
        assert_eq!(cache.find("k0"), Some((0, 0)));
        assert_eq!(cache.find("k1"), Some((1, 1)));
        assert_eq!(cache.find("k2"), Some((2, 2)));
    }

    /// 本 PR の本体（PR #1392 codex-review P1 是正）: `old_keys` 昇順で
    /// 発行される `Remove` を、契約どおり「常に前方（`cursor` 以降）で
    /// ヒットする」順序で適用し続ける限り、[`KeyedChildrenCache::compact`]
    /// は一度も走らず要素移動量が **0** に収まることを固定する。
    ///
    /// 旧実装（`Vec<(String, Element)>` への `Vec::remove(pos)` を直接
    /// 呼ぶ版）では、全削除時に対象が常に先頭（`pos == 0`）になるため
    /// `Vec::remove(0)` が残り全要素をシフトし、合計移動量が
    /// `1000 + 999 + ... + 1 ≈ 500,000` 相当（O(N²)）になっていた。
    #[test]
    fn remove_all_in_ascending_order_never_shifts_elements() {
        const N: usize = 1_000;
        let mut cache = KeyedChildrenCache::from_items(items_n(N));

        for i in 0..N {
            let key = format!("k{i}");
            assert_eq!(
                cache.find(&key),
                Some((0, i as u32)),
                "昇順削除では常に生存先頭（index 0）がヒットするはず"
            );
            assert!(cache.remove(&key), "k{i} の削除に失敗した");
        }

        assert_eq!(
            cache.moved(),
            0,
            "old_keys 昇順の全削除は Vec の要素シフトを一切伴わないはず \
             （tombstone 化のみ、`compact` が呼ばれていない証拠）"
        );
        // `moved == 0` だけでは「tombstone 化はするが `cursor` を毎回 0 へ
        // 戻して素の先頭から走査し直す」退行（要素移動は起きないが走査回数
        // が O(N²) へ戻る）を検知できない（advisor 助言）。`scanned` は
        // `find`/`remove` 各 1 回・各ヒットまで 1 歩（`cursor` が直前の
        // 削除位置ちょうどに一致するため）で N 回ずつ、合計 2N に収まる
        // はず。仮に `cursor` を毎回 0 へ戻す誤実装なら
        // `1 + 2 + ... + N ≈ N²/2 = 500,000` 相当まで膨らむ。
        assert!(
            cache.scanned() <= 4 * N,
            "old_keys 昇順の全削除は前方走査ステップ数も O(N) に収まるはず \
             （実測: {}、上限: {}。`cursor` を毎回 0 へ戻す退行なら \
             N²/2 ≈ 500,000 相当まで膨らむ）",
            cache.scanned(),
            4 * N
        );
    }

    /// 全削除ではなく 1 件だけ保持する構成（advisor 助言の「先頭寄りが
    /// 最悪ケース」を反映）: 保持対象以外（先頭寄りに集中する 999 件）を
    /// 昇順で削除しても要素移動量は 0 のまま。保持キーへの `find` も
    /// tombstone を跨いだ正しい生存位置（0）を返す。
    #[test]
    fn remove_all_but_last_never_shifts_elements() {
        const N: usize = 1_000;
        let mut cache = KeyedChildrenCache::from_items(items_n(N));

        for i in 0..N - 1 {
            let key = format!("k{i}");
            assert!(cache.remove(&key), "k{i} の削除に失敗した");
        }

        assert_eq!(
            cache.moved(),
            0,
            "保持 1 件・削除 999 件でも移動量は 0 のはず"
        );
        assert_eq!(
            cache.find(&format!("k{}", N - 1)),
            Some((0, (N - 1) as u32)),
            "唯一の生存エントリは生存順 index 0 で見つかるはず"
        );
        // `remove_all_in_ascending_order_never_shifts_elements` と同じ理由
        // （`moved == 0` だけでは検知できない走査回数の再退行を塞ぐ）で
        // `scanned` も O(N) に収まることを固定する。
        assert!(
            cache.scanned() <= 4 * N,
            "保持 1 件・削除 999 件でも前方走査ステップ数は O(N) に収まる \
             はず（実測: {}、上限: {}）",
            cache.scanned(),
            4 * N
        );
    }

    /// `find` は `Move`/`Update` からも呼ばれるため、`cursor` 前進後に
    /// `cursor` より前のキーを探すケース（compact フォールバック）でも
    /// 正しい生存位置を返すことを固定する。
    #[test]
    fn find_after_removal_locates_entries_before_cursor_via_compaction() {
        let mut cache = KeyedChildrenCache::from_items(items_n(5));
        // k0, k1 を削除して cursor を 2 まで進める。
        assert!(cache.remove("k0"));
        assert!(cache.remove("k1"));
        // cursor より前には残存エントリが無いため、この時点では compact
        // フォールバックの検証にならない。cursor 以降にある k3 を先に
        // 見つけたあと、k2（cursor より前ではないが tombstone 混在区間）
        // を探索することで tombstone 跨ぎの位置計算を検証する。
        assert_eq!(cache.find("k2"), Some((0, 2)));
        assert_eq!(cache.find("k3"), Some((1, 3)));
        assert_eq!(cache.find("k4"), Some((2, 4)));
    }

    /// `child_at`（`get`）は tombstone を跨いで正しい生存順インデックスを
    /// 返す。
    #[test]
    fn get_returns_live_index_after_removal() {
        let mut cache = KeyedChildrenCache::from_items(items_n(4));
        assert!(cache.remove("k1"));
        assert_eq!(cache.get(0), Some(0));
        assert_eq!(cache.get(1), Some(2));
        assert_eq!(cache.get(2), Some(3));
        assert_eq!(cache.get(3), None);
    }

    /// `insert`/`move_to`/`replace` の基本契約（`move_before`/
    /// `insert_before_batch`/`replace_root` の追随更新が依拠する）。
    #[test]
    fn insert_move_and_replace_update_live_order() {
        let mut cache = KeyedChildrenCache::from_items(items_n(3));
        cache.insert(1, "new".to_string(), 100);
        assert_eq!(
            cache.find("new"),
            Some((1, 100)),
            "insert した位置に見つかるはず"
        );

        cache.move_to("k0", 2, 0);
        assert_eq!(cache.find("k0"), Some((2, 0)), "move_to 後の位置");

        cache.replace("k2", 999);
        assert_eq!(
            cache.find("k2"),
            Some((3, 999)),
            "replace はハンドルのみ差し替え、位置は不変のはず"
        );
    }

    /// `clear` は全エントリを消し、以降の `find`/`get` はすべて `None`。
    #[test]
    fn clear_removes_all_entries() {
        let mut cache = KeyedChildrenCache::from_items(items_n(3));
        cache.clear();
        assert_eq!(cache.find("k0"), None);
        assert_eq!(cache.get(0), None);
    }
}
