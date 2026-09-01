//! Button（イシュー #550）: 単一 recipe styled 部品。`<button type="button">`
//! を組み立てる。
//!
//! `loading: true` のとき [`crate::spinner::spinner_decorative`]（`role`/
//! `aria-label` を持たない装飾用途の Spinner）を子ノード先頭へ埋め込む
//! （呼び出し先の契約: Spinner は状態機械を要しない静的部品であり、Button の
//! 内部でのみ組み立てて返す。ボタン自身の `aria-busy` が既に読み上げ状態を
//! 伝えるため、公開 API の [`crate::spinner::spinner`] が持つ
//! `role="status"` + `aria-label` のライブリージョンを二重に埋め込まない）。
//! ただし [`icon_button`]/[`close_button`]（icon-only）は例外で、Spinner を
//! アイコンの手前へ追加せず**置換**する（子ノードが常に 1 個の前提で正方形
//! を保つため、モジュール内 rustdoc「size スケール・icon-only・loading」
//! 節・`assemble` rustdoc 参照、イシュー #1449 Cursor Bugbot 指摘の是正）。
//! また `loading: true` のときは `disabled: true` と同様に `disabled` 属性・
//! `data-disabled`・`aria-disabled="true"` も付与し、読み込み中のクリック・
//! 暗黙 submit による重複アクションの発火を防ぐ（Medium severity のバグ
//! 指摘の是正、`aria-busy`/`data-loading` だけでは操作を止められないため）。
//! 呼び出し側 `attrs` は `class_attr::drop_class_attr` を経由して `class` を
//! 除去してから合成し、recipe が生成するクラスが常に唯一の `class` 属性値に
//! なる。
//!
//! # CloseButton / IconButton（イシュー #830）
//!
//! chakra-ui の `CloseButton`/`IconButton` に相当する部品は、
//! `docs/policy/intentional-non-adoption.md` §7・
//! `docs/design/component-coverage-map.md` で「保留（Button variant で近似
//! 可能、需要待ち）」と記録されていた。イシュー #830 で再評価トリガー
//! （`Button` variant 拡張要望 issue の起票）が充足したため保留を解除するが、
//! **専用 anatomy・新規状態機械を持つ独立部品としては新設しない**。
//! [`icon_button`]/[`close_button`] はどちらも本モジュールの `recipe()`
//! （非公開の `icon` 修飾 variant 軸を追加しただけ）と [`button`] 本体の
//! 組み立てロジックを共有する Button variant 拡張であり、`data-scope` は
//! 引き続き `"button"` のまま、専用の `data-scope="close-button"` 等は
//! 持たない（chakra 対応表 §7 の保留解除に対する Rust 最適化形の実装判断）。
//!
//! # `data-*` 語彙（イシュー #1063）
//!
//! `data-loading`: 存在属性（値は常に空文字列）。`loading: true` のときのみ
//! 付与する。`fandhe-frontend-headless-ui` に `button` に対応する部品は
//! 存在しないため、本モジュール（pre-styled-only 部品）固有の語彙として
//! 定義する（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B）。
//! 現在の recipe（[`recipe`]）はこの属性を `StateCondition` として参照
//! しない（CSS 消費者なし）。AT 向けの読み上げ意味論は併記する
//! `aria-busy="true"` が担い、`data-loading` は利用者側 CSS/JS が任意で
//! フックするための存在表示に留まる。イシュー #1449 でこの判断を再確認
//! 済み（disabled 視覚は `data-disabled` 併記経由で loading 時も成立する
//! ため、新規セレクタは追加しない）。
//!
//! # size スケール・icon-only・loading（イシュー #1449）
//!
//! [`recipe_with_scope`] の `size` variant（xs〜xl の 5 段）は
//! [`crate::theme`] の `--fandhe-size-control-{height,padding-x,font-size}-*`
//! トークン（イシュー #1678 で新設、3 系統 × 5 段）を参照する。button が
//! このトークン系統の最初の消費者であり、縦方向は `height` トークンを
//! 下限とする `min-height` で表現し（縦 padding は 0）、水平方向は
//! `padding-x` トークンのみを使う（値はいずれもソースコード内
//! `&'static str` リテラルの `var()` 参照であり、`format!` による動的
//! 合成は行わない）。**codex-review #1731 P1 指摘の是正**: 当初 `height`
//! （固定）で表現していたが、ブラウザの既定フォントサイズ拡大・
//! `--fandhe-size-control-font-size-*` の利用者上書き・ラベル折り返しで
//! 内容が固定高さを超えるとラベルがボタン外へあふれる不具合があった
//! ため、ラベル付きボタン（[`button`]）は `min-height` へ変更し、内容が
//! 下限を超える場合はボックス自体が自然に伸長するようにした。
//! [`ButtonIcon::Only`] は子ノードが常にアイコン 1 個（固定サイズ・
//! 折り返し要因なし、モジュール冒頭 rustdoc 参照）であり `min-height` の
//! ままでは `aspect-ratio: 1 / 1` が確定サイズを得られず正方形を保証
//! できないため、[`recipe`] が `icon`×`size` の compound variant として
//! 5 段ぶんの確定 `height` を追加登録し、icon-only の場合のみ正方形を
//! 復元する（正方形の成立条件は「icon-only 時の確定 `height` +
//! `aspect-ratio: 1 / 1`」であり、padding では担わない。詳細は
//! [`recipe`] rustdoc 参照）。[`ButtonIcon::Only`] の通常 variant は
//! 5 段の均等 padding リテラルをやめて `padding: 0`
//! （`aspect-ratio: 1 / 1` は不変）へ簡約したまま変更していない。
//! `recipe_with_scope` を共有する [`crate::download_trigger`] にも同じ
//! size 宣言（min-height/padding-x/font-size のトークン化）が波及する
//! （意図的、golden テスト参照。icon×size の compound variant は
//! [`recipe`]（button 専用の公開 API）にのみ追加するため download_trigger
//! へは波及しない）。
//! [`assemble`] が埋め込む loading 中の Spinner サイズはボタンの `size`
//! から決定的に写像する（`xs`/`sm`/`md` → `Size::Sm`、`lg`/`xl` →
//! `Size::Md`。ボタンの `font-size` に近い視覚サイズへ追随させるための
//! 単純な 2 分割であり、Spinner 自体は 5 段の `size` 軸を持たないため
//! 全段を写像先に持たない）。フォーカスリングは [`recipe_with_scope`] が
//! `palette` 軸を公開する部品向けの canonical 形
//! （[`focus_ring_declarations`]`(`[`FocusRingColor::Palette`]`,`
//! [`FocusRingOffset::Outside`]`)`、`docs/design/
//! pre-styled-ui-focus-ring-and-size-conventions.md` 準拠）で新規追加した
//! （`link.rs`/`radio_group.rs`/`angle_slider.rs` と同型）。
//!
//! **Outline / Solid の高さ一致（イシュー #1756）**: `recipe_with_scope` の
//! base（`box-sizing: border-box`、上記の是正参照）により、`border-box` の
//! 下では `border`/`padding` は指定した `height`/`min-height` の**内側**に
//! 含まれるため、border の有無だけでは外寸（描画高さ）が変化しない
//! （`content-box` のままだと Outline のみ border 分（上下合計 2px）外寸が
//! 大きくなっていた不具合の是正）。ただし本モジュールの size variant は
//! `height` ではなく `min-height`（ラベル付きボタン、上記の是正参照）で
//! あるため、この一致が保証されるのは (1) ラベルの内容が `min-height` の
//! 下限に収まり、ボックスの実高さが `min-height` の値そのものに固定される
//! 場合（この場合のみ border-box が border/padding を内側へ収め、
//! Outline/Solid の外寸が一致する）、または (2) 確定 `height` を持つ
//! [`ButtonIcon::Only`]（icon-only）の場合に限られる。**この一致が保証
//! されない範囲（codex-review #1756 P2 指摘の是正）**: ラベルの内容が
//! `min-height` の下限を超えると、ボックスは `min-height` に縛られず
//! 内容 + padding + border の合計まで自然に伸長するため、border の有無
//! （Outline: 1px、Solid: none）がそのまま外寸差（上下合計 2px）として
//! 再度現れる。すなわち `box-sizing: border-box` は「border/padding を
//! `height`/`min-height` の内側に含める」という、ボックスの実高さが
//! ちょうど `min-height` に等しい間だけ効く前提条件であり、内容量に
//! よらない任意ケースでの外寸完全一致を意味しない。ラベル付きボタンで
//! 内容が下限を超えるケースは意図的に保証対象外とし（アプリケーション
//! ロジックであるラベル文字列の折り返し・長さ制御は本コンポーネント層の
//! 責務外）、本モジュールが不変条件として固定するのは確定 `height` を
//! 持つ icon-only ケースのみである。この不変条件は
//! `crates/pre-styled-ui/tests/button_css.rs` の意味的回帰テストで固定する
//! （xs/sm/md/lg/xl の 5 size 全段を検証、下記参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::icon::{icon, IconProps};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_bg_solid,
    hover_surface_declarations, palette_scale_declarations, transition_declarations, when,
    ColorPalette, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};
