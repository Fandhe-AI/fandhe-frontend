//! styled Pagination（headless ラッパー、イシュー #751、親 #520/#546。
//! headless 側の保留解除は #716 → #751）。
//!
//! `fandhe_frontend_headless_ui::pagination`（#751）の Item / Ellipsis /
//! PrevTrigger / NextTrigger anatomy パーツ・[`Pagination`] 状態機械・
//! [`ItemMode`]/[`PageEntry`]/[`PaginationAction`] をそのまま再エクスポート
//! し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・variant
//! 統一方針は [`crate::toggle_group`]/[`crate::radio_group`] の rustdoc と
//! 同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::toggle_group::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`item`]/[`ellipsis`]/[`prev_trigger`]/[`next_trigger`]/
//! [`Pagination`] 等）のみを選択的に再エクスポートする。
//!
//! [`Pagination`] は inherent `root()` を持つが（`crates/headless-ui/src/pagination.rs`
//! 参照）、`crate::lib` rustdoc「複合部品の variant 統一方針」節 4 の
//! 判断（`crate::avatar::Avatar`・`crate::switch::Switch` と同じ理由）
//! により、[`Pagination`] 型自体は再エクスポートしつつ headless 自由関数
//! `root` は再エクスポートしない（未スタイル root の静かな適用漏れを防ぐ
//! fail-closed）。
//!
//! # 複合部品の variant 統一方針（root のみへクラス付与）
//!
//! `size`（[`Size`]）/`palette`（[`ColorPalette`]）はいずれも [`root`] へ
//! のみクラスを付与する。`recipe` が root スコープへ登録する custom
//! property（`--fandhe-pagination-item-size`/`-item-font-size`）は CSS の
//! 通常のプロパティ継承により `item`/`prev-trigger`/`next-trigger` へ伝わる
//! ため、これらの slot へ個別に variant クラスを付ける必要がない
//! （[`crate::toggle_group`]/[`crate::radio_group`] と同じ設計）。
//!
//! # `data-selected`/`aria-current` について
//!
//! headless 層の `item` は `data-state` ではなく `data-selected`（存在
//! マーカー）+ `aria-current="page"` で現在ページを表す
//! （`crates/headless-ui/src/pagination.rs` 参照）。`recipe` の状態規則も
//! この語彙（`StateCondition::Attr("data-selected")`）に合わせる。
//!
//! # フォーカスリング（hidden-input パターン非該当）
//!
//! `item`/`prev-trigger`/`next-trigger` はネイティブ `<button>`/`<a>` 自身が
//! 実フォーカスを直接受けるため、[`crate::toggle_group`] の `item` と同じ
//! [`StateCondition::FocusVisible`] で足りる。`data-focus-visible` 配線は
//! 不要。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`href`/`aria_label`/属性/children）へ CSS 値として流し
//! 込む経路を持たない（動的値は headless 層経由で
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る、REQ-1）。
//! styled `root` は `drop_class_attr` により呼び出し側の `class` を除去
//! してから合成するため、`class` 属性は常に単一。
//!
//! # `item`/`ellipsis` のスタイル是正（イシュー #1532、親 #1531 の 1/2 分割）
//!
//! 参考サイト（chakra-ui / Ark UI）基準へのスタイル是正 #1531 のうち、
//! イシュー #1532 では **項目ボタン `item`（current / hover / focus /
//! disabled の各状態）と省略記号 `ellipsis`** を担当した。前後トリガー
//! （`prev-trigger`/`next-trigger`）と size バリアントは 2/2（#1533）で
//! 是正済み（下記節参照）。
//!
//! - **hover**: `item` base へ [`hover_bg_muted`] を追加し `--fandhe-hover-bg`
//!   を未選択面の色（`--fandhe-color-bg-muted`）へ定義、`data-selected` 規則
//!   へ [`hover_bg_solid_with_fallback`] を追加して current ページ hover 時
//!   のみ emphasized 段へ上書きする。hover の実適用は
//!   `.state("item", StateCondition::Hover, hover_surface_declarations())`
//!   1 本のみで足りる（custom property 間接参照で variant × state の複合
//!   条件を回避する既存パターン、[`crate::toggle`] の `root` と同型）。
//! - **focus**: `item` の直書き `outline` を [`focus_ring_declarations`]
//!   （`FocusRingColor::Palette`、`FocusRingOffset::Outside`）へ置換。
//! - **disabled**: `item` の `[data-disabled]` を [`disabled_declarations`]
//!   へ置換（値は同一、宣言順のみ `opacity` → `cursor` に変わる）。
//! - **transition**: `item` base の shorthand `transition: ... 0.15s` を
//!   [`transition_declarations`]（`MotionDuration::Fast`）へ置換。
//! - **色**: `ellipsis` の `opacity: 0.6` 直書きを廃し
//!   `color: var(--fandhe-color-fg-muted)` トークンへ統一。あわせて
//!   `item` と同じ font-size 継承経路（`--fandhe-pagination-item-font-size`）
//!   に載せる。
//!
//! ## 意図的非対応
//!
//! - 色系 variant（solid/outline 等）は追加しない。`palette` 軸で既に色の
//!   切り替えが可能であり、同型部品（[`crate::toggle_group`]）との一貫性
//!   を優先する。variant 軸自体はレイアウト（下記 attached variant）に
//!   限定する（イシュー #3136 で方針を明確化）。
//! - 影（box-shadow）は追加しない。枠線 + hover 背景色のみで状態表現する
//!   既存方針を維持する（[`crate::toggle`] と同じ判断）。
//!
//! # attached variant（連結表示、イシュー #3136）
//!
//! Blocks 取り込みの対応表（ID R1136）で「既存部品で表現不可」と判定された
//! 「ページボタンを隙間なく横一列に連結する見た目」を
//! [`PaginationVariant::Attached`] として opt-in 追加した。
//!
//! ## 案 A（本モジュールの variant）を採用した理由
//!
//! 代替案（`crate::button_group` の連結対象へ pagination の各パーツを追加
//! する案）は、`button_group` が pagination の `data-scope` を知る部品横断の
//! 結合を増やし、`<nav>` 内に `role="group"` の中間 div を挟む構造変化も
//! 伴う。本案は [`root_with`] 1 箇所で完結し、`<nav>` 直下に item が並ぶ
//! 構造を変えない（[`crate::tab_nav::root_with`] と同型の opt-in
//! パターン）。
//!
//! ## 連結 CSS の方式（負マージン + z-index、`button_group` とは別方式）
//!
//! `button_group` は「先頭以外の開始側 border 幅を 0 にする」方式を採る。
//! 本 variant は代わりに「隣接マージンを -1px で重ねる + 現在ページ
//! （`data-selected`）を `z-index` で前面に出す」方式を選ぶ。理由は、
//! 現在ページの枠線も palette 色で両側とも見せたいため（border 幅 0 方式
//! だと隣の灰色枠しか見えない）。全パーツが枠線を持つ構成のため
//! `border: none` の variant との兼ね合いを考慮する必要もない。
//!
//! [`SlotRecipe`] の variant は root 自身にしかクラスを宣言できないため、
//! 子パーツ（item/ellipsis/prev-trigger/next-trigger）への連結規則は
//! [`crate::button_group::stylesheet`] と同じく [`stylesheet`] が
//! [`crate::css::serialize_rule`] で子結合子セレクタを直接末尾へ追記する。
//! 論理プロパティ（`margin-inline-start`/`border-start-start-radius` 等）を
//! 使うため RTL でも自然に反転する。位置擬似クラス（`:not(:first-child)`/
//! `:not(:last-child)`）は子結合子セレクタの直接の子側に付ける
//! （`button_group` の rustdoc にある既知の罠と同じ注意点）。
//! ellipsis は attached 時のみ枠付きセルにし、連結帯の途切れを防ぐ。
//!
//! # `prev-trigger`/`next-trigger` のスタイル是正（イシュー #1533、親 #1531 の 2/2 分割）
//!
//! #1532 が手つかずのまま残した前後トリガーを、`item` と同じ 7 軸基準へ
//! 是正した（先例: [`crate::carousel`] の `prev-trigger`/`next-trigger`
//! パターン）。
//!
//! - **hover**: `item`（#1532）と同じ未選択面 hover 色。base へ
//!   [`hover_bg_muted`] を追加し `data-disabled` 時を除いて
//!   `.state(..., StateCondition::Hover, hover_surface_declarations())`
//!   1 本を登録する（current ページ相当の状態はトリガーに存在しないため
//!   `item` の emphasized 段上書きは不要）。
//! - **focus**: 直書き `outline: 2px solid var(--fandhe-color-accent)` を
//!   [`focus_ring_declarations`]（`FocusRingColor::Palette`、
//!   `FocusRingOffset::Outside`）へ置換。`Palette` を選ぶ根拠は `item` と
//!   同じ: pagination root は palette 軸クラスで `--fandhe-palette` を
//!   定義するため、部品内のフォーカスリング色を統一する
//!   （[`crate::carousel`] の `Token` は root に palette 軸を持たない点で
//!   前提が異なる）。
//! - **disabled**: 直書き `cursor: not-allowed` + `opacity: 0.5` を
//!   [`disabled_declarations`] へ置換（値は同一、宣言順のみ変わる）。
//! - **transition**: `item`（#1532）と同型の第 2 base ブロックとして
//!   [`transition_declarations`]（`MotionDuration::Fast`）を追加する
//!   （trigger だけ状態遷移が瞬時に切り替わる不整合の解消）。
//! - **余白**: `padding-inline: var(--fandhe-space-2)` を追加する
//!   （docs-site Demo の "Prev"/"Next" テキストラベル運用でテキストが
//!   枠線に接する余白軸の欠落を解消。アイコン運用時は `min-width` により
//!   概ね正方形が保たれるため見た目への影響は小さい）。
//! - **size バリアント**: Xs〜Xl の 5 段（chakra の `xs/sm/md/lg/xl` 相当）
//!   は #1681/#1714 で既に登録済み。トリガー・`ellipsis` への反映は
//!   root スコープの custom property（`--fandhe-pagination-item-size`/
//!   `-item-font-size`）の継承で既に成立しているため、本イシューでは
//!   単体テストのカバレッジ拡充（Xs〜Xl 全段）のみ行い、`recipe()` への
//!   追加変更は行わない。
//!
//! ## 意図的非対応
//!
//! - chakra のようなアイコン専用トリガー形状は強制しない。テキスト
//!   ラベル・アイコンいずれの運用も呼び出し側の裁量とする（headless 層
//!   `prev_trigger`/`next_trigger` は children を固定しない）。
//! - variant 軸（solid/outline 等）は追加しない（`item` と同じ判断）。
//!
//! # shadcn/ui 突合（イシュー #2036）
//!
//! shadcn/ui（Base UI 版 Pagination）のスクリーンショット 3 枚
//! （`docs/design/reference-screenshots/shadcn-pagination-{1,2,3}.png`）と
//! ローカル `themes-pagination.png` を突合し、#1420（chakra-ui / Radix
//! Themes 基準）の調整では拾えなかった欠落バリアント・状態・合成パターンの
//! 有無を確認した。純追加原則（既存 variant の CSS 出力はバイト同一を
//! 維持）に従い、`recipe` の変更は不要と判断した。
//!
//! - **prev/next のアイコン+テキスト表示**: shadcn は「‹ Previous」
//!   「Next ›」の表示を持つ。[`prev_trigger`]/[`next_trigger`] は
//!   children を固定しない設計（上記「意図的非対応」節に既記載）なので、
//!   呼び出し側が単一テキストノード（例:
//!   `text("\u{2039} Previous")`）を渡すだけで再現できる（コード変更不要、
//!   docs サイト Examples「Prev/Next with label」で実演）。複数 children
//!   （アイコン部品 + テキスト）による横並びは意図的に採用せず、
//!   `prev-trigger`/`next-trigger` へ `gap` を追加する recipe 変更を発生
//!   させない。
//! - **ellipsis**: shadcn は枠なし・muted 色。ローカルも #1532 是正済みで
//!   既に枠なし・`--fandhe-color-fg-muted` トークン経由。差分なし。
//! - **size 段階**: shadcn 側スクリーンショットは単一サイズのみ。ローカル
//!   は既に Xs〜Xl の 5 段（chakra 基準）を持ち、追加で欲しい段階はない。
//! - **参照競合の判定**: pagination の current ページ表現は chakra/Radix
//!   軸の値（`data-selected` の solid fill・白文字）を採る。理由:
//!   shadcn はごく薄い outline（枠線のみ、背景はほぼ base と同色）だが、
//!   #1532/#1420 で chakra/Radix 基準として明示的に採用した設計であり、
//!   golden の色味の差だけで書き換える調整は行わない方針（純追加原則）の
//!   対象であるため、意図的に非追随とする。
//! - **合成パターン（データテーブル用フッター）**: shadcn の 3 枚目は
//!   「Rows per page」ラベル + Select + Prev/Next のみ（ページ項目なし）
//!   という構成。[`crate::native_select`](mod@crate::native_select) が既に存在するため、既存部品の
//!   組み合わせで再現可能（コード変更不要、docs サイト Examples
//!   「Rows per page + Select」で実演）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - roving focus / キーボードナビゲーションは headless 層
//!   （`crates/headless-ui/src/pagination.rs`）と同じくスコープ外
//!   （wasm keynav 層の責務）。
//! - `examples/headless-pre-styled-ui` への Pagination 追加は headless-ui
//!   0.8.0 / pre-styled-ui の crates.io 公開後の追随 PR とする（過去例:
//!   #677/#704 の追随コミットと同型）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_bg_solid_with_fallback,
    hover_surface_declarations, palette_scale_declarations, transition_declarations, ColorPalette,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

