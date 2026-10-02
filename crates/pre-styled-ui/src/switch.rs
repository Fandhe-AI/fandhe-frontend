//! styled Switch（headless ラッパー第 3 弾、イシュー #682、`size`/`palette`
//! variant 拡張はイシュー #708、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::switch`（イシュー #537/#595）の Control /
//! Thumb / Label / HiddenInput 4 anatomy パーツをそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Switch` 型・headless
//! `root` を再エクスポートしない理由、イシュー #708）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::avatar::root`]・[`crate::card::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`control`]/[`thumb`]/[`label`]/
//! [`hidden_input`]/[`SwitchAction`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::switch::Switch`] は**あえて**
//! 再エクスポートしない（[`crate::avatar`] の `Avatar` 非再エクスポートと
//! 同じ理由、イシュー #684/PR #695 Bugbot 指摘）。`Switch` は
//! `.root(disabled, attrs, children)` という inherent メソッドを持つが、
//! これは headless 自由関数 `root` へそのまま委譲するのみで `size`/
//! `palette` variant クラスを一切付与しない未スタイルの実体である。本
//! モジュールが `Switch` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`switch_instance.root(...)` を呼んでしまい、
//! `size`/`palette` が付与されず見た目が静かに崩れる事故を誘発する。
//! `Switch` による状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::switch::Switch` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` 語彙について
//!
//! headless 層は Switch を `"checked"`/`"unchecked"` 語彙（open/closed では
//! ない）で表現する（`crates/headless-ui/src/switch.rs` の
//! `crate::state::Checkable` 埋め込み参照）。`recipe` の `control`/`thumb`
//! への状態連動規則もこの語彙に合わせて `data-state="checked"` を条件とする。
//!
//! # `hidden-input` は `display: none` にしない（視覚的非表示化の判断）
//!
//! headless 層の `hidden_input` は `<input type="checkbox" role="switch">`
//! で意味論・フォーム送信・キーボード操作を担う実体であり、視覚的な見た目
//! （トラック/つまみ）は `control`/`thumb` が装飾として担う。この 2 層構造を
//! 保ちつつ `hidden_input` 自体のフォーカス・タブ順・支援技術からの到達性を
//! 失わないため、`display: none`/`visibility: hidden` ではなく
//! [`crate::select`] の `hidden-select` と同じ visually-hidden パターン
//! （`position: absolute` + 1px クリップ、PR #575 Bugbot 指摘対応の前例）を
//! 採用する。
//!
//! # `control` の `box-sizing: border-box`（PR #697 Bugbot 指摘対応）
//!
//! `control` の `width`/水平 `padding` と、checked 時の `thumb` の
//! `translateX` はいずれも border-box（`padding` を `width` に含める箱
//! モデル）を前提に値を計算している。既定の content-box のままだと
//! `width` に `padding` が加算されトラック内の実効幅がずれ、checked 時に
//! `thumb` がトラック右端まで届かない／両端の余白が不均等になる。この
//! クレート・利用側 embed にグローバルな border-box リセットは無いため、
//! `control` へ明示的に `box-sizing: border-box` を設定して自己完結させる。
//!
//! # `hidden-input` フォーカス時の `control` へのフォーカスリング反映（イシュー #709）
//!
//! `hidden-input` フォーカス時に `control` へフォーカスリングを反映する
//! 課題は、[`crate::recipe::StateCondition`] へ親子・兄弟関係の関係セレクタ
//! （`:has()`・兄弟結合子）を追加するのではなく、headless 層
//! （`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`）が
//! 出力する `data-focus-visible` 存在属性 + クライアントランタイム
//! （`fandhe-frontend-wasm-full` の focus 配線）による `root`/`control`
//! 双方への付け外しで解決する（`crates/headless-ui/src/switch.rs` の
//! フォーカスリング契約 doc 参照）。本モジュールは `control` slot へ
//! `StateCondition::Attr("data-focus-visible")` の状態規則を登録するのみで、
//! 属性の付け外し自体は headless/wasm 層の責務のまま変えない（旧版で
//! 本節に記載していた out-of-scope はこの解決により解消済み）。
//!
//! # `size`/`palette` variant（イシュー #708）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、`recipe` が登録する
//! `--fandhe-switch-track-width`/`-track-height`/`-thumb-size`/
//! `-thumb-travel`/`-label-font-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `control`/`thumb`/`label` へ伝わる。
//! `root` は `<label>` でこれらのパーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で `control`/`thumb`/`label` の寸法を切り替える。`palette`
//! （[`ColorPalette`]）は既存の [`crate::recipe::palette_scale_declarations`]
//! （chakra-ui virtual token 方式、#606）を `root` へ登録し、checked 時の
//! `control` 背景・`thumb` の色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・
//! Accent パレット相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # `control`/`thumb` の状態表現の是正（イシュー #1508、親 #1507/#1443）
//!
//! Phase 2（Themes / Forms のスタイル調整）の一環として、参考サイト
//! （chakra-ui / Radix Themes / Radix Primitives / ark-ui）基準かつ Phase 0
//! で確定した共通ビジュアル言語（イシュー #1424/#1425 の
//! [`crate::recipe`] ヘルパ・トークン）へ `control`（トラック）・`thumb`
//! （サム）を是正した。先例は checkbox（#1734、hidden-input +
//! `data-focus-visible` 構成が同型）と slider（#1777、トラック・サムの
//! radius/shadow/hover/transition が同型）。
//!
//! - **角丸**: `border-radius: 999px` 直書き → `var(--fandhe-radius-full,
//!   999px)`（slider #1777 / angle-slider #1728 と同型。フォールバックは
//!   従来リテラル値のため `--fandhe-radius-full` 未定義の既存カスタム
//!   テーマでも外観不変）
//! - **トランジション**: `transition: background 0.15s`/`transition:
//!   transform 0.15s` の shorthand 直書き →
//!   [`crate::recipe::transition_declarations`]（`MotionDuration::Fast`、
//!   150ms で従来と同値。longhand 3 宣言化により easing がトークン化され、
//!   `prefers-reduced-motion` 対応（[`crate::theme`] の duration 一括
//!   0ms 化）に載る）
//! - **hover**: `control` に `@media (hover: hover)` 経由の hover 面変化を
//!   新設（従来は皆無だった）。unchecked 面は `--fandhe-color-border` の
//!   1 段濃色として `--fandhe-color-border-emphasized` を使う
//!   （[`crate::recipe::hover_bg_muted`] の `--fandhe-color-bg-muted` は
//!   トラックより明るく hover で「薄くなる」ため不採用）。checked 面は
//!   [`crate::recipe::hover_bg_solid_with_fallback`]（palette emphasized
//!   段、未選択時は `--fandhe-color-accent-emphasized` へフォールバック）。
//!   実適用は `checkbox`/`slider` と同型の 1 本のみ
//!   （[`crate::recipe::hover_surface_declarations`]、`--fandhe-hover-bg`
//!   の間接参照経由で unchecked/checked 双方に追従）
//! - **フォーカス**: `data-focus-visible` の `outline`/`outline-offset`
//!   直書き → [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Palette`。switch は `ColorPalette` 対応部品のため
//!   palette 連動形。条件は既存の `data-focus-visible` のまま、
//!   フォールバック値は旧実装と同一のため見た目は不変）
//! - **サムの影**: `thumb` に参考サイト共通の「白面 + 影」表現
//!   （`box-shadow: var(--fandhe-shadow-sm)`）を追加（slider/angle-slider
//!   の thumb/control と同型のトークン）
//! - **disabled**: `root` の disabled 規則を canonical ヘルパ
//!   [`crate::recipe::disabled_declarations`] へ置換（値は不変、宣言順が
//!   `opacity` → `cursor` へ変わる）
//!
//! ## 意図的に参照サイトへ合わせなかった点
//!
//! - Radix Themes `classic` variant の inset shadow による立体表現・
//!   surface variant は variant 軸の新設を伴うため不採用（本イシューは
//!   既存 variant 構成を変えない是正のみを担う）
//! - hover を `data-hover` 属性ではなく CSS `:hover`
//!   （`StateCondition::Hover`）で表現する既存規約（`checkbox`/`slider`
//!   と同型）をそのまま踏襲した
//!
//! # `size` バリアントと `label` 配置の是正（イシュー #1509、親 #1507/#1443）
//!
//! 親イシュー #1507 の分割 2/2。1/2（#1508、上記節）が担わなかった残務
//! （size バリアントの寸法段階設計・`label` slot の配置）を消化する。先例は
//! checkbox（#1455、`size_variants` ヘルパへの移行・gap の size 連動・
//! label 型階層付与が同型）。
//!
//! - **size variant の一括登録**: 5 段の `.variant(Size::*, "root", ...)`
//!   を個別に手書きする代わりに [`crate::recipe::SlotRecipe::size_variants`]
//!   （既定 `md` の設定漏れを構造的に防ぐ共通生成手段、規約は
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4）を
//!   使うよう書き換えた。挙動・出力 CSS は変わらない（純粋なリファクタ）。
//! - **寸法値そのものは据え置き**: 参考スクショ（`docs/design/
//!   reference-screenshots/` 配下の switch 関連 PNG・`themes-switch.png`）
//!   と比較した結果、既存の `--fandhe-switch-track-width`/`-track-height`/
//!   `-thumb-size`/`-thumb-travel` は (1) `track-width = 2 × thumb-size +
//!   2 × padding(0.15rem)`・`thumb-travel = thumb-size`・`track-height =
//!   thumb-size + 2 × padding` の相互依存不変条件を既に満たしており、
//!   (2) `track-width`（1.5/2/2.5/3rem）は chakra switch の xs/sm/md/lg と
//!   一致済みであるため、明確な是正動機が見つからなかった。checkbox #1455
//!   の保守的方針（乖離が明確な段のみ是正し既定 md の外観は極力維持する）
//!   に倣い、寸法値の変更は行わない。padding `0 0.15rem` も checkbox の
//!   `margin-bottom: 0.1rem` と同じ「スケール外の光学調整値」として現状
//!   維持する。
//! - **root の `gap` を size 連動に**: `--fandhe-switch-gap` の root
//!   base custom property（フォールバック `var(--fandhe-space-2)`、既定
//!   md の見た目を維持）を新設し、`recipe` の `size_variants` で xs〜xl
//!   の spacing トークン（`--fandhe-space-1`/`-1-5`/`-2`/`-2-5`/`-3`）を
//!   単調増加で割り当てる（checkbox #1455 と同一の spacing トークン列）。
//! - **label に型階層を追加**: 従来 `font-size` 1 宣言のみだった `label`
//!   base へ checkbox #1455 と同一語彙（`font-weight: medium`・
//!   `line-height: normal`・`color: fg`・`user-select: none`）を追加した。
//!
//! ## 意図的に参照サイトへ合わせなかった点（#1509 分）
//!
//! - `xl` サイズは chakra（xs〜lg の 4 段）に対する過剰分だが、リポジトリ
//!   横断の 5 段 [`Size`] 語彙（checkbox 等の他部品と同一構成）に合わせた
//!   意図的な超過であり、削除しない。
//! - chakra switch の `variant`（solid/raised）軸は追加しない（1/2 と同じ
//!   判断。横断判断はイシュー #1741 で
//!   `docs/design/pre-styled-ui-size-and-color-palette-axes.md` §7 に
//!   記録済み: 現時点では見送り）。
//! - label の左右配置切り替え（label 先行レイアウト）用の専用 variant は
//!   追加しない。`root` は flex コンテナであり、anatomy の子要素（`label`/
//!   `control`）の記述順は呼び出し側の責務のため、既存構成のままで表現
//!   できる。
//!
//! # shadcn/ui との突合（イシュー #2021）
//!
//! [shadcn/ui Switch](https://ui.shadcn.com/docs/components/base/switch)
//! （`docs/design/shadcn-reference-adoption-policy.md` §8 の主基準の 1 つ）
//! と突合し、chakra-ui / Radix Themes 基準の既存調整（#1508/#1509）では
//! 拾えなかった欠落を補完した（先例は checkbox #2011）。
//!
//! ## 確認したが是正不要と判断した点
//!
//! - **`size`（xs/sm/md/lg/xl）**: shadcn/ui の Switch は単一サイズのみで
//!   size variant を持たない。本部品は既に chakra-ui 基準で xs〜xl の 5 段
//!   （`SlotRecipe::size_variants`）を備え、寸法の相互不変条件（#1509 節
//!   参照）を満たした状態で確定済みのため、shadcn 側に合わせて縮退させる
//!   動機がない。
//! - **`data-readonly`/`data-required`**: 参照サイト（shadcn 含む）いずれも
//!   readonly のみでの視覚差を持たないため、checkbox 等の他部品と同じ理由
//!   で意図的に視覚差を付けない。
//!
//! ## 是正した点
//!
//! - **`data-invalid`**: `SwitchProps.invalid`（`crates/headless-ui/src/
//!   switch.rs`、#1622）が全パーツへ出す `data-invalid` を `control` slot
//!   が未消費だった非対称を解消し、`outline` の外側リングで視覚差を
//!   追加した（`forced-colors: active` でも消えない理由・当初 `box-shadow`
//!   実装からの是正経緯は下記 `recipe` 内コメント参照）。
//!
//! ## Examples 節の合成パターン補完（コード変更なし）
//!
//! shadcn/ui の Examples（label + description の合成、枠付きボックス内
//! での利用）に相当するデモが docs サイトの Examples 節（`crates/docs-site/
//! src/component_specs/forms.rs` の `SWITCH` 定数）に 1 件も無かったため
//! 追加した。checkbox #2011 の `checkbox_with_description_example` と同型の
//! 呼び出し側合成（専用 anatomy パート・新規コンポーネントは追加しない）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - [`crate::stylesheet::StyleSheet`] の
//!   `push_recipe_is_infallible_for_all_styled_components` テストへの
//!   popover/tooltip（#664）の登録漏れは #707 で解消済み。
//! - tabs/accordion/dialog/menu/select への size（および tabs への
//!   palette）展開は本イシューの方針を第 2 弾として別途適用する
//!   （`docs/api/pre-styled-ui-api.md` の variant 表参照）。
//! - shadcn 3 枚目のスクショに相当する専用複合コンポーネント（`switch_card`
//!   等）の新設は本イシューのスコープ外（`docs/policy/
//!   intentional-non-adoption.md` §3.25 の責務境界・「純追加」方針を踏まえ、
//!   新規コンポーネント新設は別イシューの判断とすべき規模）。plain な
//!   `<div>` + 既存 `switch` パーツの合成で Examples 側から再現する。
//!
//! # 短いトラック形状・thumb アイコン slot（イシュー #3141）
//!
//! Blocks 取り込みの対応表（R1375/R1376）で「既存 `switch` では表現できない」
//! と判定された 2 点を opt-in の純追加で補った。
//!
//! - **[`SwitchTrack`]（R1375）**: つまみより細く短いトラック形状。`root`
//!   の `--fandhe-switch-track-width`/`-track-height`/`-thumb-offset` を
//!   `--fandhe-switch-thumb-size`（size variant 由来）からの `calc()` で
//!   再定義し、`control` の実効幅に対してつまみを上下左右へはみ出させる。
//!   [`Shape`](crate::recipe::Shape) と同じ理由で `default_variant` を登録
//!   しないため、`SwitchTrack::Short` を指定しない既存呼び出し元（[`root`]）
//!   の class 出力・golden CSS はバイト不変。
//! - **[`thumb_icon`]（R1376）**: `thumb` の中央に重ねる on/off アイコン
//!   slot。`data-state`（switch 自体の状態）と `data-show`（このアイコンが
//!   どちらの状態で表示されるか）の組み合わせで opacity を切り替える。
//!   装飾のため `aria-hidden="true"` を固定し、状態の意味論は既存の
//!   `hidden-input`（`role="switch"`）が担ったまま変えない。
//!
//! いずれも既存 anatomy パーツのシグネチャ・出力バイトを変更しない純追加
//! のため `crates/pre-styled-ui/Cargo.toml` は minor バンプする。RTL での
//! `thumb` 移動方向は既存の `translateX` 実装と同じく非対応のまま
//! （本イシューのスコープ外）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_solid_with_fallback,
    hover_surface_declarations, palette_scale_declarations, transition_declarations, ColorPalette,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// `Switch` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::switch::Switch`
