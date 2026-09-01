//! styled NumberInput（headless ラッパー、イシュー #738、親 #520/#545/#736）。
//!
//! `fandhe_frontend_headless_ui::number_input`（イシュー #738）の Label /
//! Control / Input / IncrementTrigger / DecrementTrigger の 5 anatomy
//! パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供
//! する。薄い委譲の根拠は [`crate::switch`]/[`crate::radio_group`] の
//! rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`NumberInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::avatar::root`] と同型）を本モジュール
//! で再定義する。headless 自由関数 `root` と名前衝突するため、`pub use
//! ...::*` ではなく必要な識別子（[`label`]/[`control`]/[`input`]/
//! [`increment_trigger`]/[`decrement_trigger`]/[`NumberInputAction`]/
//! [`NumberInputFlags`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::number_input::NumberInput`] は
//! **あえて**再エクスポートしない（[`crate::switch`] の `Switch` 非再
//! エクスポートと同じ理由）。`NumberInput` は `.root(disabled, invalid,
//! attrs, children)` という inherent メソッドを持つが、これは headless
//! 自由関数 `root` へそのまま委譲するのみで `size` variant クラスを一切
//! 付与しない未スタイルの実体である。本モジュールが `NumberInput` を丸ごと
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `number_input_instance.root(...)` を呼んでしまい、`size` が付与されず
//! 見た目が静かに崩れる事故を誘発する。`NumberInput` による状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::number_input::NumberInput` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]（および再エクスポート済み
//! のパーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` を持たない理由
//!
//! headless 層（`crates/headless-ui/src/number_input.rs`）は連続量の値を
//! 扱うため `data-state` を持たない（モジュール doc 参照）。[`recipe`] の
//! 境界到達時のスタイルは `increment-trigger`/`decrement-trigger` の
//! `data-disabled` 存在属性のみを条件にする。
//!
//! # `size` variant（イシュー #708 方針の踏襲）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-number-input-control-height`/`-font-size`/`-trigger-size`
//! （root スコープの CSS custom property。通常の CSS 継承により
//! `control`/`input`/`increment-trigger`/`decrement-trigger` へ伝わる）
//! 経由で寸法を切り替える。`color-palette` 軸は本コンポーネントでは提供
//! しない（`crate` rustdoc「複合部品の variant 統一方針」の軸提供基準 3
//! に従い、フォーム操作部品として `size` のみを対象とする。選択・チェック
//! 状態を示す色ではなく汎用フォーム入力のため）。base 規則の `var()` には
//! Md 相当のフォールバック値を書き、styled `root` を経由しない headless
//! 直接利用マークアップでも現行外観を維持する（fail-safe）。
//!
//! # トリガーの視覚配置（縦積み、chakra-ui 風）
//!
//! `control` を `position: relative` のコンテナとし、
//! `increment-trigger`/`decrement-trigger` を右端に縦に積むレイアウトを
//! 既定 CSS として提供する（chakra-ui NumberInput の既定見た目に近づける
//! 判断）。フォーカスリングは通常のフォーカス可能要素（`input`/`button`）が
//! ネイティブに受けるため、[`crate::switch`] のような hidden-input 特有の
//! `data-focus-visible` 対応は不要（`input`/トリガー自体がフォーカスを
//! 受ける契約、モジュール doc「`data-state` を持たない理由」参照）。
//!
//! # 参考サイト基準への調整（イシュー #1485）
//!
//! chakra-ui / ark-ui の NumberInput と視覚比較し、Phase 0 で確定した共通
//! 基盤（[`crate::recipe::focus_ring_declarations`]・
//! [`crate::recipe::disabled_declarations`]・
//! [`crate::recipe::transition_declarations`]・#1678 の
//! `--fandhe-size-control-height/padding-x/font-size-*` トークン）へ
//! 移行した。input #1482（[`crate::input`]）・native-select #1484
//! （[`crate::native_select`]）・date-input #1469（[`crate::date_input`]）
//! と同型の是正である。
//!
//! - **`input` パート**: `font: inherit`/`color: var(--fandhe-color-fg)`
//!   を追加し（input.rs base と同一）、border-radius・size の高さ/
//!   フォント/左 padding を #1678 のトークンへ移行。`data-disabled` は
//!   `cursor: not-allowed` のみを付与する（`opacity` は付与しない。
//!   `root` が [`crate::recipe::disabled_declarations`] で既に
//!   `opacity: 0.5` を負うため、`input` へ重ねるとネストした opacity の
//!   掛け算で約 25% まで減光してしまう。[`crate::pin_input`]/
//!   [`crate::date_input`] の segment と同型、Cursor Bugbot 指摘、
//!   イシュー #1485 PR #1764）。`data-readonly` への視覚宣言は追加しない
//!   （[`crate::input`] の「readonly（意図的非採用）」節と同型の判断。
//!   ネイティブ `<input type="text">` は選択・キャレット操作が可能な
//!   ため既定の `cursor: text` のままが適切で、date-input `segment`
//!   〔`<span role="spinbutton">` の非ネイティブ要素〕から流用した
//!   `cursor: default` を付けるとテキストキャレットが消えてしまう
//!   不具合を是正した、Cursor Bugbot 指摘、イシュー #1485 PR #1764）。
//!   フォーカスリングは
//!   `focus_ring_declarations(Token, Outside)`（input.rs と同一）。
//!   hover 背景は付与しない（input.rs と同じ判断: `<input>` は
//!   `cursor: text` でありインタラクティブ slot の hover 対象外、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` 参照）。
//! - **`increment-trigger`/`decrement-trigger` パート**: `<button>` で
//!   `cursor: pointer` を持つインタラクティブ slot のため
//!   [`crate::recipe::hover_bg_muted`] + `.state(Hover,
//!   hover_surface_declarations())` を新設する（date-input の `segment`
//!   と同型）。フォーカスリングは
//!   `focus_ring_declarations(Token, Inset)` を採用する
//!   （**Inset を選ぶ理由**: トリガーは `control` 内に
//!   `position: absolute` で密着配置されており、Outside（+2px）リングは
//!   `input` の枠線・隣接トリガーへ重なって視認性を損なうため、
//!   splitter/listbox 等と同じ符号反転 inset を採る）。境界到達時の
//!   `data-disabled` は `cursor: not-allowed` のみを [`recipe`] へ登録し
//!   （従来の `opacity: 0.4` 直書きから変更、共通 disabled ビジュアル
//!   言語の `opacity: 0.5` への統一という意図は維持）、`opacity: 0.5`
//!   自体は [`stylesheet`] が `root:not([data-disabled])` を祖先に持つ
//!   場合に限る raw CSS として追記する。`root` と両トリガーが同時に
//!   `data-disabled` になる通常の全体 disabled 構成で `root` の
//!   `opacity: 0.5` とトリガー自身の `opacity: 0.5` が二重適用され
//!   実効 0.25 まで減光してしまう不具合の是正（codex-review P1 指摘、
//!   イシュー #1485 PR #1764。詳細は [`stylesheet`] rustdoc 参照）。
//! - **`root` パート**: `data-disabled` の `opacity: 0.5` 直書きを
//!   [`crate::recipe::disabled_declarations`] へ置換する（`cursor:
//!   not-allowed` が純追加される）。
//! - **バリアント軸（意図的非採用）**: chakra の `variant`
//!   （outline/subtle/flushed）相当の軸は本イシューでは追加しない。
//!   根拠: (1) native-select #1484・date-input #1469 が同判断で見送り
//!   済み、(2) `root()` へのシグネチャ追加は 0.x の破壊的変更（minor
//!   バンプ・呼び出し元全修正）を伴い意匠調整 patch の粒度を超える、
//!   (3) input #1482 で確立した `InputVariant` 語彙を number-input へ
//!   写像する設計判断は Forms 家族横断で行うべき
//!   （`.claude/rules/out-of-scope-tracking.md` 対応）。
//!
//! 既存の CSS 変数名（`--fandhe-number-input-control-height`/
//! `-font-size`/`-trigger-size`）とクラス名・セレクタは一切削除・改名しない
//! （削除・改名は minor バンプ要件になるため）。値の付け替えと新規宣言の
//! 追加のみに留め、patch バンプ判定を維持する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく Scrubber パーツ・キーボード操作の DOM 配線は
//!   スコープ外（`fandhe_frontend_headless_ui::number_input` モジュール
//!   doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   NumberInput 追加は、未公開の新バージョンを参照できないため本イシュー
//!   のスコープ外とする（9c0e4f6 の先例どおり crates.io 公開後に追随）。
//! - chakra の `variant`（outline/subtle/flushed）相当の軸を number-input
//!   へ写像する設計判断は Forms 家族横断で検討すべきであり、本イシューの
//!   スコープ外とする（起票はユーザー承認後）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// `NumberInput` 状態機械・headless 自由関数 `root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::number_input::NumberInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::number_input::{
    control, decrement_trigger, increment_trigger, input, label, NumberInputAction,
    NumberInputFlags,
};

