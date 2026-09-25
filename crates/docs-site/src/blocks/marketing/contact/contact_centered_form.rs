//! `contact-centered-form` block（イシュー #2826。親トラッキング #2730/#2731
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0066 を主参照とし
//! R0440・R0853 の構造を集約した合成例。中央寄せの見出し + リード文の下に、
//! 幅を絞った問い合わせフォーム（氏名・メール・会社・電話・本文 + 同意
//! チェック + 送信ボタン）を配置する）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `field` / `input` / `textarea` /
//! `checkbox` / `native-select` / `button` の 9 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。送信ボタンは `button::button`
//! （既定 `type="button"`）のまま送信先・バリデーションを持たず、実際の
//! 送信処理は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。同意チェックは
//! SSR 初期状態（未チェック、`CheckedState::Unchecked`）を描画するのみと
//! しつつ、`site/blocks/contact-centered-form.md` が「会社名・電話番号
//! 以外は必須」と記載する契約に合わせて `required: true` を固定指定する
//! （`checkbox::hidden_input` の `required` 属性・`data-required` に
//! 反映される）。状態機械は持たない。
//!
//! # 同意チェックをネイティブ disabled にする理由（レビュー是正）
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input type="checkbox">`
//! であり、`disabled` を渡さない構成では docs サイトが JS ハイドレーション
//! を行わなくてもラベルクリック・キーボード操作でブラウザが `checked` を
//! ネイティブに切り替えてしまう。一方 `control`/`indicator` の見た目
//! （`data-state="unchecked"`・`hidden`）は SSR 時の `checked` 引数から
//! 固定生成されるため追従せず、利用者が同意をオンにしてもチェックマークで
//! 結果を確認できない（「常に未チェックの静的表示」という契約にも反する）。
//! `contact_split_form_image`（`crates/docs-site/src/blocks/marketing/
//! contact/contact_split_form_image.rs`）が radio group の同種の問題を
//! `RadioGroupProps { disabled: true, .. }` で解決した判断を踏襲し、
//! [`CheckboxProps`] の `disabled: true` を渡してネイティブ `disabled`
//! 属性でフォーカス・操作を不能にし、状態が二度と変化しないことを構造的に
//! 保証する（クライアント側の状態配線を追加する代替案・CSS の
//! `:has(:checked)` による見た目同期案〔`indicator` の `hidden` 属性意味論
//! を上書きすることになる〕はいずれも採らない。無 JS の静的合成例という
//! block 全体の設計方針に反するため）。`disabled: true` と `required: true`
//! は HTML 仕様上両立する（disabled 要素は制約検証の対象外になるだけで
//! `required` 属性自体は保持され、本 Demo はそもそも `<form>` を持たず
//! 実行時のネイティブ検証が発生しない）。`disabled_declarations()`（既定
//! `opacity: 0.5` + `cursor: not-allowed`）は [`LAYOUT_CSS`] で中和し、
//! 通常の checkbox と同じ見た目に保つ（`contact_split_form_image`・
//! `changelog_accordion` と同型の中和パターン）。
//!
//! 文言はすべて架空のもの（実在の人物・企業・PII を含まない）。
//!
//! # 集約元 3 件の統合
//!
//! - R0066（主参照）: タグライン + 見出し + 説明 + 氏名・メール・本文の
//!   シンプルな中央寄せフォーム構造をベースにする。
//! - R0440: 同意チェックボックスを取り込む。ただし参照元の「プライバシー
//!   ポリシーへのリンク」は死にリンクを避けるため出力せず、テキストのみの
//!   同意文言にする（本節末尾で詳述）。
//! - R0853: 会社名・電話番号（国番号付き）を含む項目の多い構成と、同意
//!   トグルの存在を取り込む。トグルは checkbox（[`checkbox::root`]）へ
//!   置き換える（UI 部品を追加せず既存部品で合成する制約のため）。
//!
//! # 全幅セルと 2 列グリッド
//!
//! [`LAYOUT_CSS`] の `.blocks-contact-centered-form-grid` は
//! `repeat(2, minmax(0, 1fr))` の 2 列グリッドを敷き、氏・名の 2 フィールド
//! のみ 1 列ずつ占有する。会社名・メール・電話番号・本文は
//! `data-blocks-contact-centered-form-wide` を付けた `field::root` に対し
//! `grid-column: 1 / -1` を適用して全幅にする。`field::root` は
//! `drop_class_attr` により呼び出し側 `class` を除去するため、CSS フックは
//! `class` ではなく `data-*` 属性で渡す（次項参照）。狭い画面
//! （`@media (max-width: 47.99rem)`）ではグリッド自体を 1 列へ戻す。
//!
//! # 電話番号（国番号付き）の横並び表現
//!
//! R0853 の「国別の電話番号」要件を、`native_select::native_select`
//! （国番号、幅固定）+ `input::input`（電話番号本体、残り幅を占有）の横並び
//! flex（`.blocks-contact-centered-form-phone`）として表現する。`<label
//! for>` は電話番号入力（[`FieldProps`] のうち電話番号用の id）だけに
//! 対応付け、国番号 select は `<label>` を持たないため `aria-label="国番号"`
//! を直接付与する（`FieldIds` を分けずに 2 つの独立した `FieldProps`
//! （id が異なる）を使うことで、id 衝突・二重 `<label for>` を避ける）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `field::root` /
//! `input::input` / `textarea::textarea` / `native_select::native_select` /
//! `button::button` / `checkbox::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-contact-centered-form-*` 属性で渡す。
//! 素の `div` には `class` がそのまま効くため、それらは
//! `.blocks-contact-centered-form-*` クラスセレクタを使う。base 宣言
//! （詳細度 0,2,0）を上書きする箇所は
//! `[data-scope=...][data-part=...][data-blocks-contact-centered-form-*]`
//! （詳細度 0,4,0 以上）まで詳細度を上げて書く
//! （`changelog_timeline_subscribe` と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む
//! （`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（`changelog_timeline_subscribe` 等と同型の
//! 判断）。
//!
//! # プライバシーポリシーへのリンクを出さない判断
//!
//! R0440 の同意文言は本来プライバシーポリシーへのリンクを伴うが、`href`
//! を出力すると遷移先を持たない死にリンクになる（`crate::blocks` の login/
//! signup 系 block と同じ制約）。本 Demo は同意チェックのラベルを平文の
//! テキストのみとし、リンクを持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-contact-centered-form-` 接頭辞を
/// 共通化し、フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-contact-centered-form-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールド（`text`/`email` 等）を組み立てる。`wide` が `true` の
/// ときは全幅セル用フックを付与する（[`LAYOUT_CSS`] の
/// `[data-blocks-contact-centered-form-wide]` 参照）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
    required: bool,
    wide: bool,
) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    };
    let mut attrs = vec![("data-blocks-contact-centered-form-field", "")];
    if wide {
        attrs.push(("data-blocks-contact-centered-form-wide", ""));
    }
    field::root(
        &orientation(),
        &props,
        attrs,
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![
                    ("type", input_type),
                    ("autocomplete", autocomplete),
                    ("placeholder", placeholder),
                ],
            ),
        ],
    )
}

