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
//! `Edge`（両端フェードのグラデーション）は新規 `data-part` を増やさず、
//! `root` への [`MarqueeEdge`] variant（イシュー #1582）として提供する
//! （下記「variant」節参照）。
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
//! # variant（`direction`・`edge`。いずれも root へのみクラス付与）
//!
//! [`MarqueeDirection`]（`Start`（既定）/`End`）は `root` パーツのみへ
//! クラスを付与し、`content` への伝搬は CSS custom property
//! （`--fandhe-marquee-direction`）の通常の CSS 継承で行う（複合部品の
//! variant 統一方針、[`crate::timeline`] と同型のパターン。かつて
//! `indicator`/`separator` に対してこの伝搬を直接セレクタで表現しようとして
//! 死んだ CSS を生んだ教訓、PR #812 修正コミット 54126cb を踏まえ、本
//! モジュールは最初から custom property 経由で設計する）。
//!
//! [`MarqueeEdge`]（`None`（既定）/`Fade`、イシュー #1582）は `root` へ
//! `mask-image` を宣言する（`None` 側も `mask-image: none` を明示登録し、
//! [`crate::skeleton::SkeletonAnimation::None`] と同型に既定側の規則も
//! golden へ出す）。`Fade` はアルファマスクの `linear-gradient` で両端を
//! 透過させる（ark-ui `Edge` パーツ・chakra `--marquee-edge-color` 相当の
//! 効果を、新規 anatomy パーツを増やさず表現する）。マスクの
//! `black`/`transparent` は alpha を表すキーワードであり、[`crate::theme`]
//! の配色トークン対象（`#rrggbb`/`rgb()` 等の生色）ではないため
//! ダークモード再定義は不要（背景色に依存しない不変条件）。フェード幅は
//! `--fandhe-marquee-edge-size`（既定 `20%`、chakra 既定値に合わせる）で
//! 呼び出し側が上書きする契約。**`-webkit-mask-image` は出力しない**:
//! [`crate::css::is_valid_identifier`] は先頭が ASCII 小文字であることを
//! 要求するため `decl("-webkit-mask-image", ...)` は
//! [`crate::recipe::SlotRecipe::css`]（[`crate::css::serialize_rule`] 経由）
//! で無音のまま破棄される（fail-closed skip）。unprefixed `mask-image` は
//! Safari 15.4+ / Chrome 120+ / Firefox 53+ で利用可能であり、対応範囲外の
//! 古いブラウザではフェードが単に効かないだけで機能自体は壊れないため、
//! prefixed 版の追加出力は行わない。
//!
//! `color-palette`/`size` 軸は提供しない（[`crate::skeleton`]/[`crate::card`]
//! と同型の「中立・装飾部品」判断）。速度・間隔は CSS custom property の
//! フォールバック（`--fandhe-marquee-duration, 20s` /
//! `--fandhe-marquee-gap, var(--fandhe-space-4)`）として与え、呼び出し側が
//! `style` 属性で上書きする契約とする。イシュー #1582 で `content` の
//! `animation` shorthand を longhand へ分解し、chakra recipe 相当の
//! `--fandhe-marquee-delay`（既定 `0s`）・`--fandhe-marquee-loop-count`
//! （既定 `infinite`）を追加公開し、`animation-fill-mode: forwards` を
//! 常時付与する（有限 loop 終了時に最終位置で停止させるため）。ark-ui の
//! `speed`（px/s）相当は呼び出し側 JS によるコンテンツ幅計測が前提で
//! CSS のみでは再現できないため、引き続き `--fandhe-marquee-duration`
//! （秒指定）契約のまま意図的に非整合とする（#1582 で再評価し確定）。
//! `autoFill`/`loopCount`（複製数の自動制御）は props へ持ち込まない
//! （本イシューのスコープ外、下記節参照）。
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
//! - 縦方向スクロール（ark `side: top/bottom` 相当）。#1582 で再評価し、
//!   `translateY` 用の第 2 `@keyframes`・`flex-direction: column`・呼び出し
//!   側による root 高さ固定が必要で #1583（コンテンツ枠）と軸が交差する
//!   ため、引き続き見送る。
//! - `autoFill` の自動複製制御（`item` の複製数は呼び出し側の責務）。
//! - `examples/headless-pre-styled-ui` への反映（crates.io 公開後の別イシュー、
//!   [`crate::stat`]/[`crate::timeline`] と同じ判断）。
//! - `docs/policy/intentional-non-adoption.md` §3.24 のもう 1 項目
//!   chakra `Theme` コンポーネント（**非採用のまま変更しない**）。
//!
//! （両端フェードの `Edge` パーツ相当は #1582 で [`MarqueeEdge`] variant
//! として実装済み。以前は「呼び出し側 CSS での代替を前提に非提供」として
//! いたが、上記「variant」節の設計へ差し替えた）。
//!
//! # スタイル調整（イシュー #1582）
//!
//! 親イシュー #1581 の 7 軸チェックリストのうち、本 issue（1/2、アニメー
//! ション担当）の担当範囲で「意図的に合わせなかった」判断・「軸を持たない」
//! 判断を記録する（親の受け入れ条件対応。枠・padding・背景・reduced-motion
//! は #1583（2/2）の担当のため未着手）。
//!
//! - **サイズ / colorPalette**: 中立・装飾部品のため軸を持たない（既存判断
//!   の再確認、上記「variant」節参照）。
//! - **色**: 新規に配色トークンを参照しない。[`MarqueeEdge::Fade`] の
//!   `black`/`transparent` は alpha を表すキーワードであり配色トークンの
//!   対象ではない。
//! - **状態（`data-*`）**: headless 側の状態属性を持たない部品のまま不変
//!   （`fandhe-frontend-headless-ui` 側に対応 primitive がない
//!   pre-styled-ui 専用部品）。
//! - **ダーク**: alpha マスクのため再定義不要（上記参照）。
//! - **フォーカス**: `root` は非フォーカス要素のまま不変。
//! - **hover**: `:hover` 一時停止は既存契約のまま不変。
//! - **disabled**: 該当なし（無効化状態を持たない部品）。
//! - **トランジション**: `animation-play-state` はアニメーション不可
//!   プロパティのため `transition` を当てない（トランジションで滑らかに
//!   一時停止させることはできない）。
//! - **縦方向スクロール・px/s 速度指定**: 上記「variant」節・「スコープ外」
//!   節のとおり、意図的に非整合のまま維持する。
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
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, Anatomy};

