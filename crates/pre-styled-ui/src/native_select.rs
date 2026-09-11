//! styled NativeSelect（イシュー #737、親 #736、祖父トラッキング #726）。
//!
//! [`crate::input`](mod@crate::input) と同型の薄い委譲層。
//! `fandhe_frontend_headless_ui::field::select`（#538/#602）が出力する
//! `data-scope="field"` `data-part="select"` へ `variant`/`size` variant
//! クラスと既定 CSS を重ねる。設計方針・状態機械を持たない理由・`field`
//! scope を共有する理由は [`crate::input`](mod@crate::input) rustdoc を参照（本モジュールは
//! 重複を避けるため差分のみ記す）。
//!
//! # ネイティブ矢印を維持する（`appearance: none` を使わない）設計判断
//!
//! chakra-ui の `NativeSelect` はカスタム矢印アイコン（`NativeSelect.Indicator`）
//! を重ねるため `appearance: none` でブラウザ既定の矢印を消す構成が一般的だが、
//! 本イシューは「ブラウザネイティブ挙動を尊重する」という本クレートの
//! 設計原則（indicator パーツはスコープ外、モジュール rustdoc・PR 本文
//! 参照）に従い、ネイティブの矢印・開閉挙動をそのまま残す最小サブセットと
//! する。`appearance` プロパティの宣言自体を持たない。
//!
//! # ネイティブ `readonly` を出力しない理由（headless 層への委譲）
//!
//! `<select readonly>` は HTML 仕様上無効な属性のため、headless
//! `field::select`（イシュー #602）はネイティブ `readonly` を出力しない
//! （`data-readonly` は他コントロール同様に出力する）。本モジュールは
//! この判断を再実装せず、そのまま委譲する。
//!
//! # `option` 子ノード
//!
//! `core` は `select` ショートカットタグを意図的に持たない
//! （`crates/core/src/tags.rs` 冒頭 doc 参照）ため、呼び出し側が
//! `fandhe_frontend_core::el("option", ..., ...)` で組み立てて `children` に
//! 渡す（headless `field::select` rustdoc と同じ契約）。
//!
//! # 参考サイト基準への調整（イシュー #1484）
//!
//! chakra-ui v3 NativeSelect と視覚比較し、Phase 0 で確定した共通基盤
//! （[`crate::recipe::focus_ring_declarations`]・
//! [`crate::recipe::disabled_declarations`]・
//! [`crate::recipe::transition_declarations`]・#1678 の
//! `--fandhe-size-control-height/padding-x/font-size-*` トークン）へ移行
//! した。[`crate::input`](mod@crate::input)（イシュー #1482）の差分をそのまま `select` slot へ
//! 写像したもので、実装差分は無い（両モジュールとも `field` scope 下の 1
//! slot・variant 3 種 × size 5 段の同型構造のため）。
//!
//! - **hover（意図的非採用）**: hover 背景は付与しない。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` の判定基準
//!   （hover はインタラクティブ slot = `cursor: pointer` を持つ slot のみ）
//!   に対しネイティブ `<select>` は既定カーソルが矢印であり対象外。chakra
//!   v3 NativeSelect recipe（`mcp__chakra-ui__get_component_example` で確認）
//!   もコンポーネント合成のみで hover 背景変化を宣言していない。
//! - **readonly（意図的非採用）**: `data-readonly` への視覚宣言は追加しない。
//!   [`crate::input`](mod@crate::input) と同判断（参照サイトも readonly の独自装飾を持たない）。
//! - **ネイティブ矢印維持**: 本モジュール冒頭の既存設計判断（`appearance:
//!   none` 不使用）を変更しない。chakra のカスタム `Indicator` への追随は
//!   引き続き意図的非採用。
//!
//! # shadcn/ui 突合（イシュー #2017）
//!
//! [shadcn/ui Native Select](https://ui.shadcn.com/docs/components/base/native-select)
//! を補完参照（ルート #2001 Phase 0 で確定の適用原則。既存の視覚言語を
//! shadcn 風へ置き換えることは目的としない。2026-09-07 のユーザー判断
//! 〔イシュー #2153、`docs/design/shadcn-reference-adoption-policy.md`
//! §8〕で shadcn/ui は主基準の 1 つへ改訂されたが、この判断は改訂後も
//! 不変）として突合した結果、
//! `recipe()`/CSS 出力に実体変更は不要と判断した。以下、確認した項目を
//! 記録する。
//!
//! - **`size` 段階（差分なし）**: shadcn の `sm`/`default` の 2 段は、
//!   既存 xs〜xl の 5 段（#1678）に包含される（[`crate::input`](mod@crate::input) 判定と
//!   同型）。
//! - **disabled（差分なし）**: shadcn は `disabled:pointer-events-none
//!   disabled:cursor-not-allowed` + ラッパー側 `opacity-50` を宣言するが、
//!   本モジュールの [`crate::recipe::disabled_declarations`] が既に
//!   `opacity: 0.5; cursor: not-allowed;` を宣言済みであり、
//!   `pointer-events: none` はネイティブ `disabled` 属性が既に操作不能化
//!   するため冗長（追加不要）。
//! - **`aria-invalid`（`data-invalid`）時の box-shadow リング（意図的
//!   非採用）**: 本フレームワークのフォーカスリング規約（#1424、
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）は
//!   実装手段を `outline` へ統一し、新規に `box-shadow` によるリングを
//!   追加しない方針を確定済み（[`crate::input`](mod@crate::input) 判定と同型）。既存の
//!   `data-invalid` → `border-color` のみを維持する。
//! - **chevron アイコンのラッパー表現（意図的非採用の再確認）**: 本
//!   モジュール冒頭「ネイティブ矢印を維持する」設計判断（`appearance:
//!   none` 不使用）を変更しない。shadcn は補完参照であり、確定済みの
//!   視覚言語を上書きしない（Phase 0 原則。#2153 改訂後も上書きしない判断は
//!   不変）。
//! - **`optgroup`（是正、コード変更なし）**: headless
//!   `field::select`（[`fandhe_frontend_headless_ui::field::select`]）は
//!   `children: Vec<Node>` をそのまま透過するため、呼び出し側が
//!   `fandhe_frontend_core::el("optgroup", vec![("label", "...")], ...)`
//!   で `<optgroup>` を組み立てれば現状の API のまま描画できる（本
//!   モジュール自体のコード変更は不要）。欠けていたのは docs サイトの
//!   Demo・Examples への可視化のみであり、`crates/docs-site/src/
//!   showcase.rs` の Native Select 節へ optgroup を含むインスタンスを
//!   追加した。
//! - **`<option>`/`<optgroup>` への system color 適用（採用、イシュー
//!   #2204。旧「暫定非採用」の記録は本節末尾の「#2017 時点の判断との
//!   差分」に残す）**: shadcn は `<option>`/`<optgroup>` へ system color
//!   キーワード（`Canvas`/`CanvasText`）を宣言し、OS 側で描画される
//!   ネイティブポップアップの配色を明示している。chakra-ui v3 も
//!   `nativeSelectSlotRecipe` で `& > option, & > optgroup` へ
//!   セマンティック `bg` トークンを宣言する（`color` は宣言せず field の
//!   `fg` 継承に委ねる）。主基準 3 者のうち chakra-ui / shadcn-ui の 2 者が
//!   着色し、Radix Themes は native select 相当の部品自体を持たないため
//!   （`docs/design/component-coverage-map.md`）、着色そのものは採用と
//!   判断した。
//!
//!   **手段の評価（DSL 拡張 vs 生 CSS 追記）**: [`crate::recipe::
//!   StateCondition`] は擬似クラス条件のみを表現でき、`select > option`
//!   のような子孫/子結合子セレクタを組み立てる手段を持たない。この
//!   子孫セレクタ機構自体を `SlotRecipe` へ追加する案（イシュー #708 で
//!   「追加しない」と確定、イシュー #2201 の `PseudoElement` rustdoc でも
//!   再確認済みの判断）は非採用とし、[`crate::status`]・[`crate::field`]
//!   が既に採る「`recipe().css()` の外側で固定 CSS 文字列を追記する」
//!   パターンを踏襲した（`css()` 内で `recipe().css()` の戻り値へ
//!   `push_str` する。セレクタはソース中の固定リテラルのみで、生セレクタ
//!   を受け取る公開 API は増えない）。
//!
//!   **値の競合判定（`docs/design/shadcn-reference-adoption-policy.md`
//!   §8 の部品ごと判断）**: 背景色は chakra-ui の値（`--fandhe-color-bg`
//!   トークン）を採る。`select` 本体の base が既に同トークンで着色されて
//!   おり、option 側も揃えることでテーマ配色と完全一致するため。shadcn の
//!   `Canvas` は `var(--fandhe-color-bg, Canvas)` の第 2 引数（フォール
//!   バック値）として取り込み、トークン未定義環境でも `color-scheme`
//!   追従の system color で可読性を確保する。文字色（`color`）は
//!   chakra-ui に倣い明示しない（`select` の `color` を継承させる）。
//!   `CanvasText` を明示すると UA の `option:disabled` 減色（`GrayText`
//!   相当）とテーマ `fg` トークンの双方を上書きしてしまうため。
//!   `optgroup > option` も対象に含める理由は `background` が継承
//!   されない CSS プロパティであり、optgroup 配下の option がポップアップ
//!   既定色に戻るのを防ぐため（shadcn も option 単位で付与しており同じ
//!   被覆範囲）。
//!
//!   **確認できたこと・できなかったこと（実機確認環境、誇張しない）**:
//!   Playwright（chromium）で docs サイトの `/themes/native-select/` を
//!   開き、light / `data-theme="dark"` の双方で `getComputedStyle(option)
//!   .backgroundColor` がトークン解決値と一致すること、`--fandhe-color-bg`
//!   未定義時に `Canvas` フォールバックへ解決すること、無効 option の
//!   `color`（UA 既定の減色）が変更前後で変化しないことを確認した。これは
//!   「CSS が要素へ適用されること」の確認であり、OS/ブラウザが描画する
//!   ネイティブポップアップの外観そのものの確認ではない。本イシュー実装
//!   時の実行環境は macOS（Chrome/Safari は OS 描画のポップアップのため
//!   ポップアップ外観は本 CSS を反映しない）で、Windows/Linux の
//!   Chrome/Firefox における実際のポップアップ描画は未検証のまま残る。
//!   **再評価トリガー**: Windows/Linux でポップアップ配色の不具合報告が
//!   あれば、本節の判定値（トークン優先・`color` 非明示）を再評価する。
//!
//!   **#2017 時点の判断との差分（歴史記録）**: PR #2154（イシュー
//!   #2017）は上記制約を理由に本項目を「暫定非採用」としてイシュー
//!   #2204 へ切り出していた。#2204 で DSL 拡張ではなく生 CSS 追記の
//!   precedent を採用することで技術的制約を解消し、採用へ転換した。
//! - **合成パターン（label/description との組み合わせ、是正）**:
//!   `native_select` はラベル・補助テキストの型階層を持たず、`field`
//!   （`/themes/field/`）が担う（[`crate::input`](mod@crate::input) の同型判断）。
//!   `field::label`/`field::helper_text`/`field::root` と組み合わせる
//!   Example を docs サイト（`crates/docs-site/src/component_specs/
//!   forms.rs` の `NATIVE_SELECT` spec）へ追加した。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, transition_declarations, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

