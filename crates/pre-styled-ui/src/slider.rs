//! styled Slider（headless ラッパー、イシュー #741、親 #736/#520/#545）。
//!
//! `fandhe_frontend_headless_ui::slider`（イシュー #741）の Label / Control /
//! Track / Thumb / HiddenInput / ValueText の 6 anatomy パーツをそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::switch`]/[`crate::number_input`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Slider` 型・headless
//! `root`/`range` を再エクスポートしない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::switch::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`label`]/[`control`]/[`track`]/[`thumb`]/[`hidden_input`]/
//! [`value_text`]/[`SliderAction`]）のみを選択的に再エクスポートする。
//!
//! `range` も再エクスポートしない。動的な塗りつぶし幅を伝える唯一の経路
//! （[`Slider::percent`](fandhe_frontend_headless_ui::slider::Slider::percent)
//! から導出する `--fandhe-slider-percent` CSS custom property、モジュール
//! doc「動的値は 1 点のみ」参照）は本モジュールの styled [`range`] が
//! 一元的に組み立てる。headless 自由関数 `range` を呼び出し側が直接使うと
//! この唯一の経路を経由せず塗りつぶしが描画されない事故を誘発するため、
//! 意図的に非公開のまま [`range`] 内部からのみ委譲する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::slider::Slider`] も**あえて**
//! 再エクスポートしない（[`crate::switch`] の `Switch` 非再エクスポートと
//! 同じ理由）。`Slider` は `.root(disabled, attrs, children)` 等の inherent
//! メソッドを持つが、これは headless 自由関数へそのまま委譲するのみで
//! `size`/`palette` variant クラス・`--fandhe-slider-percent` を一切付与
//! しない未スタイルの実体である。本モジュールが `Slider` を丸ごと再
//! エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `slider_instance.root(...)`/`slider_instance.range(...)` を呼んでしまい、
//! 見た目が静かに崩れる事故を誘発する。`Slider` による状態管理・hydration
//! が必要な呼び出し側は `fandhe_frontend_headless_ui::slider::Slider` を
//! 直接 import し、実際の描画は本モジュールの styled [`root`]/[`range`]
//! （および再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は `--fandhe-slider-percent` の 1 点のみ（chakra-ui/Zag.js 方式）
//!
//! [`range`]/[`thumb`] の位置は、headless 中立な `[`Slider::percent`]
//! （0.0..=100.0 の正規化済み有限 `f64`）から [`percent_style`] が組み立てる
//! `style="--fandhe-slider-percent: <percent>%"` の 1 属性のみで伝搬する。
//! [`crate::number_input`]/[`crate::switch`] とは異なり、本コンポーネントは
//! [`crate::css::drop_style_attr`] 相当のヘルパを本モジュール内に個別実装
//! し（[`drop_style_attr`]、`crates/headless-ui/src/progress.rs` の同名
//! ヘルパと同型の判断）、呼び出し側 `attrs` に含まれる `style`（大文字小文字
//! を無視）を除去してからフレームワーク側の `style` を優先する（重複属性
//! による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、fail-closed）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-slider-track-height`/`-thumb-size` の root スコープ custom
//! property（CSS の通常のプロパティ継承により `control`/`track`/`range`/
//! `thumb` へ伝わる）経由で寸法を切り替える（[`crate::switch`] と同型）。
//! `palette`（[`ColorPalette`]）は既存の [`crate::recipe::palette_scale_declarations`]
//! （chakra-ui virtual token 方式、#606）を `root` へ登録し、`range`/`thumb`
//! の色を `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # 縦方向（`data-orientation="vertical"`）レイアウト
//!
//! `track`/`range`/`thumb` は `data-orientation="vertical"` のとき `width`/
//! `left` 系ではなく `height`/`bottom` 系プロパティを使う（[`recipe`] の
//! `StateCondition::AttrEq("data-orientation", "vertical")` 状態規則）。
//! `range`/`thumb` の動的位置（`--fandhe-slider-percent`）は軸に関わらず
//! 同一の custom property を使い回し、CSS 側の `width`/`height` どちらを
//! 参照するかだけを orientation で切り替える（headless 側の値は 1 系統の
//! ままで済み、CSR/hydration の状態フォーマットに影響しない）。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! [`thumb`] はネイティブにフォーカス可能な要素（`tabindex`）であるため、
//! [`crate::switch`] のような hidden-input 特有の `data-focus-visible`
//! 対応は不要で、通常の `:focus-visible` 疑似クラスを [`recipe`] へ直接
//! 登録する（[`StateCondition::FocusVisible`]）。
//!
//! # イシュー #1505: トラック・レンジ・サムのスタイル調整（親 #1504 の 1/2）
//!
//! 参考サイト（chakra-ui/Radix Themes/Radix Primitives/ark-ui）視覚比較に
//! 基づき、`root`/`thumb` の disabled を共通ビジュアル言語
//! （[`crate::recipe::disabled_declarations`]、イシュー #1425）へ、
//! `thumb` の `:focus-visible` を共通フォーカスリングトークン
//! （[`crate::recipe::focus_ring_declarations`]、イシュー #1424）へ置換し、
//! `track`/`range`/`thumb` の角丸を `var(--fandhe-radius-full, 999px)`
//! トークンへ統一した（フォールバックは codex-review 指摘、PR #1777 で
//! 追加。`timeline` の `var(--fandhe-radius-full, 9999px)` と同型）。`thumb` へは参考サイト共通の「白面 + 影」表現
//! （`box-shadow: var(--fandhe-shadow-sm)`）と hover/transition
//! フィードバック（[`crate::recipe::hover_bg_muted`]/
//! [`crate::recipe::transition_declarations`]）を追加した。`thumb` は常時
//! 「白面 + palette ボーダー」の outline 表現（checked/unchecked のような
//! 二値状態を持たない）であるため、solid 面向けの `hover_bg_solid` ではなく
//! `hover_bg_muted` を選んだ（[`recipe`] 内コメント参照）。
//! `--fandhe-slider-percent` 由来の位置プロパティ（`transform`/`left`/`top`）
//! には transition を掛けない（ドラッグ追従の即時性維持、angle-slider
//! （イシュー #1445/PR #1728）と同型の判断）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく range slider（複数 thumb）・pointer ドラッグ/
//!   キーボード操作の DOM 配線はスコープ外（`fandhe_frontend_headless_ui::slider`
//!   モジュール doc 参照）。Marker/MarkerGroup は下記「イシュー #2020」節の
//!   とおり本イシューでスタイル対応済みであり、この行の対象外（#1505/#1506
//!   完了時点ではスコープ外だった名残の記述を #2020 で更新）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Slider 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::number_input`] 冒頭 rustdoc の先例どおり
//!   crates.io 公開後に追随）。
//! - marker/label/value-text の是正・`data-orientation="vertical"` の
//!   状態規則の再設計は姉妹イシュー #1506（親 #1504 の 2/2）が完了済み
//!   （下記「イシュー #1506」節参照）。
//!
//! # イシュー #1506: マーカー・ラベル・値テキストと orientation 2 方向
//! （親 #1504 の 2/2）
//!
//! 参考サイト（chakra-ui/Radix Themes/Radix Primitives/ark-ui）視覚比較に
//! 基づき、#1505 が残した残りの担当範囲（マーカー・ラベル・値テキストの
//! 型階層、vertical 方向の実バグ）を是正した。
//!
//! ## ラベル・値テキストの型階層
//!
//! - `label`: angle-slider（#1446、ark-ui 基準）の uppercase +
//!   letter-spacing は採らない（参照サイトの linear slider にはない表現の
//!   ため）。「通常ケース・medium ウェイト・前景色」の型階層のみを追加
//!   （`font-weight`/`color`/`line-height`）。
//! - `value-text`: #1505 完了時点で base 宣言が皆無で CSS ブロック自体が
//!   出力されない状態だった（angle-slider の #1446 是正前と同一）。
//!   `font-size`/`color`（`fg-muted`、`theme.rs` のコントラスト検証ペア
//!   `("fg-muted", "bg")` 登録済み）/`line-height`/
//!   `font-variant-numeric: tabular-nums`（値変動時に幅がぶれない）を
//!   base として新設した。size 連動（root スコープ custom property）は
//!   行わない。参照サイトの linear slider はラベル・値テキストをスライダー
//!   size に関わらず一定の text-sm で表示するため（angle-slider の
//!   大型ダイヤル読み出しとは役割が異なる）。
//! - `label`/`value-text` は非インタラクティブなテキストのため hover/
//!   transition は追加しない（angle-slider #1446 と同じ判断）。
//!
//! ## マーカー: #1505/#1506 完了時点は意図的非採用（#2020 で解消）
//!
//! #1505/#1506 完了時点では headless anatomy（`crates/headless-ui/src/slider.rs`）
//! に Marker/MarkerGroup パーツが存在せず、意図的非採用としていた。その後
//! イシュー #1904（PR #1875、親 #1621）で headless 側に `marker_group`/
//! `marker` anatomy パーツが追加済みであり、本モジュールはイシュー #2020 で
//! これへ追随し styled [`marker_group`]/[`marker`] を新設した（詳細は下記
//! 「イシュー #2020」節参照）。この節の記述は当時の判断の記録として残す。
//!
//! ## vertical 状態規則の再設計（実バグ修正）
//!
//! #1505 完了時点の `thumb` vertical 状態規則は `top` アンカー
//! （`top: var(--fandhe-slider-percent, 0%)`）で、`range`（`bottom: 0;
//! height: var(--fandhe-slider-percent)`、下から上へ伸長）と増加方向が
//! **逆転**していた（percent=100 のとき range は全高を塗るが thumb は
//! 最下端へ移動する不具合）。`thumb` を `bottom` アンカーへ変更し、
//! `transform` の Y 成分を `-50%`（top アンカー用）から `+50%`（bottom
//! アンカー用）へ反転してつまみ中心を位置点に載せることで、range と
//! 増加方向を一致させた。`--fandhe-slider-percent` 1 点伝搬（headless 側の
//! 値は 1 系統のまま）という既存契約は不変で、CSS の参照プロパティのみを
//! 切り替えている。`control`/`track`/`range` の既存 vertical 規則と root の
//! column レイアウトは参照サイトと整合しており変更していない。
//!
//! # イシュー #2020: shadcn/ui 突合
//!
//! shadcn/ui（<https://ui.shadcn.com/docs/components/base/slider>）の参照
//! スクリーンショット 3 枚はいずれも複数 thumb（range slider）のバリエー
//! ションで、disabled・vertical・size 等の単一値バリアントは撮影されて
//! いない。複数 thumb は headless-ui が `#741` 以来一貫してスコープ外と
//! している構造的制約（anatomy に thumb 複数化・`data-index` 等の機構が
//! ない）であり、Themes 層だけでは対応不可能なため本イシューでも意図的
//! 非採用のまま据え置く（対応する場合は headless-ui 側の新規イシューから
//! 着手する必要がある）。
//!
//! 一方、突合作業中に「マーク・値表示の有無」という既存ギャップ（shadcn/ui
//! 自体はマークを持たないが、参考 4 サイトの chakra-ui/Radix Themes/Radix
//! Primitives/ark-ui は Marker 相当を持つ）が判明した。headless anatomy
//! （`fandhe_frontend_headless_ui::slider::marker_group`/`marker`）は
//! イシュー #1904（PR #1875、親 #1621）で既に追加済みだったが、本モジュール
//! の styled 層は追随しておらず未スタイルのままだった。上記「マーカー」節の
//! とおり、本イシューでこの Themes 側追随ギャップを解消し、styled
//! [`marker_group`]/[`marker`] を新設した。
//!
//! ## マーカーのスタイル方針
//!
//! - `marker-group`: `control`（`position: relative`）に対する絶対配置
//!   オーバーレイコンテナ（`inset: 0` 相当）。`pointer-events: none` を
//!   付与し、`track`/`thumb` のクリック・ドラッグ判定を奪わない。
//! - `marker`: [`range`]/[`thumb_styled`] と同型の「動的値は 1 属性の
//!   `style` のみ」方針を踏襲し、`--fandhe-slider-marker-percent` custom
//!   property で位置を伝搬する（[`marker`] 関数のみが計算・付与する）。
//!   horizontal は `left` + `translateX(-50%)`、vertical
//!   （`data-orientation="vertical"`）は `thumb` と同じ `bottom` アンカー
//!   方式に切り替える。`data-state`（`under-value`/`over-value`/
//!   `at-value`）で色を作り分け、`under-value`/`at-value` は `range` と
//!   揃えた palette 色、`over-value` は `--fandhe-color-fg-muted`（イシュー
//!   #2020 PR #2167 レビュー指摘 P1: 旧実装は `track` と同一トークン
//!   `--fandhe-color-border` を使い track 上で完全に同化していたため、実
//!   コントラストのあるトークンへ変更した）とする。marker 自体も track と
//!   同じ丈だと range/track に重なる部分が同化するため、track 高より一回り
//!   高くして上下へ突出させ、加えて `--fandhe-color-bg`（ページ背景色）1px
//!   の `box-shadow` リングで track/range いずれの背景色に対しても輪郭が
//!   見えるようにしている（同レビュー指摘）。既存デザイントークン
//!   （`--fandhe-color-*`/`--fandhe-radius-full`）のみを再利用し、新規
//!   トークンは追加しない。
//! - angle-slider（[`crate::angle_slider`]）は円形スライダー固有の理由
//!   （目盛りを `control` の多層 `background` グラデーションで CSS-only
//!   再現しているため DOM マーカーが不要）でマーカーを意図的に非スタイル
//!   のままとしている。この判断は任意の値位置に意味を持つ linear slider の
//!   マーカーには転用できないため、`angle_slider` の前例を「マーカー
//!   非スタイルの一般方針」と誤読しないこと。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Slider` 状態機械・headless 自由関数 `root`/`range` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::slider::Slider`
// を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::slider::Slider;
pub use fandhe_frontend_headless_ui::slider::{
    control, hidden_input, label, thumb, track, value_text, SliderAction, SliderProps,
};