use crate::spinner::spinner_decorative;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::{anatomy, aria_disabled, aria_label, data_disabled, Anatomy};

/// [`icon_button`] 呼び出し時、`label` が空白のみ（trim 後空文字）の場合の
/// フォールバック `aria-label`。アクセシブルネームを欠いた icon-only ボタンを
/// 決して生成しない fail-closed 動作（イシュー #830 受け入れ条件 1）。
const ICON_BUTTON_FALLBACK_LABEL: &str = "unlabeled button";

/// `attrs` に `aria-label`（大文字小文字を無視）が既に含まれ、かつその値が
/// trim 後に空文字でないかどうかを判定する。[`icon_button`]/[`close_button`]
/// が組み立てる既定/フォールバック `aria-label` を、呼び出し側が `attrs`
/// 経由で明示指定した値と重複させないために使う（`fandhe_frontend_headless_ui::number_input`
/// の `increment_trigger`/`decrement_trigger` と同型の dedup 判断、fail-closed。
/// 重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ）。
///
/// 値が空文字・空白のみの場合はキーが存在してもフォールバックさせる
/// （呼び出し側が `aria-label=""` を渡した場合に、アイコンオンリーボタンが
/// 空のアクセシブルネームのまま出力される fail-closed 保証の穴を防ぐ。
/// イシュー #830 PR #863 Bugbot 指摘）。
fn has_caller_aria_label(attrs: &[(&str, &str)]) -> bool {
    attrs
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("aria-label") && !v.trim().is_empty())
}

/// [`close_button`] 呼び出し時、`label` が空白のみの場合の既定
/// `aria-label`（chakra-ui `CloseButton` の既定値と同値）。
const CLOSE_BUTTON_DEFAULT_LABEL: &str = "Close";

/// [`close_button`] が組み立てる装飾用途 × アイコンの SVG path
/// （Material Design の `close` グリフ相当、`viewBox="0 0 24 24"`）。
/// 外部リソース（`href`/`xlink:href`）は一切参照しない決定的なインライン
/// パスであり、ユーザー入力や実行時の変動要素を含まない。
const CLOSE_ICON_PATH: &str = "M18.3 5.71 12 12.01 5.7 5.71 4.29 7.12 10.59 13.42 4.29 19.72 5.7 21.13 12 14.83 18.3 21.13 19.71 19.72 13.41 13.42 19.71 7.12Z";

/// `label` が空白のみ（trim 後空文字）なら `fallback` へ置換する
/// （[`icon_button`]/[`close_button`] 共通の fail-closed ヘルパ。空の
/// `aria-label=""` を決して出力しない）。
fn normalize_label<'a>(label: &'a str, fallback: &'static str) -> &'a str {
    if label.trim().is_empty() {
        fallback
    } else {
        label
    }
}

/// icon-only 修飾 variant（axis `"icon"` / value `"only"`）。[`icon_button`]・
/// [`close_button`] のみが `selection` へ渡す非公開 enum で、呼び出し側の
/// 公開 API（[`ButtonProps`]）には露出しない（関数選択で表現するため）。
/// `default_variant` を登録しないため、通常の [`button`] の class 出力・
/// golden CSS は不変のまま保たれる（後方互換、イシュー #830 受け入れ条件 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonIcon {
    /// icon-only（正方形・均等 padding）。
    Only,
}

impl VariantValue for ButtonIcon {
    fn axis(self) -> &'static str {
        "icon"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Only => "only",
        }
    }
}

/// `data-scope="button"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("button");

