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
//!
//! # 属性検証拒否と「達成 Node」の整合（イシュー #1340 codex-review P1
//! 〔4 巡目〕対応）
//!
//! `sync_attrs`（in-place 更新）・`build_dom_node_with_namespace`（新規
//! 構築、`create_item`/`replace_item_children`/`replace_root` が経由）は
//! いずれも URL スキーム・イベントハンドラ・`srcset` の検証を通過しない
//! 属性の書き込みを skip する（不変条件 1〜4 の維持）。この skip は
//! `new_attrs`/`new_children` の一部だけを反映しない「部分達成」であり、
//! 旧実装は「達成 Node」の合成時にこれを無視して `new_by_key`（検証前の
//! 望ましい view）をそのまま使っていたため、拒否された属性値が実際には
//! DOM に書き込まれていないにもかかわらず「達成 Node」（＝次回 diff 基準
//! としてキャッシュされる内容）へ紛れ込み、実 DOM とキャッシュが恒久的に
//! 乖離していた（危険 URL 属性を拒否した回のキャッシュに危険値が残り、
//! 以降同じ view を再適用しても差分なしと判定されて DOM の旧値のまま
//! 固定される、PR #1340 codex-review 指摘）。
//!
//! [`compose_achieved_children`]/[`sanitize_node_for_achieved`] は、この
//! 検証拒否を「達成 Node」の合成へも一貫して反映する。検証述語
//! （`is_event_handler_attr`/`is_url_attr`+`is_safe_url`/`srcset`）は
//! `(属性名, 属性値)` のみから決定的に判定できる純粋関数であり、実際の
//! `web-sys` 呼び込みを介さず同じ判定を再現できるため、
//! `build_dom_node_with_namespace`（新規構築経路）のシグネチャ自体は変更
//! しない（呼び出し元が「達成 Node」合成時に同一ロジックを再計算するだけ
//! で十分）。決定的な拒否のため [`KeyedListApplyResult::ResyncRequired`]
//! （`crate::keyed_dom` 参照）は使わない: 同じ危険値を持つ view が再適用
//! されても毎回同じ属性が同じ理由で拒否されるだけで、無限再同期には
//! ならない。
//!
//! # `sync_attrs` の実行時失敗と「達成 Node」の整合（イシュー #1340
//! codex-review P1〔5 巡目〕対応）
//!
//! 上記の検証拒否（`is_event_handler_attr` 等、値のみから決定できる
//! ポリシー判断）とは別に、`setAttribute`/`removeAttribute` 自体が実行時に
//! 失敗しうる（`Node::Element` はタグ名と異なり属性名を構築時に検証
//! しないため、空白等を含む不正な属性名では `InvalidCharacterError` 相当の
//! `Err` を返しうる）。旧実装は `sync_attrs` 内でこの `Result` を
//! `let _ =` で破棄し無条件に「新属性へ更新済み」として扱っていたため、
//! `replace_item_children`（子ノード）は新内容へ更新できても属性だけが
//! 旧値のまま実 DOM に残る場合に、`compose_achieved_children` が新属性を
//! 達成済みとしてキャッシュしてしまい、以降差分が出ず DOM が目標状態へ
//! 収束しなくなっていた（PR #1340 codex-review 指摘）。
//!
//! この失敗は `(属性名, 属性値)` だけからは決定できず（ブラウザの属性名
//! 検証規則に依存する実行時の事実）、上記のポリシー判断のように呼び出し元
//! で再計算することができない。そのため [`KeyedListDom::sync_attrs`] の
//! 契約を「実際に達成できた属性の状態（呼び出し後にライブ DOM が実際に
//! 持つ、予約属性 `data-key` を除く属性集合）を返す」へ変更した。
//!
//! # 読み戻しの決定的正規化（イシュー #1340 codex-review Bugbot〔6 巡目〕対応）
//!
//! 上記の読み戻しを素朴に `child.attributes()`（`NamedNodeMap`）の全列挙で
//! 実装すると、生の列挙順（新規属性は末尾追加）・HTML の属性名小文字化を
//! そのまま返してしまい、後段の `diff_keyed_items`（順序・大小文字に敏感な
//! `Vec<(String, String)>` の `PartialEq` で比較）が全操作成功の正常系
//! ですら次回 view と不一致と判定し、`Update`（`replace_item_children` を
//! 含む）が毎 tick 再発火して子ノードが再マウントされ続ける不具合が
//! あった（`crate::keyed_apply::KeyedListDom::sync_attrs` 実装当初の
//! 版、codex-review 指摘）。
//!
//! [`crate::keyed_dom::WebSysKeyedDom::sync_attrs`] はこれを避けるため、
//! 戻り値の**合成**は `new_attrs`/`old_attrs`（呼び出し元
//! [`apply_ops_with_items`]/`sync_parent_attrs` が渡す、vdom 側の表記）に
//! ある属性名だけを `child.get_attribute(name)` で個別照会して行う
//! （[`KeyedListDom::sync_attrs`] doc「決定的な正規化契約」参照）。
//! 全操作成功時は戻り値が `new_attrs` とバイト等価になり、失敗時のみ実際の
//! DOM 状態が反映される。[`ApplyOutcome::achieved_attrs`] がこの戻り値を
//! キーごとに保持し、[`compose_achieved_children`] は in-place 更新
//! （`sync_attrs` 経由）のキーについてこの実測値をそのまま使う（もはや
//! `sanitize_node_for_achieved` によるポリシー再計算は不要 —— 読み戻し値が
//! ポリシー拒否も実行時失敗も両方織り込み済みのため）。
//!
//! # 削除判定とライブ属性ドリフト（イシュー #1340 codex-review Bugbot
//! 〔8 巡目〕対応）
//!
//! 戻り値の**合成**は上記のとおり `new_attrs`/`old_attrs` 基準の個別照会
//! を維持するが、**削除**（`remove_attribute` を呼ぶかどうかの判定）は
//! `old_attrs`（キャッシュされた直前の内容）だけを基準にしてはならない。
//! SSR hydrate 直後はサーバー側 HTML に含まれる属性がライブ DOM 上に存在
//! する一方、クライアント側の `old_attrs` キャッシュにその属性が含まれ
//! ない構成があり得る（hydrate 由来のライブ属性ドリフト）。`old_attrs`
//! のみを削除判定に使うと、`new_attrs` にも存在しないその属性が Update を
//! 何度適用しても永遠に除去されない不具合があった（codex-review 指摘）。
//! [`crate::keyed_dom::WebSysKeyedDom::sync_attrs`] は削除**判定**にのみ
//! `child.attributes()`（`NamedNodeMap` 全列挙）を使う（`reserved_attr`・
//! `new_attrs` 側の名前を除いた実属性すべてを削除候補とする）。列挙は
//! 削除対象の名前集合を決めるためだけに使い、値の取得・戻り値への格納は
//! 引き続き `get_attribute` の個別照会を経由するため、上記「決定的な正規化
//! 契約」（正常系のバイト等価性）と両立する。
//!
//! 新規構築経路（`build_dom_node_with_namespace` が経由する `create_item`/
//! `replace_item_children` の新規子孫・タグ変更を伴う `replace_root`）は
//! 対象外: `set_attribute` の実行時失敗は `.ok()?` で要素構築全体を
//! `None` にする既存の fail-closed 契約（本モジュール doc「セキュリティ
//! 不変条件の引き継ぎ」参照）にそのまま含まれ、当該 op 自体が丸ごと
//! 未達成（`Insert` の `failed_inserts`・`replace_root` の失敗による
//! `resync_required`・`replace_item_children` 失敗による
//! `stale_update_keys`）として扱われるため、達成 Node が「構築されなかった
//! 要素の属性」を含んでしまう余地が構造的に存在しない。

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
    ///
    /// # 戻り値（イシュー #1340 codex-review P1〔3 巡目〕全走査対応）
    ///
    /// 実 DOM への `insert_before`（`web-sys` 実装では
    /// `Element::insert_before` 相当）が失敗した場合は `false` を返し、
    /// 呼び出し元（[`apply_ops`]/[`apply_ops_with_items`]）は当該区間を
    /// 未達成スロットとして扱う（`create_item` 失敗や `Move` 対象キー未検出
    /// と同じ「未達成 op」の枠組み、[`apply_ops`] doc「未達成スロットの
    /// index 補正」参照）。実装は失敗時に**内部の索引キャッシュを更新して
    /// はならない**（実 DOM へ反映されていないノードをキャッシュ上だけ
    /// 「挿入済み」として扱うと、以後の差分基準が実 DOM と恒久的に乖離
    /// する。`replace_root`/`replace_item_children` と同じ「キャッシュ更新は
    /// 完全成功時のみ」契約）。
    fn insert_before_batch(
        &mut self,
        start_index: usize,
        items: Vec<(String, Self::NewNode)>,
        reference: Option<&Self::Handle>,
    ) -> bool;

    /// 既存の `child`（キー `key`）を `reference`（`None` なら末尾）の直前
    /// の「新しい並びでの位置」`index` へ移動する（`Element::insert_before`
    /// の Move 用途。既存ノード参照を保持したまま移動することがフォーカス・
    /// 入力途中の値の保持に直結する、`keyed_diff` モジュール doc §5.3
    /// 参照）。`index`/`key` の用途は [`Self::insert_before`] と同じ。
    ///
    /// # 戻り値（イシュー #1340 codex-review P1〔3 巡目〕全走査対応）
    ///
    /// 実 DOM への `insert_before` が失敗した場合は `false` を返す。
    /// `move_before` は単一の `insert_before` 呼び出しのみで構成され、
    /// 失敗時は仕様上 DOM を一切変更しない（既存ノードの親子関係・兄弟順は
    /// 呼び出し前のまま）ため [`Self::insert_before_batch`]/
    /// [`crate::keyed_apply::RootReplaceDom`] のような多段ロールバックは
    /// 不要。実装は失敗時に内部の索引キャッシュ（並び順）を更新しては
    /// ならない（[`Self::insert_before_batch`] doc と同じ理由）。
    fn move_before(
        &mut self,
        index: usize,
        key: &str,
        child: &Self::Handle,
        reference: Option<&Self::Handle>,
    ) -> bool;

    /// `child` をコンテナから取り除く（`Element::remove_child`）。
    ///
    /// # 戻り値（イシュー #1340 codex-review P1〔3 巡目〕全走査対応）
    ///
    /// 実 DOM への `remove_child` が失敗した場合は `false` を返す（`child`
    /// は実 DOM 上に残ったまま）。`remove_child` は単一 DOM 呼び出しのみで
    /// 構成され、失敗時は仕様上 DOM を一切変更しないためロールバックは
    /// 不要。実装は失敗時に内部の索引キャッシュから `child` を除去しては
    /// ならない（キャッシュ上だけ「削除済み」として扱うと、以後の差分基準
    /// が実 DOM と恒久的に乖離する）。
    fn remove_child(&mut self, child: &Self::Handle) -> bool;

    /// `child` の属性を `old_attrs`（直前に反映済みの、`reserved_attr` を
    /// 除く属性集合）から `new_attrs`（同じく `reserved_attr` を除く新しい
    /// 属性集合）へ同期する（[`KeyedOp::Update`] 適用の一部、イシュー
    /// #1324）。`reserved_attr` は呼び出し元の文脈で改変させない予約属性名
    /// （子アイテムなら `data-key`、`crate::keyed_apply::sync_parent_attrs`
    /// 経由の親要素なら `data-bind-list`）で、`old_attrs`/`new_attrs` には
    /// 呼び出し元が既に含めない契約（多層防御として実装側でも明示的に
    /// 除外する）。URL スキーム・イベントハンドラ属性の検証は
    /// [`crate::keyed_dom::build_dom_node`] と同一の述語を `web-sys`
    /// アダプタが共有して行う（本トレイトのモジュール doc 参照）。
    ///
    /// # 削除判定の基準（イシュー #1340 codex-review Bugbot〔8 巡目〕対応）
    ///
    /// 削除対象は `old_attrs`（キャッシュされた直前の内容）だけを基準に
    /// してはならない。SSR hydrate 直後はサーバー側 HTML に含まれる属性が
    /// ライブ DOM 上に存在する一方、クライアント側の `old_attrs`
    /// キャッシュ（`keyed_list_cache` の初期シード）にはその属性が
    /// 含まれない構成があり得る（hydrate 由来のライブ属性ドリフト）。
    /// `old_attrs` のみを削除判定に使うと、`new_attrs` にも存在しないその
    /// 属性が Update を何度適用しても永遠に除去されない。実装は
    /// **ライブ要素の実属性列挙**（`Element::attributes()`/`NamedNodeMap`）
    /// を削除候補の決定に使い、`reserved_attr` と `new_attrs` に存在する
    /// 名前を除いた全ての実属性を削除対象とすること（`old_attrs` は削除
    /// 判定には使わない。以下「決定的な正規化契約」の順序決定にのみ使う）。
    ///
    /// 属性名の一致判定は ASCII 大文字小文字を区別しない
    /// （`str::eq_ignore_ascii_case`）で行う: HTML 文書の属性名はライブ DOM
    /// 上で小文字化されて列挙されるため、`new_attrs` が大文字を含む表記
    /// （例: `"Class"`）だと単純な `==` 比較では常に不一致となり、既存
    /// 属性が誤って削除候補になってしまう（destructive: 削除後の再設定が
    /// ポリシー拒否で skip されると属性が消失したままになる）。
    ///
    /// # 戻り値（イシュー #1340 codex-review P1〔5 巡目〕・Bugbot〔6・8 巡目〕対応）
    ///
    /// 呼び出し後にライブ DOM が実際に持つ、`reserved_attr` を除く属性
    /// 集合を返す。ポリシー拒否（検証不通過で書き込みを skip）・実行時
    /// 失敗（`setAttribute`/`removeAttribute` の `Err`、不正な属性名等）の
    /// いずれによる未達成も、この戻り値には「実際に DOM に残っている
    /// 状態」として正確に反映されていること（本トレイトのモジュール doc
    /// 「`sync_attrs` の実行時失敗と「達成 Node」の整合」参照）。
    ///
    /// **決定的な正規化契約（イシュー #1340 codex-review Bugbot〔6 巡目〕
    /// 対応、削除判定基準の変更〔8 巡目〕後も維持）**: 戻り値の**合成**自体
    /// は `child.attributes()`（`NamedNodeMap`）の生の列挙順・大小文字表記
    /// を使わず、個別照会（`get_attribute`）のみに基づく（削除**判定**に
    /// ライブ列挙を使うことと矛盾しない: 列挙は「削除すべき名前の集合」を
    /// 決めるためだけに使い、値の取得・戻り値への格納は必ず
    /// `get_attribute` の個別照会を経由する）。ライブ DOM の `NamedNodeMap`
    /// は (a) 新規追加した属性を末尾へ追加する順序で `new_attrs` の順序と
    /// 一致する保証がなく、(b) HTML 文書の属性名は内部的に小文字化される
    /// ため `new_attrs` が大文字を含む表記だと一致しない。これらの
    /// アーティファクトをそのまま返すと、次回 `diff_keyed_items`
    /// （`(属性名, 属性値)` の `Vec` に対する順序・大小文字に敏感な
    /// `PartialEq` で比較）が「全書き込み成功の正常系」ですら新しい view
    /// と不一致と判定し、`Update`（`replace_item_children` を含む）が毎
    /// tick 再発火して子ノードが再マウントされ続ける（codex-review 指摘の
    /// 実害）。実装は次の手順で決定的に合成すること:
    ///
    /// 1. `new_attrs` の順序で各エントリ `(name, value)` について
    ///    `child.get_attribute(name)`（個別照会）を呼ぶ。実値が `value` と
    ///    一致すれば `(name, value)`（`new_attrs` 側の表記のまま）を採用
    ///    し、異なれば `(name, 実値)` を採用する。実値が存在しない
    ///    （未追加・削除済み）場合はこのエントリを戻り値から除外する。
    /// 2. 削除候補（ライブ列挙から `reserved_attr`・`new_attrs` 側の名前を
    ///    除いたもの）のうち、`child.get_attribute(name)` が値を返す場合
    ///    （削除が実行時に失敗し DOM に残存している場合）は実値を末尾へ
    ///    追加する。順序は `old_attrs` に同名エントリがあるものを
    ///    `old_attrs` の順序で先に並べ（全操作成功時は削除候補が空になり
    ///    この節は素通りするため、正常系の順序・バイト等価性は変わらない）、
    ///    `old_attrs` に無いもの（hydrate 由来のライブ専用属性）は残りを
    ///    ライブ列挙順で末尾に追加する。
    ///
    /// この手順により、全操作が成功した正常系では戻り値が `new_attrs` と
    /// 完全にバイト等価（順序・大小文字とも）になり、失敗系（ポリシー
    /// 拒否・実行時失敗）・hydrate ドリフト系（削除成功時は単に戻り値に
    /// 現れない）でのみ実際の DOM 状態が反映される。
    fn sync_attrs(
        &mut self,
        child: &Self::Handle,
        reserved_attr: &str,
        old_attrs: &[(String, String)],
        new_attrs: &[(String, String)],
    ) -> Vec<(String, String)>;

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

    /// `old`（キー `key` の既存要素）を `new`（[`Self::create_item`] が
    /// 構築済みの新規ノード）へ置き換える（[`KeyedOp::Update`] のうち
    /// **ルート要素のタグ自体が変わる**ケース専用、イシュー #1340
    /// codex-review P1〔2 巡目〕対応）。
    ///
    /// [`Self::sync_attrs`]/[`Self::replace_item_children`]（「浅い
    /// in-place 更新」）はルート要素自体の DOM ノード同一性維持を前提と
    /// するが、タグ自体が変わる場合は `Element.tagName` が不変（DOM 標準
    /// 仕様）のためノード同一性を維持したままの更新が原理的に不可能
    /// （`setAttribute` でタグ名は変更できない）。この場合は `Insert` と
    /// 同様に新規ノードを構築してから `old` の位置へ差し替える（`old` を
    /// 直前の参照点として使うため index 解決は不要）。
    ///
    /// # 戻り値と部分失敗時の契約（イシュー #1340 codex-review P1
    /// 〔3 巡目〕対応）
    ///
    /// 実装は `new` を `old` の直前へ挿入したうえで `old` を取り除くこと
    /// （順序を保ったまま置き換える、2 回の web-sys 呼び出し）。いずれも
    /// 失敗しうる呼び出しであり、`Result` を検査せず盲目的に進めると
    /// 「挿入失敗後も旧要素だけ削除してしまい当該キーが消滅する」「挿入
    /// 成功後の削除失敗で同一キー要素が重複する」といった部分適用が
    /// 起こりうる（`docs/design/keyed-update-op-design.md` §6 不変条件 6
    /// と同じ構造的原子性の要求、[`crate::keyed_dom::exchange_children`]
    /// と同じ流儀）:
    ///
    /// - `new` の挿入自体が失敗した場合: ライブ DOM には何も変更されて
    ///   いない（DOM 標準上 `insertBefore` 失敗時は no-op）ため、`old` に
    ///   一切触れず `false` を返す。
    /// - `new` の挿入は成功したが `old` の除去が失敗した場合: 挿入した
    ///   `new` を取り除いて挿入前の状態へロールバックしてから `false` を
    ///   返す。ロールバック自体（`new` の除去）が失敗する残余リスクは
    ///   固定英語文言の警告ログで示し（設計書 §6 不変条件 6「残る有限の
    ///   リスク」）、`unwrap()`/`panic!` は使わずベストエフォートで処理を
    ///   継続する。
    /// - 完全に成功した場合のみ `true` を返す。
    ///
    /// `children` キャッシュ等の索引更新は完全成功時（`true` を返す場合）
    /// のみ行うこと。呼び出し元（[`apply_ops_with_items`]）は `false` の
    /// 場合 `resync_required` を立て、次回はライブ DOM を直接読み出す
    /// 構造フォールバックへ委ねる。
    fn replace_root(&mut self, old: &Self::Handle, key: &str, new: Self::NewNode) -> bool;
}

