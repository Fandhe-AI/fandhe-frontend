//! Marquee（イシュー #831）: `docs/policy/intentional-non-adoption.md` §3.24
//! で意図的非採用と確定していた自動流動テキストを、CSS のみ・
//! `prefers-reduced-motion` 対応の決定的設計案で再導入する静的 styled 部品。
//!
//! # 再導入の経緯（`.claude/rules/coding-rust.md` 「意図的非採用機能の
//! 再導入提案には評価軸の充足確認が必須」対応）
//!
//! §3.24 の非採用理由は「純装飾で CSS のみで代替可能」「自動アニメーションが
//! 挙動分岐を持ち込み決定的なユニットテスト検証になじまない」の 2 点だった。
//! 本モジュールは §3.24 の再評価トリガー 1（「自動流動テキストの需要が確定し、
//! かつ `prefers-reduced-motion` 等のアクセシビリティ要件を満たす決定的な
//! 設計案が提示された場合」）を、以下の設計で充足する:
//!
//! 1. **明示性**: マークアップは `data-scope="marquee"` / `data-part` の
//!    決定的ノード木、スタイルは静的 CSS のみ。JS・状態機械・実行時分岐は
//!    一切持たない。
//! 2. **決定性**: [`css`] はソースコード中の静的リテラルのみから同一入力 →
//!    同一出力を生成する（golden CSS テストでバイト単位固定、
//!    `tests/marquee_css.rs`）。アニメーション実行はブラウザの CSS ランタイム
//!    に委ね、Rust 側出力は完全に決定的である（検証対象を CSS 出力文字列に
//!    固定することで #735 時点の懸念を解消する）。
//! 3. **機械検証可能性**: golden CSS・`render()` ユニットテスト・XSS 回帰
//!    （`tests/xss_escape_styled.rs`）・`stylesheet.rs` の全部品網羅ドリフト
//!    検知テストで機械検証する。
//! 4. **コンテキスト消費**: 新規モジュール 1 個（公開関数 2 個 + Props +
//!    enum 1 個）。[`crate::skeleton`]/[`crate::spinner`] と同型パターンの
//!    再利用で追加学習コストを最小化する。
//! 5. **不変条件を弱めない**: 新規依存クレートゼロ・既定エスケープ（REQ-1）
//!    迂回なし・`unsafe` なし・HTML/CSS への実行時入力の文字列結合なし。
//!
//! # anatomy（ark-ui 参考の縮約）
//!
//! ark-ui の Marquee anatomy（`Root`/`Viewport`/`Content`/`Item`/`Edge`）を
//! CSS のみで成立する最小 3 パーツへ縮約する: `Viewport` は `root` が兼ね、
//! `Edge`（両端フェードのグラデーション）は呼び出し側 CSS（`mask-image` 等）
//! で代替可能な純装飾のため提供しない。
//!
//! # シームレスループの実現方法
//!
//! [`marquee`] は `content` パーツ（`data-part="content"`）を **2 回複製**
//! して並べる（自動流動テキストの標準的なシームレスループ手法。`children`
//! は [`fandhe_frontend_core::Node`] が `Clone` を実装しているため
//! `children.clone()` で複製する）。**2 個目の `content` には常に
//! `aria-hidden="true"`** に加え **`inert`** を付与し、スクリーンリーダーの
//! 二重読み上げとキーボードフォーカスの二重発生を防ぐ（呼び出し側が
//! これを外すオプションは設けない、[`crate::skeleton`] と同型の
//! fail-closed 判断）。`aria-hidden` のみでは支援技術向けの意味論しか
//! 遮断せず複製内のリンク等フォーカス可能な子孫がタブ順序に残ってしまう
//! ため、キーボードフォーカスも遮断する `inert`（HTML 標準のグローバル
//! 属性）を併用する（Cursor Bugbot 指摘、PR #864）。さらに
//! `prefers-reduced-motion: reduce` 環境では 2 個目の `content` を
//! `display: none` で完全に除去し、視覚的な二重表示も防ぐ（[`css`]
//! 参照、同指摘）。
//!
//! # variant（`direction` のみ、root へのみクラス付与）
//!
//! [`MarqueeDirection`]（`Start`（既定）/`End`）は `root` パーツのみへ
//! クラスを付与し、`content` への伝搬は CSS custom property
//! （`--fandhe-marquee-direction`）の通常の CSS 継承で行う（複合部品の
//! variant 統一方針、[`crate::timeline`] と同型のパターン。かつて
//! `indicator`/`separator` に対してこの伝搬を直接セレクタで表現しようとして
//! 死んだ CSS を生んだ教訓、PR #812 修正コミット 54126cb を踏まえ、本
//! モジュールは最初から custom property 経由で設計する）。
//!
//! `color-palette`/`size` 軸は提供しない（[`crate::skeleton`]/[`crate::card`]
//! と同型の「中立・装飾部品」判断）。速度・間隔は CSS custom property の
//! フォールバック（`--fandhe-marquee-duration, 20s` / `--fandhe-marquee-gap,
//! 1rem`）として与え、呼び出し側が `style` 属性で上書きする契約とする。
//! ark-ui の `speed`/`spacing`/`autoFill`/`loopCount` 相当は props へ持ち込ま
//! ない（本イシューのスコープ外、下記節参照）。
//!
//! # a11y 契約: `decorative`/`label`
//!
//! [`MarqueeProps::decorative`]（既定 `false`）が `true` の場合、`root` へ
//! `aria-hidden="true"` を付与し純装飾として扱う（[`crate::skeleton`] と
//! 同型）。この際 `root` 自身ではなく可視の主コピー（1 個目の `content`）へ
//! `inert` を付与し、`aria-hidden` だけでは遮断できないキーボードフォーカス
//! も遮断する（複製 2 個目の `content` 側の `inert` 付与（上記「シームレス
//! ループの実現方法」節参照）と同じ理由で、支援技術からは存在しないものと
//! して扱われる主コピーに対してキーボードユーザーだけがアクセスできて
//! しまう不整合を防ぐ、Cursor Bugbot 指摘・PR #864）。`inert` を `root`
//! 自身へ付与しないのは、`inert` が HTML 標準上ヒットテストからも要素を
//! 除外し、`root` に付与すると本モジュールが常時提供する
//! `root:hover`/`root:focus-within` の一時停止 CSS（下記「常時一時停止」
//! 節）が decorative モードで機能しなくなるため（Cursor Bugbot 指摘・
//! PR #864、追補）。`false`（既定）の場合、[`MarqueeProps::label`] が `Some` なら
//! `root` へ `aria-label` を付与する（[`crate::icon`] の `label: Option<&str>`
//! と同型の判断）。呼び出し側 `attrs` の `aria-hidden`/`aria-label` は
//! 大文字小文字を無視して除去し props 由来の値へ一本化する
//! （[`crate::skeleton::skeleton`] の fail-closed 判断と同型）。
//!
//! # 常時一時停止（WCAG 2.2.2 配慮の固定挙動）
//!
//! `root` への `:hover`/`:focus-within` でアニメーションを一時停止する CSS を
//! 常時出力し、無効化するオプションは設けない。[`crate::recipe::StateCondition`]
//! はコンポーネント自身の slot への状態条件のみを表現でき、`root:hover
//! content` のような子孫コンビネータを持たないため（[`crate::recipe::SlotRecipe`]
//! は子孫セレクタ機構を持たない設計、`.claude/rules` 準拠のドキュメントを
//! 参照）、[`css`] が [`crate::skeleton::css`]/[`crate::spinner::css`] と
//! 同型の静的リテラル追記としてこの規則を出力する。
//!
//! # `prefers-reduced-motion: reduce` 対応
//!
//! [`css`] は `@media (prefers-reduced-motion: reduce)` 環境で `content` の
//! アニメーションを停止する規則を追記する（[`crate::skeleton::css`] と
//! 同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 縦方向スクロール（ark `side: top/bottom` 相当）。
//! - 両端フェードの `Edge` パーツ（呼び出し側 CSS での代替を前提とする、
//!   上記「anatomy」節参照）。
//! - `autoFill` の自動複製制御（`item` の複製数は呼び出し側の責務）。
//! - `examples/headless-pre-styled-ui` への反映（crates.io 公開後の別イシュー、
//!   [`crate::stat`]/[`crate::timeline`] と同じ判断）。
//! - `docs/policy/intentional-non-adoption.md` §3.24 のもう 1 項目
//!   chakra `Theme` コンポーネント（**非採用のまま変更しない**）。
//!
//! # イシュー #1582: アニメーションのスタイル調整（親 #1581 の 1/2）
//!
//! 親イシュー #1581（marquee のスタイルを参考サイト基準へ調整）の
//! 担当分割 1/2。担当範囲は速度・方向・`gap`・両端フェードの
//! アニメーション関連のみとし、姉妹イシュー #1583（2/2、
//! `prefers-reduced-motion` 対応・`item` の見た目・`root` の
//! padding/border/background）が担当するパートには一切触れていない
//! （同一ファイルの並行編集のため、上記「常時一時停止」節・
//! `@media (prefers-reduced-motion: reduce)` ブロックは変更しない。
//! これらは下記「イシュー #1583」節が引き継いで変更する）。
//!
//! ## 是正内容
//!
//! - `gap` のフォールバックを `1rem` から `var(--fandhe-space-4)`
//!   （chakra-ui `Marquee` の `spacing` 既定値 `1rem` と同値。テーマ
//!   トークン経由に揃えることでダークモード等の一括調整に追随する）
//!   へ変更した。`root`/`content`/`@keyframes` の 3 箇所すべてで同じ
//!   フォールバック式に揃える必要がある（不一致だとシームレスループの
//!   継ぎ目に隙間・重なりが出る）。
//! - 両端フェード（ark-ui/chakra-ui の `Edge` パーツ相当）を `root` への
//!   `mask-image`（`--fandhe-marquee-fade` の CSS custom property 契約、
//!   既定 `0px` = 無効）として **opt-in** で追加した。既存の「`Edge` は
//!   呼び出し側 CSS で代替」という判断自体は変更しないが、本イシューで
//!   代替手段の 1 つとして `mask-image` を部品側にも用意する。既定値が
//!   `0px` のため呼び出し側が明示的に上書きしない限り既存の見た目は
//!   変わらない。`mask-image` は `root` 全体へ掛かるため、`root` に
//!   枠線・背景（2/2 が追加する可能性がある）を持たせる場合はそれらの
//!   端もフェードする点に注意（境界の責務が 2/2 側に波及する）。
//!   `-webkit-mask-image` は追加しない（[`crate::css::is_valid_property`]
//!   が先頭小文字英字または `--` で始まる名前のみ許容し `-webkit-` 接頭辞は
//!   拒否するため、`decl()` に渡しても宣言が黙って落ちる。unprefixed
//!   `mask-image` は現行ブラウザで baseline サポート済みであり、
//!   未対応環境ではフェードなしへ graceful degradation する）。
//!
//! ## 意図的に合わせなかった点
//!
//! - ark-ui/chakra-ui の `speed`（px/s 指定）は要素幅に依存し純 CSS
//!   （custom property のみ）では表現できないため、本モジュールは
//!   従来どおり `--fandhe-marquee-duration`（秒指定）の contract を
//!   維持する。
//! - `delay`/`loopCount`/`autoFill`/RTL 対応（`dir="rtl"` 環境で
//!   `translateX(-100%)` 方向のシームレスループが崩れる）は本イシューの
//!   スコープ外（上記「本イシューのスコープ外」節を参照。RTL 対応は
//!   別イシュー化を提案する）。
//! - `size`・`color-palette` 軸・状態（`data-*`）・フォーカスリング・
//!   トランジションはいずれも本部品が元々持たない軸であり、参照サイトの
//!   対応要素も装飾部品としての性質上該当なしと判断し、新設しない。
//!
//! ## `--fandhe-marquee-*` custom property 一覧（本イシュール時点）
//!
//! - `--fandhe-marquee-duration`（既定 `20s`）: 変更なし。
//! - `--fandhe-marquee-direction`（`normal`/`reverse`）: 変更なし。
//! - `--fandhe-marquee-gap`（既定 `var(--fandhe-space-4)`）: 本イシューで
//!   フォールバックをテーマトークン経由へ変更。
//! - `--fandhe-marquee-fade`（既定 `0px`）: 本イシューで新設。両端の
//!   フェード幅（`mask-image` の `linear-gradient` 停止位置）を指定する。
//!
//! # イシュー #1583: reduced-motion 対応とコンテンツ枠（親 #1581 の 2/2）
//!
//! 親イシュー #1581 の担当分割 2/2。担当範囲は `prefers-reduced-motion`
//! 対応・`item` の見た目・`root` の padding/border/background のみとし、
//! 姉妹イシュー #1582（1/2、速度・方向・`gap`・両端フェード）が変更した
//! 箇所には触れていない。
//!
//! ## 是正内容
//!
//! - `prefers-reduced-motion: reduce` 時の挙動を「アニメーション停止のみ」
//!   から「停止 + `root` の横スクロール化 + フェード無効化 + 複製非表示」
//!   へ拡張した（[`css`] 3. 項参照）。従来のまま `overflow: hidden` を
//!   維持して停止するだけでは、ビューポート幅を超える内容が切れて
//!   読めなくなる問題があった（WCAG 2.2.2 の趣旨「止めた上で内容へ到達
//!   可能」）。横スクロール化は折り返しではなくテロップの横並び意味論を
//!   保ったまま最小のレイアウト変更で成立させる選択（`scroll_area.rs` の
//!   viewport と同じトークン組で `scrollbar-width`/`scrollbar-color` を
//!   与える）。
//! - `root` に opt-in の枠（`padding-block`/`background`/`border`/
//!   `border-radius`、いずれも custom property 未指定時は中立値で
//!   既存の見た目を変えない）を追加した。`padding-inline`
//!   は付けない（`overflow: hidden` のクリップは padding box 境界のため、
//!   横方向の padding は横マーキーでは視覚上無意味）。`border` は
//!   `var(--fandhe-marquee-border, none)` のように値全体を custom property
//!   にする方式を採る（`1px solid transparent` 方式だと未使用時も高さが
//!   増える）。`color: var(--fandhe-color-fg)` を付与し、ダーク時の
//!   可読性をトークン経由で担保する（[`crate::card`] と同型）。
//! - `item` を `inline-flex` + `align-items: center` + 内部 `gap` +
//!   `white-space: nowrap` にした（chakra-ui `Marquee.Item` の
//!   アイコン・テキスト横並び用法に倣う）。既定 `padding` は付けない
//!   （`content` の `gap` と二重になるため）。
//!
//! ## 意図的に合わせなかった点（7 軸チェックリスト消化結果）
//!
//! - サイズ: 参照サイトに size スケールなし → N/A。
//! - バリアント: 参照サイトに variant なし → N/A（枠は custom property
//!   opt-in のみ提供）。
//! - `data-*` 状態・hover/disabled/トランジション: フォーカス可能要素は
//!   `root`（下記参照）のみで `data-*` 状態・disabled・トランジションを
//!   持たない静的部品のため N/A（`:hover`/`:focus-within` の一時停止のみ、
//!   1/2 から不変）。
//! - フォーカス: `decorative: false`（既定）の `root` へ `tabindex="0"` と
//!   `:focus-visible` のフォーカスリング（`crate::scroll_area` の
//!   `viewport` と同型）を固定付与する（PR #1856 codex-review P1 是正、
//!   下記「既知の制限」節を参照）。`decorative: true` の場合は `root`
//!   自身が `aria-hidden` になるため付与しない（後述）。
//!
//! ## 既知の制限
//!
//! - **フェードと枠の併用制限**: `mask-image`（イシュー #1582）は `root`
//!   全体へ掛かるため、opt-in の `background`/`border` と併用すると枠の
//!   左右端もフェードする。ark anatomy 準拠の `viewport` パーツ新設で
//!   解決可能だが、anatomy 変更（`SLOTS`・DOM 構造・`wrap_state.rs` への
//!   波及、破壊的変更）を伴うため本イシューでは採らない。後続イシュー化を
//!   提案する（`.claude/rules/out-of-scope-tracking.md`）。
//! - **reduced-motion 時のキーボードスクロール到達性**: PR #1856
//!   codex-review P1 指摘により是正済み。`decorative: false`（既定）の
//!   `root` は `tabindex="0"` を常時（reduced-motion 判定に関わらず）
//!   固定付与する。CSS メディアクエリの真偽に応じて HTML 属性を静的生成側で
//!   出し分ける手段がないための選択であり、`prefers-reduced-motion` が
//!   `no-preference` の環境でも `root` はフォーカス可能になる（副次的に
//!   既存の `:focus-within` によるアニメーション一時停止もキーボードから
//!   到達可能になる、WCAG 2.2.2）。`decorative: true` の場合は `root`
//!   自身に `aria-hidden` を付与するため `tabindex` を付与しない
//!   （aria-hidden な要素自身をフォーカス可能にする既知のアンチパターンを
//!   避ける。装飾用途はそもそも支援技術・キーボード操作の対象外）。
//!
//! ## `--fandhe-marquee-*` custom property 一覧への追加（本イシュー）
//!
//! - `--fandhe-marquee-padding-y`（既定 `0`。opt-in で
//!   `var(--fandhe-space-2)` 等を指定する想定）: `root` の上下 padding。
//! - `--fandhe-marquee-bg`（既定 `transparent`）: `root` の背景。
//! - `--fandhe-marquee-border`（既定 `none`）: `root` の枠線（値全体を
//!   custom property 化）。
//! - `--fandhe-marquee-radius`（既定 `0`）: `root` の角丸。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは新規 anatomy 定義と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラル
//! であり、動的値（children・呼び出し側 `attrs`・`label`）を CSS 値として
//! 流し込む経路を持たない（動的値は `fandhe_frontend_core::render` の
//! 既定エスケープを必ず経由する、REQ-1）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, Anatomy};