/// headless `slider` anatomy の `data-part` 一覧（`crates/headless-ui/src/slider.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "track",
    "range",
    "thumb",
    "marker-group",
    "marker",
    "hidden-input",
    "value-text",
];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`range`]/[`thumb`] がフレームワーク側で `--fandhe-slider-percent` を
/// 含む `style` を組み立てた後、呼び出し側 `attrs` を連結する前に使う
/// dedup ヘルパ（`crates/headless-ui/src/progress.rs::drop_style_attr` と
/// 同型の判断。重複属性による無効な HTML 出力・後勝ちの非決定的な描画を
/// 防ぐ、fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `percent`（`[`Slider::percent`] が返す正規化済み有限 `f64`）から
/// `--fandhe-slider-percent` custom property を設定する `style` 属性値を
/// 組み立てる（動的値はこの 1 箇所のみ、モジュール doc 参照）。
fn percent_style(percent: f64) -> String {
    format!("--fandhe-slider-percent: {percent}%")
}

/// [`marker`] 専用の位置 custom property（`--fandhe-slider-marker-percent`）
/// を設定する `style` 属性値を組み立てる（イシュー #2020。[`percent_style`]
/// と同型だが、マーカーは `Slider` の現在値ではなく `marker` 自身が表す
/// 任意の目盛り値から算出するため、別 custom property・別ヘルパとする）。
///
/// `min`/`max`/`value` の非有限値（`NaN`/`Infinity`）に対する正規化は
/// headless 側 [`fandhe_frontend_headless_ui::slider::marker`] の fail-closed
/// 規約（`min`/`max` が非有限または `min >= max` なら既定 `(0.0, 100.0)` へ、
/// `value` が非有限なら `min` へフォールバック、イシュー #1621 PR #1904）と
/// 同じ計算をここでも行う。`data-value`/`data-state`（headless 側で計算）と
/// `--fandhe-slider-marker-percent`（styled 層で計算する見た目位置）の間で
/// 正規化結果が食い違うと、マーカーの描画位置が `data-*` の意味と矛盾する
/// （`value` に `NaN` を渡すアプリコードで実際に発生し得る不具合）。
fn marker_percent_style(min: f64, max: f64, value: f64) -> String {
    let (min, max) = if min.is_finite() && max.is_finite() && min < max {
        (min, max)
    } else {
        (0.0, 100.0)
    };
    let value = if value.is_finite() { value } else { min };
    let span = max - min;
    let percent = if span > 0.0 {
        ((value.clamp(min, max) - min) / span * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    format!("--fandhe-slider-marker-percent: {percent}%")
}

/// この styled Slider の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("slider", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            // イシュー #1505: `opacity`/`cursor` 直書きを共通ビジュアル言語
            // （イシュー #1425、`crate::recipe` 冒頭 doc「disabled / hover /
            // transition の共通ビジュアル言語」節）へ置換。値そのものは
            // 変わらないため見た目に差分は出ない。
            disabled_declarations(),
        )
        .base(
            "label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                // イシュー #1506: 型階層の追加宣言。angle-slider（#1446）の
                // uppercase + letter-spacing とは異なり、参照サイトの linear
                // slider ラベルは「通常ケース・medium ウェイト・前景色」の
                // ため、その表現のみを追加する（モジュール doc「イシュー
                // #1506」節参照）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "value-text",
            vec![
                // イシュー #1506: `value-text` は #1505 完了時点で base 宣言が
                // 皆無で CSS ブロック自体が出力されなかった（angle-slider の
                // #1446 是正前と同一の状態）。参照サイトの linear slider は
                // ラベル・値テキストをスライダー size に関わらず一定の
                // text-sm で表示するため、angle-slider（#1446、大型ダイヤル
                // 読み出し用に size 連動 font-size を持つ）とは異なり size
                // 連動の root スコープ custom property は導入しない
                // （モジュール doc「イシュー #1506」節参照）。
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                // 値変動時に幅がぶれないよう固定幅数字を使う（timer /
                // progress / angle-slider の value-text と同型）。
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("width", "var(--fandhe-slider-track-length, 12rem)"),
            ],
        )
        .state(
            "control",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("width", "auto"),
                decl("height", "var(--fandhe-slider-track-length, 12rem)"),
            ],
        )
        .base(
            "track",
            vec![
                decl("position", "relative"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-slider-track-height, 0.375rem)"),
                // イシュー #1505: 角丸をトークン化（`999px` リテラル →
                // `var(--fandhe-radius-full, 999px)`。angle-slider #1728 と
                // 同型の是正、参照 4 サイト（chakra-ui/Radix Themes/Radix
                // Primitives/ark-ui）いずれも完全な pill 形状のため計算結果は
                // 不変）。フォールバックは `--fandhe-radius-full` 未定義の
                // 既存カスタムテーマ（`Theme::empty()` ベース）で角丸が
                // 初期値 `0` へ落ちる互換性破壊を防ぐための codex-review
                // 指摘（PR #1777、threadId: PRRT_kwDOTarxgc6eICfZ）対応。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                // トラック色は既存の `var(--fandhe-color-border)` を維持する
                // （参照 4 サイトいずれも「淡いニュートラル面」であり、
                // ダーク対応済みの本トークンで既に基準を満たすため変更しない。
                // イシュー #1505 Step 1 差分メモ参照）。
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .state(
            "track",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("width", "var(--fandhe-slider-track-height, 0.375rem)"),
                decl("height", "100%"),
            ],
        )
        .base(
            "range",
            vec![
                decl("position", "absolute"),
                decl("top", "0"),
                decl("left", "0"),
                decl("height", "100%"),
                decl("width", "var(--fandhe-slider-percent, 0%)"),
                // イシュー #1505: `track` と同じ角丸トークン化。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "range",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("top", "auto"),
                decl("bottom", "0"),
                decl("left", "0"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-slider-percent, 0%)"),
            ],
        )
        .base(
            "thumb",
            vec![
                decl("position", "absolute"),
                decl("top", "50%"),
                decl("left", "var(--fandhe-slider-percent, 0%)"),
                decl("transform", "translate(-50%, -50%)"),
                decl("width", "var(--fandhe-slider-thumb-size, 1.1rem)"),
                decl("height", "var(--fandhe-slider-thumb-size, 1.1rem)"),
                // イシュー #1505: `track`/`range` と同じ角丸トークン化。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl(
                    "border",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // イシュー #1505: 参照 4 サイト（chakra-ui/Radix Themes/
                // Radix Primitives/ark-ui）共通の「白面サム + 影による浮き
                // 上がり」表現。`angle_slider` の `control`（面）が持つ
                // `box-shadow: var(--fandhe-shadow-sm)` と同型のトークン。
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                // hover 時に切り替える面色の間接参照先を定義する（実際の
                // `background` 適用は下記 `.state("thumb",
                // StateCondition::Hover, ...)` 1 本に集約する、`crate::radio_group`
                // の unchecked `item-control` と同型のパターン）。本サムは
                // 常時「白面 + palette ボーダー」の outline 表現であり
                // solid 面を持たないため、solid variant 向けの
                // `hover_bg_solid`（`--fandhe-palette-emphasized` 参照）は
                // 採らず、`hover_bg_muted`（`--fandhe-color-bg-muted`）で
                // 淡い面の変化のみ表現する（`crate::recipe` 冒頭 doc
                // 「hover」節の使い分け規約に従う判断）。
                hover_bg_muted(),
                decl("box-sizing", "border-box"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "thumb",
            // イシュー #1505: hover/focus 面変化を滑らかにする（`transform`/
            // `left`/`top`（`--fandhe-slider-percent` 由来の位置）は含めない
            // — ドラッグ中の位置追従に遅延を持ち込まないため。angle-slider
            // #1728 の「`--fandhe-angle` に transition を掛けない」判断と
            // 同型。`prefers-reduced-motion` 対応は `transition_declarations`
            // の呼び出し先（`Theme::to_css` の duration 一括 0ms 化）が担う。
            transition_declarations("background, border-color, box-shadow", MotionDuration::Fast),
        )
        .state(
            "thumb",
            StateCondition::AttrEq("data-orientation", "vertical"),
            // イシュー #1506: vertical 方向逆転バグの修正。`range` は
            // `bottom: 0; height: var(--fandhe-slider-percent)`（下から上へ
            // 伸長）だが、是正前の thumb は `top` アンカーで percent が
            // 増えるほど下端へ移動しており、range と thumb の増加方向が
            // 逆転していた。`bottom` アンカーへ切り替えて range と一致させ、
            // `transform` の Y 成分も `-50%`（top アンカー用）から `+50%`
            // （bottom アンカー用）へ反転してつまみ中心を位置点に載せる
            // （モジュール doc「イシュー #1506」節参照）。
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-slider-percent, 0%)"),
                decl("left", "50%"),
                decl("transform", "translate(-50%, 50%)"),
            ],
        )
        .state(
            "thumb",
            StateCondition::Attr("data-disabled"),
            // 掴めないことの表現として `cursor: not-allowed` を維持する
            // （`opacity` は `root` の `disabled_declarations()` が既に
            // 全体へ適用済みのため、`thumb` 側で重複させない）。
            vec![decl("cursor", "not-allowed")],
        )
        // イシュー #1505: hover の実適用は 1 本のみ（`--fandhe-hover-bg` の
        // 間接参照経由。`crate::radio_group`/`crate::checkbox` の
        // `item-control`/`control` と同型のパターン、`crate::recipe` 冒頭
        // doc「hover」節参照）。`StateCondition::Hover` は
        // `:hover:not([data-disabled])` へ直列化されるため disabled 時の
        // hover は自然に除外される。
        .state("thumb", StateCondition::Hover, hover_surface_declarations())
        .state(
            "thumb",
            StateCondition::FocusVisible,
            // イシュー #1505: `outline`/`outline-offset` 直書きを
            // `focus_ring_declarations`（イシュー #1424 共通トークン
            // `--fandhe-focus-ring-*`・`--fandhe-color-focus-ring` 経由）へ
            // 置換。`FocusRingColor::Palette` は選択中の palette へリング色
            // を連動させる（angle-slider #1728 と同型）。フォールバック値は
            // 旧実装と同じ `2px`/`var(--fandhe-color-accent)` のため、
            // 新トークン未定義の既存カスタムテーマでも見た目は不変。
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #2020: `marker-group`/`marker` の styled 化（headless
        // anatomy はイシュー #1904 で追加済み、モジュール doc「イシュー
        // #2020」節参照）。`marker-group` はクリック・ドラッグ判定を
        // `track`/`thumb` から奪わないよう `pointer-events: none` の
        // オーバーレイコンテナとする。
        .base(
            "marker-group",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "marker",
            vec![
                decl("position", "absolute"),
                decl("top", "50%"),
                decl("left", "var(--fandhe-slider-marker-percent, 0%)"),
                decl("transform", "translate(-50%, -50%)"),
                decl("width", "2px"),
                // イシュー #2020 PR #2167 レビュー指摘 P1（codex/Bugbot）:
                // トラック高と同じ丈だと under/at-value（range と同色）・
                // over-value（旧: track と同色）のいずれも背景へ同化して
                // 視認できない。track 高より一回り高くしてトラック上下へ
                // 突出させ、track/range に重ならない部分（`control` の空白
                // 領域）で常に輪郭が見えるようにする。
                decl(
                    "height",
                    "calc(var(--fandhe-slider-track-height, 0.375rem) + 0.5rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                // 突出のみでは track/range 上に重なる中央部分がなお同化する
                // ため、`--fandhe-color-bg`（ページ背景）1px のリング
                // （box-shadow）を輪郭として重ね、track/range いずれの色に
                // 対しても縁取りで区別できるようにする（native
                // `<input type="range">` の datalist tick と同型の視認性
                // 対処）。
                decl("box-shadow", "0 0 0 1px var(--fandhe-color-bg)"),
                // under-value/at-value（現在値に達済みのマーカー）は range
                // と同じ palette 色、over-value（未到達）は下記状態規則で
                // より濃いミュート色へ上書きする。
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "marker",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-slider-marker-percent, 0%)"),
                decl("left", "50%"),
                decl("transform", "translate(-50%, 50%)"),
                decl(
                    "width",
                    "calc(var(--fandhe-slider-track-height, 0.375rem) + 0.5rem)",
                ),
                decl("height", "2px"),
            ],
        )
        .state(
            "marker",
            StateCondition::AttrEq("data-state", "over-value"),
            // イシュー #1505 の thumb disabled 状態規則（`opacity` は root の
            // `disabled_declarations()` が既に全体へ視覚適用済みのため個別
            // 追加しない、上記コメント参照）と同じ判断で、marker の
            // disabled 専用宣言は追加しない。
            // イシュー #2020 PR #2167 レビュー指摘 P1: 旧 `--fandhe-color-border`
            // は `track` の背景そのものと同色（同一トークン参照）のため
            // over-value のマーカーが track 上で完全に同化していた。
            // `--fandhe-color-fg-muted` はライト/ダーク双方で `border` より
            // 明確に濃く（例: light `#4a4a4a` vs `border` の `#d9d9d9`）、
            // track 背景との実コントラストを確保する。
            vec![decl("background", "var(--fandhe-color-fg-muted)")],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-slider-track-height", "0.125rem"),
                decl("--fandhe-slider-thumb-size", "0.6rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-slider-track-height", "0.25rem"),
                decl("--fandhe-slider-thumb-size", "0.85rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-slider-track-height", "0.375rem"),
                decl("--fandhe-slider-thumb-size", "1.1rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-slider-track-height", "0.5rem"),
                decl("--fandhe-slider-thumb-size", "1.35rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-slider-track-height", "0.625rem"),
                decl("--fandhe-slider-thumb-size", "1.6rem"),
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
    recipe
}

/// この styled Slider が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::slider::Slider::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::slider::Slider;
/// use fandhe_frontend_pre_styled_ui::slider;
/// use fandhe_frontend_pre_styled_ui::slider::SliderProps;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = Slider::default();
/// let node = slider::root(Size::Md, ColorPalette::Accent, &s, &SliderProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="slider" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &Slider,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(props, merged, children)
}

/// styled range パーツを組み立てる。`--fandhe-slider-percent` を含む
/// `style` を付与する唯一のパーツ（[`drop_style_attr`] により呼び出し側の
/// `style` は除去してから合成する。動的値はこの 1 箇所のみ、モジュール
/// doc「動的な値は 1 点のみ」参照）。実体は
/// [`fandhe_frontend_headless_ui::slider::Slider::range`] へ委譲する。
#[must_use]
pub fn range<'a>(state: &Slider, props: &SliderProps, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let style = percent_style(state.percent());
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.range(props, merged, Vec::new())
}