pub use fandhe_frontend_headless_ui::field::{FieldIds, FieldProps};

/// この styled NativeSelect が扱う slot。
const SLOTS: &[&str] = &["select"];

/// NativeSelect の見た目 variant（chakra-ui `NativeSelect` の `variant`
/// 相当。`Flushed` の代わりに枠なしの `Plain` を持つ点が
/// [`crate::input::InputVariant`]/[`crate::textarea::TextareaVariant`] との
/// 差異）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeSelectVariant {
    /// 枠線あり（既定）。
    #[default]
    Outline,
    /// 淡色背景・枠線なし。
    Subtle,
    /// 枠・背景なし（装飾なしの最小サブセット）。
    Plain,
}

impl VariantValue for NativeSelectVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Subtle => "subtle",
            Self::Plain => "plain",
        }
    }
}

/// [`native_select`] の見た目設定。
#[derive(Debug, Clone, Copy)]
pub struct NativeSelectProps {
    /// 見た目 variant（既定 `Outline`）。
    pub variant: NativeSelectVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for NativeSelectProps {
    fn default() -> Self {
        NativeSelectProps {
            variant: NativeSelectVariant::Outline,
            size: Size::Md,
        }
    }
}

/// この styled NativeSelect の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみ
/// が呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut base = vec![
        decl("box-sizing", "border-box"),
        decl("width", "100%"),
        decl("font", "inherit"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("background", "var(--fandhe-color-bg)"),
        // input #1482 が確立した Forms 家族の標準角丸（旧
        // `--fandhe-radius-sm` から変更、イシュー #1484）。
        decl("border-radius", "var(--fandhe-radius-md)"),
    ];
    base.extend(transition_declarations(
        "border-color, background",
        MotionDuration::Fast,
    ));