/// `root` の連結表示 variant（イシュー #3136）。`Separated`（既定、従来の
/// 唯一の見た目）はクラスを付与せず、`Attached` のみ opt-in で `gap: 0` +
/// 子結合子の連結規則（[`stylesheet`] が追記）を適用する（モジュール冒頭
/// rustdoc「attached variant」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationVariant {
    /// 項目間に間隔を持つ従来の見た目（既定）。
    #[default]
    Separated,
    /// 項目を隙間なく横一列に連結する見た目（イシュー #3136）。
    Attached,
}

impl VariantValue for PaginationVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            PaginationVariant::Separated => "separated",
            PaginationVariant::Attached => "attached",
        }
    }
}

/// attached variant（root の variant クラス）が付いたときの子結合子
/// セレクタの対象パーツ（`SLOTS` のうち `root` を除く 4 パーツ）。
const ATTACHED_CHILD_PARTS: &[&str] = &["item", "ellipsis", "prev-trigger", "next-trigger"];

/// `out` へ 1 規則を追記する（空でなければ改行区切り、
/// [`crate::button_group::stylesheet`] の同名ヘルパと同型）。
fn append_rule(out: &mut String, rule: Option<String>) {
    if let Some(rule) = rule {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }
}

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::pagination::{
    ellipsis, item, next_trigger, prev_trigger, ItemMode, PageEntry, Pagination, PaginationAction,
};