/// `data-scope="marquee"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("marquee");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "content", "item"];

/// スクロールアニメーションの `@keyframes` 名リテラル。[`crate::skeleton`]
/// の `pulse_keyframes_name_lit!` と同じ理由（`decl()` の値検証は
/// `{`/`}`/`;` を拒否するため、キーフレーム本体は宣言として表現できず、
/// `animation` 宣言の値とキーフレームブロック名の単一情報源をマクロとして
/// 持つ必要がある）で同型のマクロを用意する。
macro_rules! scroll_keyframes_name_lit {
    () => {
        "fd-marquee-scroll"
    };
}

/// スクロールアニメーションの `@keyframes` 名。`recipe()` の `animation`
/// 宣言（値としてのみ参照）と [`css`] が追記する `@keyframes` ブロックの
/// 両方で共有する識別子（[`scroll_keyframes_name_lit`] を単一情報源として
/// 生成）。
const SCROLL_KEYFRAMES_NAME: &str = scroll_keyframes_name_lit!();

/// Marquee のスクロール方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeDirection {
    /// 通常方向（既定）。
    #[default]
    Start,
    /// 逆方向スクロール。
    End,
}

impl VariantValue for MarqueeDirection {
    fn axis(self) -> &'static str {
        "direction"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// [`marquee`] の設定。
///
/// `Default` は各フィールドの `Default`（`direction: Start`・
/// `decorative: false`・`label: None`）から自動導出する。
#[derive(Debug, Clone, Copy, Default)]
pub struct MarqueeProps<'a> {
    /// スクロール方向（既定 `Start`）。
    pub direction: MarqueeDirection,
    /// `true` なら装飾扱いとし `root` へ `aria-hidden="true"` を付与する
    /// （既定 `false`）。
    pub decorative: bool,
    /// `decorative` が `false` のときのみ有効なアクセシブルネーム。
    /// `Some` なら `root` へ `aria-label` を付与する（既定 `None`）。
    pub label: Option<&'a str>,
}