/// `data-scope="marquee"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("marquee");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "content", "item"];

/// スクロールアニメーションの `@keyframes` 名リテラル。[`crate::skeleton`]
/// の `pulse_keyframes_name_lit!` と同じ理由（`decl()` の値検証は
/// `{`/`}`/`;` を拒否するため、キーフレーム本体は宣言として表現できず、
/// `animation-name` 宣言の値とキーフレームブロック名の単一情報源をマクロ
/// として持つ必要がある）で同型のマクロを用意する。
macro_rules! scroll_keyframes_name_lit {
    () => {
        "fd-marquee-scroll"
    };
}

/// スクロールアニメーションの `@keyframes` 名。`recipe()` の
/// `animation-name` 宣言（値としてのみ参照）と [`css`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子（[`scroll_keyframes_name_lit`]
/// を単一情報源として生成）。
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

/// Marquee 両端のフェードマスク variant（イシュー #1582）。
///
/// `root` へ `mask-image` を宣言する（モジュール doc「variant」節参照）。
/// [`crate::skeleton::SkeletonAnimation::None`] と同型に、既定 `None` も
/// `mask-image: none` を明示 variant として登録し、golden に両クラスを
/// 出力する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeEdge {
    /// フェードなし（既定）。
    #[default]
    None,
    /// 両端をアルファマスクで透過させる。
    Fade,
}

impl VariantValue for MarqueeEdge {
    fn axis(self) -> &'static str {
        "edge"
    }

    fn value(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fade => "fade",
        }
    }
}