/// Button の見た目 variant（chakra-ui v3 準拠、イシュー #1448 で
/// `Surface`/`Plain` を追加し 6 値へ拡張。solid/outline/ghost/subtle/
/// surface/plain の chakra-ui v3 6 variant に対応する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// 塗りつぶし（既定）。
    #[default]
    Solid,
    /// 輪郭のみ。
    Outline,
    /// 背景なし・最小装飾。
    Ghost,
    /// 淡色背景（イシュー #1448 で palette 着色面へ移行、`recipe_with_scope`
    /// 参照）。
    Subtle,
    /// 淡色背景 + 輪郭（イシュー #1448 新設）。chakra-ui v3 の `surface` は
    /// box-shadow リングだが、本フレームワークは `outline`（`Outline`
    /// variant と同型の実 border）を採用する（`recipe_with_scope` 内コメント
    /// 参照。新規 box-shadow リングを増やさない #1424 の判断とも整合）。
    Surface,
    /// 背景・輪郭なしの最小装飾（イシュー #1448 新設）。chakra-ui v3 の
    /// `plain` に相当し、hover 背景変化を持たない（`recipe_with_scope`
    /// 内コメント参照）。
    Plain,
}

impl VariantValue for ButtonVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Subtle => "subtle",
            Self::Surface => "surface",
            Self::Plain => "plain",
        }
    }
}

/// [`button`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct ButtonProps {
    /// 見た目 variant（既定 `Solid`）。
    pub variant: ButtonVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色（`accent`/`info`/`success`/`warning`/`danger`）から
    /// 選択する。
    pub palette: ColorPalette,
    /// 無効化。`true` のとき `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` を付与する。
    pub disabled: bool,
    /// 読み込み中。`true` のとき `aria-busy="true"`・`data-loading` を付与し、
    /// [`crate::spinner::spinner_decorative`] を子ノード先頭へ埋め込む。
    /// [`Self::disabled`] と同様に `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` も付与し、読み込み中のクリック・暗黙 submit
    /// を止める。
    pub loading: bool,
}

impl Default for ButtonProps {
    fn default() -> Self {
        ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Md,
            palette: ColorPalette::Accent,
            disabled: false,
            loading: false,
        }
    }
}

/// Button の recipe（既定 scope `"button"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_scale_declarations`] が生成する
/// `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg`
/// （イシュー #606）経由で参照し、`var(--fandhe-color-accent)` 等の
/// セマンティック色を直接参照しない（`palette` variant の切り替えだけで
/// 全 variant の色が追従する）。
/// [`recipe_with_scope`] に Button 固有の icon-only 修飾 variant
/// （非公開 [`ButtonIcon`] 軸）を追加した recipe を返す。
///
/// icon-only 追加分は `recipe_with_scope` 自体には加えない（同関数は
/// [`crate::download_trigger`] と宣言を共有する契約のため、ここへ追加すると
/// download_trigger の golden CSS まで変えてしまう）。[`button`] 自身の
/// class 出力・golden CSS を不変に保つため `default_variant` は登録しない
/// （[`ButtonIcon`] rustdoc 参照）。
fn recipe() -> SlotRecipe {
    // イシュー #1449: size variant が `--fandhe-size-control-height-*`
    // トークンで高さを固定するようになったため、icon-only の正方形は
    // 「高さ固定 + `aspect-ratio: 1 / 1`」で成立し、5 段の均等 padding
    // リテラル（旧実装）は不要になった。`padding: 0` へ簡約し、水平方向も
    // 高さと同じ長さへ揃える（モジュール冒頭 rustdoc「size スケール・
    // icon-only・loading」節参照）。
    //
    // `.fd-button--icon-only` と `.fd-button--size-*` はセレクタ specificity
    // が同値（(0,3,0)、`data-scope`/`data-part`/単一クラス）のため、この
    // `padding: 0` が size 側の `padding` を上書きできるのは specificity
    // ではなく **CSS 出力順（後勝ち）** による（`SlotRecipe::css` は
    // variant を登録順に出力し、本 `recipe()` は `recipe_with_scope` が
    // size variant を登録し終えた後にのみ icon-only を追加する）。size
    // variant の登録順序を icon-only より後へ動かす変更は本規則を静かに
    // 破壊するため、`recipe_with_scope` 側を変更する際はこの出力順依存を
    // 意識すること（golden テスト `button_css.rs` がバイト一致で固定）。
    //
    // イシュー #1449 codex-review P1 指摘の是正: `recipe_with_scope` の
    // size variant は固定 `height` から `min-height` へ変更した（ラベル
    // 付きボタンがブラウザの既定フォントサイズ拡大・
    // `--fandhe-size-control-font-size-*` の利用者上書き・ラベル折り返しで
    // 内容が `height` を超えても、ボタンが `min-height` を下限として自然に
    // 伸長し、内容が枠外へあふれる/隣接要素と重なることを防ぐ）。ただし
    // icon-only（[`ButtonIcon::Only`]）は子ノードが常にアイコン 1 個
    // （固定サイズ・折り返し要因なし、モジュール冒頭 rustdoc 参照）で
    // あるため `min-height` だけでは正方形が保証できない（`aspect-ratio`
    // は `min-height` のような制約でなく確定サイズを要求するため、
    // コンテンツ幅次第で正方形が崩れうる）。ここで `ButtonIcon::Only` と
    // 各 `Size` の compound variant として明示的な固定 `height` を追加登録
    // し、icon-only の各 size だけ正方形の確定サイズを復元する
    // （`compound_variant` はクラス 2 個分のセレクタで size variant 単体
    // より詳細度が高いため出力順に依存せず必ず上書きする）。
    recipe_with_scope("button")
        .variant(
            ButtonIcon::Only,
            "root",
            vec![decl("aspect-ratio", "1 / 1"), decl("padding", "0")],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Xs)],
            "root",
            vec![decl("height", "var(--fandhe-size-control-height-xs, 2rem)")],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Sm)],
            "root",
            vec![decl(
                "height",
                "var(--fandhe-size-control-height-sm, 2.25rem)",
            )],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Md)],
            "root",
            vec![decl(
                "height",
                "var(--fandhe-size-control-height-md, 2.5rem)",
            )],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Lg)],
            "root",
            vec![decl(
                "height",
                "var(--fandhe-size-control-height-lg, 2.75rem)",
            )],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Xl)],
            "root",
            vec![decl("height", "var(--fandhe-size-control-height-xl, 3rem)")],
        )
}