/// Marquee の recipe（scope `"marquee"`、[`SLOTS`] の 3 パーツ）。
///
/// `direction` 軸は `root` へのみ登録し、`content` への伝搬は root スコープ
/// custom property の継承で行う（モジュール doc「variant」節参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("marquee", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("overflow", "hidden"),
                decl("gap", "var(--fandhe-marquee-gap, var(--fandhe-space-4))"),
                // イシュー #1583: root の枠（padding-block・色・背景・
                // border・radius）。padding-inline は付けない（`overflow:
                // hidden` のクリップは padding box 境界のため、横方向の
                // padding は横マーキーでは視覚上無意味、モジュール doc
                // 「イシュー #1583」節参照）。
                decl("position", "relative"),
                decl("box-sizing", "border-box"),
                decl(
                    "padding-block",
                    "var(--fandhe-marquee-padding-y, 0)",
                ),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "var(--fandhe-marquee-bg, transparent)"),
                decl("border", "var(--fandhe-marquee-border, none)"),
                decl("border-radius", "var(--fandhe-marquee-radius, 0)"),
                // イシュー #1582: 両端フェード（ark-ui/chakra-ui の `Edge`
                // パーツ相当）を opt-in で提供する。既定 `0px` のため
                // 呼び出し側が明示的に上書きしない限り既存の見た目は
                // 変わらない（モジュール doc「イシュー #1582」節参照）。
                decl(
                    "mask-image",
                    "linear-gradient(to right, transparent, black var(--fandhe-marquee-fade, 0px), black calc(100% - var(--fandhe-marquee-fade, 0px)), transparent)",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex", "none"),
                decl("align-items", "center"),
                decl("min-width", "max-content"),
                decl("gap", "var(--fandhe-marquee-gap, var(--fandhe-space-4))"),
                decl(
                    "animation",
                    concat!(
                        scroll_keyframes_name_lit!(),
                        " var(--fandhe-marquee-duration, 20s) linear infinite"
                    ),
                ),
                decl(
                    "animation-direction",
                    "var(--fandhe-marquee-direction, normal)",
                ),
            ],
        )
        .base(
            "item",
            vec![
                decl("flex", "none"),
                // イシュー #1583: item をインラインフレックス化し、
                // 内部コンテンツ（アイコン・テキスト等）を中央揃え・
                // 折り返しなしで並べる（chakra-ui `Marquee.Item` の
                // 用法に倣う、モジュール doc「イシュー #1583」節参照）。
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("white-space", "nowrap"),
            ],
        )
        .variant(
            MarqueeDirection::Start,
            "root",
            vec![decl("--fandhe-marquee-direction", "normal")],
        )
        .variant(
            MarqueeDirection::End,
            "root",
            vec![decl("--fandhe-marquee-direction", "reverse")],
        )
        .default_variant(MarqueeDirection::Start)
        // PR #1856 codex-review P1 是正: `prefers-reduced-motion: reduce`
        // 時に横スクロール化する `root`（[`css`] 3. 項）への唯一の到達手段が
        // マウス操作のみだった（`tabindex` 未付与のため、`marquee` 関数側で
        // 付与する `tabindex="0"` を対象にキーボード操作時のみのフォーカス
        // リングを付ける、`crate::scroll_area` の `viewport` と同型の判断）。
        .state(
            "root",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "-2px"),
            ],
        )
}