/// [`marquee`] の設定。
///
/// `Default` は各フィールドの `Default`（`direction: Start`・
/// `edge: None`・`decorative: false`・`label: None`）から自動導出する。
#[derive(Debug, Clone, Copy, Default)]
pub struct MarqueeProps<'a> {
    /// スクロール方向（既定 `Start`）。
    pub direction: MarqueeDirection,
    /// 両端フェードマスク（既定 `None`、イシュー #1582）。
    pub edge: MarqueeEdge,
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
                // `animation` shorthand ではなく longhand へ分解する
                // （イシュー #1582）。chakra recipe 相当の `delay`/
                // `loop-count` を CSS custom property として公開し、
                // `animation-fill-mode: forwards` で有限 loop 終了時に
                // 最終位置で停止させる（モジュール doc「variant」節参照）。
                decl("animation-name", scroll_keyframes_name_lit!()),
                decl("animation-duration", "var(--fandhe-marquee-duration, 20s)"),
                decl("animation-timing-function", "linear"),
                decl(
                    "animation-iteration-count",
                    "var(--fandhe-marquee-loop-count, infinite)",
                ),
                decl("animation-delay", "var(--fandhe-marquee-delay, 0s)"),
                decl("animation-fill-mode", "forwards"),
                decl(
                    "animation-direction",
                    "var(--fandhe-marquee-direction, normal)",
                ),
            ],
        )
        .base("item", vec![decl("flex", "none")])
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
        .variant(MarqueeEdge::None, "root", vec![decl("mask-image", "none")])
        .variant(
            MarqueeEdge::Fade,
            "root",
            vec![decl(
                "mask-image",
                "linear-gradient(to right, transparent, black var(--fandhe-marquee-edge-size, 20%), black calc(100% - var(--fandhe-marquee-edge-size, 20%)), transparent)",
            )],
        )
        .default_variant(MarqueeEdge::None)
}