/// headless `number_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/number_input.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "increment-trigger",
    "decrement-trigger",
];

/// この styled NumberInput の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("number-input", SLOTS)
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
            disabled_declarations(),
        )
        .base(
            "label",
            vec![decl(
                "font-size",
                "var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "input",
            vec![
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("font", "inherit"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "height",
                    "var(--fandhe-number-input-control-height, 2.5rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-number-input-trigger-size, 1.5rem) 0 var(--fandhe-number-input-padding-x, 1rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                // input #1482・native-select #1484 が確立した Forms 家族の
                // 標準角丸（`--fandhe-radius-md` は常時定義済みトークンの
                // ためフォールバックリテラルを持たない、input.rs と同一）。
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base(
            "input",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .state(
            "input",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // `data-readonly` への視覚宣言は追加しない（`input` パートは
        // ネイティブ `<input type="text">` であり、[`crate::input`] の
        // 「readonly（意図的非採用）」節と同型の判断。ネイティブ
        // `<input readonly>` は選択・キャレット操作が可能なため既定の
        // `cursor: text` のままが適切で、`cursor: default` を付けると
        // テキストキャレットが消え非対話に見えてしまう。date-input
        // `segment`（`<span role="spinbutton">` の非ネイティブ要素）へ
        // 付けた `cursor: default` を誤って流用していた不具合の是正
        // （Cursor Bugbot 指摘、イシュー #1485 PR #1764）。
        //
        // `opacity` は `root` のみに適用する（[`crate::pin_input`]/
        // [`crate::date_input`] の segment と同じ方針）。呼び出し側は
        // `input` が disabled になるとき常に `root` も disabled にする
        // 契約であり（`root` 独立で `input` のみ disabled になる正当な
        // 状態はない）、両パーツへ `opacity: 0.5` を重ねるとネストした
        // opacity の掛け算で `input` が実質約 25% まで減光し `root`
        // （50%）と不整合になる。`cursor: not-allowed` のみ `input` にも
        // 適用し、減光は `root` の 1 箇所に一元化する（Cursor Bugbot 指摘、
        // イシュー #1485 PR #1764）。
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
            "increment-trigger",
            vec![
                decl("position", "absolute"),
                decl("right", "1px"),
                decl("top", "1px"),
                decl("width", "var(--fandhe-number-input-trigger-size, 1.5rem)"),
                decl("height", "50%"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
                hover_bg_muted(),
            ],
        )
        .base(
            "increment-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "increment-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "increment-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // opacity は root と increment-trigger の data-disabled が同時に
        // 真になる通常の全体 disabled 構成で二重適用（0.5 × 0.5 = 0.25）
        // してしまうため、ここでは cursor のみを適用する。opacity 0.5
        // は境界到達（root は disabled でない）ときに限り [`stylesheet`]
        // が祖先セレクタ付き raw CSS として追加する
        // （[`crate::listbox`] の item hover raw CSS と同型のパターン、
        // codex-review P1 指摘、イシュー #1485 PR #1764）。
        .state(
            "increment-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "decrement-trigger",
            vec![
                decl("position", "absolute"),
                decl("right", "1px"),
                decl("bottom", "1px"),
                decl("width", "var(--fandhe-number-input-trigger-size, 1.5rem)"),
                decl("height", "50%"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
                hover_bg_muted(),
            ],
        )
        .base(
            "decrement-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "decrement-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "decrement-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // increment-trigger と同じ理由・同じ是正（上記コメント参照）。
        .state(
            "decrement-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // size（イシュー #1678 の `--fandhe-size-control-height/padding-x/
        // font-size-*` トークンへ移行、イシュー #1485。input #1482・
        // native-select #1484 と同一のフォールバック値）。
        // `--fandhe-number-input-trigger-size` は共有トークンに該当段が
        // ないため component-local のまま維持し、新高さスケールに釣り合う
        // 値へ調整する。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-number-input-control-height",
                    "var(--fandhe-size-control-height-xs, 2rem)",
                ),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
                decl(
                    "--fandhe-number-input-padding-x",
                    "var(--fandhe-size-control-padding-x-xs, 0.625rem)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.25rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-number-input-control-height",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
                decl(
                    "--fandhe-number-input-padding-x",
                    "var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.375rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-number-input-control-height",
                    "var(--fandhe-size-control-height-md, 2.5rem)",
                ),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
                decl(
                    "--fandhe-number-input-padding-x",
                    "var(--fandhe-size-control-padding-x-md, 1rem)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.5rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-number-input-control-height",
                    "var(--fandhe-size-control-height-lg, 2.75rem)",
                ),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg))",
                ),
                decl(
                    "--fandhe-number-input-padding-x",
                    "var(--fandhe-size-control-padding-x-lg, 1.25rem)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.625rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-number-input-control-height",
                    "var(--fandhe-size-control-height-xl, 3rem)",
                ),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl))",
                ),
                decl(
                    "--fandhe-number-input-padding-x",
                    "var(--fandhe-size-control-padding-x-xl, 1.5rem)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.75rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled NumberInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
///
/// # トリガー disabled の opacity を祖先セレクタ付き raw CSS で追記する理由
/// （codex-review P1 指摘、イシュー #1485 PR #1764）
///
/// headless 層の `NumberInput::increment_trigger`/`decrement_trigger`
/// （`crates/headless-ui/src/number_input.rs`）は呼び出し側の全体
/// `disabled` と境界到達（`can_increment`/`can_decrement` が偽）を
/// `||` で合成した最終値を各トリガーの `data-disabled` へ渡す。このため
/// NumberInput 全体を disabled にする通常の構成では `root` と両トリガーが
/// 同時に `data-disabled` を持つ。[`recipe`] の `increment-trigger`/
/// `decrement-trigger` の `data-disabled` 規則は `cursor: not-allowed` の
/// みを持ち `opacity` を含めない（[`input`] パートの `opacity` を `root`
/// のみに一元化した方針と同型、`recipe` 内コメント参照）ため、`root` の
/// `opacity: 0.5` がトリガーへも自然継承されるだけで済み、二重適用
/// （0.5 × 0.5 = 0.25 まで減光）が起きない。
///
/// 一方で「境界到達のみでトリガー単独が disabled、`root` は disabled で
/// ない」構成では、トリガー自身の減光が要る（ボタンが押せないことを
/// 示す視覚的フィードバック）。[`SlotRecipe::state`] が生成するセレクタは
/// 常に `[data-scope="number-input"][data-part="<slot>"]` を先頭に固定した
/// 自パーツ属性条件のみで、祖先パーツ（`root`）の属性を検査するセレクタを
/// 組めないため、本関数が [`crate::listbox::stylesheet`] の item hover
/// raw CSS 追記と同型のパターンで、`root:not([data-disabled])` を祖先に
/// 持つ場合に限りトリガーへ `opacity: 0.5` を追加する規則を末尾へ追記する。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    for part in ["increment-trigger", "decrement-trigger"] {
        let selector = format!(
            "[data-scope=\"number-input\"][data-part=\"root\"]:not([data-disabled]) \
                [data-scope=\"number-input\"][data-part=\"{part}\"][data-disabled]"
        );
        if let Some(rule) = serialize_rule(&selector, &[decl("opacity", "0.5")]) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    }
    out
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::number_input::root`]
/// へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::number_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = number_input::root(Size::Md, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="number-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::number_input::root(disabled, invalid, merged, children)
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
        assert!(a.contains(r#"[data-scope="number-input"][data-part="control"]"#));
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
        assert!(css.contains(
            r#"[data-scope="number-input"][data-part="increment-trigger"][data-disabled] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="number-input"][data-part="decrement-trigger"][data-disabled] {"#
        ));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_input_to_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="number-input"][data-part="input"][data-invalid] {"#));
        assert!(css.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains("fd-number-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-number-input--size-xs"),
            (Size::Sm, "fd-number-input--size-sm"),
            (Size::Md, "fd-number-input--size-md"),
            (Size::Lg, "fd-number-input--size-lg"),
            (Size::Xl, "fd-number-input--size-xl"),
        ] {
            let html = render(&root(size, false, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
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
        assert!(css.contains("--fandhe-number-input-control-height"));
        assert!(css.contains("--fandhe-number-input-trigger-size"));
    }

    #[test]
    fn stylesheet_references_size_control_tokens_for_all_five_sizes() {
        // イシュー #1485: #1678 の共有 size-control トークンへ移行した
        // ことを固定する（input #1482・native-select #1484 と同型）。
        let css = stylesheet();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                css.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "missing height token for {suffix}: {css}"
            );
            assert!(
                css.contains(&format!("--fandhe-size-control-padding-x-{suffix}")),
                "missing padding-x token for {suffix}: {css}"
            );
            assert!(
                css.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "missing font-size token for {suffix}: {css}"
            );
        }
    }

    #[test]
    fn input_and_triggers_use_canonical_focus_ring() {
        let css = stylesheet();
        let expected_outline =
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));";
        assert!(css.contains(expected_outline));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    #[test]
    fn triggers_hover_rule_is_wrapped_in_hover_media_query() {
        let css = stylesheet();
        let media_idx = css
            .find("@media (hover: hover) {")
            .expect("hover media query block must exist");
        let media_block = &css[media_idx..];
        assert!(media_block.contains(
            r#"[data-scope="number-input"][data-part="increment-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(media_block.contains(
            r#"[data-scope="number-input"][data-part="decrement-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(media_block.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn root_uses_canonical_disabled_declarations() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="number-input"][data-part="root"][data-disabled] {"#));
        let root_idx = css
            .find(r#"[data-scope="number-input"][data-part="root"][data-disabled] {"#)
            .expect("root disabled rule must exist");
        let root_block = &css[root_idx..];
        let block_end = root_block.find('}').unwrap_or(root_block.len());
        assert!(root_block[..block_end].contains("opacity: 0.5;"));
        assert!(root_block[..block_end].contains("cursor: not-allowed;"));
    }

    #[test]
    fn triggers_disabled_own_rule_has_cursor_only_no_opacity() {
        // codex-review P1 指摘（イシュー #1485 PR #1764）是正の固定:
        // トリガー自身の `[data-disabled]` 規則（`root` の状態を問わず
        // 常に適用される規則）は `cursor: not-allowed` のみを持ち
        // `opacity` を含まない。`opacity` は `root` からの通常継承、
        // または下記 `triggers_disabled_opacity_is_scoped_to_root_enabled`
        // が検証する祖先スコープ付き raw CSS のいずれか一方でのみ効く
        // ことで、二重適用（0.25 まで減光）を防ぐ。
        let css = stylesheet();
        for part in ["increment-trigger", "decrement-trigger"] {
            let selector =
                format!(r#"[data-scope="number-input"][data-part="{part}"][data-disabled] {{"#);
            let idx = css
                .find(&selector)
                .unwrap_or_else(|| panic!("{part} disabled rule must exist: {css}"));
            let block = &css[idx..];
            let block_end = block.find('}').unwrap_or(block.len());
            let body = &block[..block_end];
            assert!(body.contains("cursor: not-allowed;"), "{part}: {body}");
            assert!(
                !body.contains("opacity"),
                "{part} own rule must not carry opacity: {body}"
            );
        }
    }

    #[test]
    fn triggers_disabled_opacity_is_scoped_to_root_enabled() {
        // 境界到達（root は disabled でない）による trigger 単独 disabled
        // の減光（0.5）は、root 祖先が disabled でないことを条件にした
        // raw CSS 規則としてのみ出力される（[`stylesheet`] rustdoc 参照）。
        let css = stylesheet();
        for part in ["increment-trigger", "decrement-trigger"] {
            let selector = format!(
                r#"[data-scope="number-input"][data-part="root"]:not([data-disabled]) [data-scope="number-input"][data-part="{part}"][data-disabled] {{"#
            );
            assert!(
                css.contains(&selector),
                "missing scoped opacity rule for {part}: {css}"
            );
            let idx = css.find(&selector).unwrap();
            let block = &css[idx..];
            let block_end = block.find('}').unwrap_or(block.len());
            assert!(block[..block_end].contains("opacity: 0.5;"));
        }
    }

    #[test]
    fn input_does_not_style_data_readonly() {
        // Cursor Bugbot 指摘（イシュー #1485 PR #1764）是正の固定:
        // ネイティブ `<input type="text">` の readonly はテキストキャレット
        // を維持すべきで（[`crate::input`] の「readonly（意図的非採用）」
        // 節と同型）、`data-readonly` へ視覚宣言（`cursor: default` 等）を
        // 追加しない。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="number-input"][data-part="input"][data-readonly]"#));
    }

    #[test]
    fn input_disabled_still_applies_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="number-input"][data-part="input"][data-disabled] {"#));
        let idx = css
            .find(r#"[data-scope="number-input"][data-part="input"][data-disabled] {"#)
            .expect("input disabled rule must exist");
        let block = &css[idx..];
        let block_end = block.find('}').unwrap_or(block.len());
        assert!(block[..block_end].contains("cursor: not-allowed;"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            false,
            false,
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
            PAYLOAD,
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_number_input_state_machine() {
        // `NumberInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`NumberInput` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_headless_ui::number_input::NumberInput;

        let mut n = NumberInput::new(Some(0.0), 0.0, 10.0, 1.0);
        assert_eq!(n.value(), Some(0.0));

        let ssr_html = render(&n.control(false, false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-part="control""#));

        assert!(dispatch(&mut n, "increment", ""));
        let hydrate_html = render(&render_for_hydration(&n));
        assert!(hydrate_html.contains(r#"data-hydrate-value="1""#));

        let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
        assert_eq!(restored, n);
    }
}