/// styled thumb パーツを組み立てる。[`range`] と同じ `--fandhe-slider-percent`
/// 位置指定を付与する（[`drop_style_attr`] で呼び出し側 `style` を dedup）。
#[must_use]
pub fn thumb_styled<'a>(
    state: &Slider,
    aria_valuetext: Option<&'a str>,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let style = percent_style(state.percent());
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.thumb(aria_valuetext, props, merged, Vec::new())
}

/// styled marker-group パーツを組み立てる（イシュー #2020）。状態を持たない
/// コンテナのため `state` は受け取らず、headless
/// [`fandhe_frontend_headless_ui::slider::marker_group`] へそのまま委譲する
/// （モジュール doc「イシュー #2020」節参照）。
#[must_use]
pub fn marker_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::slider::marker_group(attrs, children)
}

/// styled marker パーツを組み立てる（イシュー #2020）。[`range`]/[`thumb_styled`]
/// と同じ「動的値は 1 属性の `style` のみ」方針を踏襲し、`marker` 自身の
/// 目盛り値（`value`）と `state`（`Slider`）の `min`/`max` から
/// `--fandhe-slider-marker-percent` を算出して付与する（[`drop_style_attr`]
/// で呼び出し側 `style` を dedup）。`data-value`/`data-state`/`data-disabled`
/// の算出は [`Slider::marker`] へ委譲する（現在値との大小関係の正規化は
/// headless 層の責務、モジュール doc 参照）。
#[must_use]
pub fn marker<'a>(
    state: &Slider,
    value: f64,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = marker_percent_style(state.min(), state.max(), value);
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.marker(value, disabled, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="slider"][data-part="range"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_percent_custom_property() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-slider-percent"));
    }

    #[test]
    fn stylesheet_links_track_and_range_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="slider"][data-part="track"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="slider"][data-part="range"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="slider"][data-part="thumb"][data-orientation="vertical"] {"#
        ));
    }

    #[test]
    fn stylesheet_vertical_thumb_direction_matches_range() {
        // イシュー #1506: `range` は `bottom: 0; height:
        // var(--fandhe-slider-percent)`（下から上へ伸長）であり、`thumb`
        // vertical 状態規則もこれと同じ増加方向（`bottom` アンカー）を
        // 参照する回帰テスト（是正前は `top` アンカーで方向が逆転していた）。
        let css = stylesheet();
        assert!(css.contains("bottom: var(--fandhe-slider-percent, 0%);"));
        assert!(css.contains("transform: translate(-50%, 50%);"));
    }

    #[test]
    fn stylesheet_gives_label_and_value_text_distinct_typography() {
        // イシュー #1506: label / value-text 双方が CSS ブロックを出力し、
        // 型階層（重み・色・line-height・tabular-nums）を持つことを固定する
        // （#1505 完了時点では value-text の base 宣言が皆無だった）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="slider"][data-part="label"] {"#));
        assert!(css.contains(r#"[data-scope="slider"][data-part="value-text"] {"#));
        assert!(css.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(css.contains("color: var(--fandhe-color-fg-muted);"));
        assert!(css.contains("font-variant-numeric: tabular-nums;"));
    }

    #[test]
    fn stylesheet_marker_group_is_a_pointer_events_none_overlay() {
        // イシュー #2020: `marker-group` は `track`/`thumb` のクリック・
        // ドラッグ判定を奪わないことを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="slider"][data-part="marker-group"] {"#));
        assert!(css.contains("pointer-events: none;"));
    }

    #[test]
    fn stylesheet_marker_references_marker_percent_custom_property() {
        // イシュー #2020: マーカー位置は `range`/`thumb` の
        // `--fandhe-slider-percent` とは別の custom property
        // （`--fandhe-slider-marker-percent`）で伝搬する契約を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="slider"][data-part="marker"] {"#));
        assert!(css.contains("--fandhe-slider-marker-percent"));
    }

    #[test]
    fn stylesheet_links_marker_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="slider"][data-part="marker"][data-orientation="vertical"] {"#
        ));
    }

    #[test]
    fn stylesheet_links_marker_to_over_value_state() {
        // イシュー #2020 PR #2167 レビュー指摘 P1: 未到達（over-value）の
        // マーカーは track 背景（`--fandhe-color-border`）そのものではなく、
        // 実コントラストが確保された `--fandhe-color-fg-muted` で塗る契約を
        // 固定する（track と同色だと同化して視認できないため）。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="slider"][data-part="marker"][data-state="over-value"] {"#)
        );
        assert!(css.contains("background: var(--fandhe-color-fg-muted);"));
    }

    #[test]
    fn stylesheet_marker_protrudes_past_track_height_and_has_bg_ring() {
        // イシュー #2020 PR #2167 レビュー指摘 P1: track と同じ高さの
        // マーカーは range/track に完全に重なり視認できないため、track より
        // 高くして突出させ、かつページ背景色のリング（box-shadow）で
        // track/range いずれの色に対しても輪郭が見えるようにする契約を
        // 固定する。
        let css = stylesheet();
        assert!(css.contains("height: calc(var(--fandhe-slider-track-height, 0.375rem) + 0.5rem);"));
        assert!(css.contains("box-shadow: 0 0 0 1px var(--fandhe-color-bg);"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="slider"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_thumb_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="slider"][data-part="thumb"]:focus-visible {"#));
        // イシュー #1505: `outline` 直書きから `focus_ring_declarations`
        // （共通トークン `--fandhe-focus-ring-*`/`--fandhe-color-focus-ring`、
        // `FocusRingColor::Palette` の `--fandhe-palette` 連動フォールバック
        // 連鎖）へ置換した契約を固定する。
        assert!(css.contains("var(--fandhe-focus-ring-width, 2px)"));
        assert!(css.contains(
            "var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)))"
        ));
    }

    #[test]
    fn stylesheet_uses_radius_full_token_for_track_range_thumb() {
        let css = stylesheet();
        // イシュー #1505: `999px` リテラルをトークン化した契約を固定する
        // （angle-slider #1728 と同型）。フォールバックは codex-review 指摘
        // （PR #1777）を受け `var(--fandhe-radius-full, 999px)` の形（`timeline`
        // の `var(--fandhe-radius-full, 9999px)` と同型）とし、`999px` の
        // 生リテラル自体はフォールバック値としてのみ残す。イシュー #2020 で
        // `marker` にも同トークンを追加したため、対象は track/range/thumb/
        // marker の 4 箇所（旧 3 箇所から更新）。
        assert_eq!(
            css.matches("border-radius: var(--fandhe-radius-full, 999px);")
                .count(),
            4
        );
    }

    #[test]
    fn radius_full_fallback_keeps_pill_shape_on_theme_without_radius_full_token() {
        // イシュー #1505 codex-review 指摘（PR #1777, threadId: PRRT_kwDOTarxgc6eICfZ）:
        // `var(--fandhe-radius-full)` をフォールバックなしで参照すると、
        // `--fandhe-radius-full` を定義しない `Theme::empty()` ベースの
        // 既存カスタムテーマでは宣言全体が computed-value time に無効化され、
        // `track`/`range`/`thumb` の角丸が初期値の `0` に落ちる
        // （`Cargo.toml` が謳う「パッチバンプ = 計算結果は不変」契約への
        // 違反）。`Theme::empty()` が `--fandhe-radius-full` を定義しない
        // ことを確認したうえで、`stylesheet()` の角丸宣言がフォールバック
        // 込みであることを固定し、退行を防ぐ。
        use crate::theme::Theme;

        let empty_theme_css = Theme::empty().to_css();
        assert!(!empty_theme_css.contains("--fandhe-radius-full"));

        let css = stylesheet();
        assert_eq!(
            css.matches("border-radius: var(--fandhe-radius-full, 999px);")
                .count(),
            4
        );
    }

    #[test]
    fn stylesheet_links_thumb_to_hover_and_transition() {
        let css = stylesheet();
        // イシュー #1505: outline 表現の白面サムは `hover_bg_muted`
        // （淡い面変化）を選び、solid 面向け `hover_bg_solid`
        // （`--fandhe-palette-emphasized` 参照）は使わない契約を固定する。
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(css
            .contains(r#"[data-scope="slider"][data-part="thumb"]:hover:not([data-disabled]) {"#));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm);"));
        // ドラッグ追従の即時性を保つため、位置プロパティ
        // （`transform`/`left`/`top`）を `transition-property` へ含めない。
        assert!(css.contains("transition-property: background, border-color, box-shadow;"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-slider-track-height"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = Slider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = Slider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-slider--size-md"));
        assert!(html.contains("fd-slider--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = Slider::default();
        for (size, class) in [
            (Size::Xs, "fd-slider--size-xs"),
            (Size::Sm, "fd-slider--size-sm"),
            (Size::Md, "fd-slider--size-md"),
            (Size::Lg, "fd-slider--size-lg"),
            (Size::Xl, "fd-slider--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                &s,
                &SliderProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = Slider::default();
        for (palette, class) in [
            (ColorPalette::Accent, "fd-slider--color-palette-accent"),
            (ColorPalette::Info, "fd-slider--color-palette-info"),
            (ColorPalette::Success, "fd-slider--color-palette-success"),
            (ColorPalette::Warning, "fd-slider--color-palette-warning"),
            (ColorPalette::Danger, "fd-slider--color-palette-danger"),
            (ColorPalette::Neutral, "fd-slider--color-palette-neutral"),
        ] {
            let html = render(&root(
                Size::Md,
                palette,
                &s,
                &SliderProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = Slider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = Slider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- range/thumb: --fandhe-slider-percent の唯一の動的値経路 ---

    #[test]
    fn range_outputs_percent_style() {
        let s = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
        let html = render(&range(&s, &SliderProps::default(), vec![]));
        assert!(html.contains(r#"style="--fandhe-slider-percent: 25%""#));
    }

    #[test]
    fn thumb_styled_outputs_percent_style() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&thumb_styled(&s, None, &SliderProps::default(), vec![]));
        assert!(html.contains(r#"style="--fandhe-slider-percent: 40%""#));
        assert!(html.contains(r#"role="slider""#));
    }

    #[test]
    fn range_caller_style_attr_is_dropped_not_duplicated() {
        let s = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
        let html = render(&range(
            &s,
            &SliderProps::default(),
            vec![("style", "attacker: 1")],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn thumb_styled_caller_style_attr_is_dropped_not_duplicated() {
        let s = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
        let html = render(&thumb_styled(
            &s,
            None,
            &SliderProps::default(),
            vec![("style", "attacker: 1")],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- marker-group/marker（イシュー #2020） ---

    #[test]
    fn marker_group_outputs_scope_and_part() {
        let html = render(&marker_group(vec![], vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="marker-group""#));
    }

    #[test]
    fn marker_outputs_scope_and_part() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(&s, 50.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="marker""#));
    }

    #[test]
    fn marker_outputs_marker_percent_style_at_min_and_max() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);

        let at_min = render(&marker(&s, 0.0, false, vec![], vec![]));
        assert!(at_min.contains(r#"style="--fandhe-slider-marker-percent: 0%""#));

        let at_max = render(&marker(&s, 100.0, false, vec![], vec![]));
        assert!(at_max.contains(r#"style="--fandhe-slider-marker-percent: 100%""#));
    }

    #[test]
    fn marker_percent_style_clamps_out_of_range_value() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(&s, 200.0, false, vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-slider-marker-percent: 100%""#));
    }

    /// `value` に `NaN` を渡すと headless 側 [`Slider::marker`] は `min` へ
    /// フォールバックする（`data-value`/`data-state` が `min` 基準で正しく
    /// 算出される）。styled 層の `--fandhe-slider-marker-percent` もこの
    /// フォールバックへ追随し、`NaN%` のような無効な CSS カスタムプロパティ
    /// を出力しないことを固定する（Review 指摘、イシュー #2020）。
    #[test]
    fn marker_percent_style_falls_back_to_min_on_nan_value() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(&s, f64::NAN, false, vec![], vec![]));
        assert!(!html.contains("NaN"));
        assert!(html.contains(r#"style="--fandhe-slider-marker-percent: 0%""#));
    }

    /// `min`/`max` が非有限、または `min >= max` の場合は既定 `(0.0, 100.0)`
    /// へフォールバックする（headless 側と同じ規約）。
    #[test]
    fn marker_percent_style_falls_back_to_default_range_on_invalid_min_max() {
        assert_eq!(
            marker_percent_style(f64::NAN, 100.0, 50.0),
            "--fandhe-slider-marker-percent: 50%"
        );
        assert_eq!(
            marker_percent_style(0.0, f64::INFINITY, 50.0),
            "--fandhe-slider-marker-percent: 50%"
        );
        assert_eq!(
            marker_percent_style(10.0, 10.0, 5.0),
            "--fandhe-slider-marker-percent: 5%"
        );
    }

    #[test]
    fn marker_reflects_headless_data_value_and_state() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(&s, 20.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-value="20""#));
        assert!(html.contains(r#"data-state="under-value""#));
    }

    #[test]
    fn marker_disabled_true_adds_data_disabled() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(&s, 20.0, true, vec![], vec![]));
        assert!(html.contains("data-disabled"));
    }

    #[test]
    fn marker_caller_style_attr_is_dropped_not_duplicated() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(
            &s,
            20.0,
            false,
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn marker_attrs_attribute_breakout_payload_is_escaped() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
        let html = render(&marker(
            &s,
            20.0,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn marker_group_children_are_escaped_on_render() {
        let html = render(&marker_group(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = Slider::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            &SliderProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, "40", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_slider_state_machine() {
        // `Slider` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Slider` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = Slider::new(0.0, 100.0, 10.0, 20.0, Orientation::Horizontal);
        let ssr_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "increment", ""));
        assert_eq!(s.value(), 30.0);

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-value="30""#));

        let restored = Slider::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}
