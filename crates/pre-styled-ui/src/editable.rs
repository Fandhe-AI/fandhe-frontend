//! styled Editable（headless ラッパー、イシュー #745、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::editable`（イシュー #745）の Label / Area /
//! Input / Preview / Control / EditTrigger / SubmitTrigger / CancelTrigger の
//! 8 anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::number_input`]/[`crate::slider`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Editable` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::number_input::root`]/[`crate::slider::root`] と同型）を本
//! モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::editable::Editable`] は
//! **あえて**再エクスポートしない（[`crate::number_input`] の `NumberInput`
//! 非再エクスポートと同じ理由）。`Editable` は `.root(...)` という inherent
//! メソッドを持つが、これは headless 自由関数 `root` へそのまま委譲するのみ
//! で `size` variant クラスを一切付与しない未スタイルの実体である。本
//! モジュールが `Editable` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`editable_instance.root(...)` を呼んでしまい、
//! `size` が付与されず見た目が静かに崩れる事故を誘発する。状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::editable::Editable` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # `size` variant（イシュー #708 方針の踏襲）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-editable-font-size`（root スコープの CSS custom property。
//! 通常の CSS 継承により `input`/`preview` へ伝わる）経由で寸法を切り替える。
//! `color-palette` 軸は本コンポーネントでは提供しない（`crate` rustdoc
//! 「複合部品の variant 統一方針」の軸提供基準 3 に従い、フォーム操作部品
//! として `size` のみを対象とする。[`crate::number_input`] と同じ判断）。
//! base 規則の `var()` には Md 相当のフォールバック値を書き、styled `root`
//! を経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe）。
//!
//! # `input`/`preview` の重ね合わせレイアウト（PR #792 Bugbot 指摘対応、Medium）
//!
//! `area` を CSS Grid の単一セル（`display: grid`）とし、`input`/`preview`
//! の双方に `grid-area: 1 / 1` を与えて同一グリッドセルへ重ねる
//! （chakra-ui Editable の既定見た目に近づける判断）。両者は headless 層の
//! `hidden` 属性で排他表示され、非表示側は `display: none`（`preview` は
//! 直上の `[hidden]` 規則、`input` は要素の UA 既定 `display` に対し本
//! モジュールが `display` を宣言しないため UA 既定の `[hidden]{display:none}`
//! がそのまま効く）になるため、グリッドの track サイズは表示中の 1 パーツ
//! のみで決まる。`position: relative` だけを宣言し `input`/`preview` を
//! 通常フローに残した旧実装は、両者が `area` の `inline-flex` 内で並んで
//! 描画され「重ね合わせ」にならず、chakra-ui 由来の見た目契約に反していた
//! （Bugbot 指摘）。
//!
//! # スタイル調整（イシュー #1476、`preview`/`input` パートのみ。
//! `area` は担当範囲だが後述のとおり変更なし）
//!
//! 親イシュー #1475（chakra-ui / ark-ui 基準への意匠調整）の分割 1/2。
//! `edit-trigger`/`submit-trigger`/`cancel-trigger`/`root`/`label`/`control`
//! は兄弟イシュー #1477（2/2、後述の「スタイル調整（イシュー #1477）」節）の
//! スコープだった（同一ファイルを共有する 2 PR のコンフリクト最小化。
//! combobox 1/2（PR #1744）・checkbox 1/2（PR #1734）と同型の分割運用）。
//!
//! 7 軸チェックリスト（`docs/design/pre-styled-ui-interaction-visual-language.md`）
//! との突合で担当 3 パートに加えた変更・意図的に加えなかった変更:
//!
//! - **色**: `input` に `color` 宣言がなく UA 既定色に依存していたため
//!   （ダークテーマで前景色が崩れる恐れ）、`color: var(--fandhe-color-fg)`
//!   を base へ追加した（[`crate::combobox`] の `control` と同型）。
//! - **フォーカス**: `input` にフォーカスリングが一切なかったため、
//!   `.state("input", StateCondition::FocusVisible, focus_ring_declarations(...))`
//!   を追加した。`editable` は `ColorPalette` 軸を持たないため
//!   [`FocusRingColor::Token`] を使う（`docs/design/
//!   pre-styled-ui-focus-ring-and-size-conventions.md` の canonical 形のみを
//!   使う規約に従い、`:focus` 直書きはしない）。
//! - **状態（`data-*`）**: headless が `input` へ出す `data-readonly`
//!   （`fandhe_frontend_headless_ui::editable::input`）が非視覚だったため、
//!   `cursor: default` を追加した（[`crate::date_input`] の `segment`
//!   readonly 対応と同型）。**`input[data-disabled]` へ `opacity` は追加
//!   しない**: `root[data-disabled]` が既に `opacity: 0.5`
//!   （CSS の継承により `input` へも及ぶ）を担っており、`input` 側へも
//!   同じ宣言を足すと実効 `opacity` が `0.5 * 0.5 = 0.25` へ二重適用
//!   されてしまうため、disabled の視覚化は root 側の 1 箇所に分担を
//!   固定する。
//! - **トランジション**: `input`/`preview` とも `transition` 宣言が
//!   なかったため、[`transition_declarations`] を既存 `base` ブロックを
//!   書き換えずに純追加した（`.base(slot, ...)` の複数回登録は同一 slot
//!   への出力が連結される契約、[`crate::combobox`]/[`crate::date_input`]
//!   と同型）。
//! - **hover（`preview`、意図的に非採用）**: chakra-ui `Editable.Preview`
//!   は参照ドキュメント（`.claude/skills/chakra-ui` 経由の公式リファレンス）
//!   上も淡い hover 面を持たず、`docs/design/reference-screenshots/
//!   chakra-editable-*.png`・`ark-editable-*.png` にも hover 状態のスタイル
//!   差分が確認できない（`preview` は `cursor: text` の「テキストらしさ」
//!   を保つ操作面であり、`edit-trigger` 等のボタン系パートとは異なり
//!   hover 面を持たせる参照側の意匠が存在しない）。このため
//!   `hover_bg_muted()`/`hover_surface_declarations()` は採用しない。
//! - **バリアント（variant 軸）**: chakra 相当の variant 軸追加は Forms
//!   家族横断の語彙判断であり、部品単独で先行導入しない（combobox #1467・
//!   checkbox #1454・date-input #1469 と同一の判断軸）。
//! - **サイズ・余白・角丸・影・ダーク**: 既に規約準拠（`size` は root の
//!   `--fandhe-editable-font-size` 経由で継承済み、トークンはダーク対応
//!   済み）のため変更なし。`area` の Grid 重ね合わせレイアウト（PR #792）
//!   も参照側の見た目契約を既に満たしており変更なし。
//!
//! # スタイル調整（イシュー #1477、分割 2/2。コントロール群
//! `edit-trigger`/`submit-trigger`/`cancel-trigger` と、上記 1/2 が
//! 先送りした `root`/`label`/`control` の点検）
//!
//! 親イシュー #1475 の分割 2/2。上記「スタイル調整（イシュー #1476）」節の
//! 3 パートに続き、残りのコントロール群と `root`/`label`/`control` を
//! 7 軸チェックリストと突合した。
//!
//! - **色**: `edit-trigger`/`submit-trigger`/`cancel-trigger` はいずれも
//!   `border: none; background: transparent` のみで前景色が UA 既定に
//!   依存していたため、`color: var(--fandhe-color-fg-muted)` を追加した
//!   （`date_picker::clear-trigger`〔#1747〕と同型。「テキストらしさ」を
//!   保つ `preview` とは異なりボタン系パートのため muted 色を採用）。
//!   ボタン要素は UA 既定でフォントサイズを継承しないため、
//!   `font-size: var(--fandhe-editable-font-size, ...)` も併せて追加し、
//!   `size` variant の寸法切り替えへ載せた。
//! - **面・角丸**: `display: inline-flex` + 中央寄せ（`align-items`/
//!   `justify-content: center`）・`padding: var(--fandhe-space-1)`・
//!   `border-radius: var(--fandhe-radius-sm)` を追加し、小型アイコン
//!   ボタンとしての枠を明示した（従来はテキストノードそのままの寸法に
//!   依存していた）。
//! - **hover**: 3 種とも [`hover_bg_muted`] で `--fandhe-hover-bg` を定義し、
//!   `.state(slot, StateCondition::Hover, hover_surface_declarations())`
//!   を追加した（面なし ghost 系ボタンの標準、`date_picker::trigger`/
//!   `clear-trigger` と同型）。
//! - **フォーカス**: 3 種とも `.state(slot, StateCondition::FocusVisible,
//!   focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside))`
//!   を追加した（`editable` は `ColorPalette` 軸を持たないため `input` と
//!   同じ [`FocusRingColor::Token`]）。
//! - **状態（`data-*`）・disabled の標準化**: 独自の
//!   `cursor: not-allowed; opacity: 0.4` を [`disabled_declarations`]
//!   （`opacity: 0.5` + `cursor: not-allowed`）へ置換した。`input` は
//!   root の `opacity: 0.5` 継承へ一本化する分担だったが（イシュー #1476、
//!   PR #1751 codex-review P1 対応）、トリガー 3 種は headless 層が
//!   個別に `disabled`/`data-disabled` を付与し得る独立したボタンであり、
//!   root 非 disabled のまま単独で disabled になり得るため、自前の
//!   dimming を保持する判断とした。root と同時に disabled になった場合
//!   （継承 0.5 × 自前 0.5）実効 opacity が 0.25 まで沈むが、「全体無効時に
//!   ボタンがさらに沈む」許容差として受け入れる（`input` とは異なり単独
//!   disabled が主要ユースケースであるため）。
//! - **`[hidden]` 上書き（最重要の落とし穴）**: headless 層はモードに応じて
//!   `edit-trigger`（edit 時）/`submit-trigger`・`cancel-trigger`（preview
//!   時）へ `hidden` 存在属性を出して排他表示する。base への
//!   `display: inline-flex` 追加は UA 既定 `[hidden] { display: none }`
//!   （詳細度 (0,1,0)）を `[data-scope][data-part]`（(0,2,0)）で上書きして
//!   しまうため、`preview`（イシュー #1476、PR #792 Bugbot 指摘対応と同型）
//!   に倣い 3 パートすべてへ `[hidden] { display: none }` を明示追加した
//!   （表示排他の回帰防止、[`stylesheet`] のテスト参照）。
//! - **トランジション**: [`transition_declarations`] を base へ純追加した
//!   （`background, color`、`MotionDuration::Fast`）。
//! - **`root`（点検、変更なし）**: `display: inline-flex; flex-direction:
//!   column; gap: space-1` + `[data-disabled] { opacity: 0.5 }` は
//!   [`crate::number_input`] の `root` と同型であり既に規約準拠のため
//!   変更なし。
//! - **`label`（点検、変更なし）**: `font-size` のみの宣言は
//!   [`crate::date_picker`]/[`crate::number_input`] の `label` と同水準で
//!   あり変更なし。
//! - **`control`（点検、`align-items: center` を追加）**: `display:
//!   inline-flex; gap: space-1` に `align-items: center` を追加した
//!   ([`crate::date_picker`] の `control` と同型）。トリガー間の高さの
//!   わずかな差異による縦ズレを防ぐ。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく activationMode/submitMode の実挙動・autoResize は
//!   スコープ外（`fandhe_frontend_headless_ui::editable` モジュール doc
//!   参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Editable 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::slider`] の先例どおり crates.io 公開後に
//!   追随）。
//! - headless 層の挙動・examples への Editable 追加は既存の記述どおり。
//!   `crates/docs-site/tests/wrap_state.rs::extract_headless_refs` の
//!   コメント走査 panic 対策（コメント行を
//!   `fandhe_frontend_headless_ui::` で終わらせない）はスキャナ本体の
//!   修正を伴うため引き続き本モジュールのスコープ外とする。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// `Editable` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::editable::Editable` を直接 import する。
pub use fandhe_frontend_headless_ui::editable::{
    area, cancel_trigger, control, edit_trigger, input, label, preview, submit_trigger, EditMode,
    EditableAction, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `editable` anatomy の `data-part` 一覧（`crates/headless-ui/src/editable.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "area",
    "input",
    "preview",
    "control",
    "edit-trigger",
    "submit-trigger",
    "cancel-trigger",
];

/// この styled Editable の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("editable", SLOTS)
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
            vec![decl("opacity", "0.5")],
        )
        .base(
            "label",
            vec![decl(
                "font-size",
                "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        .base(
            "area",
            vec![decl("position", "relative"), decl("display", "inline-grid")],
        )
        .base(
            "input",
            vec![
                decl("grid-area", "1 / 1"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（イシュー #1476。combobox 1/2
        // （PR #1744）の「既存 base ブロックを書き換えない」パターンを
        // 踏襲する。本モジュール冒頭 rustdoc「スタイル調整」節参照）。
        .base(
            "input",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        // headless 層が `input` へ出す `data-readonly`
        // （`fandhe_frontend_headless_ui::editable::input`）を視覚化する
        // （イシュー #1476）。`opacity` は含めない: disabled の視覚化
        // （opacity 0.5）は root 側が継承で担う分担であり、`input` 側へも
        // 重ねると実効 opacity が二重適用される（本モジュール冒頭 rustdoc
        // 参照）。
        //
        // `EditableInputFlags` は `disabled` と `readonly` を同時に
        // true にでき、その場合 `input` へ両方の data-* 属性が付与される
        // （`[data-disabled]`/`[data-readonly]` は同じ specificity
        // (0,3,0)）。このため本規則は disabled 規則より**先に**登録し、
        // `cursor: not-allowed`（disabled）が `cursor: default`
        // （readonly）を CSS のソース順で確実に上書きするようにする
        // （disabled の視覚表現を優先。イシュー #1476 PR #1751
        // codex-review P1 / Cursor Bugbot Medium 指摘「readonly 規則が
        // disabled カーソルを上書きする」対応）。
        .state(
            "input",
            StateCondition::Attr("data-readonly"),
            vec![decl("cursor", "default")],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state(
            "input",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .base(
            "preview",
            vec![
                decl("grid-area", "1 / 1"),
                decl("display", "inline-block"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid transparent"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("cursor", "text"),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（イシュー #1476。上記 `input` と
        // 同じ「既存 base ブロックを書き換えない」パターン）。hover は
        // 意図的に非採用（本モジュール冒頭 rustdoc「スタイル調整」節参照）。
        .base(
            "preview",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // PR #792 Bugbot 指摘対応（High）: preview の base 規則が
        // `display: inline-block` を宣言しており、UA 既定の
        // `[hidden] { display: none }`（詳細度 (0,1,0)）を
        // `[data-scope][data-part]`（詳細度 (0,2,0)）が上書きしてしまう。
        // edit モードで headless 層が付与する `hidden` 存在属性を確実に
        // 非表示化として機能させるため、より詳細度の高い `[hidden]`
        // 属性セレクタで `display: none` を明示的に上書きする
        // （`crate::dialog` の positioner[hidden] と同型の対処）。
        .state(
            "preview",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "preview",
            StateCondition::Attr("data-placeholder-shown"),
            vec![decl("color", "var(--fandhe-color-fg-muted, currentColor)")],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        // トリガー 3 種（edit/submit/cancel）は共通のビジュアル言語
        // （`docs/design/pre-styled-ui-interaction-visual-language.md` §5）に
        // 揃えた小型 ghost アイコンボタンとして扱う（イシュー #1477、
        // `date_picker::clear-trigger`〔#1747〕と同型）。3 種とも同一内容。
        .base(
            "edit-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "edit-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // headless 層（`fandhe_frontend_headless_ui::editable`）は edit
        // モード時に `edit-trigger` へ `hidden` 存在属性を付与して非表示化
        // する。base の `display: inline-flex`
        // （`[data-scope][data-part]`、詳細度 (0,2,0)）が UA 既定
        // `[hidden] { display: none }`（(0,1,0)）を上書きしてしまうため、
        // `preview`（イシュー #1476、PR #792 Bugbot 指摘対応と同型）に
        // 倣い `[hidden]` 属性セレクタで明示的に上書きする（表示排他の
        // 回帰防止、モジュール冒頭 rustdoc 参照）。
        .state(
            "edit-trigger",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "edit-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "edit-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "edit-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "submit-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "submit-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // headless 層は preview モード時に `submit-trigger`/`cancel-trigger`
        // へ `hidden` を付与する（`edit-trigger` と表示排他の向きが逆。
        // 同じ理由で `[hidden]` 上書きが必要、上記コメント参照）。
        .state(
            "submit-trigger",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "submit-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "submit-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "submit-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "cancel-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "cancel-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "cancel-trigger",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "cancel-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "cancel-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "cancel-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .variant(
            Size::Xs,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-xs, 0.75rem)",
            )],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-xs, 0.75rem)",
            )],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-lg)",
            )],
        )
        .default_variant(Size::Md)
}

/// この styled Editable が生成する静的 CSS 全量を返す（決定的。
/// [`crate::number_input::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::editable::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::editable;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = editable::root(
///     Size::Md,
///     editable::EditMode::Preview,
///     editable::EditableInputFlags::default(),
///     editable::EditableActivationMode::default(),
///     editable::EditableSubmitMode::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="editable" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    mode: EditMode,
    flags: EditableInputFlags,
    activation_mode: EditableActivationMode,
    submit_mode: EditableSubmitMode,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::editable::root(
        mode,
        flags,
        activation_mode,
        submit_mode,
        merged,
        children,
    )
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
        assert!(a.contains(r#"[data-scope="editable"][data-part="area"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_triggers_to_disabled_state() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="edit-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="submit-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="cancel-trigger"][data-disabled] {"#)
        );
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_preview_to_placeholder_shown_state() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="editable"][data-part="preview"][data-placeholder-shown] {"#));
    }

    #[test]
    fn input_focus_visible_uses_canonical_focus_ring() {
        // イシュー #1476: `input` にフォーカスリングが一切なかった不足を
        // 是正する。canonical 形（`outline`/`outline-offset` の 2 宣言、
        // `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`）
        // で出力されることを固定する。
        let css = stylesheet();
        let selector = r#"[data-scope="editable"][data-part="input"]:focus-visible {"#;
        assert!(css.contains(selector), "{css}");
        let rule_start = css.find(selector).expect("input focus-visible rule");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        let body = &rule_body[..rule_end];
        assert!(body.contains("outline: var(--fandhe-focus-ring-width"));
        assert!(body.contains("outline-offset: var(--fandhe-focus-ring-offset"));
    }

    #[test]
    fn input_readonly_attribute_gets_default_cursor() {
        // イシュー #1476: headless が出す `data-readonly`（非視覚）を
        // `cursor: default` として視覚化する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="editable"][data-part="input"][data-readonly] {"#));
        let selector = r#"[data-scope="editable"][data-part="input"][data-readonly] {"#;
        let rule_start = css.find(selector).expect("input readonly rule");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("cursor: default;"));
    }

    #[test]
    fn input_disabled_rule_does_not_duplicate_opacity() {
        // イシュー #1476: disabled の視覚化（opacity 0.5）は
        // `root[data-disabled]`（CSS 継承）が担う分担であり、`input` 側の
        // `[data-disabled]` 規則へ `opacity` を重ねて二重適用しないことを
        // 固定する（本モジュール冒頭 rustdoc「スタイル調整」節参照）。
        let css = stylesheet();
        let selector = r#"[data-scope="editable"][data-part="input"][data-disabled] {"#;
        let rule_start = css.find(selector).expect("input disabled rule");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        let body = &rule_body[..rule_end];
        assert!(body.contains("cursor: not-allowed;"));
        assert!(!body.contains("opacity"));
    }

    #[test]
    fn input_disabled_cursor_wins_over_readonly_cursor_when_both_present() {
        // イシュー #1476 PR #1751 codex-review P1 / Cursor Bugbot Medium
        // 指摘「readonly 規則が disabled カーソルを上書きする」対応。
        // `EditableInputFlags` は `disabled` と `readonly` を同時に true に
        // でき、その場合 `input` へ `[data-disabled]`/`[data-readonly]`
        // （同じ specificity）が両方付与される。CSS のソース順で
        // disabled 規則（`cursor: not-allowed`）が readonly 規則
        // （`cursor: default`）より後に出力され、カスケードで disabled の
        // 視覚表現が保たれることを固定する。
        let css = stylesheet();
        let readonly_selector = r#"[data-scope="editable"][data-part="input"][data-readonly] {"#;
        let disabled_selector = r#"[data-scope="editable"][data-part="input"][data-disabled] {"#;
        let readonly_pos = css.find(readonly_selector).expect("input readonly rule");
        let disabled_pos = css.find(disabled_selector).expect("input disabled rule");
        assert!(
            readonly_pos < disabled_pos,
            "disabled 規則は readonly 規則より後に出力され、カスケードで \
             cursor: not-allowed を優先しなければならない"
        );
    }

    #[test]
    fn input_and_preview_have_transition_declarations() {
        // イシュー #1476: `input`/`preview` とも transition 宣言がなかった
        // 不足を是正する（3 longhand プロパティで構成、`crate::recipe::
        // transition_declarations` 参照）。
        let css = stylesheet();
        let input_selector = r#"[data-scope="editable"][data-part="input"] {"#;
        let input_start = css.find(input_selector).expect("input base rule");
        let input_body = &css[input_start..];
        assert!(input_body.contains("transition-property: border-color, background, color;"));

        let preview_selector = r#"[data-scope="editable"][data-part="preview"] {"#;
        let preview_start = css.find(preview_selector).expect("preview base rule");
        let preview_body = &css[preview_start..];
        assert!(preview_body.contains("transition-property: background, color;"));
    }

    #[test]
    fn edit_mode_preview_hidden_attr_overrides_display_inline_block() {
        // PR #792 Bugbot 指摘対応（High）: preview の base 規則
        // `display: inline-block` が UA 既定の `[hidden] { display: none }`
        // を詳細度で上書きし、edit モードで headless 層が付与する `hidden`
        // 存在属性があっても preview が表示され続け、preview/edit の排他
        // 表示が壊れる不具合の回帰（`crate::avatar`/`crate::dialog`/
        // `crate::tooltip` で既に対処済みの同種の落とし穴）。`[hidden]`
        // 属性セレクタでの明示的な `display: none` 上書きが出力され、
        // base 規則より後段（= 詳細度同点時に優先される）で登録されることを
        // 固定する。
        let css = stylesheet();
        let preview_hidden_selector = r#"[data-scope="editable"][data-part="preview"][hidden] {"#;
        assert!(css.contains(preview_hidden_selector));
        let rule_start = css
            .find(preview_hidden_selector)
            .expect("preview[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));

        // base 規則（`display: inline-block` を含む）より後に出現すること。
        // 同一詳細度の CSS 規則はソース順で後者が勝つため、順序が逆転すると
        // 上書きが機能しない。
        let base_preview_selector = r#"[data-scope="editable"][data-part="preview"] {"#;
        let base_start = css
            .find(base_preview_selector)
            .expect("base preview rule must be present");
        assert!(base_start < rule_start);
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-editable--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-editable--size-xs"),
            (Size::Sm, "fd-editable--size-sm"),
            (Size::Md, "fd-editable--size-md"),
            (Size::Lg, "fd-editable--size-lg"),
            (Size::Xl, "fd-editable--size-xl"),
        ] {
            let html = render(&root(
                size,
                EditMode::Preview,
                EditableInputFlags::default(),
                EditableActivationMode::default(),
                EditableSubmitMode::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors_and_custom_properties() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-editable-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            EditMode::Preview,
            EditableInputFlags::default(),
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&input(
            EditMode::Edit,
            PAYLOAD,
            PAYLOAD,
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_editable_state_machine() {
        // `Editable` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`Editable` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_headless_ui::editable::Editable;

        let mut e = Editable::new("Ada", None);
        assert_eq!(e.value(), "Ada");

        let ssr_html = render(&e.control(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-part="control""#));

        assert!(dispatch(&mut e, "edit", ""));
        assert!(dispatch(&mut e, "set", "Grace Hopper"));
        let hydrate_html = render(&render_for_hydration(&e));
        assert!(hydrate_html.contains(r#"data-hydrate-draft="Grace Hopper""#));

        let restored = Editable::from_hydration_attrs(&e.hydration_attrs()).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn trigger_group_focus_visible_uses_canonical_focus_ring() {
        // イシュー #1477: トリガー 3 種にフォーカスリングが一切なかった
        // 不足を是正する。`input`（イシュー #1476）と同じ canonical 形
        // （`outline`/`outline-offset` の 2 宣言）で出力されることを固定する。
        let css = stylesheet();
        for part in ["edit-trigger", "submit-trigger", "cancel-trigger"] {
            let selector =
                format!(r#"[data-scope="editable"][data-part="{part}"]:focus-visible {{"#);
            let rule_start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("{part} focus-visible rule missing"));
            let rule_body = &css[rule_start..];
            let rule_end = rule_body.find('}').expect("rule must be closed");
            let body = &rule_body[..rule_end];
            assert!(
                body.contains("outline: var(--fandhe-focus-ring-width"),
                "{part}: {css}"
            );
            assert!(
                body.contains("outline-offset: var(--fandhe-focus-ring-offset"),
                "{part}: {css}"
            );
        }
    }

    #[test]
    fn trigger_group_hover_is_scoped_to_media_hover_query() {
        // イシュー #1477: トリガー 3 種を `hover_bg_muted()` +
        // `hover_surface_declarations()` の ghost ボタン標準へ揃える
        // （`date_picker::trigger`/`clear-trigger` と同型）。タッチ端末の
        // hover 貼り付き対策として `@media (hover: hover)` 配下へ集約され、
        // `:not([data-disabled])` で disabled 規則と衝突しないことを固定する。
        let css = stylesheet();
        let media_start = css
            .find("@media (hover: hover)")
            .expect("hover media block must be present");
        let media_body = &css[media_start..];
        for part in ["edit-trigger", "submit-trigger", "cancel-trigger"] {
            let selector = format!(
                r#"[data-scope="editable"][data-part="{part}"]:hover:not([data-disabled]) {{"#
            );
            assert!(media_body.contains(&selector), "{part}: {css}");
        }
    }

    #[test]
    fn trigger_group_hidden_attr_overrides_display_inline_flex() {
        // イシュー #1477: `edit-trigger` base の `display: inline-flex`
        // 追加は UA 既定 `[hidden] { display: none }`（(0,1,0)）を
        // `[data-scope][data-part]`（(0,2,0)）で上書きしてしまう。headless
        // 層がモードに応じて 3 パートへ `hidden` 存在属性を出す表示排他
        // （`edit-trigger` は edit 時、`submit-trigger`/`cancel-trigger` は
        // preview 時）が壊れないよう、`preview`（イシュー #1476）と同型の
        // `[hidden] { display: none }` 上書きが base より後段に出力される
        // ことを固定する（回帰防止）。
        let css = stylesheet();
        for part in ["edit-trigger", "submit-trigger", "cancel-trigger"] {
            let hidden_selector =
                format!(r#"[data-scope="editable"][data-part="{part}"][hidden] {{"#);
            let rule_start = css
                .find(&hidden_selector)
                .unwrap_or_else(|| panic!("{part}[hidden] rule missing"));
            let rule_body = &css[rule_start..];
            let rule_end = rule_body.find('}').expect("rule must be closed");
            assert!(
                rule_body[..rule_end].contains("display: none;"),
                "{part}: {css}"
            );

            let base_selector = format!(r#"[data-scope="editable"][data-part="{part}"] {{"#);
            let base_start = css
                .find(&base_selector)
                .unwrap_or_else(|| panic!("{part} base rule missing"));
            assert!(
                base_start < rule_start,
                "{part}: [hidden] 規則は base 規則より後に出力されなければならない"
            );
        }
    }

    #[test]
    fn trigger_group_disabled_uses_standard_disabled_declarations() {
        // イシュー #1477: トリガー 3 種の独自 dimming
        // （`cursor: not-allowed; opacity: 0.4`）を共通ビジュアル言語の
        // `disabled_declarations()`（`opacity: 0.5` + `cursor:
        // not-allowed`）へ標準化した（本モジュール冒頭 rustdoc
        // 「スタイル調整（イシュー #1477）」節参照。`input` の
        // root 継承一本化とは異なる分担意図を明記）。
        let css = stylesheet();
        for part in ["edit-trigger", "submit-trigger", "cancel-trigger"] {
            let selector =
                format!(r#"[data-scope="editable"][data-part="{part}"][data-disabled] {{"#);
            let rule_start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("{part}[data-disabled] rule missing"));
            let rule_body = &css[rule_start..];
            let rule_end = rule_body.find('}').expect("rule must be closed");
            let body = &rule_body[..rule_end];
            assert!(body.contains("opacity: 0.5;"), "{part}: {css}");
            assert!(body.contains("cursor: not-allowed;"), "{part}: {css}");
            assert!(!body.contains("0.4"), "{part}: {css}");
        }
    }
}