/// [`recipe`] の scope 引数化版（イシュー #828）。
///
/// [`crate::download_trigger`] が「Button recipe の流用」（`variant`/`size`/
/// `palette` の宣言・既定値を一切変えず `data-scope` セレクタとクラス接頭辞
/// のみを差し替える）であることを型で保証するために `pub(crate)` として
/// 公開する。`SlotRecipe::css` はセレクタ・クラス名の生成に `scope`
/// （[`SlotRecipe::new`] の第 1 引数）のみを使う設計であるため、宣言
/// （`base`/`variant`/`default_variant`）を 1 箇所に保ったまま scope だけを
/// 差し替えれば、機械的に「Button と同一の宣言・別 scope の CSS」が
/// 得られる（[`crate::stylesheet`] のドリフト検知テストとは独立に、
/// `crates/pre-styled-ui/tests/download_trigger_css.rs` の golden テストが
/// この流用契約自体を固定する）。
pub(crate) fn recipe_with_scope(scope: &'static str) -> SlotRecipe {
    let mut recipe = SlotRecipe::new(scope, &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                // イシュー #1449 codex-review P1 指摘の是正: size variant が
                // `height` をトークンで固定する公開 CSS 契約（モジュール冒頭
                // rustdoc 参照）を border 付き variant（Outline）でも成立させる
                // ため `box-sizing: border-box` を base へ追加する。UA 既定の
                // `content-box` のままだと Outline の `border: 1px solid` が
                // `height` の外側に積み増され、実際の外寸が
                // `--fandhe-size-control-height-*` トークン値より上下合計 2px
                // 大きくなってしまう（`download_trigger` は recipe を共有する
                // ため同じ是正が及ぶ）。
                decl("box-sizing", "border-box"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "0.5rem"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("cursor", "pointer"),
                // `<button>` は UA 既定で text-decoration を持たないため従来は
                // 無指定でも問題なかったが、本 recipe を `<a>` へ流用する
                // `download_trigger`（イシュー #828、scope 切替のみで宣言は
                // 完全共有、モジュール冒頭 rustdoc 参照）では UA のリンク下線が
                // 残ってしまう（`link`/`nav_list`/`breadcrumb` の a ベース部品が
                // 同じ理由でリセット済み）。Button recipe を 1 箇所で共有する
                // 設計上、ここでリセットして両方の実体（button/a）へ一律適用する。
                decl("text-decoration", "none"),
            ],
        )
        .base(
            "root",
            // hover/disabled の見た目切り替えを滑らかにする（イシュー #1425、
            // 共通ビジュアル言語の参照実装）。`base` は同一 slot への複数回
            // 登録が許され出力順で連結されるため、既存の base ブロックを
            // 書き換えずに純追加できる。
            transition_declarations(
                "background, border-color, color, box-shadow",
                MotionDuration::Fast,
            ),
        )
        // イシュー #1449: size variant を `--fandhe-size-control-*`
        // トークン（イシュー #1678 新設、`crate::theme::DEFAULT_SIZES`）で
        // 表現する。button が本トークン系統の最初の消費者。縦方向は
        // `height` で固定し（縦 padding は 0）、水平方向のみ `padding-x`
        // トークンを使う（モジュール冒頭 rustdoc 参照）。値はすべて
        // ソースコード内 `&'static str` リテラルの `var()` 参照。
        //
        // 各 `var()` は第 2 引数へ `DEFAULT_SIZES` の既定値をそのまま
        // フォールバックとして持つ（`focus_ring_declarations` の rustdoc・
        // イシュー #1424 レビュー指摘と同じ理由）。`--fandhe-size-control-*`
        // は本イシュー以前には存在しなかったトークンであり、`Theme::empty()`
        // ベースの既存カスタムテーマはこれらを定義していない。フォール
        // バックなしで直接参照すると `height`/`padding`/`font-size` の
        // computed-value time 無効化により無効な最終値（`auto`/`0`/
        // 初期値）へ落ち、パッチバンプ（0.54.2 → 0.54.3、§3.1）の
        // 「破壊的変更ではない CSS 実体変更」という前提と矛盾する。
        // `control-font-size-*` の既定値自体が
        // `var(--fandhe-font-font-size-<段>)` 参照のため、フォールバックは
        // その参照をそのまま埋め込む（`DEFAULT_SIZES` 定義と同値）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("min-height", "var(--fandhe-size-control-height-xs, 2rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-xs, 0.625rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "min-height",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("min-height", "var(--fandhe-size-control-height-md, 2.5rem)"),
                decl("padding", "0 var(--fandhe-size-control-padding-x-md, 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "min-height",
                    "var(--fandhe-size-control-height-lg, 2.75rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-lg, 1.25rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("min-height", "var(--fandhe-size-control-height-xl, 3rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-xl, 1.5rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl))",
                ),
            ],
        )
        .variant(
            ButtonVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
                decl("border", "none"),
                // hover 時の背景色をこの palette の emphasized 段へ差し替える
                // （イシュー #1425。`--fandhe-hover-bg` の実値は variant ごとに
                // ここで定義し、実際の `background: var(--fandhe-hover-bg)`
                // 適用は下記 `.state(..., StateCondition::Hover, ...)` 1 本に
                // 集約する）。
                hover_bg_solid(),
            ],
        )
        .variant(
            ButtonVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-palette)"),
                // 面を持たない variant は淡い bg-muted 段で hover を表現する
                // （イシュー #1425、[`ButtonVariant::Solid`] と対称の設計）。
                hover_bg_muted(),
            ],
        )
        .variant(
            ButtonVariant::Ghost,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "none"),
                hover_bg_muted(),
            ],
        )
        .variant(
            ButtonVariant::Subtle,
            "root",
            vec![
                // イシュー #1448: 中立色 `--fandhe-color-bg-subtle` から
                // palette 着色面（`palette_scale_declarations` が定義する
                // `--fandhe-palette-subtle`/`-fg-subtle`）へ移行する。
                // `palette` variant の切り替えだけで全 variant の色が追従する
                // という本 recipe の設計方針（モジュール冒頭 rustdoc 参照）を
                // `Subtle` にも適用する。
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "none"),
                // #1425 の「面なし系は bg-muted」原則は palette 着色面の
                // 導入前の語彙であり、tint 面に中立色 bg-muted を当てると
                // tint→gray の不自然な遷移になるため、同原則の趣旨
                // （既存段の再利用・新段を作らない）に沿って
                // `--fandhe-palette-muted` を用いる（`Outline`/`Ghost` の
                // hover は #1425 の参照実装のまま変更しない意図的差分）。
                decl("--fandhe-hover-bg", "var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            ButtonVariant::Surface,
            "root",
            vec![
                // イシュー #1448 新設: 淡色背景 + 輪郭。chakra-ui v3 の
                // `surface` は box-shadow リングだが、`Outline` と同じ実
                // border を採用し、部品間の視覚言語の一貫性を優先する
                // （#1424 が新規 box-shadow リングを増やさない判断とも
                // 整合。理由はモジュール冒頭 `ButtonVariant::Surface` の
                // rustdoc 参照）。
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
                decl("--fandhe-hover-bg", "var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            ButtonVariant::Plain,
            "root",
            vec![
                // イシュー #1448 新設: 背景・輪郭なしの最小装飾。
                // `--fandhe-hover-bg` を `transparent` として明示定義する
                // （未定義のまま共有 Hover state が `background:
                // var(--fandhe-hover-bg)` を適用すると computed-value time
                // に無効化される非決定的挙動を避ける fail-closed。chakra-ui
                // v3 の `plain` は hover 背景変化を持たないため、この
                // 明示定義が挙動としても正しい）。
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "none"),
                decl("--fandhe-hover-bg", "transparent"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ButtonVariant::Solid)
        .default_variant(ColorPalette::Accent)
        .state("root", StateCondition::Hover, hover_surface_declarations())
        // イシュー #1448/#1449: キーボードフォーカス表示（`recipe::
        // focus_ring_declarations` 経由の canonical outline、#1424 §3/§6
        // 規約、`link.rs`/`radio_group.rs`/`angle_slider.rs` と同型）。
        // palette 連動色（`FocusRingColor::Palette`）を使い、`data-disabled`
        // 状態より前に登録して disabled の後勝ち（opacity/cursor 上書き）を
        // 保つ。`recipe_with_scope` を共有する `download_trigger`（`<a>`）
        // にも波及するが、`<a>` はフォーカス可能なので妥当（モジュール冒頭
        // rustdoc 参照）。
        //
        // 両イシューが独立に同一の `.state(..., StateCondition::FocusVisible,
        // focus_ring_declarations(...))` 呼び出しを追加していたため（#1448
        // は data-disabled の後、#1449 は data-disabled の前）、base 取り込み
        // マージで重複登録にならないよう #1448 側の 1 本（disabled 後勝ちの
        // 根拠が明示されている方）へ統合した。
        .state(
            "root",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // `data-disabled` を recipe が消費する（イシュー #1425 で方針転換）。
        // `recipe_with_scope` は `download_trigger` と宣言を共有するため、
        // `download_trigger` 側にも同じ規則が波及するが、`download_trigger`
        // は `disabled` を持たない設計（`crate::download_trigger` rustdoc
        // 参照）のため実害はなく dead CSS の混入に留まる。
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        );

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

/// Button の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Button 1 個を組み立てる。
///
/// `type="button"` を既定固定し、フォーム内の暗黙 submit（`type` 省略時の
/// HTML 既定値 `"submit"`）による事故を防ぐ（安全側既定、
/// `.claude/rules/security.md` セキュリティ設定ミス対策相当）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
///
/// let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
/// let html = render(&node);
/// assert!(html.contains(r#"type="button""#));
/// assert!(html.contains("Save"));
/// ```
#[must_use]
pub fn button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    assemble(props, false, attrs, children)
}

/// [`assemble`] が `loading: true` のとき埋め込む装飾用途 Spinner の
/// サイズを、ボタンの [`Size`]（5 段）から決定的に写像する（イシュー
/// #1449）。[`crate::spinner`] は `size` 軸自体を 5 段持たないため
/// （`SpinnerProps` の既定は `Sm`/`Md` 中心）、ボタンの `font-size` に近い
/// 視覚サイズへ追随させる単純な 2 分割とする: `Xs`/`Sm`/`Md` →
/// `Size::Sm`、`Lg`/`Xl` → `Size::Md`（モジュール冒頭 rustdoc「size
/// スケール・icon-only・loading」節参照）。
fn spinner_size_for(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm | Size::Md => Size::Sm,
        Size::Lg | Size::Xl => Size::Md,
    }
}

/// `button()`/[`icon_button`]/[`close_button`] 共有の組み立てロジック
/// （内部専用）。`type="button"` 固定・`disabled`/`loading` の三点セット・
/// `loading` 時の spinner 埋め込み・`drop_class_attr` による `class` 一意化を
/// 一箇所へ集約し、3 つの公開関数がこの契約を完全に共有することを保証する
/// （イシュー #830。挙動の分岐は `icon_only` による class 選択への
/// `("icon", "only")` 追加のみ）。
///
/// `icon_only && loading` の子ノード置換（イシュー #1449 Cursor Bugbot
/// Medium 指摘の是正）: icon-only の正方形は `padding: 0` +
/// `aspect-ratio: 1 / 1`（[`recipe`] 参照）で成立し、内容量に依存しない
/// 前提で保たれている。テキストボタン（非 icon-only）と同じく Spinner を
/// 呼び出し側アイコンの手前へ**追加**すると、`gap` を挟んで 2 個の
/// 子要素（Spinner + アイコン）が横並びになり、コンテンツ幅が正方形の幅を
/// 超えて横長化してしまう（アイコンボタンは元々アイコン 1 個分の内容量を
/// 前提にしている）。icon-only かつ loading のときは呼び出し側 `children`
/// （アイコン）を描画せず Spinner のみへ**置換**し、常に子ノード 1 個の
/// まま正方形の前提を保つ（chakra-ui `IconButton` の `loading` 実装と同型:
/// アイコンと Spinner を並べず Spinner がアイコンの代役を務める）。
fn assemble<'a>(
    props: &ButtonProps,
    icon_only: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut selection: Vec<(&str, &str)> = vec![
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ];
    if icon_only {
        selection.push(("icon", "only"));
    }
    let class = recipe.variant_classes(&selection);

    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), ("class", class.as_str())];
    if props.disabled || props.loading {
        merged.push(("disabled", ""));
        merged.extend(data_disabled(true));
        merged.push(aria_disabled(true));
    }
    if props.loading {
        merged.push(("aria-busy", "true"));
        merged.push(("data-loading", ""));
    }
    merged.extend(drop_class_attr(attrs));

    let node_children = if props.loading {
        let spinner = spinner_decorative(spinner_size_for(props.size), props.palette);
        if icon_only {
            // 上記 rustdoc 参照: icon-only は正方形（padding: 0 +
            // aspect-ratio 1/1）を子ノード 1 個の内容量前提で保っている
            // ため、アイコンを描画に含めず Spinner のみへ置換する。
            vec![spinner]
        } else {
            let mut node_children = Vec::with_capacity(children.len() + 1);
            node_children.push(spinner);
            node_children.extend(children);
            node_children
        }
    } else {
        children
    };

    ANATOMY.part("root", "button", merged, node_children)
}