// を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::switch::{
    control, hidden_input, label, thumb, SwitchAction, SwitchProps,
};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="switch"` を固定した本モジュール独自パート（`thumb-icon`）用の
/// anatomy（イシュー #3141）。headless-ui 側の `switch::ANATOMY`
/// （`crates/headless-ui/src/switch.rs`）とは別インスタンスだが `scope`
/// 文字列は同一値であり、出力される `data-scope` 属性値は一致する
/// （[`crate::avatar::ANATOMY`] と同型）。
const ANATOMY: Anatomy = anatomy("switch");

/// headless `switch` anatomy の `data-part` 一覧（`crates/headless-ui/src/switch.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。イシュー #3141 で pre-styled-only
/// 1 パート（`thumb-icon`。headless-ui の anatomy には存在せず、本モジュール
/// だけが出力する）を末尾へ追加し、計 6 件になった（[`crate::avatar`] の
/// `group`/`badge` と同型）。
const SLOTS: &[&str] = &[
    "root",
    "control",
    "thumb",
    "label",
    "hidden-input",
    "thumb-icon",
];

/// この styled Switch の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("switch", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                // イシュー #1509: `gap` を size 連動 custom property 化
                // （`--fandhe-switch-gap`、checkbox #1455 と同型）。
                // フォールバックは従来の固定値 `var(--fandhe-space-2)`
                // のため、styled `root` 非経由の headless 直接利用でも
                // 現行外観（md 相当）を維持する。
                decl("gap", "var(--fandhe-switch-gap, var(--fandhe-space-2))"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            // イシュー #1508: `cursor`/`opacity` 直書きを共通ビジュアル言語
            // （イシュー #1425、`crate::recipe` 冒頭 doc「disabled / hover /
            // transition の共通ビジュアル言語」節）へ置換。宣言順は
            // `opacity` → `cursor` に変わるが値そのものは不変のため見た目に
            // 差分は出ない。
            disabled_declarations(),
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-switch-track-width, 2.5rem)"),
                decl("height", "var(--fandhe-switch-track-height, 1.4rem)"),
                // イシュー #1508: 角丸をトークン化（`999px` リテラル →
                // `var(--fandhe-radius-full, 999px)`。slider #1777 /
                // angle-slider #1728 と同型の是正。フォールバックは従来
                // リテラル値のため `--fandhe-radius-full` 未定義の既存
                // カスタムテーマでも見た目は不変）。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl("background", "var(--fandhe-color-border)"),
                decl("padding", "0 0.15rem"),
                // unchecked 時の hover 面（イシュー #1425、`crate::checkbox`
                // の `control` と同型のパターン）。トラック base 背景
                // `--fandhe-color-border` の 1 段濃色として
                // `--fandhe-color-border-emphasized` を使う（`hover_bg_muted`
                // の `--fandhe-color-bg-muted` はトラックより明るく hover で
                // 「薄くなって」しまうため不採用）。checked 時は下記 state
                // 規則が同名カスタムプロパティを上書きし、hover セレクタ側は
                // `hover_surface_declarations()` 1 本のみで両方の面色に
                // 追従する。
                decl("--fandhe-hover-bg", "var(--fandhe-color-border-emphasized)"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（`checkbox.rs`/
        // `slider.rs` の transition 追加と同型のパターン、イシュー #1425
        // 参照実装）。
        .base(
            "control",
            transition_declarations("background", MotionDuration::Fast),
        )
        .state(
            "control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // checked 面の hover は palette の emphasized 段へ
                // （`checkbox` の checked `control` と同型、モジュール
                // rustdoc 参照）。`hover_bg_solid_with_fallback` は
                // `--fandhe-palette-emphasized` 未定義時も
                // `--fandhe-color-accent-emphasized` へ確実にフォールバック
                // する（styled `root` 非経由の headless 直接利用でも hover
                // 面が消えない fail-safe）。
                hover_bg_solid_with_fallback(),
            ],
        )
        // イシュー #2021: shadcn/ui との突合で headless 層が出す
        // `data-invalid`（`SwitchProps.invalid`、`crates/headless-ui/src/
        // switch.rs`）が control slot で未消費だったことを確認し追加した。
        // `checkbox`/`radio_group` の invalid 表現は `border-color` 上書き
        // だが、`control`（トラック）は `border` 宣言を base に一切
        // 持たない塗りつぶし pill であり、`box-sizing: border-box` で
        // `track-width = 2 × thumb-size + 2 × padding(0.15rem)` /
        // `thumb-travel = thumb-size` の相互不変条件（#1508/#1509
        // モジュール rustdoc 参照）を満たした実効幅で成立している。ここへ
        // `border` を新設すると border-box の実効幅・thumb の translateX
        // 整合に影響しうるため、`border`/`outline` いずれも box 寸法へ
        // 参加しない点は共通だが、後者を採用する（`outline` はフォーカス
        // リング規約（#1424、`docs/design/
        // pre-styled-ui-focus-ring-and-size-conventions.md` §3・
        // `recipe::focus_ring_declarations` doc 参照）が「新規に
        // `box-shadow` によるリングを追加しない」と確定させた理由と同じ
        // く、`forced-colors: active` で `outline` の色はシステム色へ
        // 強制置換され必ず描画されるが `box-shadow` は除去され消える
        // ため。Bugbot 指摘〔PR #2169〕を機に、当初実装していた
        // `box-shadow` 版から本実装へ是正した）。下記 `data-focus-visible`
        // 状態も同じ `control` slot へ `outline` を宣言するが、両状態が
        // 同時成立（invalid かつ focus-visible）する場合は後続登録の
        // `data-focus-visible` 側が同一プロパティを上書きしフォーカス
        // リングが優先表示される（`thumb` は既に別 slot で
        // `box-shadow: var(--fandhe-shadow-sm)` を持つが slot が異なり
        // 衝突しない）。
        .state(
            "control",
            StateCondition::Attr("data-invalid"),
            vec![
                decl("outline", "2px solid var(--fandhe-color-danger)"),
                decl("outline-offset", "2px"),
            ],
        )
        // イシュー #709: 実フォーカスは hidden-input が受けるため、wasm 層
        // （`fandhe-frontend-wasm-full` の focus 配線）が `control` へも
        // 付け外しする `data-focus-visible` をキーボード操作専用のフォーカス
        // リング条件として使う（`checkbox` の `control`
        // `StateCondition::Attr("data-focus-visible")` と同型の視覚言語、
        // モジュール rustdoc 参照）。イシュー #1508 でリング宣言を canonical
        // ヘルパ（`recipe::focus_ring_declarations`）へ置換し、`palette` 軸
        // を持つ本部品ではリング色も選択中 palette へ連動させる
        // （`FocusRingColor::Palette`）。フォールバック値は旧実装と同じ
        // `2px`/`var(--fandhe-color-accent)` のため、新トークン未定義の
        // 既存カスタムテーマでも見た目は不変。
        .state(
            "control",
            StateCondition::Attr("data-focus-visible"),
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // hover の実適用は 1 本のみ（`--fandhe-hover-bg` の間接参照経由で
        // unchecked/checked いずれの面色にも追従する。`checkbox` の
        // `control` hover と同型のパターン）。
        .state(
            "control",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .base(
            "thumb",
            vec![
                decl("width", "var(--fandhe-switch-thumb-size, 1.1rem)"),
                decl("height", "var(--fandhe-switch-thumb-size, 1.1rem)"),
                // イシュー #1508: `control` と同じ角丸トークン化。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl("background", "var(--fandhe-color-bg)"),
                // 参照 4 サイト（chakra-ui/Radix Themes/Radix Primitives/
                // ark-ui）共通の「白面サム + 影による浮き上がり」表現
                // （`slider`/`angle-slider` の thumb/control と同型の
                // トークン）。
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
            ],
        )
        .base(
            "thumb",
            // イシュー #1508: `transform` は `--fandhe-switch-thumb-travel`
            // 由来の checked 移動を含むため transition から外さない
            // （slider の `left`/`top` 除外〔ドラッグ追従の遅延回避〕とは
            // 異なり、switch の checked 切り替えは離散的なトグルであり
            // 追従遅延の懸念がないため、旧実装どおり `transform` へ
            // トランジションを掛ける）。`prefers-reduced-motion` 対応は
            // `transition_declarations` の呼び出し先（`Theme::to_css` の
            // duration 一括 0ms 化）が担う。
            transition_declarations("transform", MotionDuration::Fast),
        )
        .state(
            "thumb",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl(
                "transform",
                "translateX(var(--fandhe-switch-thumb-travel, 1.1rem))",
            )],
        )
        .base(
            "label",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-switch-label-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1509: checkbox（#1455）の label と同一語彙の
                // 型階層を追加する。`font-weight: medium` + `color: fg` で
                // 通常テキストより強調し、`line-height: normal` で複数行
                // ラベルの行送りを整え、`user-select: none` はクリックで
                // トグルする `<label>` 内テキストの誤選択を防ぐ
                // （chakra の switch label と同じ挙動）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        // hidden-input の視覚的非表示化（[`crate::select`] の `hidden-select` と
        // 同じ visually-hidden パターン。モジュール doc 参照）。
        .base(
            "hidden-input",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        // イシュー #1509: 5 段の `.variant(Size::*, "root", ...)` を個別に
        // 手書きする代わりに `size_variants`（イシュー #1424 の共通生成
        // 手段、checkbox #1455 / slider #1777 と同型）を使い、既定 `md` の
        // 設定漏れを構造的に防ぐ（規約は
        // `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §4）。寸法値自体は不変条件（`track-width = 2 × thumb-size + 2 ×
        // padding(0.15rem)` / `thumb-travel = thumb-size` / `track-height =
        // thumb-size + 2 × padding`）を既に満たし、track-width は chakra の
        // xs/sm/md/lg（1.5/2/2.5/3rem）と一致済みのため据え置く（参考
        // スクショ比較で明確な乖離が見つからなかった、モジュール rustdoc
        // 「イシュー #1509」節参照）。`--fandhe-switch-gap` は本イシューで
        // 新設した root 余白の size 連動 custom property で、checkbox と
        // 同じ spacing トークンを xs〜xl まで単調増加させる。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-switch-track-width", "1.5rem"),
                        decl("--fandhe-switch-track-height", "0.9rem"),
                        decl("--fandhe-switch-thumb-size", "0.6rem"),
                        decl("--fandhe-switch-thumb-travel", "0.6rem"),
                        decl(
                            "--fandhe-switch-label-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-switch-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-switch-track-width", "2rem"),
                        decl("--fandhe-switch-track-height", "1.15rem"),
                        decl("--fandhe-switch-thumb-size", "0.85rem"),
                        decl("--fandhe-switch-thumb-travel", "0.85rem"),
                        decl(
                            "--fandhe-switch-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-switch-gap", "var(--fandhe-space-1-5)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-switch-track-width", "2.5rem"),
                        decl("--fandhe-switch-track-height", "1.4rem"),
                        decl("--fandhe-switch-thumb-size", "1.1rem"),
                        decl("--fandhe-switch-thumb-travel", "1.1rem"),
                        decl(
                            "--fandhe-switch-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-switch-gap", "var(--fandhe-space-2)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-switch-track-width", "3rem"),
                        decl("--fandhe-switch-track-height", "1.65rem"),
                        decl("--fandhe-switch-thumb-size", "1.35rem"),
                        decl("--fandhe-switch-thumb-travel", "1.35rem"),
                        decl(
                            "--fandhe-switch-label-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-switch-gap", "var(--fandhe-space-2-5)"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-switch-track-width", "3.5rem"),
                        decl("--fandhe-switch-track-height", "1.9rem"),
                        decl("--fandhe-switch-thumb-size", "1.6rem"),
                        decl("--fandhe-switch-thumb-travel", "1.6rem"),
                        decl(
                            "--fandhe-switch-label-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl("--fandhe-switch-gap", "var(--fandhe-space-3)"),
                    ],
                ),
            ],
        )
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

    // イシュー #3141 (R1375): 短いトラック形状。`control`/`thumb` の実寸法は
    // size variant が設定する `--fandhe-switch-thumb-size` 系 custom
    // property のみに依存するため、root scope への 1 宣言のみで `control`/
    // `thumb` へ継承させる（size variant と同じ「root の custom property
    // 経由」の表現手段、[`SlotRecipe`] は子孫セレクタ機構を持たないため）。
    // トラック幅 = つまみサイズ T の 1.8 倍・高さ = 0.8 倍、`control` の
    // 既存 `padding: 0 0.15rem` を相殺するオフセットを `thumb` の
    // `margin-left` へ加えると、checked 時の移動量（1.8T - T + 0.2T = T）が
    // 既存 `--fandhe-switch-thumb-travel`（全 size 段で T と同値）と一致し、
    // 上書き不要になる。palette variant（この for 文）より後に登録する
    // ことで、同じ詳細度で size variant（`default_variant` より前に登録
    // 済み）のトラック寸法宣言を上書きできる。`Short` を指定しない既存
    // 呼び出し元は `variant_classes` が本 axis を選択しないため class・
    // golden CSS ともバイト不変（モジュール冒頭 rustdoc 参照）。
    recipe = recipe
        .variant(
            SwitchTrack::Short,
            "root",
            vec![
                decl(
                    "--fandhe-switch-track-width",
                    "calc(var(--fandhe-switch-thumb-size, 1.1rem) * 1.8)",
                ),
                decl(
                    "--fandhe-switch-track-height",
                    "calc(var(--fandhe-switch-thumb-size, 1.1rem) * 0.8)",
                ),
                decl(
                    "--fandhe-switch-thumb-offset",
                    "calc(-0.15rem - var(--fandhe-switch-thumb-size, 1.1rem) * 0.1)",
                ),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 既存 2 件の `thumb` base ブロックを書き換えずに純追加する（上記
        // transition 追加と同型のパターン）。`margin-left` の既定値は
        // `0`（フォールバック）のため、`SwitchTrack::Short` 非経由では
        // 見た目は不変。`position: relative` は下記 `thumb-icon` の絶対
        // 配置の基準にもなる。
        .base(
            "thumb",
            vec![
                decl("position", "relative"),
                decl("margin-left", "var(--fandhe-switch-thumb-offset, 0)"),
            ],
        )
        // イシュー #3141 (R1376): thumb アイコン slot。`data-state`（switch
        // 自体の checked/unchecked）と `data-show`（このアイコンが表示対象
        // とする状態、[`thumb_icon`] 参照）の両方が一致したときのみ
        // 不透明にする（不一致の 2 パターンを明示的に `opacity: 0` にし、
        // 一致せず宣言も無い既定状態は CSS 既定値 `opacity: 1` のまま
        // 隠さない。checked かつ show=checked のときだけ on アイコンを
        // palette 色にする）。
        .base(
            "thumb-icon",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl(
                    "color",
                    "var(--fandhe-switch-thumb-icon-color, var(--fandhe-color-fg-muted))",
                ),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "thumb-icon",
            transition_declarations("opacity", MotionDuration::Fast),
        )
        .state(
            "thumb-icon",
            StateCondition::AttrEqAll(&[("data-state", "checked"), ("data-show", "unchecked")]),
            vec![decl("opacity", "0")],
        )
        .state(
            "thumb-icon",
            StateCondition::AttrEqAll(&[("data-state", "unchecked"), ("data-show", "checked")]),
            vec![decl("opacity", "0")],
        )
        .state(
            "thumb-icon",
            StateCondition::AttrEqAll(&[("data-state", "checked"), ("data-show", "checked")]),
            vec![decl(
                "color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        );

    recipe
}

/// 短いトラック形状 variant（イシュー #3141、R1375）。つまみより細く短い
/// トラックで、つまみがトラックの上下左右からはみ出す形状を表現する。
/// `size`/[`ColorPalette`] と異なり `default_variant` を各部品側で登録
/// しない契約（[`crate::recipe::Shape`] の先例と同型）のため、`Short` を
/// 指定しない既存呼び出し元の class 出力・golden CSS はバイト不変のまま
/// 保たれる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchTrack {
    /// 既定のトラック形状（`control` の size variant がそのまま寸法を決める）。
    Default,
    /// 短いトラック形状（`recipe` 内コメント参照、R1375）。
    Short,
}

impl VariantValue for SwitchTrack {
    fn axis(self) -> &'static str {
        "track"
    }

    fn value(self) -> &'static str {
        match self {
            SwitchTrack::Default => "default",
            SwitchTrack::Short => "short",
        }
    }
}

/// [`thumb_icon`] が表示対象とする状態（イシュー #3141、R1376）。呼び出し
/// 側は `checked`/`unchecked` 用の 2 個の [`thumb_icon`] を `thumb` の子
/// として両方配置し、switch 自体の状態（`checked` 引数）と組み合わせて
/// どちらか一方だけが見えるよう CSS（`recipe` 内コメント参照）が切り替える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbIconShow {
    /// checked 状態でのみ表示する（on 用）アイコン。
    Checked,
    /// unchecked 状態でのみ表示する（off 用）アイコン。
    Unchecked,
}

impl ThumbIconShow {
    fn value(self) -> &'static str {
        match self {
            ThumbIconShow::Checked => "checked",
            ThumbIconShow::Unchecked => "unchecked",
        }
    }
}