/// headless `pagination` anatomy の `data-part` 一覧
/// (`crates/headless-ui/src/pagination.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約)。
const SLOTS: &[&str] = &["root", "item", "ellipsis", "prev-trigger", "next-trigger"];

/// この styled Pagination の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("pagination", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
                // イシュー #1532: 未選択面の hover 色。base 背景
                // `--fandhe-color-bg` より 1 段濃い `--fandhe-color-bg-muted`
                // を `--fandhe-hover-bg` へ定義する（toggle #1785 の off 面と
                // 同型）。current ページ（`data-selected`）は下記 state 規則
                // が同名カスタムプロパティを emphasized 段へ上書きする。
                hover_bg_muted(),
            ],
        )
        .base(
            "item",
            // イシュー #1532: `transition: background 0.15s, ...` の
            // shorthand 直書きを canonical ヘルパへ置換（toggle #1785 等と
            // 同型。150ms で従来と同値、longhand 3 宣言化により easing が
            // トークン化され `prefers-reduced-motion` 対応に載る）。
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        .base(
            "ellipsis",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                // イシュー #1532: muted 表現を `--fandhe-color-fg-muted`
                // トークン経由へ統一（`color` + `opacity: 0.6` の直書きを
                // 廃止。ダーク側はトークン再定義で自動成立し、コントラスト
                // 検査対象トークンのため 4.5:1 が担保される）。
                decl("color", "var(--fandhe-color-fg-muted)"),
                // item と同じ font-size 継承経路に載せる（size 変更時に
                // 省略記号だけ大きさが取り残される不整合を防ぐ）。
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                // イシュー #1533: docs-site Demo の "Prev" テキストラベル
                // 運用でテキストが枠線に接しないよう横 padding を追加する
                // （アイコン運用時は `min-width` により概ね正方形が保たれる
                // ため見た目への影響は小さい）。
                decl("padding-inline", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
                // イシュー #1533: item（#1532）と同じ未選択面 hover 色
                // （`item` に hover が無かった旧実装との不整合を解消）。
                hover_bg_muted(),
            ],
        )
        .base(
            "prev-trigger",
            // イシュー #1533: item は transition 済みなのにトリガーだけ状態
            // 遷移が瞬時に切り替わる不整合を解消する（item と同型の第 2
            // base ブロック）。
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("padding-inline", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "next-trigger",
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        // 現在ページ（`data-selected` 存在マーカー、headless 層
        // `crates/headless-ui/src/pagination.rs` 参照）の見た目。
        .state(
            "item",
            StateCondition::Attr("data-selected"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-palette-fg)"),
                // イシュー #1532: current ページの hover は palette の
                // emphasized 段へ（toggle #1785 の on 面と同型）。
                // `hover_bg_solid_with_fallback` は `--fandhe-palette-
                // emphasized` 未定義時も `--fandhe-color-accent-emphasized`
                // へ確実にフォールバックする。
                hover_bg_solid_with_fallback(),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            // イシュー #1532: `cursor`/`opacity` 直書きを共通ヘルパへ置換
            // （出力順が `opacity` → `cursor` に変わるが値は不変）。
            disabled_declarations(),
        )
        // イシュー #1532: hover の実適用は 1 本のみ（`--fandhe-hover-bg` の
        // 間接参照経由で未選択面・current 面いずれの色にも追従する。toggle
        // #1785 の `root` hover と同型のパターン）。`Hover` は
        // `:not([data-disabled])` 込みで `@media (hover: hover)` へ集約
        // 出力される既存機構。
        .state("item", StateCondition::Hover, hover_surface_declarations())
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            // イシュー #1533: `cursor`/`opacity` 直書きを共通ヘルパへ置換
            // （item・#1532 と同じ置換、出力順のみ変わる）。
            disabled_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1533: item（#1532）と同型の hover 適用
        // （`@media (hover: hover)` + `:not([data-disabled])` へ集約出力）。
        .state(
            "prev-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // item/prev-trigger/next-trigger はネイティブ button/a 自身が実
        // フォーカスを受けるため、hidden-input パターンの
        // data-focus-visible 配線は不要（[`crate::toggle_group`] と同じ判断）。
        .state(
            "item",
            StateCondition::FocusVisible,
            // イシュー #1532: outline 直書きを共通フォーカスリングトークン
            // 経由の canonical ヘルパへ置換（`FocusRingColor::Palette`。
            // pagination は root に palette 軸を持つため toggle_group の
            // item と同じ選定。フォールバック値は旧実装と同一のため新
            // トークン未定義の既存カスタムテーマでも見た目は不変）。
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .state(
            "prev-trigger",
            StateCondition::FocusVisible,
            // イシュー #1533: outline 直書きを共通フォーカスリングトークン
            // 経由の canonical ヘルパへ置換。`FocusRingColor::Palette` を
            // 選ぶ根拠は item（#1532）と同じ: pagination root は palette
            // 軸クラスで `--fandhe-palette` を定義するため、部品内の
            // フォーカスリング色を統一する（carousel の `Token` とは
            // root に palette 軸を持たない点で前提が異なる）。
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .state(
            "next-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #1681: Xs/Xl は item-size の Sm→Md→Lg 等差進行（0.5rem
        // 刻み）を両端へ外挿。font-size は Sm=Md=sm、Lg=md の段差を踏襲し、
        // Xs=xs（1 段下）、Xl=lg（1 段上）とする。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "1rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "1.5rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "2rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "2.5rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "3rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }

    // イシュー #3136: attached variant（root の `gap` のみ root 自身へ
    // 宣言できる。子パーツへの連結規則は [`stylesheet`] が子結合子
    // セレクタで追記する、モジュール冒頭 rustdoc 参照）。
    recipe = recipe.variant(PaginationVariant::Attached, "root", vec![decl("gap", "0")]);

    recipe
}

/// この styled Pagination が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle_group::stylesheet`] と同じ契約）。attached variant
/// （[`PaginationVariant::Attached`]）向けに子パーツの連結規則を
/// [`crate::css::serialize_rule`] で追記する（モジュール冒頭 rustdoc
/// 「連結 CSS の方式」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    let attached_root =
        r#"[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached"#;

    for part in ATTACHED_CHILD_PARTS {
        let child = format!(r#"[data-scope="pagination"][data-part="{part}"]"#);

        let not_first = format!("{attached_root} > {child}:not(:first-child)");
        append_rule(
            &mut out,
            serialize_rule(
                &not_first,
                &[
                    decl("margin-inline-start", "-1px"),
                    decl("border-start-start-radius", "0"),
                    decl("border-end-start-radius", "0"),
                ],
            ),
        );

        let not_last = format!("{attached_root} > {child}:not(:last-child)");
        append_rule(
            &mut out,
            serialize_rule(
                &not_last,
                &[
                    decl("border-start-end-radius", "0"),
                    decl("border-end-end-radius", "0"),
                ],
            ),
        );

        let focus_visible = format!("{attached_root} > {child}:focus-visible");
        append_rule(
            &mut out,
            serialize_rule(
                &focus_visible,
                &[decl("position", "relative"), decl("z-index", "2")],
            ),
        );
    }

    let selected_item =
        format!(r#"{attached_root} > [data-scope="pagination"][data-part="item"][data-selected]"#);
    append_rule(
        &mut out,
        serialize_rule(
            &selected_item,
            &[decl("position", "relative"), decl("z-index", "1")],
        ),
    );

    // イシュー #3136 レビュー是正: 上記 `selected_item` 規則（`z-index: 1`）が
    // 子ループで出力済みの `item:focus-visible`（`z-index: 2`、同一詳細度）を
    // ソース順で上書きし、選択中ページのフォーカス時に前面表示が失われる
    // 不具合があった。選択中 かつ フォーカス中の複合セレクタを
    // `selected_item` より後段へ追記し、`z-index: 2` で確実に勝たせる。
    let selected_item_focus_visible = format!(
        r#"{attached_root} > [data-scope="pagination"][data-part="item"][data-selected]:focus-visible"#
    );
    append_rule(
        &mut out,
        serialize_rule(
            &selected_item_focus_visible,
            &[decl("position", "relative"), decl("z-index", "2")],
        ),
    );

    let ellipsis_cell =
        format!(r#"{attached_root} > [data-scope="pagination"][data-part="ellipsis"]"#);
    append_rule(
        &mut out,
        serialize_rule(
            &ellipsis_cell,
            &[
                decl("box-sizing", "border-box"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                // イシュー #3136 レビュー是正: 他パーツ（item/prev-trigger/
                // next-trigger）と同じ基本角丸を持たせる。先頭・末尾の
                // ellipsis は `not_first`/`not_last` 規則（上記ループ、
                // `ATTACHED_CHILD_PARTS` に ellipsis を含む）が内側角のみ
                // `0` へ上書きするため、連結帯の外側角は丸いまま保たれる。
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        ),
    );

    out
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（`drop_class_attr` により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::pagination::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::pagination;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = pagination::root(Size::Md, ColorPalette::Accent, "pagination", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="pagination" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    aria_label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    root_with(
        size,
        PaginationVariant::Separated,
        palette,
        aria_label,
        attrs,
        children,
    )
}

/// styled root パーツを組み立てる（[`root`] の opt-in 拡張版、イシュー
/// #3136）。`variant == `[`PaginationVariant::Separated`]` のときは [`root`]
/// とバイト単位で同一の出力になる（`recipe()` が `Separated` へ
/// `default_variant` を登録しないため `variant-` クラスが付かない、
/// [`crate::tab_nav::root_with`] と同型の契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::pagination::{self, PaginationVariant};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = pagination::root_with(
///     Size::Md,
///     PaginationVariant::Attached,
///     ColorPalette::Accent,
///     "pagination",
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains("fd-pagination--variant-attached"));
/// ```
#[must_use]
pub fn root_with<'a>(
    size: Size,
    variant: PaginationVariant,
    palette: ColorPalette,
    aria_label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut selection: Vec<(&str, &str)> = vec![("size", size.value())];
    if variant != PaginationVariant::Separated {
        selection.push(("variant", variant.value()));
    }
    selection.push(("color-palette", palette.value()));
    let class = recipe.variant_classes(&selection);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::pagination::root(aria_label, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="pagination"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_selected_item_to_accent_style() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="item"][data-selected] {"#));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent))"));
    }

    #[test]
    fn stylesheet_links_disabled_triggers_to_not_allowed_cursor() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="pagination"][data-part="prev-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="pagination"][data-part="next-trigger"][data-disabled] {"#)
        );
    }

    #[test]
    fn stylesheet_links_item_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="item"]:focus-visible {"#));
    }

    // イシュー #1532: item hover が `@media (hover: hover)` へ集約出力され、
    // `--fandhe-hover-bg` の間接参照経由で背景色を切り替えることを確認する。
    #[test]
    fn stylesheet_defines_item_hover_via_media_hover() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="pagination"][data-part="item"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg)"));
    }

    // イシュー #1532: current ページ（`data-selected`）の hover 時は
    // emphasized 段（フォールバック連鎖付き）へ上書きされることを確認する。
    #[test]
    fn selected_item_overrides_hover_bg_to_emphasized() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="item"][data-selected] {"#));
        assert!(css.contains(
            "--fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized))"
        ));
    }

    // イシュー #1532: item の focus-visible が直書き outline ではなく共通
    // フォーカスリングトークン経由になっていることを確認する。
    #[test]
    fn item_focus_ring_uses_common_tokens() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-focus-ring-width"));
        assert!(css.contains("--fandhe-color-focus-ring"));
    }

    // イシュー #1532: ellipsis が `opacity` 直書きではなく fg-muted トークン
    // 経由になっていることを確認する。
    #[test]
    fn ellipsis_uses_fg_muted_token() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="ellipsis"] {"#));
        assert!(css.contains("color: var(--fandhe-color-fg-muted)"));
        assert!(!css.contains("opacity: 0.6"));
    }

    // イシュー #1533: prev-trigger/next-trigger の hover が item と同じ
    // `@media (hover: hover)` + `--fandhe-hover-bg` 間接参照経由で出力
    // されることを確認する。
    #[test]
    fn stylesheet_defines_trigger_hover_via_media_hover() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        for part in ["prev-trigger", "next-trigger"] {
            let selector = format!(
                r#"[data-scope="pagination"][data-part="{part}"]:hover:not([data-disabled]) {{"#
            );
            assert!(
                css.contains(&selector),
                "missing hover selector for {part}: {css}"
            );
        }
    }

    // イシュー #1533: prev-trigger/next-trigger の focus-visible が直書き
    // outline ではなく共通フォーカスリングトークン経由になっていることを
    // 確認する（item と同じトークン経由の検証パターン）。
    #[test]
    fn trigger_focus_ring_uses_common_tokens() {
        let css = stylesheet();
        for part in ["prev-trigger", "next-trigger"] {
            let selector =
                format!(r#"[data-scope="pagination"][data-part="{part}"]:focus-visible {{"#);
            assert!(
                css.contains(&selector),
                "missing focus-visible selector for {part}: {css}"
            );
        }
        assert!(css.contains("--fandhe-focus-ring-width"));
        assert!(css.contains("--fandhe-color-focus-ring"));
        assert!(!css.contains("outline: 2px solid var(--fandhe-color-accent)"));
    }

    // イシュー #1533: prev-trigger/next-trigger の transition がトークン化
    // された longhand（`--fandhe-motion-duration-fast` 等）であることを
    // 確認する（`transition: ... 0.15s` 直書き shorthand を使っていない）。
    #[test]
    fn trigger_transition_uses_tokenized_longhand() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-motion-duration-fast"));
        assert!(!css.contains("transition: background 0.15s"));
        assert!(!css.contains("transition: background, border-color, color 0.15s"));
    }

    // イシュー #1533: prev-trigger/next-trigger に横 padding が追加され、
    // テキストラベル運用時に枠線へ接しないことを確認する。
    #[test]
    fn trigger_has_inline_padding() {
        let css = stylesheet();
        for part in ["prev-trigger", "next-trigger"] {
            let selector = format!(r#"[data-scope="pagination"][data-part="{part}"] {{"#);
            let idx = css
                .find(&selector)
                .unwrap_or_else(|| panic!("missing base selector for {part}"));
            let block_end = css[idx..].find('}').map(|i| idx + i).unwrap_or(css.len());
            assert!(
                css[idx..block_end].contains("padding-inline: var(--fandhe-space-2)"),
                "missing padding-inline for {part}: {}",
                &css[idx..block_end]
            );
        }
    }

    // モジュール冒頭 rustdoc「複合部品の variant 統一方針」節が謳う「root の
    // --fandhe-pagination-item-font-size は item/prev-trigger/next-trigger
    // すべてに反映される」を base スタイルの実体で保証する回帰テスト
    // （Size::Sm/Lg 指定時に Prev/Next ラベルのテキストサイズが変わらない
    // 見た目不整合の再発防止、Cursor Bugbot 指摘対応）。
    #[test]
    fn prev_and_next_trigger_inherit_item_font_size_variable() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="prev-trigger"] {"#));
        assert!(css.contains(r#"[data-scope="pagination"][data-part="next-trigger"] {"#));
        assert!(css.contains(
            "font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))"
        ));
    }

    // --- variant クラス（root のみ） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("<nav"));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-pagination--size-md"));
        assert!(html.contains("fd-pagination--color-palette-accent"));
    }

    // イシュー #1533: Xs/Xl（#1681/#1714 で登録済み）を含む 5 段全網羅へ
    // テストカバレッジを拡充する。
    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-pagination--size-xs"),
            (Size::Sm, "fd-pagination--size-sm"),
            (Size::Md, "fd-pagination--size-md"),
            (Size::Lg, "fd-pagination--size-lg"),
            (Size::Xl, "fd-pagination--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                "pagination",
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-pagination--color-palette-accent"),
            (ColorPalette::Info, "fd-pagination--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-pagination--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-pagination--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-pagination--color-palette-danger"),
            (
                ColorPalette::Neutral,
                "fd-pagination--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, "pagination", vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn reexported_item_is_not_given_variant_classes() {
        // item は root のみへクラスが付く複合部品の variant 統一方針
        // （モジュール rustdoc 参照）。item 自体には class 属性がない。
        let html = render(&item(ItemMode::Button, 1, false, false, vec![], vec![]));
        assert!(!html.contains("class="));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_aria_label_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_href_and_children_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item(
            ItemMode::Link { href: PAYLOAD },
            1,
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_pagination_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Hydrate,
        };

        let mut p = Pagination::new(200, 10, 1, 1, 1);
        assert!(dispatch(&mut p, "goto", "5"));
        assert_eq!(p.page(), 5);

        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains("data-hydrate-page=\"5\""));

        let restored = Pagination::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    // --- attached variant（イシュー #3136） ---

    #[test]
    fn root_and_root_with_separated_match_byte_for_byte() {
        let a = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        let b = render(&root_with(
            Size::Md,
            PaginationVariant::Separated,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert_eq!(a, b);
    }

    #[test]
    fn root_never_outputs_variant_class() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(!html.contains("fd-pagination--variant-"));
    }

    #[test]
    fn root_with_attached_outputs_variant_class_and_keeps_other_classes() {
        let html = render(&root_with(
            Size::Md,
            PaginationVariant::Attached,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-pagination--variant-attached"));
        assert!(html.contains("fd-pagination--size-md"));
        assert!(html.contains("fd-pagination--color-palette-accent"));
    }

    #[test]
    fn root_with_class_attr_spoofing_is_dropped_for_attached_variant() {
        let html = render(&root_with(
            Size::Md,
            PaginationVariant::Attached,
            ColorPalette::Accent,
            "pagination",
            vec![
                ("class", "attacker-controlled"),
                ("data-scope", "attacker"),
                ("data-part", "attacker"),
            ],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn root_with_attached_label_is_escaped() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root_with(
            Size::Md,
            PaginationVariant::Attached,
            ColorPalette::Accent,
            PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn stylesheet_contains_attached_connective_rules() {
        let css = stylesheet();
        let attached_root =
            r#"[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached"#;
        for part in ["item", "ellipsis", "prev-trigger", "next-trigger"] {
            let not_first = format!(
                "{attached_root} > [data-scope=\"pagination\"][data-part=\"{part}\"]:not(:first-child) {{"
            );
            assert!(css.contains(&not_first), "missing {not_first}: {css}");
        }
        assert!(css.contains("margin-inline-start: -1px"));
        assert!(css.contains(
            r#"[data-scope="pagination"][data-part="item"][data-selected] {"#
        ) || css.contains(
            &format!(
                r#"{attached_root} > [data-scope="pagination"][data-part="item"][data-selected] {{"#
            )
        ));
        assert!(css.contains("gap: 0"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences_with_attached_rules() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // イシュー #3136 レビュー是正: 選択中ページ（`item[data-selected]`、
    // `z-index: 1`）がフォーカスされたとき `item:focus-visible` の
    // `z-index: 2` に埋もれず前面表示を保つことを固定する回帰テスト。
    #[test]
    fn stylesheet_keeps_selected_item_above_siblings_on_focus_visible() {
        let css = stylesheet();
        let attached_root =
            r#"[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached"#;
        let selected_focus_visible_rule = format!(
            r#"{attached_root} > [data-scope="pagination"][data-part="item"][data-selected]:focus-visible {{"#
        );
        assert!(
            css.contains(&selected_focus_visible_rule),
            "missing {selected_focus_visible_rule}: {css}"
        );

        let selected_rule_pos = css
            .find(r#"[data-scope="pagination"][data-part="item"][data-selected] {"#)
            .expect("selected_item rule must exist");
        let selected_focus_visible_pos = css
            .find(&selected_focus_visible_rule)
            .expect("selected_item_focus_visible rule must exist");
        assert!(
            selected_focus_visible_pos > selected_rule_pos,
            "selected+focus-visible override must come after the plain selected rule \
             so it wins under equal CSS specificity"
        );
    }

    // イシュー #3136 レビュー是正: Attached 構成で先頭・末尾の ellipsis が
    // 他パーツと同じ基本角丸を持ち、連結帯の外側角が四角くならないことを
    // 固定する回帰テスト。
    #[test]
    fn stylesheet_gives_ellipsis_cell_base_border_radius() {
        let css = stylesheet();
        let attached_root =
            r#"[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached"#;
        let ellipsis_cell =
            format!(r#"{attached_root} > [data-scope="pagination"][data-part="ellipsis"] {{"#);
        let rule_start = css
            .find(&ellipsis_cell)
            .unwrap_or_else(|| panic!("missing {ellipsis_cell}: {css}"));
        let rule_end = css[rule_start..]
            .find('}')
            .map(|offset| rule_start + offset)
            .expect("rule must be closed");
        assert!(
            css[rule_start..rule_end].contains("border-radius: var(--fandhe-radius-md)"),
            "ellipsis cell rule missing base border-radius: {}",
            &css[rule_start..rule_end]
        );
    }
}