/// Marquee の静的 CSS 全文。
///
/// recipe が生成する規則群に続けて、以下を静的リテラルとして追記する
/// （[`crate::skeleton::css`] と同型。値はソースコード中のリテラルのみで
/// 構成され、外部入力は一切混入しない）:
///
/// 1. `animation` 宣言が参照する `@keyframes`（[`SCROLL_KEYFRAMES_NAME`]）。
/// 2. `root` への `:hover`/`:focus-within` で `content` のアニメーションを
///    一時停止する規則（子孫コンビネータのため recipe では表現できない、
///    モジュール doc「常時一時停止」節参照）。
/// 3. `prefers-reduced-motion: reduce` 環境で `content` のアニメーションを
///    停止し、`root` を横スクロール可能にした上でフェードを無効化する
///    `@media` ブロック（受け入れ条件、イシュー #1583 で拡張）。`root` へ
///    `overflow-x: auto` を付与するのは、単純な停止だけではビューポート幅を
///    超える内容が `overflow: hidden` のまま切れて読めなくなるため（WCAG
///    2.2.2 の趣旨「止めた上で内容へ到達可能」、モジュール doc「イシュー
///    #1583」節参照）。同時に `mask-image: none` で両端フェード
///    （イシュー #1582）を無効化する。静止・スクロール可能な内容に対して
///    フェードが残ると末尾の item とスクロールバー端が視覚的に隠れるため。
///    加えて、シームレスループ用に複製した 2 個目の `content`
///    （`aria-hidden="true"`）へ `display: none` も追加する。アニメーション
///    停止のみでは複製 2 本がそのまま横スクロール領域に残り、スクロール幅が
///    2 倍になってしまう不具合（Cursor Bugbot 指摘、PR #864 の指摘を
///    横スクロール化後も踏襲）への是正。
///
/// `root` の両端フェード（`mask-image`、イシュー #1582）・枠
/// （`padding-block`/`background`/`border`/`border-radius`、イシュー
/// #1583）は recipe の `base("root", ...)` 側の宣言として出力されるため、
/// 本関数が追記する静的リテラルには含まれない（モジュール doc
/// 「イシュー #1582」「イシュー #1583」節参照）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {SCROLL_KEYFRAMES_NAME} {{\n  from {{\n    transform: translateX(0);\n  }}\n  to {{\n    transform: translateX(calc(-100% - var(--fandhe-marquee-gap, var(--fandhe-space-4))));\n  }}\n}}\n"
    ));
    out.push_str(
        "\n[data-scope=\"marquee\"][data-part=\"root\"]:hover [data-part=\"content\"],\n[data-scope=\"marquee\"][data-part=\"root\"]:focus-within [data-part=\"content\"] {\n  animation-play-state: paused;\n}\n",
    );
    out.push_str(
        "\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"marquee\"][data-part=\"root\"] {\n    overflow-x: auto;\n    scrollbar-width: thin;\n    scrollbar-color: var(--fandhe-color-border) transparent;\n    mask-image: none;\n  }\n\n  [data-scope=\"marquee\"][data-part=\"content\"] {\n    animation: none;\n  }\n\n  [data-scope=\"marquee\"][data-part=\"content\"][aria-hidden=\"true\"] {\n    display: none;\n  }\n}\n",
    );
    out
}