/// Marquee の静的 CSS 全文。
///
/// recipe が生成する規則群に続けて、以下を静的リテラルとして追記する
/// （[`crate::skeleton::css`] と同型。値はソースコード中のリテラルのみで
/// 構成され、外部入力は一切混入しない）:
///
/// 1. `animation-name` 宣言が参照する `@keyframes`（[`SCROLL_KEYFRAMES_NAME`]）。
/// 2. `root` への `:hover`/`:focus-within` で `content` のアニメーションを
///    一時停止する規則（子孫コンビネータのため recipe では表現できない、
///    モジュール doc「常時一時停止」節参照）。
/// 3. `prefers-reduced-motion: reduce` 環境でアニメーションを停止する
///    `@media` ブロック（受け入れ条件）。同ブロック内でシームレスループ用に
///    複製した 2 個目の `content`（`aria-hidden="true"`）へ `display: none`
///    も追加する。アニメーション停止のみでは複製 2 本がそのまま flex
///    レイアウトへ残り、メッセージがビューポートより狭い場合に視認可能な
///    ユーザーへ内容が二重表示されてしまう不具合（Cursor Bugbot 指摘、
///    PR #864）への是正。
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
    // `animation: none`（shorthand）は上記 recipe が出す全 longhand（イシュー
    // #1582 で追加した animation-delay/-iteration-count/-fill-mode 含む）を
    // 一括でリセットするため、longhand 化後もこのブロックは変更不要（#1583
    // の担当領域であり本 PR では触らない）。
    out.push_str(
        "\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"marquee\"][data-part=\"content\"] {\n    animation: none;\n  }\n\n  [data-scope=\"marquee\"][data-part=\"content\"][aria-hidden=\"true\"] {\n    display: none;\n  }\n}\n",
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
    let class = recipe.variant_classes(&[
        ("direction", props.direction.value()),
        ("edge", props.edge.value()),
    ]);
    // `aria-hidden`/`aria-label` は呼び出し側の偽装を大文字小文字無視で除去し、
    // props 由来の値へ一本化する（`crate::skeleton::skeleton` と同型の
    // fail-closed 判断）。
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| {
            !k.eq_ignore_ascii_case("aria-hidden") && !k.eq_ignore_ascii_case("aria-label")
        })
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
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
        // イシュー #1582: `animation` shorthand を longhand へ分解し、
        // `delay`/`loop-count` の custom property 公開・`fill-mode:
        // forwards` を追加した（モジュール doc「variant」節参照）。
        assert!(out.contains("animation-name: fd-marquee-scroll;"));
        assert!(out.contains("animation-duration: var(--fandhe-marquee-duration, 20s);"));
        assert!(out.contains("animation-timing-function: linear;"));
        assert!(
            out.contains("animation-iteration-count: var(--fandhe-marquee-loop-count, infinite);")
        );
        assert!(out.contains("animation-delay: var(--fandhe-marquee-delay, 0s);"));
        assert!(out.contains("animation-fill-mode: forwards;"));
        assert!(out.contains("@keyframes fd-marquee-scroll {"));
        assert!(out.contains("transform: translateX(0);"));
        assert!(out.contains(
            "transform: translateX(calc(-100% - var(--fandhe-marquee-gap, var(--fandhe-space-4))));"
        ));
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

    /// 受け入れ条件（イシュー #1582）: 既定 `edge: None` は
    /// `fd-marquee--edge-none` クラスを付与し、`fd-marquee--edge-fade` は
    /// 付与しない（[`crate::skeleton::SkeletonAnimation::None`] と同型に
    /// 既定側も明示 variant として golden へ出す設計、モジュール doc
    /// 「variant」節参照）。
    #[test]
    fn default_props_render_edge_none_class() {
        let html = render(&marquee(&MarqueeProps::default(), vec![], vec![]));
        assert!(html.contains("fd-marquee--edge-none"));
        assert!(!html.contains("fd-marquee--edge-fade"));
    }

    #[test]
    fn edge_enumeration_maps_to_expected_classes() {
        for (edge, class) in [
            (MarqueeEdge::None, "fd-marquee--edge-none"),
            (MarqueeEdge::Fade, "fd-marquee--edge-fade"),
        ] {
            let props = MarqueeProps {
                edge,
                ..MarqueeProps::default()
            };
            let html = render(&marquee(&props, vec![], vec![]));
            assert!(html.contains(class), "edge={edge:?} -> {html}");
        }
    }

    /// 受け入れ条件（イシュー #1582）: `mask-image` の gradient は
    /// `fd-marquee--edge-fade` ブロック内にのみ出現し、`--fandhe-marquee-
    /// edge-size` フォールバック `20%` を参照する。`-webkit-mask-image` は
    /// [`crate::css::is_valid_identifier`] により無音で破棄されるため出力に
    /// 含まれない（モジュール doc「variant」節参照）。
    #[test]
    fn css_output_declares_edge_fade_mask_only_under_fade_variant() {
        let out = css();
        let fade_block_start = out
            .find(".fd-marquee--edge-fade {")
            .expect("fade variant のブロックが存在する");
        let fade_block = &out[fade_block_start..];
        let fade_block_end = fade_block.find('}').map_or(fade_block.len(), |i| i + 1);
        let fade_block = &fade_block[..fade_block_end];
        assert!(fade_block.contains("mask-image: linear-gradient("));
        assert!(fade_block.contains("var(--fandhe-marquee-edge-size, 20%)"));
        assert!(!out.contains("-webkit-mask-image"));
        assert!(out.contains(".fd-marquee--edge-none {"));
        assert!(out.contains("mask-image: none;"));
    }

    /// 受け入れ条件（イシュー #1582）: gap のフォールバックが `1rem` の
    /// 生リテラルではなく共通トークン `--fandhe-space-4` を参照する
    /// （root/content の `gap` と `@keyframes` の `translateX` の 3 箇所、
    /// モジュール doc「variant」節参照）。
    #[test]
    fn css_output_uses_space_token_for_gap_fallback() {
        let out = css();
        assert_eq!(
            out.matches("var(--fandhe-marquee-gap, var(--fandhe-space-4))")
                .count(),
            3,
            "root gap・content gap・keyframes translateX の 3 箇所で参照される: {out}"
        );
        assert!(!out.contains("gap, 1rem)"));
    }
}