/// IconButton（イシュー #830）: アイコンのみを表示する正方形の Button
/// variant 拡張。`children` へ呼び出し側が構築したアイコンノード
/// （[`crate::icon::icon`] 等）を渡す。
///
/// `label` はアクセシブルネームとして必須の `aria-label` を組み立てる
/// （視覚的にテキストラベルを持たないボタンのため）。`label.trim()` が
/// 空文字の場合は固定フォールバック（`"unlabeled button"`）へ置換し、
/// 空の `aria-label=""` を決して出力しない（fail-closed、安全側既定）。
/// ただし `attrs` に呼び出し側が既に `aria-label`（大文字小文字を無視）を
/// 指定している場合はそちらを優先し、既定/フォールバック値は追加しない
/// （`aria-label` の重複出力による無効な HTML・後勝ちの非決定的な描画を
/// 防ぐ、`fandhe_frontend_headless_ui::number_input` の
/// `increment_trigger`/`decrement_trigger` と同型の dedup 契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, render};
/// use fandhe_frontend_pre_styled_ui::button::{icon_button, ButtonProps};
/// use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
///
/// let node = icon_button(
///     &ButtonProps::default(),
///     "Search",
///     vec![],
///     vec![icon(
///         &IconProps { label: None, ..IconProps::default() },
///         vec![],
///         vec![el("path", vec![("d", "M12 2L2 22h20z")], vec![])],
///     )],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-label="Search""#));
/// assert!(html.contains("fd-button--icon-only"));
/// ```
#[must_use]
pub fn icon_button<'a>(
    props: &ButtonProps,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label = normalize_label(label, ICON_BUTTON_FALLBACK_LABEL);
    let mut merged_attrs = attrs;
    if !has_caller_aria_label(&merged_attrs) {
        // 呼び出し側が空文字/空白のみの `aria-label` を渡した場合、そのまま
        // 残すとフォールバック値と合わせて `aria-label` が 2 個出力されて
        // しまう（dedup 契約違反）。無効な既存エントリを除去してから
        // フォールバックを追加し、常に高々 1 個の `aria-label` を保証する。
        merged_attrs.retain(|(k, _)| !k.eq_ignore_ascii_case("aria-label"));
        merged_attrs.push(aria_label(label));
    }
    assemble(props, true, merged_attrs, children)
}

