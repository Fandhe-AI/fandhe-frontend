//! styled Dialog（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::dialog`（イシュー #531）の Root / Trigger /
//! Backdrop / Positioner / Content / Title / Description / CloseTrigger
//! 8 anatomy パーツを再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Dialog` 型・headless
//! `root` を再エクスポートしない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::switch::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子（[`trigger`]/
//! [`backdrop`]/[`positioner`]/[`content`]/[`title`]/[`description`]/
//! [`close_trigger`]/[`ContentIds`]/[`DialogRole`]）のみを選択的に再エクスポート
//! する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::dialog::Dialog`] は**あえて**
//! 再エクスポートしない（[`crate::switch`] の `Switch` 非再エクスポートと
//! 同じ理由、イシュー #684/PR #695 Bugbot 指摘の一般化）。`Dialog` は
//! `.root(attrs, children)` という inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size` variant クラス
//! を一切付与しない未スタイルの実体である。本モジュールが `Dialog` を丸ごと
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `dialog_instance.root(...)` を呼んでしまい、`size` が付与されず見た目が
//! 静かに崩れる事故を誘発する。`Dialog` による状態管理・hydration が必要な
//! 呼び出し側は `fandhe_frontend_headless_ui::dialog::Dialog` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]（および再エクスポート済み
//! のパーツ関数）を組み合わせて構築すること。
//!
//! # 薄い委譲の根拠（本モジュールが新たな出力経路を持たない理由）
//!
//! headless 層の [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] は
//! 各パーツへ必ず `data-scope="dialog"` / `data-part="<slot>"` を付与する
//! （呼び出し側の偽装値は fail-closed で除去される、headless 側の既存保証）。
//! [`crate::recipe::SlotRecipe`] が生成する CSS のセレクタは
//! この `[data-scope][data-part]` 属性を直接ターゲットにするため、styled 層は
//! パーツ関数へ手を加えず再エクスポートするだけで既定スタイルを効かせられる
//! （クラス名注入を必要としない）。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! [`fandhe_frontend_headless_ui::state::Disclosure`] が出力する
//! `data-state="open"`/`"closed"`（headless 側の既存保証）に応じて
//! backdrop/content の見た目を切り替える CSS を [`recipe`] へ登録する。
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）を通じて登録し、
//! `data-state` を含むセレクタも `SlotRecipe` の識別子検証・fail-closed
//! 除外を経由させる（`serialize_rule` を直接呼ぶ手書きセレクタ機構は
//! 廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみフォーカスリングを表示する `:focus-visible`
//! （[`crate::recipe::StateCondition::FocusVisible`]）を [`recipe`] へ登録する。
//!
//! # `size` variant（イシュー #729）
//!
//! `size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-dialog-content-padding`/`-content-max-width`/`-title-font-size`
//! の root スコープ CSS custom property（通常の CSS 継承により `content`/
//! `title` へ伝わる。`root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`base` 規則の `var()` には Md サイズ相当の
//! フォールバック値を書き、styled `root` を経由しない headless 直接利用
//! マークアップでも現行外観を維持する（fail-safe、`crate::lib` rustdoc
//! 「複合部品の variant 統一方針」節参照）。dialog は `color-palette` 軸を
//! 持たない（variant 表の方針、`docs/api/pre-styled-ui-api.md` §4d 参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・アニメーションは
//!   headless 層のドキュメント（`crates/headless-ui/src/dialog.rs`）で既に
//!   スコープ外と明記済みであり、本モジュールもそれを継承する。
//!
//! # overlay の stacking context（PR #575 Bugbot 指摘対応）
//!
//! `backdrop`/`positioner` は `position: fixed; inset: 0` のビューポート全体
//! オーバーレイだが、`z-index` を宣言しないとページ内の他の position 指定 UI
//! （ヘッダー・スティッキーバー・[`crate::menu`]/[`crate::select`] の
//! `positioner` 等）の下に隠れて操作不能になり得る。[`recipe`] の base 規則で
//! 両パーツに `z-index` を設定し、常に最前面に来るようにする（menu/select の
//! dropdown positioner（z-index: 10）より高い値にする）。
//!
//! # 内部パートのスタイル調整と開閉トランジション（イシュー #1693、親 #1520）
//!
//! 参照サイト基準の意匠調整として、`title`/`description`/`close-trigger`
//! （内部パート）と `backdrop`/`content` の開閉トランジションを本イシューで
//! 追加する。兄弟イシュー #1692 が担う `trigger`/`backdrop`/`positioner`/
//! `content` の枠・影・サイズ（border/box-shadow/max-width 等）には触れない。
//!
//! - **`content` の `position: relative`**: 絶対配置する `close-trigger`
//!   （後述）の配置基準。枠・影・サイズの変更ではないため #1692 側ではなく
//!   本イシューで追加する（`content` base 宣言を両イシューが編集するため、
//!   マージ順によっては手動 conflict 解消が必要になる点に注意）。
//! - **`title`/`description` の行送り**: [`crate::recipe`] のタイポグラフィ
//!   トークン（`--fandhe-font-line-height-tight`/`-normal`）を追加し、
//!   `description` の下余白を広げて後続のアクション行（footer 相当）との
//!   縦リズムを確保する。
//! - **`close-trigger` を content 右上のゴーストボタン化**: `position:
//!   absolute` で右上に固定し、hover 時のみ背景が付く（[`hover_bg_muted`] +
//!   [`hover_surface_declarations`]）ghost ボタンの見た目にする。focus-visible
//!   リングは [`focus_ring_declarations`]（イシュー #1424 共通トークン）へ
//!   移行する（`trigger` 側のフォーカスリングは #1692 のスコープのため
//!   本イシューでは変更しない）。
//! - **開閉トランジション**: `backdrop`/`content` の base へ
//!   [`transition_declarations`]（`MotionDuration::Slow`、モーダル等の
//!   強調遷移向け 300ms 既定、イシュー #1425 規約）を追加し、`content` の
//!   `data-state` 連動規則へ `opacity` を追加して fade + scale の複合遷移に
//!   する。`prefers-reduced-motion` は `Theme::to_css` が duration トークンを
//!   一括 `0ms` 化する共通経路で担保されるため、本モジュールでは
//!   `@media (prefers-reduced-motion)` を書かない（#1425 規約）。
//!   なお headless 層（`crates/headless-ui/src/dialog.rs`）は closed 時に
//!   `positioner`/`backdrop`/`content` へ `hidden` 属性を即時付与する契約の
//!   ため、closed 側の遷移は `display: none` により視覚上は省略される
//!   （既知の制約。headless 側の契約変更は本イシューのスコープ外）。
//! - **footer 相当のアクション配置**: headless anatomy に `footer` パートが
//!   存在せず、[`crate::recipe::SlotRecipe`] は子孫セレクタ機構を持たない
//!   （イシュー #708 で不採用確定）ため、専用 footer パートの CSS を
//!   pre-styled 側だけで新設することはできない。本イシューでは
//!   `description` の下余白確保までに留め、showcase デモ
//!   （`crates/docs-site/src/showcase.rs::dialog_section`）でアクション行の
//!   掲示例を示す。`dialog` への `footer` anatomy パート追加は headless-ui
//!   の anatomy 変更を伴うため、別イシュー・ユーザー承認が必要な対象外事項
//!   として記録する（`.claude/rules/out-of-scope-tracking.md` 対応）。
//!
//! # closed 時の `positioner` は必ず非表示化する（PR #575 Bugbot 指摘対応、High）
//!
//! headless 層（`crates/headless-ui/src/dialog.rs`）は dialog が closed の
//! とき `positioner`（`backdrop`/`content` も同様）に `hidden` 存在属性を
//! 付与し、UA 既定スタイル `[hidden] { display: none }` によって非表示化
//! させる契約になっている。ところが [`recipe`] の base 規則は `positioner`
//! に `display: flex` を宣言しており、この author スタイルが UA スタイルより
//! 詳細度で優先されるため `[hidden]` 単体では非表示化できず、closed でも
//! `position: fixed; inset: 0; z-index: 1001` のフルビューポート層が残存して
//! 背後のページのクリックを遮断してしまう（`backdrop`/`content` は
//! base 規則が `display` を宣言しないため UA 既定で問題ない）。
//! [`state_css`] に `[data-scope="dialog"][data-part="positioner"][hidden]`
//! に対する `display: none` の明示的な上書き規則を追加し、`display: flex`
//! より詳細度・出現順の両方で優先させることでこれを固定する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// headless 自由関数 `root`・状態機械 `Dialog` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::dialog` を直接 import する。
pub use fandhe_frontend_headless_ui::dialog::{
    backdrop, close_trigger, content, description, positioner, title, trigger, ContentIds,
    DialogRole,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `trigger`/`backdrop` 等の `state` 引数はいずれも `state` モジュール由来で
// 上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `dialog` anatomy の `data-part` 一覧（`crates/headless-ui/src/dialog.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "backdrop",
    "positioner",
    "content",
    "title",
    "description",
    "close-trigger",
];