/// [`thumb_icon`] が全パーツへ一律付与する属性キー一覧（呼び出し側 `attrs`
/// からの偽装を fail-closed で除去する対象。`crate::checkbox_card` の
/// `STATE_RESERVED`/`drop_reserved` と同型）。
///
/// イシュー #3141 codex-review 指摘（P1）対応: `data-disabled`/
/// `data-invalid`/`data-required`/`data-readonly` も [`SwitchProps`] から
/// 全パーツへ一律反映する既存契約（`control`/`thumb` 等、headless
/// `state_attrs` 参照）に合わせ、`thumb_icon` のみ欠落していた 4 属性を
/// 予約キーへ追加する。
const THUMB_ICON_RESERVED: &[&str] = &[
    "data-state",
    "data-show",
    "aria-hidden",
    "data-disabled",
    "data-invalid",
    "data-required",
    "data-readonly",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crate::checkbox_card::drop_reserved` と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// この styled Switch が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`]/[`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（`drop_class_attr` により呼び出し側の `class` は除去して
/// から合成する）。実体は [`fandhe_frontend_headless_ui::switch::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::switch::{self, SwitchAction as _, SwitchProps};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = switch::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     &SwitchProps::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="switch" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    root_with(
        size,
        palette,
        SwitchTrack::Default,
        checked,
        props,
        attrs,
        children,
    )
}