/// [`KeyedListDom::replace_item_children`]（子ノード列交換）のコミット
/// フェーズの下位 DOM 操作を抽象化するトレイト（イシュー #1340
/// codex-review P1〔2 巡目〕対応）。
///
/// [`KeyedListDom`] と同じ「純粋層 + wasm32 配線層」の 2 層構成方針
/// （本モジュール doc 参照）をもう一段適用したもの: `replace_item_children`
/// の部分失敗時ロールバック（`docs/design/keyed-update-op-design.md` §6
/// 不変条件 6「子ノード交換」）は個々の `remove_child`/`insert_before`
/// 呼び出しの成否を判定しながら分岐する非自明なアルゴリズムであり、
/// native `cargo test` から到達できない `web-sys` 実装（`crate::keyed_dom::WebSysKeyedDom`）
/// 内に直接書くと「n 回目の呼び出しだけ失敗する」ケースを決定的に注入
/// できない。本トレイトへ抽象化し、走査本体を [`exchange_children`] へ
/// 切り出すことで、`#[cfg(test)]` のモック実装から任意の呼び出し回数目を
/// 失敗させて復元手順の回帰を固定できる。
pub(crate) trait ChildExchangeDom {
    /// 交換対象のノード（`web-sys` 実装では `web_sys::Node`）。
    type Node: Clone + PartialEq;

    /// 現在のライブ子ノード列を、一切変更せず出現順のまま読み出す
    /// （ロールバックの復元先として保持するための事前読み取り。副作用を
    /// 持たない）。
    fn current_children(&mut self) -> Vec<Self::Node>;

    /// `node` を親から取り除く。失敗した場合 `false`
    /// （`Node::removeChild`相当、失敗しうる `web_sys` 呼び出し）。
    fn remove_child(&mut self, node: &Self::Node) -> bool;

    /// `node` を `reference`（`None` なら末尾）の直前へ挿入する。失敗した
    /// 場合 `false`（`Node::insertBefore` 相当。`reference` が `None` の
    /// 呼び出しは `appendChild` と等価、DOM 標準仕様）。
    fn insert_before(&mut self, node: &Self::Node, reference: Option<&Self::Node>) -> bool;

    /// ロールバック手順自体（`remove_child`/`insert_before` の逆操作）が
    /// 失敗した場合に呼ばれる（設計書 §6 不変条件 6「残る有限のリスク」）。
    /// 既定は no-op（native モックはログ出力不要）。`web-sys` 実装は固定
    /// 英語文言の警告ログ（不変条件 7）を出す。
    fn on_rollback_failed(&mut self) {}
}

/// [`ChildExchangeDom`] を介して `dom` の現在の子ノード列を `built`
/// （構築済みの新しい子ノード列）へ交換する（`docs/design/keyed-update-op-design.md`
/// §6 不変条件 6「子ノード交換」の構造的原子性の実装本体、イシュー #1340
/// codex-review P1〔2 巡目〕対応）。
///
/// 呼び出し元（[`KeyedListDom::replace_item_children`]）は「新しい子ノード
/// 列を detached な状態で先に構築し、構築が全件成功した場合にのみ本関数を
/// 呼ぶ」契約を満たすこと（構築フェーズの失敗は本関数の対象外、旧稿から
/// 不変）。本関数はコミットフェーズ（ライブ DOM への `remove_child`/
/// `insert_before` の実適用）のみを担う:
///
/// 1. 旧子ノード列を（一切変更せず）読み出す。
/// 2. 旧子ノードを先頭から順に取り外す。`i` 件目で失敗した場合、既に
///    取り外し済みの `0..i` 件を、まだ付いたままの `i` 件目（取り外しに
///    失敗したノード自身、ルート要素に残っている未取り外し suffix の
///    先頭）の直前へ元の順序で再度取り付け、`false` を返す
///    （`append_child`〔末尾追加〕では suffix の後ろへ回り込み元の順序が
///    壊れるため使わない）。
/// 3. 旧子ノードの取り外しをすべて終えた後、新子ノード（`built`）を先頭
///    から順に取り付ける。`j` 件目で失敗した場合、既に取り付け済みの
///    `0..j` 件を取り除き、保持しておいた旧子ノード列を元の順序で再度
///    取り付け、`false` を返す。
/// 4. 全件成功した場合のみ `true` を返す。
///
/// ロールバック自体（`insert_before`/`remove_child` の逆操作）が失敗する
/// 残余リスクは [`ChildExchangeDom::on_rollback_failed`] を経由して
/// `web-sys` 実装が警告ログを出す（設計書 §6 不変条件 6「残る有限の
/// リスク」、本関数自体は `unwrap()`/`panic!` を使わずベストエフォートで
/// 処理を継続する）。
pub(crate) fn exchange_children<D: ChildExchangeDom>(dom: &mut D, built: &[D::Node]) -> bool {
    let old_children = dom.current_children();

    for (i, old_child) in old_children.iter().enumerate() {
        if !dom.remove_child(old_child) {
            for removed in &old_children[..i] {
                if !dom.insert_before(removed, Some(old_child)) {
                    dom.on_rollback_failed();
                }
            }
            return false;
        }
    }

    for (j, node) in built.iter().enumerate() {
        if !dom.insert_before(node, None) {
            for appended in &built[..j] {
                if !dom.remove_child(appended) {
                    dom.on_rollback_failed();
                }
            }
            for old in &old_children {
                if !dom.insert_before(old, None) {
                    dom.on_rollback_failed();
                }
            }
            return false;
        }
    }
    true
}