/// この styled Dialog の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("dialog", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                decl("z-index", "1000"),
                decl("background", "rgba(0, 0, 0, 0.4)"),
            ],
        )
        // イシュー #1693: 開閉トランジション（モーダルの強調遷移向け
        // `MotionDuration::Slow`、#1425 規約）。closed 側は headless 層が
        // `hidden` 属性を即時付与するため視覚上は省略される（モジュール
        // 冒頭 rustdoc「内部パートのスタイル調整と開閉トランジション」節参照）。
        .base(
            "backdrop",
            transition_declarations("opacity", MotionDuration::Slow),
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                decl("z-index", "1001"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "content",
            [
                vec![
                    decl("position", "relative"),
                    decl("background", "var(--fandhe-color-bg)"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("border-radius", "0.5rem"),
                    decl(
                        "padding",
                        "var(--fandhe-dialog-content-padding, var(--fandhe-space-6))",
                    ),
                    decl("max-width", "var(--fandhe-dialog-content-max-width, 32rem)"),
                    decl("width", "100%"),
                ],
                // イシュー #1693: 開閉トランジション（#1425 規約、モジュール
                // 冒頭 rustdoc 参照）。`position: relative`（上記）は
                // close-trigger の絶対配置基準（枠・影・サイズではないため
                // #1692 側ではなく本イシューで追加、両イシューが `content`
                // base を編集する点に注意）。
                transition_declarations("opacity, transform", MotionDuration::Slow),
            ]
            .concat(),
        )
        .base(
            "title",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-dialog-title-font-size, var(--fandhe-font-font-size-lg))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
                // Medium 指摘（イシュー #1693 レビュー）: close-trigger を
                // content 右上へ絶対配置で重ねているため、title 側に
                // インライン終端方向のガター（close-trigger の想定占有幅
                // 相当）を確保しないと、title が折り返す/長い場合に
                // テキストと close-trigger が重なる。参照サイト実装
                // （Radix 等）が header/title 側にガターを設ける慣行に倣う。
                decl("padding-inline-end", "var(--fandhe-space-8)"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #1693: content 右上のゴーストボタン化（参照サイト標準）。
        // `position: absolute` は同イシューで追加した `content` の
        // `position: relative` を基準とする。位置指定は論理プロパティで
        // 統一する（`inset-block-start`/`inset-inline-end`、一貫性のため
        // 物理プロパティの `top` は使わない）。
        .base(
            "close-trigger",
            [
                vec![
                    decl("position", "absolute"),
                    decl("inset-block-start", "var(--fandhe-space-2)"),
                    decl("inset-inline-end", "var(--fandhe-space-2)"),
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("background", "transparent"),
                    decl("padding", "var(--fandhe-space-1)"),
                    decl("cursor", "pointer"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                ],
                vec![hover_bg_muted()],
                // `hover_surface_declarations()`（下記 state 登録）は
                // `background` のみを変更し `color` を変える規則を持たない
                // ため、到達しない宣言を避けて `background` のみ
                // transition 対象にする。
                transition_declarations("background", MotionDuration::Fast),
            ]
            .concat(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #551 受け入れ条件: `backdrop`/`content` の開閉状態に応じた
        // 見た目の切り替え。
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1"), decl("transform", "scale(1)")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0"), decl("transform", "scale(0.95)")],
        )
        // PR #575 Bugbot 指摘対応（High）: positioner の base 規則が
        // `display: flex` を宣言しており、UA 既定の `[hidden] { display: none }`
        // を詳細度で上書きしてしまう。closed 時に headless 層が付与する
        // `hidden` 属性を確実に非表示化として機能させるため、より詳細度の高い
        // `[hidden]` 属性セレクタで `display: none` を明示的に上書きする。
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // イシュー #1693: close-trigger の focus-visible をイシュー #1424
        // 共通トークンへ移行する（trigger 側は #1692 のスコープのため
        // リテラルのまま維持する）。
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の等差進行（padding 2 段刻み・
        // max-width 8〜10rem 刻み・font-size 1 段オフセット）を両端へ外挿。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-dialog-content-max-width", "16rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-4)"),
                decl("--fandhe-dialog-content-max-width", "24rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-6)"),
                decl("--fandhe-dialog-content-max-width", "32rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-8)"),
                decl("--fandhe-dialog-content-max-width", "42rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-xl)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-10)"),
                decl("--fandhe-dialog-content-max-width", "52rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-2xl)",
                ),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Dialog が生成する静的 CSS 全量を返す（決定的。同一プロセス内で
/// 複数回呼んでも常にバイト単位で同一の文字列を返す、[`SlotRecipe::css`](crate::recipe::SlotRecipe::css)
/// の契約をそのまま継承する）。
///
/// 呼び出し元は返り値を静的 `.css` ファイルとして配信する、または
/// [`crate::stylesheet::StyleSheet::push_css`] へ渡して `<style>` 要素へ
/// 埋め込む（#605、[`crate`] 冒頭の不変条件を参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::dialog::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::dialog::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = dialog::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="dialog" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::dialog::root(state, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="dialog"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="dialog"][data-part="backdrop"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn backdrop_and_positioner_declare_stacking_order() {
        // PR #575 Bugbot 指摘対応: backdrop/positioner が z-index を宣言し、
        // 他の position 指定 UI の下に隠れないことを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"] {"#));
        assert!(css.contains("z-index: 1000;"));
        assert!(css.contains("z-index: 1001;"));
    }

    #[test]
    fn closed_positioner_hidden_attr_overrides_display_flex() {
        // PR #575 Bugbot 指摘対応（High）: positioner の base 規則
        // `display: flex` が UA 既定の `[hidden] { display: none }` を
        // 上書きし、closed でもフルビューポート層が残存して背後のページの
        // クリックを遮断する不具合の回帰。`[hidden]` 属性セレクタでの
        // 明示的な `display: none` 上書きが出力されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="positioner"][hidden] {"#));
        let positioner_hidden_rule_start = css
            .find(r#"[data-scope="dialog"][data-part="positioner"][hidden] {"#)
            .expect("positioner[hidden] rule must be present");
        let rule_body = &css[positioner_hidden_rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        // styled root が headless 層と同一の data-scope/data-part 出力になる
        // ことを固定する（薄い委譲であることの回帰）。
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-dialog--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(
            css.contains("padding: var(--fandhe-dialog-content-padding, var(--fandhe-space-6));")
        );
        assert!(css.contains("max-width: var(--fandhe-dialog-content-max-width, 32rem);"));
        assert!(css.contains(
            "font-size: var(--fandhe-dialog-title-font-size, var(--fandhe-font-font-size-lg));"
        ));
    }

    #[test]
    fn trigger_and_close_trigger_declare_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。イシュー #1693 で
        // close-trigger 側は共通トークン（#1424）へ移行したため、
        // trigger（#1692 のスコープのためリテラルのまま）と分けて検証する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="close-trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn close_trigger_declares_hover_surface_inside_hover_media_query() {
        // イシュー #1693: close-trigger の hover 規則が `@media (hover:
        // hover)` 内に `:hover:not([data-disabled])` で出力されることを
        // 固定する（#1425 規約、`SlotRecipe::css` の集約契約）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="dialog"][data-part="close-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn backdrop_and_content_declare_transition_for_open_close() {
        // イシュー #1693: 開閉トランジション（backdrop/content の base に
        // transition-property/-duration/-timing-function が含まれること）。
        let css = stylesheet();
        assert!(css.contains("transition-property: opacity;"));
        assert!(css.contains("transition-property: opacity, transform;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-slow);"));
        assert!(css.contains("transition-timing-function: var(--fandhe-motion-easing-standard);"));
    }

    #[test]
    fn content_open_and_closed_states_include_opacity() {
        // イシュー #1693: content の data-state 連動規則に opacity が
        // 含まれ、fade + scale の複合遷移になることを固定する。
        let css = stylesheet();
        let open_start = css
            .find(r#"[data-scope="dialog"][data-part="content"][data-state="open"] {"#)
            .expect("content open rule must be present");
        let open_end = css[open_start..].find('}').unwrap() + open_start;
        assert!(css[open_start..open_end].contains("opacity: 1;"));
        assert!(css[open_start..open_end].contains("transform: scale(1);"));

        let closed_start = css
            .find(r#"[data-scope="dialog"][data-part="content"][data-state="closed"] {"#)
            .expect("content closed rule must be present");
        let closed_end = css[closed_start..].find('}').unwrap() + closed_start;
        assert!(css[closed_start..closed_end].contains("opacity: 0;"));
        assert!(css[closed_start..closed_end].contains("transform: scale(0.95);"));
    }

    #[test]
    fn content_and_close_trigger_declare_positioning_pair() {
        // イシュー #1693: close-trigger の絶対配置は content の
        // `position: relative` を基準とする（対で出力されることを固定）。
        let css = stylesheet();
        let content_start = css
            .find(r#"[data-scope="dialog"][data-part="content"] {"#)
            .expect("content base rule must be present");
        let content_end = css[content_start..].find('}').unwrap() + content_start;
        assert!(css[content_start..content_end].contains("position: relative;"));

        let close_trigger_start = css
            .find(r#"[data-scope="dialog"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let close_trigger_end = css[close_trigger_start..].find('}').unwrap() + close_trigger_start;
        assert!(css[close_trigger_start..close_trigger_end].contains("position: absolute;"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="closed"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="content"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_dialog_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Dialog`（headless の Component/Hydrate 実装を継承。イシュー
        // #729 により本モジュールから再エクスポートしないため、状態機械を
        // 使う呼び出し側と同じくエスケープハッチ経由で直接 import する。
        // モジュール冒頭の rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::dialog::Dialog;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut d = Dialog::default();
        assert_eq!(d.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&d.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut d, "open", ""));
        let hydrate_html = render(&render_for_hydration(&d));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        // クライアント側の改ざん耐性のある復元経路が Dialog 経由でも機能する。
        let restored = Dialog::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