    SlotRecipe::new("field", SLOTS)
        .base("select", base)
        .state(
            "select",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "select",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "select",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // size（イシュー #1678 の `--fandhe-size-control-height/padding-x/
        // font-size-*` トークンへ移行、イシュー #1484。input #1482 と同一値）。
        .variant(
            Size::Xs,
            "select",
            vec![
                decl("height", "var(--fandhe-size-control-height-xs, 2rem)"),
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
            "select",
            vec![
                decl("height", "var(--fandhe-size-control-height-sm, 2.25rem)"),
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
            "select",
            vec![
                decl("height", "var(--fandhe-size-control-height-md, 2.5rem)"),
                decl("padding", "0 var(--fandhe-size-control-padding-x-md, 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "select",
            vec![
                decl("height", "var(--fandhe-size-control-height-lg, 2.75rem)"),
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
            "select",
            vec![
                decl("height", "var(--fandhe-size-control-height-xl, 3rem)"),
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
            NativeSelectVariant::Outline,
            "select",
            vec![decl("border", "1px solid var(--fandhe-color-border)")],
        )
        .variant(
            NativeSelectVariant::Subtle,
            "select",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border", "1px solid transparent"),
            ],
        )
        .variant(
            NativeSelectVariant::Plain,
            "select",
            vec![
                decl("background", "transparent"),
                decl("border", "1px solid transparent"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(NativeSelectVariant::Outline)
}

/// この styled NativeSelect が生成する静的 CSS 全量を返す（決定的。
/// [`crate::field::css`]/[`crate::status::css`] と同じ契約）。
///
/// [`SlotRecipe`] の `state()` は同一 slot 内の属性条件のみを表現でき、
/// `select` の子孫である `<option>`/`<optgroup>` へのネストセレクタは
/// 表現できないため、`field.rs`/`status.rs` と同型の直接追記（recipe
/// 出力の末尾に静的 CSS 文字列を連結）で表す（system color 適用手段の
/// 評価、イシュー #2204。モジュール doc「shadcn/ui 突合」節参照）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    out.push('\n');
    out.push_str(
        // `color` は明示しない（`select` の `color` 継承に委ね、UA の
        // `option:disabled` 減色を上書きしない。モジュール doc「値の
        // 競合判定」節参照）。`optgroup > option` を含めるのは
        // `background-color` が継承されないため。
        "[data-scope=\"field\"][data-part=\"select\"] > option,\n\
         [data-scope=\"field\"][data-part=\"select\"] > optgroup,\n\
         [data-scope=\"field\"][data-part=\"select\"] > optgroup > option {\n  \
         background-color: var(--fandhe-color-bg, Canvas);\n}\n",
    );
    out
}

/// styled `select` パーツを組み立てる。`variant`/`size` に応じたクラスを
/// 付与し（`drop_class_attr` により呼び出し側の `class` は除去してから
/// 合成する）、アクセシビリティ配線は
/// [`fandhe_frontend_headless_ui::field::select`] へそのまま委譲する。
///
/// `children` は `fandhe_frontend_core::el("option", ..., ...)` で組み立てた
/// `<option>` 要素列を渡す（モジュール rustdoc「`option` 子ノード」参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, render, text};
/// use fandhe_frontend_pre_styled_ui::native_select::{
///     self, FieldIds, FieldProps, NativeSelectProps,
/// };
///
/// let field = FieldProps {
///     id: "country",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let option = el("option", vec![("value", "jp")], vec![text("Japan")]);
/// let node = native_select::native_select(
///     &NativeSelectProps::default(),
///     &field,
///     vec![],
///     vec![option],
/// );
/// assert!(render(&node).contains(r#"data-scope="field" data-part="select""#));
/// ```
#[must_use]
pub fn native_select<'a>(
    props: &NativeSelectProps,
    field: &FieldProps<'_>,
    extra_attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(extra_attrs));
    fandhe_frontend_headless_ui::field::select(field, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, render, text};

    fn default_field(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    fn option(value: &str, label: &str) -> Node {
        el("option", vec![("value", value)], vec![text(label)])
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="field"][data-part="select"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_never_declares_appearance() {
        // ネイティブ矢印を維持する設計判断（モジュール rustdoc 参照）の回帰。
        let out = css();
        assert!(!out.contains("appearance"));
    }

    #[test]
    fn stylesheet_uses_canonical_focus_ring_declarations() {
        // イシュー #1484: focus ring がリテラル値ではなく canonical ヘルパ
        // （`focus_ring_declarations`）由来のトークン参照であることを固定。
        let out = css();
        assert!(out.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(out.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn stylesheet_uses_motion_token_transition() {
        // イシュー #1484: transition がリテラル秒数ではなく motion トークン
        // （`transition_declarations`）由来であることを固定。
        let out = css();
        assert!(out.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(out.contains("transition-property: border-color, background;"));
    }

    #[test]
    fn stylesheet_size_variants_use_control_tokens() {
        // イシュー #1484: 各 size が #1678 の control トークンへ移行した
        // ことを固定（input #1482 と同型の 3 点セット）。
        let out = css();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                out.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "height token missing for {suffix} -> {out}"
            );
            assert!(
                out.contains(&format!("--fandhe-size-control-padding-x-{suffix}")),
                "padding-x token missing for {suffix} -> {out}"
            );
            assert!(
                out.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "font-size token missing for {suffix} -> {out}"
            );
        }
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![option("jp", "Japan")],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="select""#));
        assert!(html.contains(r#"<option value="jp">Japan</option>"#));
    }

    #[test]
    fn default_variant_is_outline_and_md() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-field--variant-outline"));
        assert!(html.contains("fd-field--size-md"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (NativeSelectVariant::Outline, "fd-field--variant-outline"),
            (NativeSelectVariant::Subtle, "fd-field--variant-subtle"),
            (NativeSelectVariant::Plain, "fd-field--variant-plain"),
        ] {
            let field = default_field("f");
            let props = NativeSelectProps {
                variant,
                ..NativeSelectProps::default()
            };
            let html = render(&native_select(&props, &field, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-field--size-xs"),
            (Size::Sm, "fd-field--size-sm"),
            (Size::Md, "fd-field--size-md"),
            (Size::Lg, "fd-field--size-lg"),
            (Size::Xl, "fd-field--size-xl"),
        ] {
            let field = default_field("f");
            let props = NativeSelectProps {
                size,
                ..NativeSelectProps::default()
            };
            let html = render(&native_select(&props, &field, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn readonly_does_not_emit_native_attribute_but_keeps_data_readonly() {
        let mut field = default_field("f");
        field.readonly = true;
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(!html.contains(r#" readonly="""#));
        assert!(html.contains(r#"data-readonly=""#));
    }

    #[test]
    fn invalid_and_disabled_flags_propagate_from_field_props() {
        let mut field = default_field("f");
        field.invalid = true;
        field.disabled = true;
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid=""#));
        assert!(html.contains(r#"data-disabled=""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"disabled=""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn option_children_text_payload_is_escaped_on_render() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![option("x", "<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn extra_attrs_attribute_breakout_payload_is_escaped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn optgroup_label_attribute_breakout_payload_is_escaped() {
        // イシュー #2017: shadcn/ui 突合で「optgroup は現状 API のまま描画
        // できる」と判定した際、`<optgroup label="...">` の `label` 属性値が
        // `el()`/`render()` の既定エスケープ経路を通ることを固定する
        // （`extra_attrs_attribute_breakout_payload_is_escaped` と同型だが、
        // native_select 自身の extra_attrs ではなく children 側の optgroup
        // 属性という別コンテキストの回帰）。
        let field = default_field("f");
        let optgroup = el(
            "optgroup",
            vec![("label", "\" onmouseover=\"alert(1)")],
            vec![option("jp", "Japan")],
        );
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![optgroup],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
        assert!(html.contains("<optgroup"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="select""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn stylesheet_styles_option_and_optgroup_children() {
        // イシュー #2204: option/optgroup の背景色を select と同じ
        // `--fandhe-color-bg` トークン（`Canvas` フォールバック）で着色する
        // 生 CSS 追記の回帰。
        let out = css();
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"select\"] > option,\n\
             [data-scope=\"field\"][data-part=\"select\"] > optgroup,\n\
             [data-scope=\"field\"][data-part=\"select\"] > optgroup > option {\n  \
             background-color: var(--fandhe-color-bg, Canvas);\n}"
        ));
    }

    #[test]
    fn stylesheet_never_sets_option_color() {
        // イシュー #2204: option/optgroup ブロックへ `color:` 宣言
        // （`CanvasText` 等）を追加しない回帰。`select` の `color` 継承に
        // 委ね、UA の `option:disabled` 減色を上書きしないための判断
        // （モジュール rustdoc「値の競合判定」節参照）。
        let out = css();
        assert!(!out.contains("CanvasText"));
        let option_block_start = out
            .find("[data-part=\"select\"] > option,")
            .expect("option block should exist");
        // `background-color:` は含むため単純な "color:" 部分一致では誤検知
        // する。`color:` 単独の宣言（改行/インデント直後の "color:"）が
        // 存在しないことを確認する。
        assert!(!out[option_block_start..].contains("\n  color:"));
    }
}