/// Marquee 1 個を組み立てる（`<div>`。`content` パーツを内部で 2 回複製し
/// シームレスループを実現する、モジュール doc「シームレスループの実現方法」
/// 節参照）。
///
/// `class` は [`crate::class_attr::drop_class_attr`] により常に単一化される
/// （呼び出し側由来のクラスは recipe 生成クラスへ合成されず破棄する、
/// [`crate::skeleton::skeleton`] と同じ方針）。`aria-hidden`/`aria-label` も
/// 同様に呼び出し側の値（大文字小文字を無視）を除去し、`props` 由来の値へ
/// 一本化する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::marquee::{item, marquee, MarqueeProps};
///
/// let node = marquee(
///     &MarqueeProps {
///         decorative: true,
///         ..MarqueeProps::default()
///     },
///     vec![],
///     vec![item(vec![], vec![text("Breaking news")])],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn marquee<'a>(
    props: &MarqueeProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("direction", props.direction.value())]);
    // `aria-hidden`/`aria-label`/`tabindex` は呼び出し側の偽装を大文字小文字
    // 無視で除去し、`props`・本関数由来の値へ一本化する
    // （`crate::skeleton::skeleton` と同型の fail-closed 判断）。`tabindex`
    // は PR #1856 是正で本関数が固定付与するようになったため、呼び出し側の
    // 重複指定（属性の二重出力）を防ぐ目的で追加した。
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| {
            !k.eq_ignore_ascii_case("aria-hidden")
                && !k.eq_ignore_ascii_case("aria-label")
                && !k.eq_ignore_ascii_case("tabindex")
        })
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    // PR #1856 codex-review P1 是正: `prefers-reduced-motion: reduce` 時に
    // `root` が横スクロール領域になる（[`css`] 3. 項）が、`tabindex` が
    // 無いとキーボード操作のみのユーザーがブラウザ既定の挙動に依存せずには
    // その領域へフォーカスできず、画面外の内容へ到達できなかった
    // （モジュール doc「イシュー #1583」節「既知の制限」を撤回・是正）。
    // `crate::scroll_area` の `viewport` と同じく `tabindex="0"` を固定
    // 付与する（reduced-motion でない環境でも常に付与する。CSS メディア
    // クエリの真偽に応じて HTML 属性を出し分ける手段はサーバー側静的
    // 生成では持てないため、常時フォーカス可能にすることで両条件を
    // カバーする。副次的に、既存の `:focus-within` によるアニメーション
    // 一時停止（モジュール doc「常時一時停止」節、WCAG 2.2.2）も
    // マウスに加えキーボードから到達可能になる）。
    //
    // `decorative: true` の場合は付与しない: `root` 自身に `aria-hidden`
    // を付与するモードであり（直下の分岐）、`aria-hidden` な要素自身を
    // フォーカス可能にすると支援技術がフォーカス位置を見失う既知の
    // アンチパターン（axe-core `aria-hidden-focus` 相当）になるため。
    // 装飾用途はそもそも支援技術・キーボード操作の対象から除外される
    // 設計であり、フォーカス到達性は不要（モジュール doc「シームレス
    // ループ」節の `inert` 方針と同じ判断軸）。
    if !props.decorative {
        merged.push(("tabindex", "0"));
    }
    if props.decorative {
        // `aria-hidden` を root へ付与すると ARIA の仕様上サブツリー全体
        // （可視の主コピーを含む）が支援技術からは存在しないものとして
        // 扱われる。ただし `aria-hidden` 単独ではキーボードフォーカスを
        // 遮断しないため、`root` 自身ではなく可視の主コピー
        // （`content_visible`、直下）へ `inert` を付与し支援技術・
        // キーボード操作の双方から除外する。`root` 自身には `inert` を
        // 付与しない: `inert` は HTML 標準上ヒットテスト（マウスの当たり
        // 判定）からも要素を除外するため、`root` へ付与すると本モジュールが
        // 常時提供する `root:hover`/`root:focus-within` の一時停止 CSS
        // （下記「常時一時停止」節）が decorative モードで機能しなくなって
        // しまう（Cursor Bugbot 指摘・PR #864、`decorative: true` でも
        // WCAG 2.2.2 のホバー一時停止契約を維持する）。
        merged.push(aria_hidden(true));
    } else if let Some(label) = props.label {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);

    // シームレスループ: content を 2 回並べる。2 個目は常に aria-hidden で
    // スクリーンリーダーの二重読み上げを防ぐ（呼び出し側が外すオプションは
    // 設けない）。加えて `inert` を付与し、複製内部にリンク等フォーカス
    // 可能な子孫を含む場合でもタブ移動対象から除外する（`aria-hidden` は
    // 支援技術向けの意味論のみでキーボードフォーカスを遮断しないため、
    // 単独では複製内の子孫がタブ順序に残ってしまう不具合があった、
    // Cursor Bugbot 指摘・PR #864）。
    //
    // `decorative: true` の場合はさらに可視の主コピー（`content_visible`）
    // にも `inert` を付与する（上記コメント参照。`root` ではなく `content`
    // 側へ付与することで `root` の hit-testing・`:hover`/`:focus-within`
    // を温存する）。
    let visible_attrs: Vec<(&str, &str)> = if props.decorative {
        vec![("inert", "")]
    } else {
        vec![]
    };
    let content_visible = ANATOMY.part("content", "div", visible_attrs, children.clone());
    let content_hidden = ANATOMY.part(
        "content",
        "div",
        vec![aria_hidden(true), ("inert", "")],
        children,
    );

    ANATOMY.part("root", "div", merged, vec![content_visible, content_hidden])
}