/// CloseButton（イシュー #830）: 装飾用途の × アイコンを内包する IconButton
/// 特化版（Button variant 拡張、[`icon_button`] 経由）。
///
/// アイコンは本関数が内部で組み立てる（[`crate::icon::icon`] +
/// 決定的なインライン SVG path、外部リソース非参照）ため、`children` 引数を
/// 取らない。`label` は [`icon_button`] と同じ fail-closed 規約に従うが、
/// 空文字時の既定値は chakra-ui `CloseButton` と同値の `"Close"`。
/// `attrs` 経由の呼び出し側 `aria-label` 優先・重複防止の契約も
/// [`icon_button`] と同一。
///
/// variant の既定は [`ButtonProps::default`]（`Solid`）のまま変更しない
/// （暗黙の既定差し替えをしない Rust 最適化形の判断）。chakra-ui の
/// `ghost` 既定相当の見た目にしたい場合は、呼び出し側が
/// `ButtonProps { variant: ButtonVariant::Ghost, .. }` を明示的に渡す。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::button::{close_button, ButtonProps, ButtonVariant};
///
/// let node = close_button(
///     &ButtonProps { variant: ButtonVariant::Ghost, ..ButtonProps::default() },
///     "",
///     vec![],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-label="Close""#));
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn close_button<'a>(
    props: &ButtonProps,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let label = normalize_label(label, CLOSE_BUTTON_DEFAULT_LABEL);
    let icon_node = icon(
        &IconProps {
            size: props.size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", CLOSE_ICON_PATH)], vec![])],
    );
    let mut merged_attrs = attrs;
    if !has_caller_aria_label(&merged_attrs) {
        // icon_button と同じ理由で、無効な既存 `aria-label` を除去してから
        // フォールバックを追加する（dedup 契約、高々 1 個の `aria-label`）。
        merged_attrs.retain(|(k, _)| !k.eq_ignore_ascii_case("aria-label"));
        merged_attrs.push(aria_label(label));
    }
    assemble(props, true, merged_attrs, vec![icon_node])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_solid_md_type_button() {
        let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<button data-scope="button" data-part="root" type="button" "#,
                r#"class="fd-button--size-md fd-button--variant-solid fd-button--color-palette-accent">Save</button>"#,
            )
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ButtonVariant::Solid, "fd-button--variant-solid"),
            (ButtonVariant::Outline, "fd-button--variant-outline"),
            (ButtonVariant::Ghost, "fd-button--variant-ghost"),
            (ButtonVariant::Subtle, "fd-button--variant-subtle"),
            (ButtonVariant::Surface, "fd-button--variant-surface"),
            (ButtonVariant::Plain, "fd-button--variant-plain"),
        ] {
            let props = ButtonProps {
                variant,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md {class} fd-button--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-button--size-xs"),
            (Size::Sm, "fd-button--size-sm"),
            (Size::Md, "fd-button--size-md"),
            (Size::Lg, "fd-button--size-lg"),
            (Size::Xl, "fd-button--size-xl"),
        ] {
            let props = ButtonProps {
                size,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "{class} fd-button--variant-solid fd-button--color-palette-accent"
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラス
    /// （`fd-button--color-palette-<value>`）へ写像されることを固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-button--color-palette-accent"),
            (ColorPalette::Info, "fd-button--color-palette-info"),
            (ColorPalette::Success, "fd-button--color-palette-success"),
            (ColorPalette::Warning, "fd-button--color-palette-warning"),
            (ColorPalette::Danger, "fd-button--color-palette-danger"),
            (ColorPalette::Neutral, "fd-button--color-palette-neutral"),
        ] {
            let props = ButtonProps {
                palette,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md fd-button--variant-solid {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    /// イシュー #606: recipe の静的 CSS に `--fandhe-palette` 系の宣言と
    /// `var(--fandhe-radius-md)` の参照が含まれることを固定する。
    #[test]
    fn css_output_declares_palette_custom_properties_and_radius_token() {
        let out = css();
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-accent)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-info)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-success)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-warning)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
        assert!(out.contains("background: var(--fandhe-palette);"));
        assert!(out.contains("color: var(--fandhe-palette-fg);"));
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
    }

    #[test]
    fn disabled_adds_disabled_data_disabled_and_aria_disabled() {
        let props = ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_adds_aria_busy_data_loading_and_spinner_child() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-loading="""#));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        // spinner は children の先頭に挿入される。
        let spinner_pos = html.find("data-scope=\"spinner\"").unwrap();
        let save_pos = html.find("Save").unwrap();
        assert!(spinner_pos < save_pos);
    }

    #[test]
    fn loading_also_disables_button_to_prevent_duplicate_actions() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_spinner_is_decorative_and_does_not_break_button_name() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(!html.contains(r#"role="status""#));
        assert!(!html.contains("aria-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    /// Bugbot 指摘（PR #628）の回帰テスト: 非 accent palette かつ
    /// `loading: true` のボタンで、埋め込まれる装飾用途 Spinner が
    /// ボタン自身の `colorPalette` 軸を継承すること（`variant_classes` が
    /// `color-palette` 軸未指定時に既定の accent へ補完し、親ボタンの
    /// palette を上書きしてしまう不具合の是正）。
    #[test]
    fn loading_spinner_inherits_button_palette_instead_of_default_accent() {
        let props = ButtonProps {
            loading: true,
            palette: ColorPalette::Danger,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains("fd-spinner--color-palette-danger"));
        assert!(!html.contains("fd-spinner--color-palette-accent"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![("class", "attacker-controlled"), ("id", "save-btn")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
        assert!(html.contains(r#"id="save-btn""#));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![],
            vec![text("<script>alert('xss')</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    }

    // --- イシュー #830: icon_button / close_button ---------------------

    #[test]
    fn icon_button_outputs_icon_only_class_aria_label_and_type_button() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![],
            vec![text("icon")],
        ));
        assert!(html.contains("fd-button--icon-only"));
        assert!(html.contains(r#"aria-label="Search""#));
        assert!(html.contains(r#"type="button""#));
    }

    /// 受け入れ条件 1: `label` が空文字・空白のみの場合にフォールバック
    /// ラベルへ置換し、空の `aria-label=""` を決して出力しない。
    #[test]
    fn icon_button_empty_label_falls_back_and_never_emits_empty_aria_label() {
        for label in ["", "   "] {
            let html = render(&icon_button(&ButtonProps::default(), label, vec![], vec![]));
            assert!(html.contains(r#"aria-label="unlabeled button""#), "{html}");
            assert!(!html.contains(r#"aria-label="""#), "{html}");
        }
    }

    #[test]
    fn icon_button_preserves_loading_and_disabled_three_attrs() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&icon_button(&props, "Search", vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
    }

    /// イシュー #1449 Cursor Bugbot（Medium）指摘の是正: icon-only +
    /// `loading: true` のとき、呼び出し側アイコンを Spinner の隣へ
    /// 追加せず Spinner のみへ置換する（正方形の前提である「子ノード
    /// 1 個」を保つ、`assemble` rustdoc 参照）。
    #[test]
    fn icon_button_loading_replaces_icon_with_spinner_instead_of_appending() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&icon_button(
            &props,
            "Search",
            vec![],
            vec![text("caller-icon-marker")],
        ));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        assert!(
            !html.contains("caller-icon-marker"),
            "loading 中は呼び出し側アイコンを描画せず Spinner のみへ置換するべき: {html}"
        );
    }

    /// 非 icon-only（テキストボタン）は従来どおり Spinner を子ノード先頭へ
    /// 追加し、既存の `children` は描画され続ける（icon-only 例外の
    /// 対象外であることの回帰固定）。
    #[test]
    fn button_loading_still_prepends_spinner_and_keeps_children() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        assert!(html.contains("Save"));
    }

    #[test]
    fn close_button_embeds_decorative_icon_and_default_aria_label_close() {
        let html = render(&close_button(&ButtonProps::default(), "", vec![]));
        assert!(html.contains(r#"aria-label="Close""#));
        assert!(html.contains(r#"data-scope="icon" data-part="root""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains(r#"role="img""#));
        assert!(html.contains("fd-button--icon-only"));
    }

    #[test]
    fn close_button_overridden_label_is_used_and_escaped() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "<script>alert(1)</script>",
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("aria-label=\"&lt;script&gt;alert(1)&lt;/script&gt;\""));
    }

    /// Review 指摘の是正回帰: 呼び出し側が `attrs` 経由で既に `aria-label`
    /// を指定している場合、`icon_button` の既定 `aria-label` を二重に
    /// 出力しない（`aria-label` は高々 1 個。`number_input::increment_trigger`
    /// と同じ dedup 契約）。
    #[test]
    fn icon_button_does_not_duplicate_caller_supplied_aria_label() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "custom label")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="custom label""#));
        assert!(!html.contains(r#"aria-label="Search""#));
    }

    /// 大文字小文字違いの `Aria-Label` でも同一属性とみなして dedup する
    /// （`has_caller_aria_label` は大文字小文字を無視する契約）。
    #[test]
    fn icon_button_dedup_is_case_insensitive() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("Aria-Label", "custom label")],
            vec![],
        ));
        // 属性名は呼び出し側指定の表記（大文字小文字）のまま出力されるため、
        // 小文字化してから件数を数える（dedup 判定自体が大文字小文字を
        // 無視することの確認が目的であり、出力側の表記は問わない）。
        assert_eq!(html.to_lowercase().matches("aria-label=").count(), 1);
        assert!(html.contains(r#"Aria-Label="custom label""#));
    }

    /// Review 指摘の是正回帰: `close_button` も同様に呼び出し側指定の
    /// `aria-label` を優先し、既定値 `"Close"` と重複させない。
    #[test]
    fn close_button_does_not_duplicate_caller_supplied_aria_label() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "",
            vec![("aria-label", "custom close label")],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="custom close label""#));
        assert!(!html.contains(r#"aria-label="Close""#));
    }

    /// Bugbot 指摘の是正回帰（PR #863）: 呼び出し側が `aria-label=""`
    /// （空文字）を渡した場合、`has_caller_aria_label` はキーの存在のみで
    /// 判定してはならない。フォールバック `aria-label`（`"Search"` 相当の
    /// 正規化ラベル）へ必ず差し替え、空のアクセシブルネームを出力しない
    /// （icon-only ボタンの fail-closed 保証）。
    #[test]
    fn icon_button_falls_back_when_caller_aria_label_is_empty() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Search""#));
        assert!(!html.contains(r#"aria-label="""#));
    }

    /// 同様に空白のみの `aria-label` もフォールバック対象とする
    /// （`trim()` 後に空文字と判定される契約）。
    #[test]
    fn icon_button_falls_back_when_caller_aria_label_is_whitespace_only() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "   ")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Search""#));
    }

    /// `close_button` も同様に空文字 `aria-label` をフォールバック
    /// （既定ラベル `"Close"`）させる。
    #[test]
    fn close_button_falls_back_when_caller_aria_label_is_empty() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "",
            vec![("aria-label", "")],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Close""#));
        assert!(!html.contains(r#"aria-label="""#));
    }

    /// 後方互換回帰: 通常の [`button`] は icon 軸に `default_variant` を
    /// 持たないため、`fd-button--icon-` を含む class を一切出力しない
    /// （イシュー #830 受け入れ条件 2、既存 golden HTML の不変性）。
    #[test]
    fn plain_button_never_emits_icon_only_class() {
        let html = render(&button(&ButtonProps::default(), vec![], vec![text("Save")]));
        assert!(!html.contains("fd-button--icon-"));
    }

    /// イシュー #1449: icon-only 修飾 variant 単体（size 非依存）は
    /// `aspect-ratio: 1 / 1` + `padding: 0` のみを持ち、正方形の確定
    /// `height` は icon×size の compound variant 側が担う
    /// （[`css_output_icon_only_size_compound_declares_fixed_height`]
    /// 参照。codex-review #1731 P1 是正後の役割分担、`recipe()` rustdoc
    /// 参照）。
    #[test]
    fn css_output_contains_icon_only_variant_rule() {
        let out = css();
        assert!(out.contains(".fd-button--icon-only"));
        assert!(out.contains("aspect-ratio: 1 / 1;"));
        assert!(out.contains("padding: 0;"));
    }

    /// イシュー #1449: size variant の `min-height`/`padding`/`font-size` が
    /// `--fandhe-size-control-*` トークン（イシュー #1678 新設）を参照する
    /// ことを固定する。**codex-review #1731 P1 指摘の是正**: ラベル折り返し・
    /// フォント拡大時にラベルがボタン外へあふれないよう、固定 `height` から
    /// `min-height` へ変更した（モジュール冒頭 rustdoc「size スケール・
    /// icon-only・loading」節参照）。
    #[test]
    fn css_output_declares_size_control_tokens() {
        let out = css();
        // `(suffix, height フォールバック, padding-x フォールバック)`。
        // `crate::theme::DEFAULT_SIZES` の既定値と同値（イシュー #1424
        // レビュー指摘と同型のフォールバック契約、モジュール冒頭 rustdoc
        // 参照）。
        for (suffix, height_fallback, padding_x_fallback) in [
            ("xs", "2rem", "0.625rem"),
            ("sm", "2.25rem", "0.75rem"),
            ("md", "2.5rem", "1rem"),
            ("lg", "2.75rem", "1.25rem"),
            ("xl", "3rem", "1.5rem"),
        ] {
            assert!(
                out.contains(&format!(
                    "min-height: var(--fandhe-size-control-height-{suffix}, {height_fallback});"
                )),
                "size={suffix} の min-height トークンが見つからない: {out}"
            );
            assert!(
                out.contains(&format!(
                    "padding: 0 var(--fandhe-size-control-padding-x-{suffix}, {padding_x_fallback});"
                )),
                "size={suffix} の padding-x トークンが見つからない: {out}"
            );
            assert!(
                out.contains(&format!(
                    "font-size: var(--fandhe-size-control-font-size-{suffix}, var(--fandhe-font-font-size-{suffix}));"
                )),
                "size={suffix} の font-size トークンが見つからない: {out}"
            );
        }
    }

    /// **codex-review #1731 P1 指摘の是正の回帰テスト**: icon-only は
    /// `min-height` だけでは `aspect-ratio: 1 / 1` が確定サイズを得られず
    /// 正方形を保証できないため、`icon`×`size` の compound variant
    /// （`.fd-button--icon-only.fd-button--size-<suffix>`）が 5 段ぶんの
    /// 確定 `height` を追加することを固定する（`recipe()` rustdoc 参照）。
    #[test]
    fn css_output_icon_only_size_compound_declares_fixed_height() {
        let out = css();
        for (suffix, height_fallback) in [
            ("xs", "2rem"),
            ("sm", "2.25rem"),
            ("md", "2.5rem"),
            ("lg", "2.75rem"),
            ("xl", "3rem"),
        ] {
            assert!(
                out.contains(&format!(
                    ".fd-button--icon-only.fd-button--size-{suffix} {{\n  height: var(--fandhe-size-control-height-{suffix}, {height_fallback});\n}}"
                )),
                "size={suffix} の icon-only 確定 height compound variant が見つからない: {out}"
            );
        }
    }

    /// イシュー #1449: `loading: true` 時に埋め込む Spinner のサイズが
    /// ボタンの `size` から決定的に写像されることを固定する（`spinner_size_for`
    /// rustdoc 参照: xs/sm/md → Sm、lg/xl → Md）。
    #[test]
    fn loading_spinner_size_follows_button_size() {
        for (size, expected_class) in [
            (Size::Xs, "fd-spinner--size-sm"),
            (Size::Sm, "fd-spinner--size-sm"),
            (Size::Md, "fd-spinner--size-sm"),
            (Size::Lg, "fd-spinner--size-md"),
            (Size::Xl, "fd-spinner--size-md"),
        ] {
            let props = ButtonProps {
                size,
                loading: true,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(expected_class),
                "size={size:?} -> {html} (expected {expected_class})"
            );
        }
    }

    /// イシュー #1449（#1424 §3/§6 準拠）: `:focus-visible` で
    /// `focus_ring_declarations(FocusRingColor::Palette, ...)` の canonical
    /// 宣言（`outline`/`outline-offset`）が出力されることを固定する。
    #[test]
    fn css_output_declares_focus_visible_ring() {
        let out = css();
        assert!(out.contains(r#"[data-scope="button"][data-part="root"]:focus-visible {"#));
        assert!(out.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, \
             var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
        assert!(out.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }
}