/// styled root パーツを組み立てる（[`root`] の `track` 軸付き版、イシュー
/// #3141）。[`root`] は `track: SwitchTrack::Default` で本関数へ委譲する
/// だけの薄いラッパーであり、`SwitchTrack::Default` を渡した場合の出力は
/// [`root`] とバイト一致する（`track` 軸は `variant_classes` が選択時のみ
/// class を emit するため、`Default` では axis 自体が選択列に現れない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::switch::{self, SwitchAction as _, SwitchProps, SwitchTrack};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = switch::root_with(
///     Size::Md,
///     ColorPalette::Accent,
///     SwitchTrack::Short,
///     false,
///     &SwitchProps::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains("fd-switch--track-short"));
/// ```
#[must_use]
pub fn root_with<'a>(
    size: Size,
    palette: ColorPalette,
    track: SwitchTrack,
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut selection: Vec<(&str, &str)> =
        vec![("size", size.value()), ("color-palette", palette.value())];
    if track == SwitchTrack::Short {
        selection.push(("track", track.value()));
    }
    let class = recipe.variant_classes(&selection);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::switch::root(checked, props, merged, children)
}

/// つまみ中央に重ねる on/off アイコン slot（イシュー #3141、R1376）。
/// [`thumb`] の子として `checked`/`unchecked` 用を 1 個ずつ配置する想定
/// （`recipe` 内コメント参照）。装飾のため `aria-hidden="true"` を固定し、
/// 呼び出し側 `attrs` からの `data-state`/`data-show`/`aria-hidden`/
/// `data-disabled`/`data-invalid`/`data-required`/`data-readonly` 偽装は
/// 除去する（`data-scope`/`data-part` の偽装は [`Anatomy::part`] が除去）。
///
/// `props`（[`SwitchProps`]）は `control`/`thumb` 等の既存パーツと同じく
/// `data-disabled`/`data-invalid`/`data-required`/`data-readonly` を全パーツ
/// 一律反映する契約（headless `state_attrs` 参照）に揃えるために受け取る
/// （イシュー #3141 codex-review 指摘〔P1〕対応: 新設時に本属性が欠落して
/// いた）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps, ThumbIconShow};
///
/// let node = switch::thumb_icon(
///     false,
///     ThumbIconShow::Unchecked,
///     &SwitchProps::default(),
///     vec![],
///     vec![],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="switch" data-part="thumb-icon""#));
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn thumb_icon<'a>(
    checked: bool,
    show: ThumbIconShow,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-state", if checked { "checked" } else { "unchecked" }),
        ("data-show", show.value()),
        ("aria-hidden", "true"),
    ];
    merged.extend(fandhe_frontend_headless_ui::data_attrs::data_disabled(
        props.disabled,
    ));
    merged.extend(fandhe_frontend_headless_ui::data_attrs::data_invalid(
        props.invalid,
    ));
    merged.extend(fandhe_frontend_headless_ui::data_attrs::data_required(
        props.required,
    ));
    merged.extend(fandhe_frontend_headless_ui::data_attrs::data_readonly(
        props.readonly,
    ));
    merged.extend(drop_reserved(attrs, THUMB_ICON_RESERVED));
    ANATOMY.part("thumb-icon", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="switch"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_and_thumb_to_checked_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"][data-state="checked"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb"][data-state="checked"] {
  transform: translateX(var(--fandhe-switch-thumb-travel, 1.1rem));
}"#
        ));
    }

    #[test]
    fn stylesheet_links_control_to_data_invalid_state() {
        // イシュー #2021: shadcn/ui との突合で headless 層が出す
        // `data-invalid` が control slot で未消費だったことを確認し追加した
        // （`checkbox` の同名テストと同型）。PR #2169 の Bugbot 指摘を受け、
        // `forced-colors: active` で消える `box-shadow` から、システム色へ
        // 強制置換され必ず描画される `outline` へ是正した（フォーカス
        // リング規約 #1424 と同じ理由、[`recipe`] 内コメント参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"][data-invalid] {
  outline: 2px solid var(--fandhe-color-danger);
  outline-offset: 2px;
}"#
        ));
    }

    #[test]
    fn stylesheet_registers_control_hover_inside_hover_media_query() {
        // イシュー #1508: タッチ端末の hover 貼り付き対策として
        // `@media (hover: hover)` 配下へ集約される 1 本のみの hover 規則。
        // `--fandhe-hover-bg` 経由の間接参照により unchecked/checked いずれの
        // 面色にも追従する（`checkbox`/`slider` と同型、モジュール
        // rustdoc「イシュー #1508」節参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }"#
        ));
    }

    #[test]
    fn control_focus_visible_ring_uses_palette_connected_focus_ring_token() {
        // イシュー #1508: `outline`/`outline-offset` 直書きから
        // `focus_ring_declarations(FocusRingColor::Palette, ...)` へ置換。
        // `--fandhe-color-focus-ring` → `--fandhe-color-accent` の
        // フォールバック連鎖を経由し、`Theme::empty()` ベースの既存テーマ
        // でもフォーカスリングが消えないことを固定する。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"][data-focus-visible] {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}"#
        ));
    }

    #[test]
    fn thumb_has_shadow_and_tokenized_radius() {
        // イシュー #1508: 参考 4 サイト共通の「白面 + 影」表現。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb"] {
  width: var(--fandhe-switch-thumb-size, 1.1rem);
  height: var(--fandhe-switch-thumb-size, 1.1rem);
  border-radius: var(--fandhe-radius-full, 999px);
  background: var(--fandhe-color-bg);
  box-shadow: var(--fandhe-shadow-sm);
}"#
        ));
    }

    #[test]
    fn control_uses_border_box_so_thumb_travel_matches_track_bounds() {
        // Cursor Bugbot 指摘（PR #697, review 3636964684）対応の回帰:
        // `control` の `width`/`padding` と `thumb` の `translateX` は
        // border-box を前提に計算されている。`box-sizing: border-box` が
        // 欠けると content-box 既定によりつまみの移動量とトラック内幅が
        // ずれる（checked 時につまみが手前で止まる／両端の余白が不均等）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: var(--fandhe-switch-track-width, 2.5rem);"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="switch"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_is_visually_hidden_not_display_none() {
        // フォーカス・フォーム送信・支援技術の到達性を保つため
        // `display: none` を使わないことをモジュール doc 通りに固定する
        // （フォーカス到達性の回帰防止）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="switch"][data-part="hidden-input"] {"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(!css.contains("display: none"));
    }

    // --- variant クラス（イシュー #708） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-switch--size-md"));
        assert!(html.contains("fd-switch--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-switch--size-xs"),
            (Size::Sm, "fd-switch--size-sm"),
            (Size::Md, "fd-switch--size-md"),
            (Size::Lg, "fd-switch--size-lg"),
            (Size::Xl, "fd-switch--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                &SwitchProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-switch--color-palette-accent"),
            (ColorPalette::Info, "fd-switch--color-palette-info"),
            (ColorPalette::Success, "fd-switch--color-palette-success"),
            (ColorPalette::Warning, "fd-switch--color-palette-warning"),
            (ColorPalette::Danger, "fd-switch--color-palette-danger"),
            (ColorPalette::Neutral, "fd-switch--color-palette-neutral"),
        ] {
            let html = render(&root(
                Size::Md,
                palette,
                false,
                &SwitchProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-switch-track-width"));
    }

    #[test]
    fn size_variants_set_label_font_size_custom_property() {
        // Cursor Bugbot 指摘（PR #719 レビュー）対応の回帰: `label` の base
        // 規則が参照する `--fandhe-switch-label-font-size` を各 size
        // variant が設定していないと、control 自体はスケールしてもラベル
        // 文字サイズがフォールバック（sm 相当）のまま変わらない
        // （`radio_group.rs` の `--fandhe-radio-group-font-size` と対称の
        // 契約）。
        let css = stylesheet();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="switch"][data-part="root"].fd-switch--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            assert!(
                css[start..block_end].contains("--fandhe-switch-label-font-size"),
                "size={size:?} variant block missing --fandhe-switch-label-font-size: {}",
                &css[start..block_end]
            );
        }
    }

    /// イシュー #1509: label が checkbox（#1455）と同一語彙の型階層
    /// （medium font-weight・前景色・行送り・誤選択防止）を持つことを固定する。
    #[test]
    fn label_has_typography_hierarchy_declarations() {
        let css = stylesheet();
        let selector = r#"[data-scope="switch"][data-part="label"]"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("label base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
        assert!(
            block.contains("font-weight: var(--fandhe-font-font-weight-medium);"),
            "label block missing font-weight: {block}"
        );
        assert!(
            block.contains("line-height: var(--fandhe-font-line-height-normal);"),
            "label block missing line-height: {block}"
        );
        assert!(
            block.contains("color: var(--fandhe-color-fg);"),
            "label block missing color: {block}"
        );
        assert!(
            block.contains("user-select: none;"),
            "label block missing user-select: {block}"
        );
    }

    /// イシュー #1509: `--fandhe-switch-gap` が xs〜xl で spacing トークン
    /// 経由の単調増加になることを固定する（root 余白の size 連動）。
    #[test]
    fn size_variants_set_gap_custom_property_monotonically() {
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-1)"),
            (Size::Sm, "var(--fandhe-space-1-5)"),
            (Size::Md, "var(--fandhe-space-2)"),
            (Size::Lg, "var(--fandhe-space-2-5)"),
            (Size::Xl, "var(--fandhe-space-3)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="switch"][data-part="root"].fd-switch--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            let block = &css[start..block_end];
            let expected_decl = format!("--fandhe-switch-gap: {gap};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        // headless anatomy の fail-closed 偽装除去を styled root 経由でも
        // 継承していることの回帰（[`crate::avatar`] の同型テストに準拠）。
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        // イシュー #682: styled Switch 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー第 1・2 弾と同じ回帰）。
        let html = render(&label(
            false,
            &SwitchProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(
            PAYLOAD,
            PAYLOAD,
            false,
            &SwitchProps::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_switch_state_machine() {
        // `Switch` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Switch` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::switch::Switch;

        let mut s = Switch::default();
        assert!(!s.is_checked());

        let ssr_html = render(&s.root(&SwitchProps::default(), vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut s, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-checked="checked""#));

        let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    // --- イシュー #3141: SwitchTrack::Short / thumb_icon ---

    #[test]
    fn root_with_default_track_matches_root_byte_for_byte_and_has_no_track_class() {
        let via_root = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        let via_root_with = render(&root_with(
            Size::Md,
            ColorPalette::Accent,
            SwitchTrack::Default,
            false,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        assert_eq!(via_root, via_root_with);
        assert!(!via_root.contains("fd-switch--track-"));
    }

    #[test]
    fn root_with_short_track_adds_single_track_class() {
        let html = render(&root_with(
            Size::Md,
            ColorPalette::Accent,
            SwitchTrack::Short,
            false,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-switch--track-short"));
        assert_eq!(html.matches("class=\"").count(), 1);
    }

    #[test]
    fn thumb_icon_outputs_scope_part_state_show_and_aria_hidden() {
        let html = render(&thumb_icon(
            true,
            ThumbIconShow::Checked,
            &SwitchProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="thumb-icon""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"data-show="checked""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn thumb_icon_reflects_switch_props_state_attrs() {
        // イシュー #3141 codex-review 指摘（P1）回帰: control/thumb 等の
        // 既存パーツと同じく SwitchProps の 4 状態が thumb-icon へも
        // 一律反映されることを固定する。
        let props = SwitchProps {
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
        };
        let html = render(&thumb_icon(
            true,
            ThumbIconShow::Checked,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn thumb_icon_drops_reserved_attribute_spoofing_case_insensitively() {
        let html = render(&thumb_icon(
            false,
            ThumbIconShow::Unchecked,
            &SwitchProps::default(),
            vec![
                ("DATA-STATE", "attacker"),
                ("data-show", "attacker"),
                ("Aria-Hidden", "false"),
                ("data-scope", "attacker"),
                ("data-part", "attacker"),
                ("data-disabled", "attacker"),
                ("data-invalid", "attacker"),
                ("data-required", "attacker"),
                ("data-readonly", "attacker"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="thumb-icon""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains(r#"data-show="unchecked""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains(r#"aria-hidden="false""#));
    }

    #[test]
    fn thumb_icon_attrs_and_children_are_escaped() {
        let html = render(&thumb_icon(
            false,
            ThumbIconShow::Unchecked,
            &SwitchProps::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn stylesheet_registers_track_short_variant_and_thumb_icon_opacity_states() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="root"].fd-switch--track-short {
  --fandhe-switch-track-width: calc(var(--fandhe-switch-thumb-size, 1.1rem) * 1.8);
  --fandhe-switch-track-height: calc(var(--fandhe-switch-thumb-size, 1.1rem) * 0.8);
  --fandhe-switch-thumb-offset: calc(-0.15rem - var(--fandhe-switch-thumb-size, 1.1rem) * 0.1);
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb-icon"][data-state="checked"][data-show="unchecked"] {
  opacity: 0;
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb-icon"][data-state="unchecked"][data-show="checked"] {
  opacity: 0;
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb-icon"][data-state="checked"][data-show="checked"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
    }

    #[test]
    fn stylesheet_still_never_contains_style_breakout_sequences_after_thumb_icon_addition() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