/// [`KeyedListDom::replace_root`] のコミット手順を DOM 非依存に表現する
/// トレイト（[`ChildExchangeDom`] と同じ理由・同じ「純粋層 + wasm32 配線
/// 層」方針、イシュー #1340 codex-review P1〔3 巡目〕対応）。
///
/// `replace_root` の実 DOM 実装（`crate::keyed_dom::WebSysKeyedDom`）は
/// `insert_before`/`remove_child` という 2 つの独立した失敗しうる web-sys
/// 呼び出しから成るため、native テストから「挿入だけ失敗」「削除だけ
/// 失敗」をそれぞれ決定的に注入できるよう本トレイトへ抽象化する。
pub(crate) trait RootReplaceDom {
    /// 交換対象のノード（`web-sys` 実装では `web_sys::Node`）。
    type Node: Clone;

    /// `new` を `old` の直前へ挿入する。失敗した場合 `false`
    /// （`Node::insertBefore` 相当）。
    fn insert_before(&mut self, new: &Self::Node, old: &Self::Node) -> bool;

    /// `node` を親から取り除く。失敗した場合 `false`
    /// （`Node::removeChild` 相当）。
    fn remove(&mut self, node: &Self::Node) -> bool;

    /// ロールバック手順自体（挿入済み `new` の除去）が失敗した場合に
    /// 呼ばれる（設計書 §6 不変条件 6「残る有限のリスク」と同種）。既定は
    /// no-op（native モックはログ出力不要）。`web-sys` 実装は固定英語
    /// 文言の警告ログ（不変条件 7）を出す。
    fn on_rollback_failed(&mut self) {}
}

/// [`RootReplaceDom`] を介して `old` を `new` へ置き換える
/// （[`KeyedListDom::replace_root`] doc「戻り値と部分失敗時の契約」の
/// 実装本体、イシュー #1340 codex-review P1〔3 巡目〕対応）。
///
/// 1. `new` を `old` の直前へ挿入する。失敗した場合、ライブ DOM には
///    何も変更が加わっていない（DOM 標準上 `insertBefore` 失敗時は
///    no-op）ため `old` には一切触れず `false` を返す（`old` を誤って
///    削除してしまうと当該キーが DOM 上から消滅する codex-review 指摘の
///    再現を防ぐ）。
/// 2. `old` を取り除く。失敗した場合（挿入は成功済みのため同一キー要素の
///    重複を防ぐ必要がある）、挿入した `new` を取り除いて挿入前の状態へ
///    ロールバックしてから `false` を返す。ロールバック自体が失敗する
///    残余リスクは [`RootReplaceDom::on_rollback_failed`] 経由で警告する。
/// 3. 両方成功した場合のみ `true` を返す。
pub(crate) fn replace_root_node<D: RootReplaceDom>(
    dom: &mut D,
    old: &D::Node,
    new: &D::Node,
) -> bool {
    if !dom.insert_before(new, old) {
        return false;
    }
    if !dom.remove(old) {
        if !dom.remove(new) {
            dom.on_rollback_failed();
        }
        return false;
    }
    true
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
                    if !dom.remove_child(&child) {
                        // 実 DOM への `remove_child` 自体が失敗（`child` は
                        // 実 DOM 上に残存）。当該キーは目標状態（削除済み）
                        // に到達していないため未達成として扱う（イシュー
                        // #1340 codex-review P1〔3 巡目〕全走査対応）。
                        fully_achieved = false;
                    }
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
                    let batch_len = items.len();
                    if !dom.insert_before_batch(adjusted_start, items, reference.as_ref()) {
                        // 実 DOM への挿入自体が失敗（`insert_before_batch`
                        // 契約により DOM・内部キャッシュとも無変更のまま）。
                        // 区間内の全アイテムが未達成スロットになるため
                        // `failed_in_run` へ合算する（イシュー #1340
                        // codex-review P1〔3 巡目〕全走査対応）。
                        failed_in_run += batch_len;
                    }
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
                    if !dom.move_before(adjusted, key, &existing, reference.as_ref()) {
                        // 実 DOM への移動自体が失敗（`move_before` 契約により
                        // DOM・内部キャッシュとも無変更のまま、`existing` は
                        // 移動前の位置に残る）。目標スロットは埋まらないため
                        // `Move` 対象キー未検出時と同じ扱いにする（イシュー
                        // #1340 codex-review P1〔3 巡目〕全走査対応）。
                        fully_achieved = false;
                    }
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
    /// in-place 更新（`sync_attrs` 経由）が成功したキーについて、
    /// [`KeyedListDom::sync_attrs`] が返した「実際に達成できた属性状態」
    /// （予約属性 `data-key` を除く、イシュー #1340 codex-review P1
    /// 〔5 巡目〕対応）。`sync_attrs` を呼ばなかったキー（`stale_update_keys`
    /// に記録された子ノード構築失敗・タグ変更を伴う `replace_root`・
    /// `Insert` 等）はここに含まれない。[`compose_achieved_children`] は
    /// このキーの有無で「in-place 更新の実測値を使う」か「新規構築の
    /// ポリシー再計算〔[`sanitize_node_for_achieved`]〕を使う」かを分岐する。
    pub(crate) achieved_attrs: std::collections::HashMap<String, Vec<(String, String)>>,
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
    // `sync_attrs`（in-place 更新）が返した「実際に達成できた属性状態」の
    // キーごとの記録（[`ApplyOutcome::achieved_attrs`] doc・モジュール冒頭
    // doc「`sync_attrs` の実行時失敗と「達成 Node」の整合」参照）。
    let mut achieved_attrs: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
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
                    if !dom.remove_child(&child) {
                        // 実 DOM への `remove_child` 自体が失敗（`child` は
                        // 実 DOM 上に残存）。`Remove` は位置を持たない op の
                        // ため `index_offset` は変えないが、対象キーが最終的
                        // に存在しないことを前提とする `final_keys` と実 DOM
                        // が乖離するため `resync_required` を立てる（イシュー
                        // #1340 codex-review P1〔3 巡目〕全走査対応）。
                        resync_required = true;
                    }
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
                if !dom.insert_before_batch(
                    adjusted,
                    vec![(key.clone(), new_node)],
                    reference.as_ref(),
                ) {
                    // 実 DOM への挿入自体が失敗（`insert_before_batch` 契約
                    // により DOM・内部キャッシュとも無変更のまま）。
                    // `create_item` 失敗と同様に未達成スロットとして扱う
                    // （イシュー #1340 codex-review P1〔3 巡目〕全走査対応）。
                    failed_inserts.insert(key);
                    index_offset += 1;
                    resync_required = true;
                }
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
                if !dom.move_before(adjusted, &key, &existing, reference.as_ref()) {
                    // 実 DOM への移動自体が失敗（`move_before` 契約により
                    // DOM・内部キャッシュとも無変更のまま、`existing` は
                    // 移動前の位置に残る）。アイテム数自体は変わらないため
                    // `index_offset` は増やさないが、並び順が計画と乖離する
                    // ため `resync_required` を立てる（イシュー #1340
                    // codex-review P1〔3 巡目〕全走査対応）。
                    resync_required = true;
                }
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
                    tag: new_tag,
                    attrs: new_attrs,
                    children: new_children,
                } = new_node
                else {
                    // keyed list アイテムは keyed_list() 構築時点で
                    // Node::Element であることが保証される
                    // （KeyedListError::NonElementItem で fail-closed に
                    // 拒否済み）。到達しない想定だが安全側に skip する。
                    resync_required = true;
                    continue;
                };

                // codex-review P1 対応（PR #1340 push 後の再レビュー、
                // イシュー #1340）: ルート要素のタグ一致を検証する。
                // `diff_keyed_items`（core、`docs/design/keyed-update-op-design.md`
                // §3.1）は新旧 `Node` の `PartialEq` 不一致（タグ差分を含む
                // 部分木全体の構造比較）のみで `Update` を発行し、タグの
                // 一致は前提としない。一方 §3.2 c 案「浅い in-place 更新」は
                // ルート要素自体の DOM ノード同一性維持（`sync_attrs`/
                // `replace_item_children` はいずれもタグを一切書き換え
                // ない）を前提とした適用方式であり、`Element.tagName` は
                // DOM 標準仕様上不変のためタグ変更を伴う更新はこの前提が
                // 構造的に成り立たない。タグが不一致の場合は「浅い
                // in-place 更新」を諦め、`Insert` と同じ構築手順（[`KeyedListDom::create_item`]）
                // で新規ルート要素を構築してから [`KeyedListDom::replace_root`]
                // で旧ルート要素と差し替える（アイテム全置換、§3.2 却下案 a
                // と同じ手段だが対象はタグ変更が実際に起きたこの 1 アイテム
                // のみに限定される）。この経路により、旧タグのまま Achieved
                // として確定されてしまい以後の同一 view 再適用で差分が出ず
                // 収束しない不具合（codex-review 指摘）を解消する。
                let old_tag_matches = old_items.iter().find(|(k, _)| k == &key).is_some_and(
                    |(_, old_node)| matches!(old_node, Node::Element { tag, .. } if tag == new_tag),
                );
                if !old_tag_matches {
                    let Some(new_dom_node) = dom.create_item(&key) else {
                        // 構築失敗（`RawHtml` 混入等）。旧ルート要素には
                        // 一切触れない fail-closed（`create_item` の既存
                        // 契約と同じ）。旧タグのまま残る事実の検出は次回
                        // 診断（`resync_required` による構造フォール
                        // バック）へ委ねる。
                        resync_required = true;
                        continue;
                    };
                    if !dom.replace_root(&existing, &key, new_dom_node) {
                        // 挿入・除去いずれかが失敗し完全な置換ができな
                        // かった（`replace_root` doc「戻り値と部分失敗時の
                        // 契約」参照）。ライブ DOM は旧要素・新要素いずれか
                        // 一方（実装がベストエフォートでロールバックした
                        // 結果）または稀に両方が残る不定状態になりうるため
                        // `final_keys` だけからは判別できず、次回はライブ
                        // DOM から再構築する構造フォールバックへ委ねる。
                        resync_required = true;
                    }
                    continue;
                }

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
                // `sync_attrs` の決定的正規化契約（`KeyedListDom::sync_attrs`
                // doc・モジュール冒頭 doc「読み戻しの決定的正規化」参照）が
                // 「削除に失敗して残存した旧属性は old_attrs 側の表記で
                // 末尾へ追加する」ために必要とする、直前に反映済みの
                // （`data-key` を除く）属性集合。`old_tag_matches` の判定と
                // 同じ `old_items` 引き当てを再利用する。
                let filtered_old_attrs: Vec<(String, String)> = old_items
                    .iter()
                    .find(|(k, _)| k == &key)
                    .and_then(|(_, old_node)| match old_node {
                        Node::Element { attrs, .. } => Some(
                            attrs
                                .iter()
                                .filter(|(name, _)| name != fandhe_frontend_core::keyed::KEY_ATTR)
                                .cloned()
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default();
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
                    // `sync_attrs` は `setAttribute`/`removeAttribute` 個々の
                    // 実行時失敗（不正な属性名等）をロールバック対象とは
                    // しない（本モジュール doc「Update op の DOM 適用」の
                    // 単純化根拠と同じ）が、呼び出し後にライブ DOM が実際に
                    // 持つ属性状態を戻り値として返す契約へ変更した
                    // （イシュー #1340 codex-review P1〔5 巡目〕対応、
                    // `KeyedListDom::sync_attrs` doc・モジュール冒頭 doc
                    // 「`sync_attrs` の実行時失敗と「達成 Node」の整合」
                    // 参照）。子ノード構築の成功を確認した後でのみ呼ぶため、
                    // 呼び出し自体は常に行われるが、「達成」の中身（新値か
                    // 旧値かの取捨）はこの戻り値が正確に表す。
                    let synced = dom.sync_attrs(
                        &existing,
                        fandhe_frontend_core::keyed::KEY_ATTR,
                        &filtered_old_attrs,
                        &filtered_attrs,
                    );
                    achieved_attrs.insert(key, synced);
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
        achieved_attrs,
    }
}

/// [`crate::keyed_dom::apply_keyed_list_with_previous`] が「達成 Node」
/// （実 DOM が実際に表す内容、設計書 §4.2）の子ノード列を合成するために
/// 呼ぶ純粋関数（DOM 非依存、native `cargo test` から到達可能。イシュー
/// #1340 codex-review P1〔4 巡目〕対応、モジュール冒頭 doc「属性検証拒否と
/// 「達成 Node」の整合」参照）。
///
/// [`ApplyOutcome::stale_update_keys`] のキーは旧内容をまるごと使う（子
/// ノード構築失敗〔`RawHtml` 混入等〕で据え置かれたため、属性同期
/// `sync_attrs` 自体も呼ばれておらずライブ DOM は完全に旧内容のまま）。
/// それ以外のキーは新内容を使うが、[`sanitize_node_for_achieved`] を経由
/// して「実際に DOM へ書き込まれた属性値」へ正規化する。
pub(crate) fn compose_achieved_children(
    old_items: &[(String, Node)],
    new_items: &[(String, Node)],
    outcome: &ApplyOutcome,
) -> Vec<Node> {
    let old_by_key: std::collections::HashMap<&str, &Node> =
        old_items.iter().map(|(k, n)| (k.as_str(), n)).collect();
    let new_by_key: std::collections::HashMap<&str, &Node> =
        new_items.iter().map(|(k, n)| (k.as_str(), n)).collect();

    outcome
        .final_keys
        .iter()
        .filter_map(|key| {
            if outcome.stale_update_keys.contains(key) {
                return old_by_key.get(key.as_str()).map(|n| (*n).clone());
            }
            let new_node = new_by_key.get(key.as_str())?;
            if let Some(achieved) = outcome.achieved_attrs.get(key) {
                // in-place 更新（`sync_attrs` 経由）: `sync_attrs` が実 DOM を
                // 読み戻して返した「実際に達成できた属性状態」をそのまま
                // 使う（ポリシー拒否・実行時失敗のいずれも織り込み済み、
                // モジュール冒頭 doc「`sync_attrs` の実行時失敗と「達成
                // Node」の整合」参照）。子ノードは `replace_item_children`
                // が新規構築するため、その内部の検証拒否だけは
                // `sanitize_node_for_achieved` で反映する。
                Some(compose_in_place_updated_node(new_node, achieved))
            } else {
                // 新規構築経路（`Insert`・タグ変更を伴う `replace_root`）:
                // `sync_attrs` を経由しないため実測値がなく、ポリシー
                // 拒否のみを再計算する fresh 扱い（実行時失敗は
                // `build_dom_node_with_namespace` の `.ok()?` が要素構築
                // 全体を失敗させるため、ここへは到達しない）。
                Some(sanitize_node_for_achieved(new_node))
            }
        })
        .collect()
}

/// keyed list の親要素（`list_element` 自身）の属性を `old_parent_attrs`
/// から `new_parent_attrs` へ同期し、「達成 Node」へ格納する属性列を合成
/// する（[`crate::keyed_dom::apply_keyed_list_with_previous`] 専用ヘルパー、
/// DOM 非依存・`D: KeyedListDom` 越しに native `cargo test` から到達可能。
/// イシュー #1340 codex-review P1〔7 巡目〕対応）。
///
/// `list_handle`（`list_element` 自身のハンドル）を
/// [`KeyedListDom::sync_attrs`]（子アイテムの `Update` と同じ実 DOM
/// 読み戻し + 決定的正規化契約）へ渡すことで、「達成 Node ＝ 実 DOM の
/// 実際の状態」の不変条件を親要素にも拡張する。旧実装は `list_element`
/// のタグ・属性を一切同期せず `new_list_node` の `parent_attrs` をそのまま
/// 「達成 Node」へ格納していたため、親属性が変わる更新では実 DOM が旧
/// 状態のままキャッシュだけ新値へ進み、以後 diff で検出できなくなって
/// いた（codex-review 指摘）。
///
/// 予約属性 `data-bind-list`（[`fandhe_frontend_core::keyed::BIND_LIST_ATTR`]）
/// は子アイテムの `data-key` と同じ理由で同期対象から除外する（呼び出し
/// 元がこの `list_element` を解決するための識別子であり、Update 経路から
/// 改変させない多層防御）。`new_parent_attrs` 側の値を戻り値の末尾へ
/// そのまま保持する（`fandhe_frontend_core::keyed::keyed_list` が常に
/// 末尾へ付加する契約と同型、[`compose_in_place_updated_node`] の
/// `data-key` 末尾追加と同じ合成規則）。
pub(crate) fn sync_parent_attrs<D: KeyedListDom>(
    dom: &mut D,
    list_handle: &D::Handle,
    old_parent_attrs: &[(String, String)],
    new_parent_attrs: &[(String, String)],
) -> Vec<(String, String)> {
    let filtered_old: Vec<(String, String)> = old_parent_attrs
        .iter()
        .filter(|(name, _)| name != fandhe_frontend_core::keyed::BIND_LIST_ATTR)
        .cloned()
        .collect();
    let filtered_new: Vec<(String, String)> = new_parent_attrs
        .iter()
        .filter(|(name, _)| name != fandhe_frontend_core::keyed::BIND_LIST_ATTR)
        .cloned()
        .collect();
    let achieved = dom.sync_attrs(
        list_handle,
        fandhe_frontend_core::keyed::BIND_LIST_ATTR,
        &filtered_old,
        &filtered_new,
    );

    let bind_list_attr_entry = new_parent_attrs
        .iter()
        .find(|(name, _)| name == fandhe_frontend_core::keyed::BIND_LIST_ATTR)
        .cloned();
    let mut result = achieved;
    result.extend(bind_list_attr_entry);
    result
}

/// in-place 更新（`sync_attrs` 経由）が成功したキーの「達成 Node」を、
/// `sync_attrs` の実測 `achieved_attrs`（ground truth）と `new_node` の
/// 子ノードから合成する（[`compose_achieved_children`] 専用ヘルパー）。
///
/// 予約属性 `data-key` は `sync_attrs` の管理対象外（呼び出し元
/// `apply_ops_with_items` が `filtered_attrs` から除外して渡す契約、
/// [`KeyedListDom::sync_attrs`] doc 参照）のため `achieved_attrs` には
/// 含まれない。Update 経路から改変されない不変条件により `new_node` 側の
/// 値をそのまま保持する。
fn compose_in_place_updated_node(new_node: &Node, achieved_attrs: &[(String, String)]) -> Node {
    match new_node {
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let key_attr = attrs
                .iter()
                .find(|(name, _)| name == fandhe_frontend_core::keyed::KEY_ATTR)
                .cloned();
            let mut result_attrs = achieved_attrs.to_vec();
            result_attrs.extend(key_attr);
            let sanitized_children = children.iter().map(sanitize_node_for_achieved).collect();
            Node::Element {
                tag,
                attrs: result_attrs,
                children: sanitized_children,
            }
        }
        other => other.clone(),
    }
}