/// 国番号付き電話番号フィールド（R0853 の差分。全幅・任意項目）。
fn phone_field() -> Node {
    let phone_id = field_id("phone");
    let country_id = field_id("country");
    let phone_props = FieldProps {
        id: phone_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let country_props = FieldProps {
        id: country_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let country_options = vec![
        el("option", vec![("value", "jp")], vec![text("+81 (JP)")]),
        el("option", vec![("value", "us")], vec![text("+1 (US)")]),
        el("option", vec![("value", "gb")], vec![text("+44 (GB)")]),
    ];

    field::root(
        &orientation(),
        &phone_props,
        vec![
            ("data-blocks-contact-centered-form-field", ""),
            ("data-blocks-contact-centered-form-wide", ""),
        ],
        vec![
            field::label(&phone_props, vec![], vec![text("電話番号（任意）")]),
            div(
                vec![("class", "blocks-contact-centered-form-phone")],
                vec![
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &country_props,
                        vec![("aria-label", "国番号")],
                        country_options,
                    ),
                    input::input(
                        &InputProps::default(),
                        &phone_props,
                        vec![
                            ("type", "tel"),
                            ("autocomplete", "tel"),
                            ("placeholder", "90-1234-5678"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// お問い合わせ内容（全幅・必須の `textarea`）。
fn message_field() -> Node {
    let message_id = field_id("message");
    let props = FieldProps {
        id: message_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![
            ("data-blocks-contact-centered-form-field", ""),
            ("data-blocks-contact-centered-form-wide", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text("お問い合わせ内容")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("placeholder", "ご相談内容をご記入ください。"),
                    ("data-blocks-contact-centered-form-message", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// プライバシーポリシー同意チェック（R0440/R0853 の差分。SSR 初期状態
/// 〔未チェック〕を描画する、リンクは持たない。ネイティブ操作は
/// `disabled: true` で不能にする（モジュール doc「`<form>` を持たない・
/// 送信処理を持たない」節参照）。
fn consent_checkbox() -> Node {
    let props = CheckboxProps {
        required: true,
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-contact-centered-form-consent", "")],
        vec![
            checkbox::hidden_input(&props, "contact-centered-form-consent", "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(
                &props,
                vec![],
                vec![text("プライバシーポリシーに同意します")],
            ),
        ],
    )
}

/// 中央寄せのタグライン + 見出し + 説明文（`header` 領域）。
fn header() -> Node {
    div(
        vec![("class", "blocks-contact-centered-form-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-contact-centered-form-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("ご相談・ご質問をお寄せください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("2 営業日以内に担当者からご連絡します。")],
            ),
        ],
    )
}

/// `contact-centered-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-contact-centered-form-grid")],
        vec![
            text_field(
                field_id("first-name"),
                "姓",
                "text",
                "family-name",
                "山田",
                true,
                false,
            ),
            text_field(
                field_id("last-name"),
                "名",
                "text",
                "given-name",
                "太郎",
                true,
                false,
            ),
            text_field(
                field_id("company"),
                "会社名（任意）",
                "text",
                "organization",
                "株式会社サンプル",
                false,
                true,
            ),
            text_field(
                field_id("email"),
                "メールアドレス",
                "email",
                "email",
                "you@example.com",
                true,
                true,
            ),
            phone_field(),
            message_field(),
        ],
    );

    let form = div(
        vec![("class", "blocks-contact-centered-form-form")],
        vec![
            grid,
            consent_checkbox(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-contact-centered-form-submit", "")],
                vec![text("送信する")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-contact-centered-form-layout")],
        vec![header(), form],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-centered-form/",
    title: "contact-centered-form",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_centered_form.rs",
    demo_class: "blocks-contact-centered-form",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_centered_form` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。他 block と同型に本ファイル内 private 定数として
/// [`BLOCK`] の `layout_css`（[`LayoutCss::Static`]）で自己申告し、
/// `crate::blocks::stylesheet` が `all_blocks()` 走査で `push_css` する。
///
/// セレクタは `.blocks-contact-centered-form-*` と
/// `[data-blocks-contact-centered-form-*]`、および styled `field`/`button`
/// の `[data-scope=...]` 系セレクタへの上書き（モジュール doc「全幅セルと
/// 2 列グリッド」節参照）のみを用い、他 block や部品の素のセレクタへ
/// 影響させない。
const LAYOUT_CSS: &str = "\
.blocks-contact-centered-form-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-contact-centered-form-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-3);\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
.blocks-contact-centered-form-form {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  width: 100%;\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
.blocks-contact-centered-form-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-contact-centered-form-wide] {\n  grid-column: 1 / -1;\n}\n\
.blocks-contact-centered-form-phone {\n  display: flex;\n  flex-wrap: nowrap;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-contact-centered-form-phone [data-scope=\"field\"][data-part=\"select\"] {\n  flex: 0 0 auto;\n  width: auto;\n}\n\
.blocks-contact-centered-form-phone [data-scope=\"field\"][data-part=\"input\"] {\n  flex: 1;\n  min-width: 0;\n}\n\
[data-scope=\"field\"][data-part=\"textarea\"][data-blocks-contact-centered-form-message] {\n  min-height: 8rem;\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-contact-centered-form-consent][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-contact-centered-form-submit] {\n  width: 100%;\n}\n\
@media (max-width: 47.99rem) {\n  \
.blocks-contact-centered-form-grid {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 9 種の部品を含むことを固定する（`crates/docs-site/
    /// tests/blocks_contract.rs` の横断検査と重複し過ぎない範囲での個別
    /// 固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"field\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="textarea""#));
        assert!(html.contains(r#"data-part="select""#));
        assert!(html.contains(r#"data-part="input""#));
    }

    /// `<form>` を出力しない・送信先を持たない静的表示で、`type="button"`
    /// がちょうど 1 個であること。
    #[test]
    fn demo_has_exactly_one_type_button_and_no_form() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 1);
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("action="));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    /// 各 field の `<label for>` が対応する input/textarea の `id` と一致し、
    /// 国番号 select には `aria-label` が付与されていること。
    #[test]
    fn labels_point_at_their_controls() {
        let html = render(&demo());
        for suffix in ["first-name", "last-name", "company", "email", "phone"] {
            let control_id = format!("blocks-contact-centered-form-{suffix}-control");
            assert!(
                html.contains(&format!(r#"for="{control_id}""#)),
                "expected a label pointing at {control_id} in {html}"
            );
            assert!(
                html.contains(&format!(r#"id="{control_id}""#)),
                "expected control id {control_id} in {html}"
            );
        }
        assert!(html.contains(r#"aria-label="国番号""#));
    }

    /// 同意チェックが静的な未チェック表示であること。
    #[test]
    fn consent_checkbox_is_unchecked_static() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-contact-centered-form-consent"#));
        assert!(html.contains(r#"data-state="unchecked""#));
    }

    /// 同意チェックがネイティブ `disabled` により操作不能であること、
    /// つまり `checked`/視覚表示が永続的に食い違わないことを固定する
    /// （codex-review P1 是正、モジュール doc「同意チェックをネイティブ
    /// disabled にする理由」節）。
    #[test]
    fn consent_checkbox_is_natively_disabled() {
        let html = render(&demo());
        assert!(
            html.contains(" disabled=\"\""),
            "the consent hidden input should carry the native disabled attribute"
        );
        assert!(
            html.contains("data-disabled=\"\""),
            "the consent checkbox root/control/label should carry data-disabled"
        );
    }

    /// [`LAYOUT_CSS`] が同意チェックの disabled 化に伴う視覚中和規則
    /// （`opacity: 1`/`cursor: default`）を持つことを固定する。
    #[test]
    fn layout_css_neutralizes_disabled_consent_checkbox_opacity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-contact-centered-form-consent][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-centered-form-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-contact-centered-form-layout");
    }

    /// [`LAYOUT_CSS`] の `min-height: 8rem` セレクタ
    /// （`[data-scope="field"][data-part="textarea"]
    /// [data-blocks-contact-centered-form-message]`）が実際に一致する
    /// 要素が出力に存在すること。CSS フック
    /// （`data-blocks-contact-centered-form-message`）を `field::root`
    /// （`data-part="root"`）へ付け違えると、セレクタが要求する
    /// `data-part="textarea"` を持つ要素と一致せず `min-height` が適用
    /// されない配線ミスになるため、単に属性・part の存在を別々に確認する
    /// だけでなく、同一タグ内での共起を固定する（イシュー #2826 PR #3193
    /// レビュー指摘）。
    #[test]
    fn message_hook_attribute_is_on_the_textarea_part_element() {
        let html = render(&demo());
        // `<` で分割し、`data-part="textarea"` を含むタグのみを抽出して
        // 同じタグ内に CSS フック属性も含まれることを検証する。
        let textarea_tags: Vec<&str> = html
            .split('<')
            .filter(|tag| tag.contains(r#"data-part="textarea""#))
            .collect();
        assert_eq!(
            textarea_tags.len(),
            1,
            "expected exactly one textarea part element in {html}"
        );
        assert!(
            textarea_tags[0].contains("data-blocks-contact-centered-form-message"),
            "expected the message CSS hook attribute on the same element as \
             data-part=\"textarea\", found tag: {}",
            textarea_tags[0]
        );
        // CSS セレクタが要求する 3 属性の組が LAYOUT_CSS 側にも存在することを
        // あわせて固定する（セレクタ自体の書き換えによる無害化を防ぐ）。
        assert!(LAYOUT_CSS.contains(
            r#"[data-scope="field"][data-part="textarea"][data-blocks-contact-centered-form-message] {"#
        ));
    }

    /// [`LAYOUT_CSS`] が 2 列グリッド・全幅セル・狭幅ブレークポイント・
    /// 送信ボタン全幅のセレクタを含み、`<` を含まないこと。
    #[test]
    fn layout_css_declares_two_column_grid_and_narrow_breakpoint() {
        assert!(LAYOUT_CSS.contains("repeat(2"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains(
            r#"[data-scope="button"][data-part="root"][data-blocks-contact-centered-form-submit] {"#
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }
}