/// item パーツ（`<div>`）を組み立てる。呼び出し側が [`marquee`] の
/// `children` として並べる個々の要素の入れ物（モジュール doc「anatomy」
/// 節参照）。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_start_direction_and_duplicated_content() {
        let node = marquee(&MarqueeProps::default(), vec![], vec![text("news")]);
        let html = render(&node);
        assert!(html.contains(r#"data-scope="marquee" data-part="root""#));
        assert!(html.contains("fd-marquee--direction-start"));
        assert_eq!(html.matches(r#"data-part="content""#).count(), 2);
        // 2 個目の content のみ aria-hidden="true" を持つ。
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 1);
        assert_eq!(html.matches(">news<").count(), 2);
    }

    #[test]
    fn direction_enumeration_maps_to_expected_classes() {
        for (direction, class) in [
            (MarqueeDirection::Start, "fd-marquee--direction-start"),
            (MarqueeDirection::End, "fd-marquee--direction-end"),
        ] {
            let props = MarqueeProps {
                direction,
                ..MarqueeProps::default()
            };
            let html = render(&marquee(&props, vec![], vec![]));
            assert!(html.contains(class), "direction={direction:?} -> {html}");
        }
    }

    #[test]
    fn decorative_true_sets_root_aria_hidden() {
        let props = MarqueeProps {
            decorative: true,
            ..MarqueeProps::default()
        };
        let html = render(&marquee(&props, vec![], vec![]));
        // root と 2 個目の content の 2 箇所で aria-hidden="true" が出現する。
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
        assert!(!html.contains("aria-label"));
        // 可視の主コピー（1 個目 content）・2 個目 content の 2 箇所で
        // inert="" が出現する（root 自身には付与しない。root:hover による
        // 一時停止 CSS を decorative モードでも機能させるため、Cursor
        // Bugbot 指摘・PR #864 追補）。
        assert_eq!(html.matches(r#"inert="""#).count(), 2);
        let root_open = html
            .split_once('>')
            .map(|(head, _)| head)
            .expect("root の開始タグが存在する");
        assert!(
            !root_open.contains("inert"),
            "root タグに inert が付与されている（root:hover の一時停止が機能しなくなる）: {html}"
        );
    }

    #[test]
    fn label_some_sets_root_aria_label_when_not_decorative() {
        let props = MarqueeProps {
            label: Some("Breaking news"),
            ..MarqueeProps::default()
        };
        let html = render(&marquee(&props, vec![], vec![]));
        assert!(html.contains(r#"aria-label="Breaking news""#));
        // root 自体は aria-hidden を持たない（2 個目 content 分の 1 回のみ）。
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 1);
        // 非 decorative の root はキーボードフォーカスも遮断しない
        // （inert は 2 個目 content 分の 1 回のみ）。
        assert_eq!(html.matches(r#"inert="""#).count(), 1);
    }

    #[test]
    fn decorative_true_takes_precedence_over_label() {
        let props = MarqueeProps {
            decorative: true,
            label: Some("ignored"),
            ..MarqueeProps::default()
        };
        let html = render(&marquee(&props, vec![], vec![]));
        assert!(!html.contains("aria-label"));
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
        // decorative=true が label より優先され、root にも inert が付与される。
        assert_eq!(html.matches(r#"inert="""#).count(), 2);
    }

    #[test]
    fn item_uses_div_and_expected_data_part() {
        let html = render(&item(vec![], vec![text("a")]));
        assert!(html.starts_with(r#"<div data-scope="marquee" data-part="item""#));
        assert!(html.contains(">a<"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&marquee(
            &MarqueeProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_supplied_aria_hidden_and_aria_label_are_dropped_case_insensitively() {
        for key in ["aria-hidden", "Aria-Hidden", "ARIA-HIDDEN"] {
            let html = render(&marquee(
                &MarqueeProps::default(),
                vec![(key, "false")],
                vec![],
            ));
            // root 自体は aria-hidden を持たない契約（decorative=false・
            // label=None）。呼び出し側偽装が root へ漏れ出ないことを固定する。
            assert!(!html.contains(r#"aria-hidden="false""#), "html={html}");
        }
        for key in ["aria-label", "Aria-Label", "ARIA-LABEL"] {
            let html = render(&marquee(
                &MarqueeProps::default(),
                vec![(key, "attacker")],
                vec![],
            ));
            assert!(!html.contains("attacker"), "html={html}");
        }
    }

    /// 受け入れ条件（PR #1856 codex-review P1 是正）: `decorative: false`
    /// （既定）の `root` は `reduced-motion` の判定に関わらず常に
    /// `tabindex="0"` を持つ（キーボード操作のみのユーザーが
    /// `prefers-reduced-motion: reduce` 時の横スクロール領域へブラウザの
    /// 既定挙動に依存せず到達できることの固定）。
    #[test]
    fn non_decorative_root_has_tabindex_zero_for_keyboard_reduced_motion_access() {
        let html = render(&marquee(&MarqueeProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="marquee" data-part="root""#));
        assert!(
            html.contains(r#"tabindex="0""#),
            "非 decorative の root はキーボード到達性のため tabindex=\"0\" を持つ必要がある: html={html}"
        );
    }

    /// 受け入れ条件（PR #1856 codex-review P1 是正）: `decorative: true` は
    /// `root` 自身が `aria-hidden` になるため `tabindex` を付与しない
    /// （aria-hidden な要素自身をフォーカス可能にするアンチパターンの回避）。
    #[test]
    fn decorative_root_has_no_tabindex() {
        let props = MarqueeProps {
            decorative: true,
            ..MarqueeProps::default()
        };
        let html = render(&marquee(&props, vec![], vec![]));
        assert!(!html.contains("tabindex"), "html={html}");
    }

    /// 受け入れ条件（PR #1856 codex-review P1 是正）: 呼び出し側が偽装した
    /// `tabindex` は大文字小文字を無視して除去され、本関数が固定付与する
    /// 値のみが出力される（属性の二重出力を防ぐ）。
    #[test]
    fn caller_supplied_tabindex_is_dropped_case_insensitively() {
        for key in ["tabindex", "Tabindex", "TABINDEX"] {
            let html = render(&marquee(
                &MarqueeProps::default(),
                vec![(key, "-1")],
                vec![],
            ));
            assert_eq!(
                html.matches("tabindex").count(),
                1,
                "tabindex は 1 回のみ出力されるべき: html={html}"
            );
            assert!(!html.contains(r#"tabindex="-1""#), "html={html}");
            assert!(html.contains(r#"tabindex="0""#), "html={html}");
        }
    }

    /// 受け入れ条件（PR #1856 codex-review P1 是正）: `root` の
    /// `:focus-visible` にフォーカスリング宣言（`crate::scroll_area` の
    /// `viewport` と同型）が生成される。
    #[test]
    fn css_output_declares_root_focus_visible_ring() {
        let out = css();
        assert!(
            out.contains("[data-scope=\"marquee\"][data-part=\"root\"]:focus-visible"),
            "css={out}"
        );
        assert!(out.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn xss_payload_in_label_is_escaped() {
        let props = MarqueeProps {
            label: Some("\"><script>alert(1)</script>"),
            ..MarqueeProps::default()
        };
        let html = render(&marquee(&props, vec![], vec![]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn xss_payload_in_children_text_is_escaped() {
        let html = render(&marquee(
            &MarqueeProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&marquee(
            &MarqueeProps::default(),
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn css_output_is_deterministic_and_non_empty() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="marquee"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_scroll_animation_and_keyframes() {
        let out = css();
        assert!(out.contains(
            "animation: fd-marquee-scroll var(--fandhe-marquee-duration, 20s) linear infinite;"
        ));
        assert!(out.contains("@keyframes fd-marquee-scroll {"));
        assert!(out.contains("transform: translateX(0);"));
        assert!(out.contains(
            "transform: translateX(calc(-100% - var(--fandhe-marquee-gap, var(--fandhe-space-4))));"
        ));
    }

    /// 受け入れ条件（イシュー #1582）: `gap` フォールバックがテーマ
    /// トークン（`--fandhe-space-4`）へ揃っていることを固定する。
    /// `root`/`content`/`@keyframes` の 3 箇所すべてで同一式でなければ
    /// シームレスループの継ぎ目に隙間・重なりが出るため、出現回数まで
    /// 検証する（モジュール doc「イシュー #1582」節参照）。
    #[test]
    fn css_output_gap_fallback_is_theme_token_in_all_three_locations() {
        let out = css();
        let occurrences = out
            .matches("var(--fandhe-marquee-gap, var(--fandhe-space-4))")
            .count();
        assert_eq!(
            occurrences, 3,
            "root/content/@keyframes の 3 箇所で一致するはず: {out}"
        );
        assert!(!out.contains("--fandhe-marquee-gap, 1rem"));
    }

    /// 受け入れ条件（イシュー #1582）: 両端フェード（`Edge` 相当）を
    /// `root` の `mask-image` として opt-in で提供することを固定する。
    /// `decl()` は不正プロパティ名を黙って落とすため、この出力アサーション
    /// のみが `mask-image` が実際に出力されていることを証明する。
    #[test]
    fn css_output_declares_root_mask_image_fade() {
        let out = css();
        assert!(out.contains("mask-image: linear-gradient("));
        assert!(out.contains("--fandhe-marquee-fade, 0px"));
    }

    /// 受け入れ条件（イシュー #1582）: 新設した `mask-image` の値は
    /// `transparent`/`black` キーワードのみで構成され、`#` 始まりの
    /// 色リテラルを持ち込まないことを固定する（本モジュールは
    /// 色トークン軸を持たない中立部品判断、モジュール doc「イシュー
    /// #1582」節参照）。
    #[test]
    fn css_output_mask_image_uses_no_hex_color_literals() {
        let out = css();
        assert!(!out.contains('#'));
    }

    /// 受け入れ条件: `:hover`/`:focus-within` で常時一時停止する CSS を含む
    /// ことを固定する（無効化オプションを設けない契約、モジュール doc
    /// 「常時一時停止」節参照）。
    #[test]
    fn css_output_declares_hover_and_focus_within_pause() {
        let out = css();
        assert!(
            out.contains(r#"[data-scope="marquee"][data-part="root"]:hover [data-part="content"]"#)
        );
        assert!(out.contains(
            r#"[data-scope="marquee"][data-part="root"]:focus-within [data-part="content"]"#
        ));
        assert!(out.contains("animation-play-state: paused;"));
    }

    /// 受け入れ条件: `prefers-reduced-motion: reduce` でアニメーションを
    /// 停止する CSS を含むことを固定する。
    #[test]
    fn css_output_declares_reduced_motion_media_query() {
        let out = css();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(out.contains(r#"[data-scope="marquee"][data-part="content"] {"#));
        assert!(out.contains("animation: none;"));
    }

    #[test]
    fn css_output_declares_direction_custom_properties() {
        let out = css();
        assert!(out.contains("--fandhe-marquee-direction: normal;"));
        assert!(out.contains("--fandhe-marquee-direction: reverse;"));
    }

    /// 受け入れ条件（イシュー #1583）: `prefers-reduced-motion: reduce`
    /// 環境で `root` を横スクロール可能にし、両端フェード（イシュー #1582）
    /// を無効化することを固定する。複製 2 個目の `content` 非表示
    /// （`display: none;`）が引き続き維持されていることも合わせて固定する
    /// （モジュール doc「イシュー #1583」節参照）。
    #[test]
    fn css_output_reduced_motion_makes_root_scrollable_and_disables_fade() {
        let out = css();
        let media_start = out
            .find("@media (prefers-reduced-motion: reduce) {")
            .expect("reduced-motion media query must exist");
        let media = &out[media_start..];
        assert!(media.contains(r#"[data-scope="marquee"][data-part="root"] {"#));
        assert!(media.contains("overflow-x: auto;"));
        assert!(media.contains("scrollbar-width: thin;"));
        assert!(media.contains("scrollbar-color: var(--fandhe-color-border) transparent;"));
        assert!(media.contains("mask-image: none;"));
        assert!(media.contains("display: none;"));
    }

    /// 受け入れ条件（イシュー #1583）: `root` の枠（padding-block・色・
    /// 背景・border・radius）がすべてテーマトークン既定のフォールバック
    /// 付き custom property として出力されることを固定する。
    #[test]
    fn css_output_root_frame_uses_theme_tokens_with_neutral_fallbacks() {
        let out = css();
        assert!(out.contains("padding-block: var(--fandhe-marquee-padding-y, 0);"));
        assert!(out.contains("color: var(--fandhe-color-fg);"));
        assert!(out.contains("background: var(--fandhe-marquee-bg, transparent);"));
        assert!(out.contains("border: var(--fandhe-marquee-border, none);"));
        assert!(out.contains("border-radius: var(--fandhe-marquee-radius, 0);"));
    }

    /// 受け入れ条件（イシュー #1583）: `item` がインラインフレックスで
    /// 折り返しなし・内部 `gap` を持つことを固定する。
    #[test]
    fn css_output_item_is_inline_flex_and_nowrap() {
        let out = css();
        let item_start = out
            .find(r#"[data-scope="marquee"][data-part="item"] {"#)
            .expect("item rule must exist");
        let item_block = &out[item_start..];
        let item_end = item_block.find('}').expect("item rule must be closed");
        let item_block = &item_block[..item_end];
        assert!(item_block.contains("display: inline-flex;"));
        assert!(item_block.contains("align-items: center;"));
        assert!(item_block.contains("gap: var(--fandhe-space-2);"));
        assert!(item_block.contains("white-space: nowrap;"));
    }

    /// 受け入れ条件（イシュー #1583）: 新設した枠・reduced-motion 宣言も
    /// 含め、CSS 全文に `#` 色リテラル・`rgb(` 色リテラルのいずれも
    /// 含まれないことを固定する（本モジュールは色トークン軸を持たない
    /// 中立部品判断、既存 `css_output_mask_image_uses_no_hex_color_literals`
    /// の拡張）。
    #[test]
    fn css_output_uses_no_hex_or_rgb_color_literals() {
        let out = css();
        assert!(!out.contains('#'));
        assert!(!out.contains("rgb("));
    }
}