/// 検証を通過しない属性（危険 URL スキーム・イベントハンドラ・不正
/// `srcset`）を丸ごと除外した [`Node`] を合成する（新規構築経路専用、
/// [`compose_achieved_children`]/[`compose_in_place_updated_node`] の子孫
/// ノード再帰から呼ばれる。モジュール冒頭 doc「属性検証拒否と「達成
/// Node」の整合」参照）。新規構築（`create_item`/タグ変更を伴う
/// `replace_root`・`replace_item_children` の新規子孫）はいずれも旧値を
/// 持たないため、拒否属性は無条件で除外する（`build_dom_node_with_namespace`
/// の `set_attribute` skip と等価）。
///
/// `crate::keyed_dom::apply_keyed_list_with_previous`（親要素自身のタグが
/// 変わる更新、イシュー #1340 codex-review P1〔9 巡目〕対応）も、
/// `list_element` 自身を含む部分木全体を新規構築するため本関数を再利用
/// する（`pub(crate)` として公開）。
pub(crate) fn sanitize_node_for_achieved(node: &Node) -> Node {
    match node {
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let sanitized_attrs = attrs
                .iter()
                .filter(|(name, value)| !is_attr_write_rejected(name, value))
                .cloned()
                .collect();
            let sanitized_children = children.iter().map(sanitize_node_for_achieved).collect();
            Node::Element {
                tag,
                attrs: sanitized_attrs,
                children: sanitized_children,
            }
        }
        other => other.clone(),
    }
}

/// `build_dom_node_with_namespace` の書き込み拒否判定と同一の述語
/// （イベントハンドラ属性・危険 URL スキーム・不正 `srcset`）。
/// `crate::keyed_dom::WebSysKeyedDom::sync_attrs` も同一の述語を使うが、
/// そちらは実 DOM 読み戻しにより ground truth を直接返すため、この述語を
/// 呼ばずに済む（新規構築経路専用の再計算、`is_attr_write_rejected` の
/// 呼び出し元コメント参照）。
fn is_attr_write_rejected(name: &str, value: &str) -> bool {
    fandhe_frontend_core::is_event_handler_attr(name)
        || (fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value))
        || (name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value))
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
        /// `replace_root` に DOM 操作失敗（`insert_before`/`remove_child`
        /// いずれかの失敗相当）を注入するキー集合。イシュー #1340
        /// codex-review P1〔3 巡目〕の呼び出し元（`apply_ops_with_items`）
        /// 側の `resync_required` 反映テスト用のフック（`RootReplaceDom`
        /// レベルの挙動細分は `replace_root_node` 専用の
        /// `VecRootReplace` モックが担う）。
        fail_replace_root_for: std::collections::HashSet<String>,
        /// `insert_before_batch` に実 DOM 挿入失敗を注入するキー集合（バッチ
        /// 中のいずれかのキーが含まれていれば区間全体を失敗させる）。
        /// イシュー #1340 codex-review P1〔3 巡目〕全走査対応の呼び出し元
        /// （`apply_ops`/`apply_ops_with_items`）側の未達成反映テスト用の
        /// フック。
        fail_insert_before_batch_for: std::collections::HashSet<String>,
        /// `move_before` に実 DOM 移動失敗を注入するキー集合（同上）。
        fail_move_before_for: std::collections::HashSet<String>,
        /// `remove_child` に実 DOM 削除失敗を注入するキー集合（同上）。
        fail_remove_child_for: std::collections::HashSet<String>,
        /// `sync_attrs` 内の `setAttribute` 相当に実行時失敗（不正な属性名
        /// 等の `InvalidCharacterError` 相当）を注入する `(キー, 属性名)`
        /// 集合。イシュー #1340 codex-review P1〔5 巡目〕対応: 該当エントリ
        /// では属性値の書き込みが失敗した扱いにする（現在値があればそれを
        /// 保持、無ければ追加されない）。
        fail_set_attribute_for: std::collections::HashSet<(String, String)>,
        /// `sync_attrs` 内の `removeAttribute` 相当に実行時失敗を注入する
        /// `(キー, 属性名)` 集合（同上）。該当エントリは `new_attrs` に
        /// 含まれなくなっても実 DOM 上に残存する扱いにする。
        fail_remove_attribute_for: std::collections::HashSet<(String, String)>,
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
        replace_root: usize,
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
                + self.replace_root
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
        ) -> bool {
            self.calls.insert_before_batch += 1;
            if items
                .iter()
                .any(|(key, _)| self.fail_insert_before_batch_for.contains(key))
            {
                // 実 DOM 挿入失敗を模擬: `items`/`self.items` のいずれも
                // 変更せず `false` を返す（トレイト契約「失敗時は DOM・
                // 内部キャッシュとも無変更」、イシュー #1340 codex-review
                // P1〔3 巡目〕全走査対応）。
                return false;
            }
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
            true
        }

        fn move_before(
            &mut self,
            _index: usize,
            key: &str,
            child: &Self::Handle,
            reference: Option<&Self::Handle>,
        ) -> bool {
            self.calls.move_before += 1;
            if self.fail_move_before_for.contains(key) {
                // 実 DOM 移動失敗を模擬: `self.items` を一切変更せず `false`
                // を返す（イシュー #1340 codex-review P1〔3 巡目〕全走査
                // 対応）。
                return false;
            }
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
            true
        }

        fn remove_child(&mut self, child: &Self::Handle) -> bool {
            self.calls.remove_child += 1;
            if self.fail_remove_child_for.contains(child) {
                // 実 DOM 削除失敗を模擬: `self.items` を一切変更せず `false`
                // を返す（イシュー #1340 codex-review P1〔3 巡目〕全走査
                // 対応）。
                return false;
            }
            if let Some(pos) = self.items.iter().position(|k| k == child) {
                self.items.remove(pos);
            }
            true
        }

        /// 実 DOM の `sync_attrs`（`crate::keyed_dom::WebSysKeyedDom`）を
        /// 模擬する: `self.attrs` を「ライブ DOM の実属性列挙」の代理と
        /// して扱う。削除判定は `self.attrs`（`old_attrs` キャッシュでは
        /// なく）から `reserved_attr`・`new_attrs` 側の名前を除いた全件を
        /// 候補とする（Bugbot〔8 巡目〕対応: hydrate 由来のライブ属性
        /// ドリフト、`old_attrs` に無いが `self.attrs` にはある属性も削除
        /// 対象になることを検証できる）。ポリシー拒否（`is_attr_write_rejected`、
        /// `web-sys` 実装と同一の述語）・実行時失敗注入
        /// （`fail_set_attribute_for`/`fail_remove_attribute_for`）の
        /// いずれでも書き込みが skip され、`self.attrs` は「実際に達成
        /// できた状態」のみを反映するよう更新する。戻り値は
        /// `KeyedListDom::sync_attrs` doc「決定的な正規化契約」と同じ手順
        /// （合成は `new_attrs`/`old_attrs` の表記基準の個別照会、削除
        /// 判定のみライブ列挙基準）で合成する（イシュー #1340 codex-review
        /// P1〔5 巡目〕・Bugbot〔6・8 巡目〕対応）。
        fn sync_attrs(
            &mut self,
            child: &Self::Handle,
            reserved_attr: &str,
            old_attrs: &[(String, String)],
            new_attrs: &[(String, String)],
        ) -> Vec<(String, String)> {
            self.calls.sync_attrs += 1;
            let mut current = self.attrs.remove(child).unwrap_or_default();

            // 削除判定: ライブ状態の代理である `current` の列挙から
            // `reserved_attr`・`new_attrs` 側の名前を除いたものを削除候補
            // とする（`old_attrs` は削除判定に使わない、Bugbot〔8 巡目〕
            // 対応）。属性名の比較は大小文字を区別しない
            // （`eq_ignore_ascii_case`、`KeyedListDom::sync_attrs` doc
            // 「削除判定の基準」参照）。
            let removal_candidates: Vec<String> = current
                .iter()
                .map(|(name, _)| name.clone())
                .filter(|name| {
                    !name.eq_ignore_ascii_case(reserved_attr)
                        && !new_attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
                })
                .collect();
            for name in &removal_candidates {
                if self
                    .fail_remove_attribute_for
                    .contains(&(child.clone(), name.clone()))
                {
                    // 実行時失敗を模擬: 現在値を残したまま skip する。
                    continue;
                }
                current.retain(|(k, _)| k != name);
            }

            for (name, value) in new_attrs {
                if name.eq_ignore_ascii_case(reserved_attr) {
                    // 多層防御: 呼び出し元が既に除外済みの前提だが、万一
                    // 予約属性が紛れ込んでも書き込まない。
                    continue;
                }
                if is_attr_write_rejected(name, value) {
                    // ポリシー拒否を模擬: 現在値（旧値、または未追加なら
                    // 不在）をそのまま保持する。
                    continue;
                }
                if self
                    .fail_set_attribute_for
                    .contains(&(child.clone(), name.clone()))
                {
                    // 実行時失敗を模擬: 現在値（旧値、または未追加なら
                    // 不在）をそのまま保持する。
                    continue;
                }
                if let Some(entry) = current.iter_mut().find(|(k, _)| k == name) {
                    entry.1 = value.clone();
                } else {
                    current.push((name.clone(), value.clone()));
                }
            }

            self.attrs.insert(child.clone(), current.clone());

            // `get_attribute` 相当の個別照会（`current` への線形探索）で
            // 決定的に合成する（`KeyedListDom::sync_attrs` doc「決定的な
            // 正規化契約」の手順 1・2 と同一のロジック。手順 2 は削除判定
            // 基準の変更〔8 巡目〕に合わせ `removal_candidates` を走査
            // する）。
            let get = |name: &str| {
                current
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.clone())
            };
            let mut achieved: Vec<(String, String)> = Vec::with_capacity(new_attrs.len());
            for (name, value) in new_attrs {
                if name.eq_ignore_ascii_case(reserved_attr) {
                    continue;
                }
                if let Some(actual) = get(name) {
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
            // 専用属性）は `removal_candidates` の列挙順で末尾に追加する。
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
                if let Some(actual) = get(name) {
                    achieved.push((name.clone(), actual));
                }
            }
            achieved
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

        /// `Handle`/`NewNode` がともに `String`（key そのもの）であるため、
        /// `old` を `new` へ置き換えるのは `items` 内の該当位置の値を
        /// 差し替えるだけで表現できる（`old` の位置を保ったまま `new` へ
        /// 差し替える契約、`replace_root` トレイト doc 参照）。
        /// `fail_replace_root_for` に含まれるキーは DOM 操作失敗を模擬し、
        /// `items` を一切変更せず `false` を返す（呼び出し元
        /// `apply_ops_with_items` が `resync_required` を正しく立てる
        /// ことを確認するためのフック）。
        fn replace_root(&mut self, old: &Self::Handle, key: &str, new: Self::NewNode) -> bool {
            self.calls.replace_root += 1;
            if self.fail_replace_root_for.contains(key) {
                return false;
            }
            if let Some(pos) = self.items.iter().position(|k| k == old) {
                self.items[pos] = new;
            }
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
        ) -> bool {
            self.inner
                .insert_before_batch(start_index, items, reference)
        }
        fn move_before(
            &mut self,
            index: usize,
            key: &str,
            child: &Self::Handle,
            reference: Option<&Self::Handle>,
        ) -> bool {
            self.inner.move_before(index, key, child, reference)
        }
        fn remove_child(&mut self, child: &Self::Handle) -> bool {
            self.inner.remove_child(child)
        }
        fn sync_attrs(
            &mut self,
            child: &Self::Handle,
            reserved_attr: &str,
            old_attrs: &[(String, String)],
            new_attrs: &[(String, String)],
        ) -> Vec<(String, String)> {
            self.inner
                .sync_attrs(child, reserved_attr, old_attrs, new_attrs)
        }
        fn replace_item_children(&mut self, child: &Self::Handle, new_children: &[Node]) -> bool {
            self.inner.replace_item_children(child, new_children)
        }
        fn find_by_key(&mut self, key: &str) -> Option<Self::Handle> {
            self.inner.find_by_key(key)
        }
        fn replace_root(&mut self, old: &Self::Handle, key: &str, new: Self::NewNode) -> bool {
            self.inner.replace_root(old, key, new)
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

    // --- 実 DOM 操作失敗の未達成反映（イシュー #1340 codex-review P1
    // 〔3 巡目〕全走査対応。`insert_before_batch`/`move_before`/
    // `remove_child` の Result 破棄を是正した契約の回帰テスト） ---

    /// `insert_before_batch` が実 DOM 挿入失敗を返した場合、`apply_ops` は
    /// `false`（未達成）を返し、`items` キャッシュは無変更のまま
    /// （挿入されていないノードをキャッシュ上だけ「挿入済み」にしない）。
    #[test]
    fn apply_ops_returns_false_and_leaves_items_untouched_when_insert_before_batch_fails() {
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            fail_insert_before_batch_for: std::collections::HashSet::from(["b".to_string()]),
            ..Default::default()
        };
        let new_keys = vec!["a".to_string(), "b".to_string()];

        let fully_achieved = apply_ops(&mut dom, &new_keys);

        assert!(
            !fully_achieved,
            "挿入失敗があったので全体としては未達成のはず"
        );
        assert_eq!(
            dom.items,
            vec!["a".to_string()],
            "挿入失敗時は items キャッシュを一切変更してはならない"
        );
    }

    /// `move_before` が実 DOM 移動失敗を返した場合、`apply_ops` は
    /// `false`（未達成）を返し、対象キーは移動前の位置に残ったまま
    /// （移動していないのに移動済みとしてキャッシュを更新しない）。
    #[test]
    fn apply_ops_returns_false_and_leaves_order_untouched_when_move_before_fails() {
        let mut dom = CountingDom {
            items: keys_n(3),
            fail_move_before_for: std::collections::HashSet::from(["k2".to_string()]),
            ..Default::default()
        };
        let new_keys = vec!["k2".to_string(), "k0".to_string(), "k1".to_string()];

        let fully_achieved = apply_ops(&mut dom, &new_keys);

        assert!(!fully_achieved, "移動失敗があったので未達成のはず");
        assert_eq!(
            dom.items,
            keys_n(3),
            "移動失敗時は items キャッシュ（並び順）を一切変更してはならない"
        );
    }

    /// `remove_child` が実 DOM 削除失敗を返した場合、`apply_ops` は
    /// `false`（未達成）を返し、対象キーは items キャッシュに残ったまま
    /// （削除していないのに削除済みとしてキャッシュを更新しない）。
    #[test]
    fn apply_ops_returns_false_and_leaves_items_untouched_when_remove_child_fails() {
        let mut dom = CountingDom {
            items: keys_n(3),
            fail_remove_child_for: std::collections::HashSet::from(["k1".to_string()]),
            ..Default::default()
        };
        let new_keys = vec!["k0".to_string(), "k2".to_string()];

        let fully_achieved = apply_ops(&mut dom, &new_keys);

        assert!(!fully_achieved, "削除失敗があったので未達成のはず");
        assert_eq!(
            dom.items,
            keys_n(3),
            "削除失敗時は items キャッシュから対象キーを取り除いてはならない"
        );
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

    // --- 属性検証拒否と「達成 Node」の整合（イシュー #1340 codex-review
    // P1〔4 巡目〕対応） ---

    /// 既存属性（`href="/safe"`）を危険値（`javascript:` スキーム）へ
    /// 更新しようとした場合、`sync_attrs`（`web-sys` 実装）は書き込みを
    /// skip して実 DOM は旧値のまま据え置かれる契約
    /// （`crate::keyed_dom::WebSysKeyedDom::sync_attrs` doc 参照）。
    /// `compose_achieved_children` が合成する「達成 Node」もこの実 DOM の
    /// 実際の状態（旧値）と一致しなければならない: 修正前は `new_by_key`
    /// （検証前の望ましい view）をそのまま使っていたため、危険値が
    /// 「達成 Node」へ紛れ込み、次回 `previous_list_node` としてキャッシュ
    /// された結果、以降同じ危険な view を再適用しても差分なしと判定され
    /// 実 DOM の乖離が解消されなくなっていた（PR #1340 codex-review 指摘）。
    #[test]
    fn compose_achieved_children_keeps_old_value_when_attr_update_is_rejected() {
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("href", "/safe")],
                vec![text("x")],
            ),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("href", "javascript:alert(1)")],
                vec![text("x")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            // `sync_attrs` は呼び出し後にライブ DOM を読み戻して「達成
            // 属性状態」を返す契約（イシュー #1340 codex-review P1〔5 巡目〕
            // 対応）のため、モックにも `old_items` と同じ「更新前のライブ
            // DOM 状態」を種付けしておく必要がある（実 DOM なら
            // `href="/safe"` が既に書き込まれている状態に相当）。
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![("href".to_string(), "/safe".to_string())],
            )]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(
            outcome.stale_update_keys.is_empty(),
            "属性検証拒否は子ノード構築失敗ではないため stale_update_keys \
             の対象にはならないはず"
        );
        assert!(
            !outcome.resync_required,
            "属性検証拒否は決定的な skip であり op 自体は計画どおり適用\
             できているため resync_required の対象にはならないはず"
        );

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(achieved.len(), 1);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            attrs.contains(&("href".to_string(), "/safe".to_string())),
            "実 DOM が書き込みを skip して旧値のまま据え置く契約と一致する \
             よう、達成 Node も旧値を保持するはず: {attrs:?}"
        );
        assert!(
            !attrs.iter().any(|(_, v)| v.contains("javascript:")),
            "拒否された危険値が達成 Node へ紛れ込んではならない: {attrs:?}"
        );
    }

    /// 既存アイテムに新規属性として危険値（`src="javascript:..."`）を
    /// 追加しようとした場合、`sync_attrs` は書き込みを skip し実 DOM には
    /// 当該属性が一切追加されない（旧値も存在しない、新規追加拒否のケース）。
    /// `compose_achieved_children` の達成 Node もこの「属性が存在しない」
    /// 状態と一致しなければならない。
    #[test]
    fn compose_achieved_children_omits_attr_when_new_attr_addition_is_rejected() {
        let old_items = vec![(
            "a".to_string(),
            el("li", vec![("data-key", "a")], vec![text("x")]),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("src", "javascript:alert(1)")],
                vec![text("x")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(outcome.stale_update_keys.is_empty());
        assert!(!outcome.resync_required);

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(achieved.len(), 1);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            !attrs.iter().any(|(k, _)| k == "src"),
            "旧値を持たない新規属性の書き込みが拒否された場合、達成 Node \
             にも当該属性を含めてはならない（実 DOM に一切追加されない \
             契約と一致させる）: {attrs:?}"
        );
    }

    /// 新規構築経路（`Insert`）でも同じ検証拒否が反映されることを確認する
    /// （`build_dom_node_with_namespace` の `set_attribute` skip と
    /// `compose_achieved_children` の整合、モジュール冒頭 doc「属性検証
    /// 拒否と「達成 Node」の整合」参照）。新規要素はそもそも旧値を持た
    /// ないため、拒否属性は無条件で除外される。
    #[test]
    fn compose_achieved_children_omits_attr_on_fresh_insert_when_validation_rejects() {
        let old_items: Vec<(String, Node)> = vec![];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("href", "javascript:alert(1)")],
                vec![text("x")],
            ),
        )];
        // `compose_achieved_children` は `ApplyOutcome` のみを入力とする
        // 純粋関数のため、実際の DOM 適用（`apply_ops_with_items`）を経由
        // せず「Insert が計画どおり完全に適用できた」場合の `ApplyOutcome`
        // を直接構築する（`final_keys` に対象キーを含み、`stale_update_keys`
        // は空・`resync_required` は `false` が「完全に適用できた」ことを
        // 表す契約、[`ApplyOutcome`] doc 参照）。
        // `achieved_attrs` を空（`Default`）のまま残すことで、`sync_attrs`
        // を経由しない新規構築経路（`compose_achieved_children` の else 腕）
        // をこのテストが正しく踏むことを示す。
        let outcome = ApplyOutcome {
            final_keys: vec!["a".to_string()],
            ..Default::default()
        };

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(achieved.len(), 1);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            !attrs.iter().any(|(k, _)| k == "href"),
            "新規構築（Insert）で検証拒否された属性は達成 Node にも含めて \
             はならない: {attrs:?}"
        );
    }

    /// 正常系の回帰固定（イシュー #1340 codex-review Bugbot〔6 巡目〕対応の
    /// 核心）: 全ての属性書き込みが成功した場合、`compose_achieved_children`
    /// が合成する「達成 Node」は `new_list_node`（の該当子ノード）と
    /// バイト等価（属性の順序・大小文字とも一致）になるはず。旧実装は
    /// `sync_attrs` の戻り値を `NamedNodeMap` の生の列挙順（新規属性は
    /// 末尾追加）で合成していたため、`new_attrs` の順序（`id`→`class`→
    /// `data-extra`）と異なる並びが返り、全操作成功の正常系ですら
    /// `diff_keyed_items`（順序に敏感な `PartialEq`）が不一致と判定して
    /// `Update` が毎 tick 再発火する不具合があった。属性名は
    /// `fandhe_frontend_core::keyed::keyed_list` が実際に構築する形
    /// （利用者属性の後ろに予約属性 `data-key` を付加する順序）に合わせて
    /// 構築する。
    #[test]
    fn compose_achieved_children_is_byte_equal_to_new_node_when_all_writes_succeed() {
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("id", "x"), ("class", "old"), ("data-key", "a")],
                vec![text("old")],
            ),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![
                    ("id", "x"),
                    ("class", "new"),
                    ("data-extra", "y"),
                    ("data-key", "a"),
                ],
                vec![text("new")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![
                    ("id".to_string(), "x".to_string()),
                    ("class".to_string(), "old".to_string()),
                ],
            )]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(outcome.stale_update_keys.is_empty());
        assert!(!outcome.resync_required);

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(
            achieved,
            vec![new_items[0].1.clone()],
            "全書き込み成功時、達成 Node は new_list_node とバイト等価\
             （属性の順序・大小文字とも一致）であるはず: {achieved:?}"
        );
    }

    // --- 親タグ変更時の丸ごと置換（イシュー #1340 codex-review P1
    // 〔9 巡目〕対応）: `crate::keyed_dom::replace_list_element_for_tag_change`
    // が「達成 Node」合成に使う `sanitize_node_for_achieved` の native
    // 検証。実 DOM 操作（`build_dom_node_with_namespace`/`replace_root_node`
    // の wasm-only 配線）自体は `crate::keyed_dom` の wasm_bindgen_test
    // （browser、成功系・detached fail-closed 系）が担う。ロールバック
    // 契約自体は `replace_root_node_*`（本ファイル、`VecRootReplace` 汎用
    // モック）が既に native 検証済みで、`ParentNodeRootReplace` は
    // `ListElementRootReplace` と同型のコンテナ差し替えのみの薄い
    // アダプタのため、この既存 native カバレッジをそのまま継承する。 ---

    /// 新 baseline 確定の回帰固定: 全属性が検証を通過する場合、
    /// `sanitize_node_for_achieved` は親要素・子アイテムを含む部分木全体を
    /// `new_list_node` とバイト等価のまま返す（`data-bind-list` 予約属性も
    /// 含めて引き継がれる。`build_dom_node_with_namespace` は予約属性を
    /// 特別扱いせず無条件で書き込むため、実 DOM 構築結果と一致する）。
    #[test]
    fn sanitize_node_for_achieved_preserves_full_tree_including_bind_list_attr_when_nothing_rejected(
    ) {
        let items: Vec<(String, Node)> = vec![
            ("a".to_string(), el("li", vec![], vec![text("a")])),
            ("b".to_string(), el("li", vec![], vec![text("b")])),
        ];
        let new_list_node =
            fandhe_frontend_core::keyed::keyed_list("ol", vec![("class", "list")], "items", items)
                .expect("valid keyed list");

        let achieved = sanitize_node_for_achieved(&new_list_node);

        assert_eq!(
            achieved, new_list_node,
            "検証拒否属性が無い場合、達成 Node（新 baseline）は \
             new_list_node（data-bind-list・子アイテムの data-key を含む）と \
             バイト等価であるはず: {achieved:?}"
        );
        let Node::Element { attrs, .. } = &achieved else {
            panic!("要素ノードのはず");
        };
        assert!(
            attrs
                .iter()
                .any(|(k, v)| k == fandhe_frontend_core::keyed::BIND_LIST_ATTR && v == "items"),
            "予約属性 data-bind-list が新コンテナへ引き継がれているはず: \
             {attrs:?}"
        );
    }

    /// 親タグ変更に伴う丸ごと再構築でも、検証を拒否された属性（危険 URL
    /// スキーム等）は親要素・子アイテムのいずれでも達成 Node から除外
    /// されること（新規構築経路のセキュリティ不変条件が置換パスにも
    /// 及ぶことの確認）。
    #[test]
    fn sanitize_node_for_achieved_strips_rejected_attrs_from_parent_and_children_on_replace() {
        let items: Vec<(String, Node)> = vec![(
            "a".to_string(),
            el("a", vec![("href", "javascript:alert(1)")], vec![text("a")]),
        )];
        let new_list_node = fandhe_frontend_core::keyed::keyed_list(
            "ol",
            vec![("onclick", "alert(1)"), ("class", "list")],
            "items",
            items,
        )
        .expect("valid keyed list");

        let achieved = sanitize_node_for_achieved(&new_list_node);

        let Node::Element {
            attrs: parent_attrs,
            children,
            ..
        } = &achieved
        else {
            panic!("要素ノードのはず");
        };
        assert!(
            !parent_attrs.iter().any(|(k, _)| k == "onclick"),
            "親要素のイベントハンドラ属性は除外されるはず: {parent_attrs:?}"
        );
        assert!(
            parent_attrs.contains(&(
                fandhe_frontend_core::keyed::BIND_LIST_ATTR.to_string(),
                "items".to_string()
            )),
            "拒否属性のフィルタ後も予約属性 data-bind-list は残るはず: \
             {parent_attrs:?}"
        );
        let Node::Element {
            attrs: child_attrs, ..
        } = &children[0]
        else {
            panic!("要素ノードのはず");
        };
        assert!(
            !child_attrs.iter().any(|(k, _)| k == "href"),
            "子アイテムの危険 URL 属性も除外されるはず: {child_attrs:?}"
        );
    }

    /// keyed list の親要素（`list_element` 自身）の属性同期
    /// （`crate::keyed_apply::sync_parent_attrs`、イシュー #1340
    /// codex-review P1〔7 巡目〕対応）の正常系: 全操作成功時、戻り値は
    /// `new_parent_attrs` とバイト等価（`data-bind-list` は末尾に保持）に
    /// なるはず。
    #[test]
    fn sync_parent_attrs_applies_new_values_and_keeps_bind_list_attr_at_tail() {
        let old_parent_attrs = vec![
            ("class".to_string(), "old".to_string()),
            (
                fandhe_frontend_core::keyed::BIND_LIST_ATTR.to_string(),
                "items".to_string(),
            ),
        ];
        let new_parent_attrs = vec![
            ("class".to_string(), "new".to_string()),
            ("id".to_string(), "list".to_string()),
            (
                fandhe_frontend_core::keyed::BIND_LIST_ATTR.to_string(),
                "items".to_string(),
            ),
        ];
        let mut dom = CountingDom {
            attrs: std::collections::HashMap::from([(
                "list".to_string(),
                vec![("class".to_string(), "old".to_string())],
            )]),
            ..Default::default()
        };

        let achieved = sync_parent_attrs(
            &mut dom,
            &"list".to_string(),
            &old_parent_attrs,
            &new_parent_attrs,
        );

        assert_eq!(
            achieved, new_parent_attrs,
            "全書き込み成功時、親要素の達成属性列は new_parent_attrs と \
             バイト等価であるはず（data-bind-list は末尾のまま）: {achieved:?}"
        );
        assert!(
            !dom.attrs
                .get("list")
                .expect("sync_attrs が呼ばれているはず")
                .iter()
                .any(|(k, _)| k == fandhe_frontend_core::keyed::BIND_LIST_ATTR),
            "予約属性 data-bind-list は sync_attrs（実 DOM 書き込み）へ \
             一切渡されないはず（同期対象から除外、多層防御）"
        );
    }

    /// `sync_parent_attrs` の実行時失敗反映: `setAttribute` の実行時失敗が
    /// 親要素の達成属性列にも正しく反映されること（子アイテムの
    /// `compose_achieved_children_keeps_old_value_when_set_attribute_fails_at_runtime`
    /// と同型の契約を親要素へも拡張する）。
    #[test]
    fn sync_parent_attrs_keeps_old_value_when_set_attribute_fails_at_runtime() {
        let old_parent_attrs = vec![("class".to_string(), "old".to_string())];
        let new_parent_attrs = vec![("class".to_string(), "new".to_string())];
        let mut dom = CountingDom {
            attrs: std::collections::HashMap::from([(
                "list".to_string(),
                vec![("class".to_string(), "old".to_string())],
            )]),
            fail_set_attribute_for: std::collections::HashSet::from([(
                "list".to_string(),
                "class".to_string(),
            )]),
            ..Default::default()
        };

        let achieved = sync_parent_attrs(
            &mut dom,
            &"list".to_string(),
            &old_parent_attrs,
            &new_parent_attrs,
        );

        assert_eq!(
            achieved,
            vec![("class".to_string(), "old".to_string())],
            "setAttribute 失敗時、達成属性列は実 DOM の実際の状態（旧値）を \
             反映するはず: {achieved:?}"
        );
    }

    // --- 削除判定のライブ属性ドリフト対応（イシュー #1340 codex-review
    // Bugbot〔8 巡目〕対応） ---

    /// SSR hydrate 直後の想定回帰固定: ライブ DOM 上にはあるが
    /// `old_attrs`（クライアント側キャッシュ）には無い属性（hydrate 由来の
    /// ライブ属性ドリフト）が、`new_attrs` にも存在しなければ削除される
    /// こと。旧実装は `old_attrs` のみを削除判定に使っていたため、この
    /// ケースの属性が Update を何度適用しても永遠に除去されなかった。
    #[test]
    fn apply_ops_with_items_removes_live_only_attr_absent_from_old_attrs_cache() {
        // `old_attrs`（キャッシュ）は "class" のみを知っているが、
        // モックの「ライブ DOM」（`dom.attrs`）には hydrate 由来の
        // "data-ssr-only" も実際に存在する状況を再現する。
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "old")],
                vec![text("x")],
            ),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "new")],
                vec![text("x")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![
                    ("class".to_string(), "old".to_string()),
                    ("data-ssr-only".to_string(), "server-value".to_string()),
                ],
            )]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(outcome.stale_update_keys.is_empty());
        assert!(!outcome.resync_required);

        assert_eq!(
            dom.attrs.get("a"),
            Some(&vec![("class".to_string(), "new".to_string())]),
            "old_attrs キャッシュに存在しない hydrate 由来のライブ属性も、\
             new_attrs に無ければ削除されるはず（ライブ列挙基準の削除判定）: \
             {:?}",
            dom.attrs.get("a")
        );

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            !attrs.iter().any(|(k, _)| k == "data-ssr-only"),
            "削除に成功した hydrate 由来の属性は達成 Node に含まれないはず: \
             {attrs:?}"
        );
    }

    /// 予約属性（`data-key`）はライブ属性列挙による削除判定からも除外
    /// されること（`reserved_attr` ガードが `old_attrs`/`new_attrs` の
    /// 事前フィルタだけに依存しない多層防御であることの確認）。ライブ DOM
    /// 上に `data-key` が存在する状態を明示的にモックへ種付けし、
    /// `sync_attrs` 呼び出し後も残存すること・達成 Node に重複して含まれ
    /// ないこと（`compose_in_place_updated_node` が別途 1 回だけ追加する
    /// 契約）を確認する。
    #[test]
    fn apply_ops_with_items_never_removes_reserved_attr_via_live_enumeration() {
        let old_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "old")],
                vec![text("x")],
            ),
        )];
        let new_items = vec![(
            "a".to_string(),
            el(
                "li",
                vec![("data-key", "a"), ("class", "new")],
                vec![text("x")],
            ),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            // ライブ DOM（モック）上に予約属性 `data-key` が実在する状態を
            // 明示的に種付けする（実 DOM では `data-key` は常に存在するが、
            // `sync_attrs`/`CountingDom.attrs` は他テストでは通常
            // `data-key` を含めずに管理しているため、ここで明示する）。
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![
                    ("data-key".to_string(), "a".to_string()),
                    ("class".to_string(), "old".to_string()),
                ],
            )]),
            ..Default::default()
        };

        apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            dom.attrs
                .get("a")
                .expect("sync_attrs が呼ばれているはず")
                .iter()
                .any(|(k, _)| k == "data-key"),
            "予約属性 data-key はライブ属性列挙による削除判定の対象から \
             除外され、実 DOM（モック）に残り続けるはず: {:?}",
            dom.attrs.get("a")
        );
    }

    /// `setAttribute` の実行時失敗（不正な属性名等の `InvalidCharacterError`
    /// 相当、`fail_set_attribute_for` で注入）を模した回帰固定（イシュー
    /// #1340 codex-review P1〔5 巡目〕対応）。ポリシー拒否（危険 URL 等）とは
    /// 異なり `(属性名, 属性値)` だけからは判定できない失敗のため、
    /// `sync_attrs` の戻り値（実 DOM 読み戻し）を経由しないと「達成 Node」
    /// へ正しく反映できないことを確認する。旧実装は `sync_attrs` の
    /// `Result` を無条件に破棄し新値を達成済み扱いにしていたため、この
    /// ケースで実 DOM（旧値のまま）とキャッシュ（新値）が乖離していた。
    #[test]
    fn compose_achieved_children_keeps_old_value_when_set_attribute_fails_at_runtime() {
        let old_items = vec![item_with_attr("a", "class", "old")];
        let new_items = vec![item_with_attr("a", "class", "new")];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![("class".to_string(), "old".to_string())],
            )]),
            fail_set_attribute_for: std::collections::HashSet::from([(
                "a".to_string(),
                "class".to_string(),
            )]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(
            outcome.stale_update_keys.is_empty(),
            "属性の実行時失敗は子ノード構築失敗ではないため stale_update_keys \
             の対象にはならないはず"
        );
        assert!(
            !outcome.resync_required,
            "op 自体（Update）は計画どおり実行された（属性 1 件の書き込みが \
             実 DOM 側で失敗しただけ）ため resync_required の対象にはならない \
             はず（本モジュール doc 「Update op の DOM 適用」参照）"
        );
        assert_eq!(
            dom.attrs.get("a"),
            Some(&vec![("class".to_string(), "old".to_string())]),
            "実 DOM（モック）は setAttribute 失敗により旧値のまま据え置かれる \
             はず"
        );

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(achieved.len(), 1);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            attrs.contains(&("class".to_string(), "old".to_string())),
            "達成 Node も実 DOM の実際の状態（旧値のまま）と一致するはず: \
             {attrs:?}"
        );
        assert!(
            !attrs.iter().any(|(_, v)| v == "new"),
            "setAttribute 失敗で書き込まれなかった新値が達成 Node へ紛れ込ん \
             ではならない: {attrs:?}"
        );
    }

    /// `removeAttribute` の実行時失敗（`fail_remove_attribute_for` で注入）を
    /// 模した回帰固定（イシュー #1340 codex-review P1〔5 巡目〕対応）。
    /// 新しい view が当該属性を持たなくなった（`new_attrs` から消えた）にも
    /// かかわらず削除が実 DOM 側で失敗した場合、「達成 Node」は旧属性が
    /// 残存している実 DOM の実際の状態と一致しなければならない。
    #[test]
    fn compose_achieved_children_keeps_stale_attr_when_remove_attribute_fails_at_runtime() {
        let old_items = vec![item_with_attr("a", "class", "old")];
        let new_items = vec![(
            "a".to_string(),
            el("li", vec![("data-key", "a")], vec![text("x")]),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            attrs: std::collections::HashMap::from([(
                "a".to_string(),
                vec![("class".to_string(), "old".to_string())],
            )]),
            fail_remove_attribute_for: std::collections::HashSet::from([(
                "a".to_string(),
                "class".to_string(),
            )]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);
        assert!(outcome.stale_update_keys.is_empty());
        assert!(
            !outcome.resync_required,
            "removeAttribute の単発失敗は Update op 自体の未達成ではないため \
             resync_required の対象にはならないはず"
        );
        assert_eq!(
            dom.attrs.get("a"),
            Some(&vec![("class".to_string(), "old".to_string())]),
            "実 DOM（モック）は removeAttribute 失敗により旧属性が残存する \
             はず"
        );

        let achieved = compose_achieved_children(&old_items, &new_items, &outcome);
        assert_eq!(achieved.len(), 1);
        let Node::Element { attrs, .. } = &achieved[0] else {
            panic!("要素ノードのはず");
        };
        assert!(
            attrs.contains(&("class".to_string(), "old".to_string())),
            "実 DOM に残存したままの旧属性は達成 Node にも反映されなければ \
             ならない（削除できたと誤ってキャッシュしてはならない）: {attrs:?}"
        );
    }

    /// `item`（`(key, text_value)`）と同型の、単一属性付き要素を組み立てる
    /// テスト用ヘルパー（属性検証拒否・実行時失敗系のテストで使う）。
    fn item_with_attr(key: &str, attr_name: &'static str, attr_value: &str) -> (String, Node) {
        (
            key.to_string(),
            el(
                "li",
                vec![("data-key", key), (attr_name, attr_value)],
                vec![text("x")],
            ),
        )
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

    /// codex-review P1 回帰固定（PR #1340 push 後の再レビュー、イシュー
    /// #1340）: 同一キーでルート要素のタグが変わる `KeyedOp::Update`
    /// （`li` → `div`）は「浅い in-place 更新」（`sync_attrs`/
    /// `replace_item_children`）を経由せず、`create_item` + `replace_root`
    /// （アイテム全置換）で処理されること。`sync_attrs`/
    /// `replace_item_children` が一切呼ばれないこと（タグを書き換えられない
    /// 経路が誤って使われていないことの直接確認）・`replace_root` がちょうど
    /// 1 回呼ばれること・`stale_update_keys`/`resync_required` のいずれも
    /// 立たない（新規構築のためこの回で完全達成している）ことを確認する。
    #[test]
    fn apply_ops_with_items_replaces_root_when_update_changes_tag() {
        let old_items = vec![(
            "a".to_string(),
            el("li", vec![("data-key", "a")], vec![text("old")]),
        )];
        let new_items = vec![(
            "a".to_string(),
            el("div", vec![("data-key", "a")], vec![text("new")]),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert_eq!(
            dom.calls.replace_root, 1,
            "タグ変更は replace_root（アイテム全置換）でちょうど 1 回 \
             処理されるはず（内訳: {:?}）",
            dom.calls
        );
        assert_eq!(
            dom.calls.sync_attrs, 0,
            "タグ変更を伴う Update は「浅い in-place 更新」経路 \
             （sync_attrs）を経由してはならない（内訳: {:?}）",
            dom.calls
        );
        assert_eq!(
            dom.calls.replace_item_children, 0,
            "タグ変更を伴う Update は replace_item_children も経由しては \
             ならない（内訳: {:?}）",
            dom.calls
        );
        assert_eq!(dom.calls.create_item, 1);
        assert!(
            outcome.stale_update_keys.is_empty(),
            "新規構築による全置換のため stale 扱いにはならないはず"
        );
        assert!(
            !outcome.resync_required,
            "構築が成功した場合は完全達成のため再同期は不要のはず"
        );
        assert_eq!(outcome.final_keys, vec!["a".to_string()]);
    }

    /// 上記のタグ変更ケースで、新しい要素の構築自体が失敗（`RawHtml`
    /// 混入相当）した場合は、旧ルート要素に一切触れず（fail-closed）
    /// `resync_required` を立てること。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_tag_change_construction_fails() {
        let old_items = vec![(
            "a".to_string(),
            el("li", vec![("data-key", "a")], vec![text("old")]),
        )];
        let new_items = vec![(
            "a".to_string(),
            el("div", vec![("data-key", "a")], vec![text("new")]),
        )];
        let mut dom = PoisonedCreateDom {
            inner: CountingDom {
                items: vec!["a".to_string()],
                ..Default::default()
            },
            poisoned_key: "a".to_string(),
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "構築失敗時は再同期を要求するはず（旧タグのまま残る事実を \
             final_keys だけからは判別できないため）"
        );
        assert_eq!(
            dom.inner.items,
            vec!["a".to_string()],
            "構築失敗時はライブ側に一切変更が起きないはず（fail-closed）"
        );
        assert_eq!(dom.inner.calls.replace_root, 0);
    }

    // --- exchange_children（イシュー #1340 codex-review P1〔2 巡目〕、
    // 子ノード交換のコミットフェーズ部分失敗ロールバック）---

    /// `ChildExchangeDom` の native モック（`char` を子ノード識別子として
    /// 使う `Vec` ベースの実装）。`fail_remove_at`/`fail_insert_at`
    /// （0-origin の呼び出し回数目）で任意の `remove_child`/`insert_before`
    /// 呼び出しを決定的に失敗させられる（codex-review 指摘「n 回目の
    /// append_child を失敗させるテストダブル」に対応。`insert_before` は
    /// `reference: None` の呼び出しが `append_child` 相当、トレイト doc
    /// 参照）。
    #[derive(Default)]
    struct VecChildExchange {
        children: Vec<char>,
        fail_remove_at: Option<usize>,
        fail_insert_at: Option<usize>,
        remove_calls: usize,
        insert_calls: usize,
        rollback_failed_calls: usize,
    }

    impl ChildExchangeDom for VecChildExchange {
        type Node = char;

        fn current_children(&mut self) -> Vec<char> {
            self.children.clone()
        }

        fn remove_child(&mut self, node: &char) -> bool {
            let call_index = self.remove_calls;
            self.remove_calls += 1;
            if self.fail_remove_at == Some(call_index) {
                return false;
            }
            if let Some(pos) = self.children.iter().position(|c| c == node) {
                self.children.remove(pos);
            }
            true
        }

        fn insert_before(&mut self, node: &char, reference: Option<&char>) -> bool {
            let call_index = self.insert_calls;
            self.insert_calls += 1;
            if self.fail_insert_at == Some(call_index) {
                return false;
            }
            let pos = match reference {
                Some(r) => self
                    .children
                    .iter()
                    .position(|c| c == r)
                    .unwrap_or(self.children.len()),
                None => self.children.len(),
            };
            self.children.insert(pos, *node);
            true
        }

        fn on_rollback_failed(&mut self) {
            self.rollback_failed_calls += 1;
        }
    }

    /// 旧子ノード取り外しフェーズの `i` 件目（0-origin）で `remove_child`
    /// が失敗した場合、既に取り外し済みの `0..i` 件が、まだ付いたままの
    /// `i` 件目の直前へ元の順序で再度取り付けられ、構造が Update 適用開始
    /// 前の状態へ完全に復元されること（`docs/design/keyed-update-op-design.md`
    /// §6 不変条件 6「取り外しフェーズでの失敗」の回帰固定）。
    #[test]
    fn exchange_children_restores_old_structure_when_nth_remove_fails() {
        let mut dom = VecChildExchange {
            children: vec!['a', 'b', 'c'],
            fail_remove_at: Some(1), // 2 件目（'b'）の remove_child が失敗
            ..Default::default()
        };
        let built = vec!['x', 'y'];

        let achieved = exchange_children(&mut dom, &built);

        assert!(!achieved, "部分失敗時は false（未達成）を返すはず");
        assert_eq!(
            dom.children,
            vec!['a', 'b', 'c'],
            "取り外し済み分がロールバックにより元の順序で復元され、\
             構造は Update 適用開始前と一致するはず"
        );
        assert_eq!(
            dom.rollback_failed_calls, 0,
            "ロールバック自体は成功しているはず"
        );
    }

    /// 旧子ノードの取り外しをすべて終えた後、新子ノード `j` 件目
    /// （0-origin）の `insert_before`（`append_child` 相当）が失敗した
    /// 場合、追加済み `0..j` 件を取り除き、保持しておいた旧子ノード列が
    /// 元の順序で再度取り付けられること（設計書 §6 不変条件 6「取り付け
    /// フェーズでの失敗」の回帰固定）。
    #[test]
    fn exchange_children_restores_old_structure_when_nth_append_fails() {
        let mut dom = VecChildExchange {
            children: vec!['a', 'b'],
            fail_insert_at: Some(1), // 2 件目（'y'）の insert_before が失敗
            ..Default::default()
        };
        let built = vec!['x', 'y', 'z'];

        let achieved = exchange_children(&mut dom, &built);

        assert!(!achieved, "部分失敗時は false（未達成）を返すはず");
        assert_eq!(
            dom.children,
            vec!['a', 'b'],
            "追加済み新子ノードが取り除かれ、旧子ノード列が元の順序で \
             再度取り付けられ、構造は Update 適用開始前と一致するはず"
        );
        assert_eq!(
            dom.rollback_failed_calls, 0,
            "ロールバック自体は成功しているはず"
        );
    }

    /// ロールバック手順自体（取り外し済みノードの再取り付け）が失敗する
    /// 残余リスク（設計書 §6 不変条件 6「残る有限のリスク」）が発生した
    /// 場合、`on_rollback_failed` が呼ばれ、当該アイテムの構造が不定状態
    /// （部分的にしか復元されない）になりうることを許容すること。
    #[test]
    fn exchange_children_reports_rollback_failure_without_panicking() {
        let mut dom = VecChildExchange {
            children: vec!['a', 'b', 'c'],
            fail_remove_at: Some(1), // 'b' の remove_child が失敗
            fail_insert_at: Some(0), // ロールバックの再取り付けも失敗
            ..Default::default()
        };
        let built = vec!['x'];

        let achieved = exchange_children(&mut dom, &built);

        assert!(!achieved);
        assert_eq!(
            dom.rollback_failed_calls, 1,
            "ロールバック失敗が検知され on_rollback_failed が呼ばれるはず \
             （`unwrap()`/`panic!` を使わず処理を継続する）"
        );
        assert_eq!(
            dom.children,
            vec!['b', 'c'],
            "ロールバック自体が失敗したため完全復元はできないが（残余 \
             リスクとして許容）、それ以上の破壊（例: built の混入）は \
             起きないはず"
        );
    }

    // --- replace_root_node（イシュー #1340 codex-review P1〔3 巡目〕、
    // ルート要素置換のコミットフェーズ部分失敗ロールバック）---

    /// `RootReplaceDom` の native モック（`char` をノード識別子として使う）。
    /// `fail_insert`/`fail_remove_old`/`fail_remove_new` で
    /// `insert_before`/`remove`（`old` 対象）/`remove`（ロールバックの
    /// `new` 対象）をそれぞれ独立に決定的に失敗させられる（codex-review
    /// 指摘「insert_before / remove_child それぞれの失敗を注入する
    /// テストダブル」に対応）。
    #[derive(Default)]
    struct VecRootReplace {
        fail_insert: bool,
        fail_remove_old: bool,
        fail_remove_new: bool,
        inserted: Vec<char>,
        removed: Vec<char>,
        rollback_failed_calls: usize,
    }

    impl RootReplaceDom for VecRootReplace {
        type Node = char;

        fn insert_before(&mut self, new: &char, _old: &char) -> bool {
            if self.fail_insert {
                return false;
            }
            self.inserted.push(*new);
            true
        }

        fn remove(&mut self, node: &char) -> bool {
            if *node == 'o' && self.fail_remove_old {
                return false;
            }
            if *node == 'n' && self.fail_remove_new {
                return false;
            }
            self.removed.push(*node);
            true
        }

        fn on_rollback_failed(&mut self) {
            self.rollback_failed_calls += 1;
        }
    }

    /// 挿入・除去のいずれも成功した場合、`true` を返し `new` の挿入・
    /// `old` の除去がちょうど 1 回ずつ行われること（正常系の裏付け）。
    #[test]
    fn replace_root_node_succeeds_when_both_operations_succeed() {
        let mut dom = VecRootReplace::default();

        let achieved = replace_root_node(&mut dom, &'o', &'n');

        assert!(achieved);
        assert_eq!(dom.inserted, vec!['n']);
        assert_eq!(dom.removed, vec!['o']);
        assert_eq!(dom.rollback_failed_calls, 0);
    }

    /// 新要素の挿入自体が失敗した場合、`old` には一切触れず（`remove` が
    /// 呼ばれない）`false` を返すこと（codex-review 指摘の再現手順その
    /// もの: 挿入失敗後に旧要素だけ削除してキーが消滅する不具合の防止）。
    #[test]
    fn replace_root_node_leaves_old_untouched_when_insert_fails() {
        let mut dom = VecRootReplace {
            fail_insert: true,
            ..Default::default()
        };

        let achieved = replace_root_node(&mut dom, &'o', &'n');

        assert!(!achieved);
        assert!(
            dom.removed.is_empty(),
            "挿入が失敗した場合、old の除去を試みてはならない（キー消滅の \
             防止）"
        );
        assert_eq!(dom.rollback_failed_calls, 0);
    }

    /// 新要素の挿入は成功したが旧要素の除去が失敗した場合、挿入済みの
    /// `new` を取り除いて挿入前の状態へロールバックし `false` を返すこと
    /// （codex-review 指摘の再現手順: 挿入成功後の削除失敗による同一キー
    /// 要素の重複を防ぐ）。
    #[test]
    fn replace_root_node_rolls_back_inserted_new_when_remove_old_fails() {
        let mut dom = VecRootReplace {
            fail_remove_old: true,
            ..Default::default()
        };

        let achieved = replace_root_node(&mut dom, &'o', &'n');

        assert!(!achieved);
        assert_eq!(dom.inserted, vec!['n'], "new の挿入自体は成功しているはず");
        assert_eq!(
            dom.removed,
            vec!['n'],
            "old の除去は失敗するため、代わりに挿入済みの new がロール \
             バックで取り除かれ、old のみが残る状態へ復元されるはず \
             （old 自体は一度も除去されていないため removed には含まれ \
             ない）"
        );
        assert_eq!(dom.rollback_failed_calls, 0);
    }

    /// ロールバック手順自体（挿入済み `new` の除去）も失敗した場合、
    /// `on_rollback_failed` が呼ばれ、`unwrap()`/`panic!` を使わず処理を
    /// 継続すること（設計書 §6 不変条件 6「残る有限のリスク」と同種の
    /// 許容）。
    #[test]
    fn replace_root_node_reports_rollback_failure_without_panicking() {
        let mut dom = VecRootReplace {
            fail_remove_old: true,
            fail_remove_new: true,
            ..Default::default()
        };

        let achieved = replace_root_node(&mut dom, &'o', &'n');

        assert!(!achieved);
        assert_eq!(
            dom.rollback_failed_calls, 1,
            "ロールバック失敗が検知され on_rollback_failed が呼ばれるはず"
        );
        assert!(
            dom.removed.is_empty(),
            "old も new もいずれも除去できておらず、new が挿入されたまま \
             old と共存する不定状態が残りうる（残余リスクとして許容）"
        );
    }

    /// `apply_ops_with_items` の呼び出し元側（イシュー #1340 codex-review
    /// P1〔3 巡目〕）: `KeyedListDom::replace_root` が `false`（DOM 操作
    /// 失敗、`CountingDom::fail_replace_root_for` で模擬）を返した場合、
    /// `resync_required` が立ち、`items`（ライブ側）が一切変更されない
    /// こと（`replace_root` が返す `false` を無視して無条件に「達成」と
    /// キャッシュしてしまう codex-review 指摘の再発防止）。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_replace_root_fails() {
        let old_items = vec![(
            "a".to_string(),
            el("li", vec![("data-key", "a")], vec![text("old")]),
        )];
        let new_items = vec![(
            "a".to_string(),
            el("div", vec![("data-key", "a")], vec![text("new")]),
        )];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            fail_replace_root_for: std::collections::HashSet::from(["a".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "replace_root が false を返した場合、キャッシュを確定させず \
             再同期を要求するはず"
        );
        assert_eq!(
            dom.items,
            vec!["a".to_string()],
            "replace_root 失敗時はライブ側（items）が一切変更されない \
             はず（CountingDom::replace_root の fail-closed 契約）"
        );
        assert_eq!(dom.calls.replace_root, 1);
    }

    /// `apply_ops_with_items` の `Insert` op で `insert_before_batch` が実
    /// DOM 挿入失敗を返した場合、`resync_required` が立ち `final_keys` から
    /// 対象キーが除外され、`items`（ライブ側）が一切変更されないこと
    /// （イシュー #1340 codex-review P1〔3 巡目〕全走査対応）。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_insert_before_batch_fails() {
        let old_items: Vec<(String, Node)> = vec![];
        let new_items = vec![item("a", "new")];
        let mut dom = CountingDom {
            items: vec![],
            fail_insert_before_batch_for: std::collections::HashSet::from(["a".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "insert_before_batch が false を返した場合、キャッシュを確定 \
             させず再同期を要求するはず"
        );
        assert!(
            outcome.final_keys.is_empty(),
            "未達成の Insert 対象キーは final_keys から除外されるはず"
        );
        assert!(
            dom.items.is_empty(),
            "insert_before_batch 失敗時はライブ側（items）が一切変更 \
             されないはず"
        );
    }

    /// `apply_ops_with_items` の `Move` op で `move_before` が実 DOM 移動
    /// 失敗を返した場合、`resync_required` が立ち `items`（ライブ側の並び）
    /// が一切変更されないこと（イシュー #1340 codex-review P1〔3 巡目〕
    /// 全走査対応）。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_move_before_fails() {
        let old_items = vec![item("a", "a-text"), item("b", "b-text")];
        let new_items = vec![item("b", "b-text"), item("a", "a-text")];
        let mut dom = CountingDom {
            items: vec!["a".to_string(), "b".to_string()],
            fail_move_before_for: std::collections::HashSet::from(["b".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "move_before が false を返した場合、再同期を要求するはず"
        );
        assert_eq!(
            dom.items,
            vec!["a".to_string(), "b".to_string()],
            "move_before 失敗時はライブ側（並び順）が一切変更されない \
             はず"
        );
    }

    /// `apply_ops_with_items` の `Remove` op で `remove_child` が実 DOM
    /// 削除失敗を返した場合、`resync_required` が立ち `items`（ライブ側）
    /// から対象キーが取り除かれないこと（イシュー #1340 codex-review P1
    /// 〔3 巡目〕全走査対応）。
    #[test]
    fn apply_ops_with_items_signals_resync_required_when_remove_child_fails() {
        let old_items = vec![item("a", "a-text")];
        let new_items: Vec<(String, Node)> = vec![];
        let mut dom = CountingDom {
            items: vec!["a".to_string()],
            fail_remove_child_for: std::collections::HashSet::from(["a".to_string()]),
            ..Default::default()
        };

        let outcome = apply_ops_with_items(&mut dom, &old_items, &new_items);

        assert!(
            outcome.resync_required,
            "remove_child が false を返した場合、再同期を要求するはず"
        );
        assert_eq!(
            dom.items,
            vec!["a".to_string()],
            "remove_child 失敗時はライブ側（items）から対象キーを取り除いて \
             はならない"
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
