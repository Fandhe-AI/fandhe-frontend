//! `checkout-form-summary-split` block（イシュー #3042/#3043、親トラッキング
//! #3041「Ecommerce / Checkout カテゴリ初の block」）。規模 L のため前半
//! （骨格・主要領域・Blocks 登録、#3042）と後半（簡易決済ボタン行版・
//! サマリ反転配色版・割引バッジ・請求先住所・状態違いの並記、#3043）に
//! 分割実装した。入力フォーム（連絡先 → 配送先 → 配送方法 → 支払い情報）と
//! 注文サマリ（商品行・割引コード・集計・確定ボタン）を並べる購入手続き
//! 画面の合成例を、集約元 6 件（R0833/R0834/R0836/R0837/R0429/R0431、対応表
//! ID のみ・出典固有名は書かない）の差分を読み取れる 3 版で並記する
//! （下記「状態違いの並記」節）。
//!
//! # 使用部品
//!
//! `field` / `input` / `input-group` / `native-select` /
//! `radio-card` / `radio-group` / `fieldset` / `checkbox` / `badge` /
//! `button` / `image` / `separator` / `data-list` / `heading` の 14 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 状態違いの並記（版 A/B/C と集約元の対応、#3043）
//!
//! `product_overview_gallery_split`（#3520）と同型に、3 版を `caption()` +
//! `fandhe_frontend_core::section` で縦に並べる（[`Variant`] 1 個へ差分を
//! 集約し、各ヘルパへ版ごとの `suffix`/フラグを渡して id・`name` を一意化
//! する。抽象化はこの 1 構造体に限り、trait・builder は作らない）。
//!
//! - **版 A（代表構成、R0833/R0429）**: #3042 の骨格そのまま。サマリ左
//!   （狭幅ではサマリが先頭）・配送方法あり・割引コード入力欄あり。
//! - **版 B（簡易決済・割引適用・請求先、R0836/R0431）**: フォーム列先頭に
//!   `express_checkout_row`（簡易決済ボタン 2 個 + 「または」区切り）を置き、
//!   フォーム左・サマリ右（狭幅ではサマリが末尾）にする。サマリは割引コード
//!   適用済み（[`badge`] で「適用中のコード」を明示 + 集計に「割引」行）で
//!   操作用の入力欄は持たない。支払い節の後ろへ、[`fieldset`] +
//!   読み取り専用（disabled）[`radio_group`] の請求先住所セクション
//!   （「配送先と同じ」選択済み/「別の住所を指定」）を追加する。R0429 の
//!   5 節構成（連絡先/配送先/配送方法/支払い/請求先）はこの請求先節で近似
//!   する（Demo へ 5 節目を独立追加せず、支払い節の続きとして配置）。
//! - **版 C（反転配色・閲覧専用サマリ、R0837/R0834）**: サマリ面を
//!   `data-blocks-checkout-form-summary-split-tone="inverted"` で反転配色
//!   にし、操作要素（割引コード入力・確定ボタン）を持たない閲覧専用にする
//!   （下記「反転配色の面に入力・ボタンを置かない理由」節）。確定ボタンは
//!   フォーム列の末尾へ移し、配送方法の節は持たない（R0834 の構成）。
//!
//! Demo に入れなかった集約元の差分: R0429 の請求先専用ページ遷移（本
//! Demo は単一画面合成のため対象外、静的 Demo 全体の判断と同型）。
//!
//! # 反転配色の面に入力・ボタンを置かない理由（版 C、#3043）
//!
//! 版 C のサマリ面は `[data-...-tone="inverted"]` で前景色・背景色を入れ
//! 替える（`cta_split_actions::instance_inverted` と同型の判断）。input の
//! 既定背景・枠線トークンは反転を考慮しないため、反転面に input/button を
//! 置くとコントラストが崩れうる。本 Demo は反転面を「閲覧専用サマリ」
//! （商品行・集計の表示のみ）に限定することで、この崩れを構造的に避ける
//! （個別の色上書きを積み増すより安全、`LAYOUT_CSS` 側の宣言数も最小になる）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。確定ボタンは [`button::button`] の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。カード番号・CVC 等の決済情報
//! 入力欄は置かない（実在の決済フォームに見せないための判断、
//! `card_form_footer` と同じ）。文言・金額はすべて架空のもの（実企業名・
//! 実クレデンシャル・PII を含まない）。簡易決済ボタン（版 B）は実在ブランド
//! の名称・ロゴ・配色を使わない汎用名にする（`checkout_step_sections::
//! express_checkout_buttons` と同じラベルを再利用する）。
//!
//! # 配送方法・支払い方法 radio card をネイティブ disabled にする理由
//!
//! [`radio_card::item_hidden_input`] は有効なネイティブ
//! `<input type="radio">` であり、docs サイトは JS ハイドレーションを
//! 行わないため選択状態を追従できない。`card_form_footer::payment_item`
//! と同じ判断で、配送方法・支払い方法の両 radio card グループへ
//! `disabled: true` を共有してネイティブ操作を構造的に禁止し、`root` へ
//! `aria-disabled="true"` を明示付与したうえで現在の選択を独立した
//! [`styled_text::text`] で明文化する（disabled radio がフォームモード
//! 走査から除外されても選択が支援技術へ伝わるようにするため）。disabled の
//! 見た目（`opacity: 0.5; cursor: not-allowed;`）は中和せずそのまま残し、
//! 操作できない固定表示であることを視覚的にも示す（中和すると通常の選択
//! 項目に見える、#3462 codex 指摘。`cart_line_item_table` の数量 select と
//! 同じ扱い）。
//!
//! # DOM 順を視覚順へ一致させる理由（サマリ → フォーム、#3462 レビュー
//! 指摘対応）
//!
//! `order` プロパティは視覚順のみを変え、キーボード操作順（Tab 移動）は
//! DOM 順のまま変わらないため、CSS `order` で狭幅時だけサマリを先頭へ
//! 動かす旧実装は狭幅でキーボード操作順（フォーム → サマリ）と視覚順
//! （サマリ → フォーム）が食い違っていた（#3462 codex 指摘）。本実装は
//! `order` を一切使わず、DOM 順自体をサマリ → フォームに固定することで
//! 両順序を常に一致させる。狭幅（既定、1 列）ではサマリが先に表示され、
//! 購入内容を先に確認してからフォームへ進める導線（`docs/design/
//! docs-site-blocks-section.md` の 2 カラム系 block と同じ判断）を
//! `order` なしで実現する。`64rem` 以上ではサマリ = 固定幅・フォーム =
//! 可変幅の 2 カラムへ切り替え、`grid-template-columns` の並び順のみで
//! サマリを左（固定幅）・フォームを右（可変幅）に配置する（`order` は
//! 使わないため、この列配置でも視覚順は DOM 順＝サマリ→フォームのまま
//! 一致する）。
//!
//! 版 B はこの逆順（フォーム左・サマリ右、狭幅ではサマリ末尾）にする
//! ため、DOM 順自体をフォーム → サマリへ入れ替える（`order` は使わない
//! 方針を維持）。`64rem` 以上の列幅配分は
//! `[data-blocks-checkout-form-summary-split-summary-position="end"]`
//! （`.blocks-checkout-form-summary-split-columns` へ付与）の有無で
//! `grid-template-columns` を左右反転させるだけで、DOM 順＝視覚順の
//! 不変条件・コンテナクエリ自体は版 A/C と共有する。
//!
//! # 2 カラム切り替え・行内 2 列化はコンテナクエリで判定する（`@container`、
//! #3462 Bugbot 指摘対応）
//!
//! Demo 枠の幅はビューポート幅と一致しない（サイドバー分だけ実際の
//! コンテナ幅が狭い）ため、`@media (min-width: ...)` をブレークポイント
//! 判定に使うと、ビューポートは `64rem` 以上でも Demo の実コンテナ幅は
//! それ未満のままレイアウトが崩れる（`cart_two_column_summary` と同型の
//! 判断）。外側の `.blocks-checkout-form-summary-split-layout` へ
//! `container-type: inline-size` を宣言し、サマリ/フォームの 2 カラム
//! 切り替えは子の `.blocks-checkout-form-summary-split-columns` を
//! `@container` 内で選んで行う（コンテナクエリは祖先のコンテナを参照
//! するため、コンテナ自身を `@container` 内で選んでも規則は適用されない、
//! #3462 codex 指摘。`notification_tray` の `-stack`/`-row` と同型）。姓名・市区町村/都道府県の行内
//! 2 列化はフォーム列（`.blocks-checkout-form-summary-split-form`）自体の
//! 幅に依存する（2 カラム化後はフォーム列がレイアウト全体より狭くなる）
//! ため、フォーム列へネストした `container-type: inline-size` を別途
//! 宣言し、レイアウト全体のコンテナクエリとは独立した
//! `blocks-checkout-form-summary-split-form` コンテナで判定する（レイア
//! ウト全体の `64rem` ブレークポイントへ相乗りさせていた旧実装は、フォー
//! ム列が狭くなる条件下でかえって 2 列化するという意図と逆の挙動を生ん
//! でいた、#3462 Bugbot 指摘）。
//!
//! # サマリの `position: sticky` は使わない（#3462 Bugbot 指摘対応）
//!
//! `.blocks-demo` は `overflow-x: auto` の横スクロールコンテナのため、
//! Demo 内でサマリを `position: sticky` 追従させても意図どおりに機能し
//! ない（`cart_two_column_summary`・`content_article_toc` の判断を踏襲）。
//! 実アプリで組み込む際は呼び出し側で `sticky` を付与してよい。
//!
//! # 都道府県 select をネイティブのまま操作可能にする理由
//!
//! [`native_select::native_select`] は `<select>` 自体をそのまま使う薄い
//! 委譲層（開閉 JS 配線を持つ styled `select` とは異なる）。選択操作が
//! ブラウザネイティブに閉じており、視覚表示との食い違いが生じないため
//! `disabled` にしない（`contact_centered_form` の国番号 select と同じ
//! 判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root` / `input::input` /
//! `native_select::native_select` / `radio_card::root` /
//! `checkbox::root` / `button::button` / `input_group::root` /
//! `data_list::root` / `image::image` / `heading::heading` /
//! `separator::separator` / `styled_text::text` はいずれも `drop_class_attr`
//! により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-checkout-form-summary-split-*` 属性で渡す。素の
//! `div`/`ul`/`li` には `class` がそのまま効くため、それらは
//! `.blocks-checkout-form-summary-split-*` クラスセレクタを使う。
//!
//! # id / ARIA の方針
//!
//! id 接頭辞は `blocks-checkout-form-summary-split-` に統一する。
//! `field::root` の `id` から `for`/コントロール `id` が導出される
//! （headless field 契約）ため各フィールドで一意にする。配送方法・支払い
//! 方法は `fieldset` を使わず、`radio_card::label` の id を
//! `radio_card::root` の `labelled_by` へ渡す（`card_form_footer::
//! payment_method_field` と同型）。版 B の請求先住所のみ `fieldset` +
//! `radio_group`（`notifications_section`〔`form_layout_stacked`〕と同型
//! に `fieldset::legend` の id を `radio_group::root` の `labelled_by` へ
//! 渡す構成）を使う（`radio_card` と `radio_group` を使い分ける理由は
//! モジュール doc「状態違いの並記」節参照）。3 版とも id・`name` は
//! `format!("{BASE}-{suffix}")`（`suffix` は `"a"`/`"b"`/`"c"`）で一意化する
//! （`product_overview_gallery_split` と同型）。商品画像は装飾扱いの
//! `alt=""`。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（既存 block と同じ回避方法）。
//!
//! # 参照について
//!
//! 主参照 R0833、集約元 R0834/R0836/R0837/R0429/R0431（対応表 ID のみ、
//! 出典固有名は原稿・コードのいずれにも書かない）。文言・配色・アイコン
//! は独自に書く（他 block と同じライセンス上の転記制限）。簡易決済
//! ボタン行（R0836/R0431）・反転配色サマリ（R0837/R0834）・割引バッジ
//! （R0431）・請求先住所（R0429 近似）は版 B/C（#3043）で追加済み
//! （対応関係は「状態違いの並記」節参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, p, section, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 版ごとの構成差分（モジュール doc「状態違いの並記」節）。抽象化は本
/// 構造体 1 個に限り、trait・builder は作らない（ponytail: 1 abstraction）。
struct Variant {
    /// id・`name` を一意化する接尾辞（`"a"`/`"b"`/`"c"`）。
    suffix: &'static str,
    /// 簡易決済ボタン行（版 B）を持つか。
    express: bool,
    /// 配送方法の節を持つか（版 C は持たない）。
    shipping_method: bool,
    /// 支払い節の後ろへ独立した請求先住所セクション（`fieldset` +
    /// `radio_group`）を追加するか（版 B のみ）。`false` の版は従来どおり
    /// 支払い節内に請求先の宛名フィールドを inline する。
    billing_section: bool,
    /// サマリの割引コードが適用済みか（版 B）。`true` のとき操作用の入力欄
    /// は出さず、代わりに badge と集計の「割引」行を出す。
    discount_applied: bool,
    /// サマリに割引コード入力欄を出すか（版 A のみ）。
    show_discount_input: bool,
    /// `true` で列順を「フォーム→サマリ」（版 B、狭幅ではサマリが末尾）に
    /// する。`false` は既定の「サマリ→フォーム」（版 A/C）。
    summary_end: bool,
    /// サマリ面を反転配色の閲覧専用にするか（版 C のみ）。`true` のとき
    /// 確定ボタンはフォーム列の末尾へ移る。
    inverted: bool,
}

const VARIANT_A: Variant = Variant {
    suffix: "a",
    express: false,
    shipping_method: true,
    billing_section: false,
    discount_applied: false,
    show_discount_input: true,
    summary_end: false,
    inverted: false,
};

const VARIANT_B: Variant = Variant {
    suffix: "b",
    express: true,
    shipping_method: true,
    billing_section: true,
    discount_applied: true,
    show_discount_input: false,
    summary_end: true,
    inverted: false,
};

const VARIANT_C: Variant = Variant {
    suffix: "c",
    express: false,
    shipping_method: false,
    billing_section: false,
    discount_applied: false,
    show_discount_input: false,
    summary_end: false,
    inverted: true,
};

/// 配送方法 radio card のネイティブ `<input>` 共通 `name`。
const SHIPPING_METHOD_NAME: &str = "blocks-checkout-form-summary-split-shipping-method";
const SHIPPING_METHOD_LABEL_ID: &str = "blocks-checkout-form-summary-split-shipping-method-label";

/// 支払い方法 radio card のネイティブ `<input>` 共通 `name`。
const PAYMENT_METHOD_NAME: &str = "blocks-checkout-form-summary-split-payment-method";
const PAYMENT_METHOD_LABEL_ID: &str = "blocks-checkout-form-summary-split-payment-method-label";

const EMAIL_ID: &str = "blocks-checkout-form-summary-split-email";
const FIRST_NAME_ID: &str = "blocks-checkout-form-summary-split-first-name";
const LAST_NAME_ID: &str = "blocks-checkout-form-summary-split-last-name";
const ADDRESS_ID: &str = "blocks-checkout-form-summary-split-address";
const CITY_ID: &str = "blocks-checkout-form-summary-split-city";
const PREFECTURE_ID: &str = "blocks-checkout-form-summary-split-prefecture";
const POSTAL_CODE_ID: &str = "blocks-checkout-form-summary-split-postal-code";
const BILLING_NAME_ID: &str = "blocks-checkout-form-summary-split-billing-name";
const DISCOUNT_CODE_ID: &str = "blocks-checkout-form-summary-split-discount-code";
const BILLING_ADDRESS_ID: &str = "blocks-checkout-form-summary-split-billing-address";

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ（モジュール doc
/// 「id / ARIA の方針」節）。`id` は版ごとの一意な借用先（`format!` した
/// ローカル `String` 等）を受け取れるよう、`'static` ではなく汎用の
/// ライフタイムで受ける。
fn field_props(id: &str, required: bool) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    }
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 見出し 1 件（`## Demo` がページ側で `h2` を出すため `h3` に固定、
/// 既存 block と同じ判断）。
fn section_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(title)],
    )
}

/// 連絡先セクション（メールアドレス + お知らせ配信 checkbox。checkbox は
/// 未チェック固定の静的表示のため、JS ハイドレーションなしでもネイティブ
/// input の checked 状態とカスタム indicator の表示が食い違わないよう
/// `disabled: true` でネイティブ操作を止める（#3462 レビュー指摘対応）。
/// styled checkbox の disabled の見た目（`opacity: 0.5; cursor:
/// not-allowed;`）は中和せず残し、操作できない固定表示であることを示す
/// （配送方法・支払い方法 radio card と同じ判断、#3462 codex 指摘）。
fn contact_section(suffix: &str) -> Node {
    let email_id = format!("{EMAIL_ID}-{suffix}");
    let email = field_props(&email_id, true);
    let checkbox_name = format!("blocks-checkout-form-summary-split-newsletter-{suffix}");
    let checkbox_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("連絡先"),
            field::root(
                &orientation(),
                &email,
                vec![],
                vec![
                    field::label(&email, vec![], vec![text("メールアドレス")]),
                    input::input(
                        &InputProps::default(),
                        &email,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            checkbox::root(
                Size::Sm,
                ColorPalette::Accent,
                &checkbox_props,
                vec![],
                vec![
                    checkbox::hidden_input(&checkbox_props, &checkbox_name, "on", vec![]),
                    checkbox::control(
                        &checkbox_props,
                        vec![],
                        vec![checkbox::indicator(&checkbox_props, vec![], vec![])],
                    ),
                    checkbox::label(&checkbox_props, vec![], vec![text("お知らせを受け取る")]),
                ],
            ),
        ],
    )
}

/// 姓・名 2 欄の行（狭幅は 1 列、`40rem` 以上は 2 列。モジュール doc
/// 「id / ARIA の方針」節）。
fn name_row(suffix: &str) -> Node {
    let first_name_id = format!("{FIRST_NAME_ID}-{suffix}");
    let last_name_id = format!("{LAST_NAME_ID}-{suffix}");
    let first_name = field_props(&first_name_id, true);
    let last_name = field_props(&last_name_id, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-name-row")],
        vec![
            field::root(
                &orientation(),
                &first_name,
                vec![],
                vec![
                    field::label(&first_name, vec![], vec![text("姓")]),
                    input::input(
                        &InputProps::default(),
                        &first_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "山田"),
                            ("autocomplete", "family-name"),
                        ],
                    ),
                ],
            ),
            field::root(
                &orientation(),
                &last_name,
                vec![],
                vec![
                    field::label(&last_name, vec![], vec![text("名")]),
                    input::input(
                        &InputProps::default(),
                        &last_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "太郎"),
                            ("autocomplete", "given-name"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 都道府県 select の選択肢（一部抜粋の固定リスト、モジュール doc「都道
/// 府県 select をネイティブのまま操作可能にする理由」節）。
fn prefecture_options() -> Vec<Node> {
    [
        ("tokyo", "東京都", true),
        ("osaka", "大阪府", false),
        ("aichi", "愛知県", false),
        ("fukuoka", "福岡県", false),
    ]
    .into_iter()
    .map(|(value, label, selected)| {
        let mut attrs = vec![("value", value)];
        if selected {
            attrs.push(("selected", "selected"));
        }
        el("option", attrs, vec![text(label)])
    })
    .collect()
}

/// 配送先セクション（姓・名・住所・市区町村・都道府県・郵便番号）。
fn shipping_section(suffix: &str) -> Node {
    let address_id = format!("{ADDRESS_ID}-{suffix}");
    let city_id = format!("{CITY_ID}-{suffix}");
    let prefecture_id = format!("{PREFECTURE_ID}-{suffix}");
    let postal_code_id = format!("{POSTAL_CODE_ID}-{suffix}");
    let address = field_props(&address_id, true);
    let city = field_props(&city_id, true);
    let prefecture = field_props(&prefecture_id, true);
    let postal_code = field_props(&postal_code_id, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送先"),
            name_row(suffix),
            field::root(
                &orientation(),
                &address,
                vec![],
                vec![
                    field::label(&address, vec![], vec![text("住所")]),
                    input::input(
                        &InputProps::default(),
                        &address,
                        vec![
                            ("type", "text"),
                            ("placeholder", "1-2-3 サンプル町"),
                            ("autocomplete", "address-line1"),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-checkout-form-summary-split-city-row")],
                vec![
                    field::root(
                        &orientation(),
                        &city,
                        vec![],
                        vec![
                            field::label(&city, vec![], vec![text("市区町村")]),
                            input::input(
                                &InputProps::default(),
                                &city,
                                vec![
                                    ("type", "text"),
                                    ("placeholder", "渋谷区"),
                                    ("autocomplete", "address-level2"),
                                ],
                            ),
                        ],
                    ),
                    field::root(
                        &orientation(),
                        &prefecture,
                        vec![],
                        vec![
                            field::label(&prefecture, vec![], vec![text("都道府県")]),
                            native_select::native_select(
                                &NativeSelectProps::default(),
                                &prefecture,
                                vec![("autocomplete", "address-level1")],
                                prefecture_options(),
                            ),
                        ],
                    ),
                ],
            ),
            field::root(
                &orientation(),
                &postal_code,
                vec![],
                vec![
                    field::label(&postal_code, vec![], vec![text("郵便番号")]),
                    input::input(
                        &InputProps::default(),
                        &postal_code,
                        vec![
                            ("type", "text"),
                            ("placeholder", "150-0002"),
                            ("autocomplete", "postal-code"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// radio card 1 件（`checked`/`disabled`/`value`/`label`/`description` から
/// 組み立てる。配送方法・支払い方法の双方で共有する、モジュール doc
/// 「配送方法・支払い方法 radio card をネイティブ disabled にする理由」
/// 節）。
fn radio_card_item(
    checked: bool,
    name: &str,
    value: &'static str,
    label: &'static str,
    description: &'static str,
) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(name), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![
                            radio_card::item_text(vec![], vec![text(label)]),
                            radio_card::item_description(vec![], vec![text(description)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 配送方法セクション（fieldset は使わず `radio_card::label` の id を
/// `labelled_by` へ渡す構成、`card_form_footer::payment_method_field` と
/// 同型）。「通常配送」を選択済みで固定する。
fn shipping_method_section(suffix: &str) -> Node {
    let name = format!("{SHIPPING_METHOD_NAME}-{suffix}");
    let label_id = format!("{SHIPPING_METHOD_LABEL_ID}-{suffix}");
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送方法"),
            radio_card::label(Some(&label_id), vec![], vec![text("配送方法を選択")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(&label_id),
                vec![("aria-disabled", "true")],
                vec![
                    radio_card_item(
                        true,
                        &name,
                        "standard",
                        "通常配送",
                        "3〜5 営業日でお届けします。",
                    ),
                    radio_card_item(
                        false,
                        &name,
                        "express",
                        "お急ぎ便",
                        "1〜2 営業日でお届けします。",
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 通常配送")],
            ),
        ],
    )
}

/// 支払い方法セクション（「カード払い」選択済みで固定）。カード番号等の
/// 決済情報入力欄は置かない（モジュール doc「`<form>` を持たない」節）。
/// `include_billing_name` が `true`（版 A/C）のときのみ請求先の宛名欄を
/// inline する。`false`（版 B）のときは呼び出し側が [`billing_address_section`]
/// を別途続ける（モジュール doc「状態違いの並記」節）。
fn payment_method_section(suffix: &str, include_billing_name: bool) -> Node {
    let name = format!("{PAYMENT_METHOD_NAME}-{suffix}");
    let label_id = format!("{PAYMENT_METHOD_LABEL_ID}-{suffix}");
    let billing_name_id = format!("{BILLING_NAME_ID}-{suffix}");
    let billing_name = field_props(&billing_name_id, true);
    let mut children = vec![
        section_heading("お支払い方法"),
        radio_card::label(Some(&label_id), vec![], vec![text("お支払い方法を選択")]),
        radio_card::root(
            Size::Sm,
            ColorPalette::Accent,
            true,
            None::<Orientation>,
            Some(&label_id),
            vec![("aria-disabled", "true")],
            vec![
                radio_card_item(
                    true,
                    &name,
                    "card",
                    "カード払い",
                    "登録済みのカードから引き落とします。",
                ),
                radio_card_item(
                    false,
                    &name,
                    "bank",
                    "銀行振込",
                    "指定口座へお振込みいただきます。",
                ),
                radio_card_item(
                    false,
                    &name,
                    "cod",
                    "代金引換",
                    "商品お受け取り時にお支払いいただきます。",
                ),
            ],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("現在の選択: カード払い")],
        ),
    ];
    if include_billing_name {
        children.push(field::root(
            &orientation(),
            &billing_name,
            vec![],
            vec![
                field::label(&billing_name, vec![], vec![text("請求先の宛名")]),
                input::input(
                    &InputProps::default(),
                    &billing_name,
                    vec![("type", "text"), ("placeholder", "山田 太郎")],
                ),
            ],
        ));
    }
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        children,
    )
}

/// 請求先住所の radio group 項目 1 件（`fieldset::legend` の id を
/// `radio_group::root` の `labelled_by` へ渡す構成、`form_layout_stacked::
/// notifications_section` の `push_notification_item` と同型）。
fn billing_address_item(
    checked: bool,
    props: &RadioGroupProps,
    name: &str,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![],
        vec![
            radio_group::item_hidden_input(checked, props, Some(name), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 請求先住所セクション（版 B 固有、R0429 の 5 節構成を近似。モジュール
/// doc「状態違いの並記」節）。`fieldset` + 読み取り専用（`disabled: true`）
/// の `radio_group` で「配送先と同じ」を選択済みにする。配送方法・支払い
/// 方法 radio card と同じ理由でネイティブ disabled にする（モジュール doc
/// 「配送方法・支払い方法 radio card をネイティブ disabled にする理由」
/// 節、radio_group 版は `radio_card::item_hidden_input` の代わりに
/// `RadioGroupProps::disabled` で表現する）。
fn billing_address_section(suffix: &str) -> Node {
    let fieldset_id = format!("{BILLING_ADDRESS_ID}-{suffix}");
    let legend_id = format!("{fieldset_id}-legend");
    let name = format!("{BILLING_ADDRESS_ID}-{suffix}");
    let fieldset_props = FieldsetProps {
        id: &fieldset_id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("請求先住所")]),
            radio_group::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(&legend_id),
                vec![],
                vec![
                    billing_address_item(true, &radio_props, &name, "same", "配送先と同じ"),
                    billing_address_item(false, &radio_props, &name, "different", "別の住所を指定"),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 配送先と同じ")],
            ),
        ],
    )
}

/// 簡易決済ボタン行（版 B 固有。`checkout_step_sections::
/// express_checkout_buttons` と同じラベルを再利用し、実在ブランドの
/// 名称・ロゴ・配色を持ち込まない、モジュール doc「`<form>` を持たない」
/// 節）。
fn express_checkout_row() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-express")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ウォレットで支払う")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ワンタップ決済")],
            ),
        ],
    )
}

/// 確定ボタン（`注文を確定する`、送信しない種別のまま）。版 A/B はサマリ
/// 列末尾、版 C はフォーム列末尾に置く（モジュール doc「状態違いの並記」
/// 節）。
fn confirm_button() -> Node {
    button::button(
        &ButtonProps::default(),
        vec![("data-blocks-checkout-form-summary-split-confirm", "")],
        vec![text("注文を確定する")],
    )
}

/// 右カラム（入力フォーム）全体。版 B は先頭に簡易決済ボタン行を、版 C は
/// 末尾に確定ボタンを追加する（モジュール doc「状態違いの並記」節）。
fn form_column(v: &Variant) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if v.express {
        children.push(express_checkout_row());
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("またはメールアドレスで手続きを続ける")],
        ));
    }
    children.push(contact_section(v.suffix));
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    children.push(shipping_section(v.suffix));
    if v.shipping_method {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(shipping_method_section(v.suffix));
    }
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    children.push(payment_method_section(v.suffix, !v.billing_section));
    if v.billing_section {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(billing_address_section(v.suffix));
    }
    if v.inverted {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(confirm_button());
    }
    div(
        vec![("class", "blocks-checkout-form-summary-split-form")],
        children,
    )
}

/// 商品行 1 件（画像 + 名前・バリエーション + 価格）。
fn product_row(name: &'static str, variant_label: &'static str, price: &'static str) -> Node {
    li(
        vec![("class", "blocks-checkout-form-summary-split-product-row")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-checkout-form-summary-split-product-image", "")],
            ),
            div(
                vec![("class", "blocks-checkout-form-summary-split-product-detail")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(variant_label)],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("data-blocks-checkout-form-summary-split-product-price", "")],
                vec![text(price)],
            ),
        ],
    )
}

/// 割引コード欄（input + 「適用」ボタンを 1 本の入力グループへ一体化、
/// `hero_email_signup::signup_group` と同型）。版 A のみが持つ（モジュール
/// doc「状態違いの並記」節、版 B は適用済みバッジ、版 C は閲覧専用で
/// どちらも持たない）。
fn discount_code_group(suffix: &str) -> Node {
    let discount_id = format!("{DISCOUNT_CODE_ID}-{suffix}");
    let discount = field_props(&discount_id, false);
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &orientation(),
        &discount,
        vec![],
        vec![
            field::label(&discount, vec![], vec![text("割引コード")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-checkout-form-summary-split-discount-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &discount,
                        vec![("type", "text"), ("placeholder", "コードを入力")],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("適用")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn total_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 集計（小計・送料・税・合計）の定義リスト。`discount_applied` が `true`
/// （版 B）のとき「割引」行を挟み、送料・税・合計を割引後の金額で再計算
/// する（モジュール doc「状態違いの並記」節、架空値の整合: 小計 12,800 −
/// 割引 1,280 + 送料 600 = 課税対象 12,120、税 10% = 1,212、合計 13,332）。
fn totals(discount_applied: bool) -> Node {
    let mut rows = vec![total_row("小計", "¥12,800")];
    if discount_applied {
        rows.push(total_row("割引", "-¥1,280"));
    }
    rows.push(total_row("送料", "¥600"));
    if discount_applied {
        rows.push(total_row("税", "¥1,212"));
        rows.push(total_row("合計", "¥13,332"));
    } else {
        rows.push(total_row("税", "¥1,340"));
        rows.push(total_row("合計", "¥14,740"));
    }
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        rows,
    )
}

/// 適用済み割引コードの表示行（版 B 固有、[`badge`] で明示する。モジュール
/// doc「状態違いの並記」節）。
fn discount_applied_badge() -> Node {
    div(
        vec![(
            "class",
            "blocks-checkout-form-summary-split-discount-badge-row",
        )],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("適用中のコード")],
            ),
            badge::badge(&BadgeProps::default(), vec![], vec![text("WELCOME10")]),
        ],
    )
}

/// 左カラム（注文サマリ）全体。商品行 → （割引コード入力 or 適用済み
/// バッジ）→ 集計 → 確定ボタンの順に縦積みする。版 C（`v.inverted`）は
/// 反転配色の閲覧専用にし、割引コード入力・確定ボタンを持たない（モジュール
/// doc「反転配色の面に入力・ボタンを置かない理由」節）。
fn summary_column(v: &Variant) -> Node {
    let mut attrs = vec![("data-blocks-checkout-form-summary-split-summary", "")];
    if v.inverted {
        attrs.push(("data-blocks-checkout-form-summary-split-tone", "inverted"));
    }
    let mut children = vec![section_heading("ご注文内容")];
    if v.discount_applied {
        children.push(discount_applied_badge());
    }
    children.push(ul(
        vec![("class", "blocks-checkout-form-summary-split-product-list")],
        vec![
            product_row("キャンバストートバッグ", "カラー: ナチュラル", "¥6,400"),
            product_row("セラミックマグカップ", "カラー: ホワイト", "¥3,200"),
            product_row("コットンソックス 2 足組", "サイズ: M", "¥3,200"),
        ],
    ));
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    if v.show_discount_input {
        children.push(discount_code_group(v.suffix));
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
    }
    children.push(totals(v.discount_applied));
    if !v.inverted {
        children.push(confirm_button());
    }
    div(attrs, children)
}

/// 版キャプション（`product_overview_gallery_split::caption` と同型、素の
/// `<p>` で `heading` 部品を使わない）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-checkout-form-summary-split-caption")],
        vec![text(label)],
    )
}

/// 版 1 件のレイアウト（`-layout` → `-columns` → [サマリ列, フォーム列]
/// または逆順）。`v.summary_end` で列順・列幅配分を切り替える（モジュール
/// doc「DOM 順を視覚順へ一致させる理由」節、`order` は使わない）。
fn variant_layout(v: &Variant) -> Node {
    let mut columns_attrs = vec![("class", "blocks-checkout-form-summary-split-columns")];
    if v.summary_end {
        columns_attrs.push((
            "data-blocks-checkout-form-summary-split-summary-position",
            "end",
        ));
    }
    let columns = if v.summary_end {
        vec![form_column(v), summary_column(v)]
    } else {
        vec![summary_column(v), form_column(v)]
    };
    div(
        vec![
            ("class", "blocks-checkout-form-summary-split-layout"),
            ("data-blocks-checkout-form-summary-split-variant", v.suffix),
        ],
        vec![div(columns_attrs, columns)],
    )
}

/// `checkout-form-summary-split` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。3 版をキャプション付きで縦に並べる（モジュール doc
/// 「状態違いの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-demo")],
        vec![
            caption("代表構成（サマリ先頭・配送方法あり・割引コード入力欄）"),
            section(vec![], vec![variant_layout(&VARIANT_A)]),
            caption("簡易決済 + 割引適用済み + 請求先指定（フォーム先頭・サマリ末尾）"),
            section(vec![], vec![variant_layout(&VARIANT_B)]),
            caption("反転配色・閲覧専用サマリ（配送方法なし・確定ボタンはフォーム末尾）"),
            section(vec![], vec![variant_layout(&VARIANT_C)]),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/checkout-form-summary-split/",
    title: "checkout-form-summary-split",
    category: BlockCategory::Checkout,
    rust_source: "crates/docs-site/src/blocks/ecommerce/checkout/checkout_form_summary_split.rs",
    demo_class: "blocks-checkout-form-summary-split",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Radio Group",
            path: "/themes/radio-group/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `checkout_form_summary_split` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-checkout-form-summary-split-demo {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-checkout-form-summary-split-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-checkout-form-summary-split-layout {\n  container-type: inline-size;\n  container-name: blocks-checkout-form-summary-split;\n}\n\
.blocks-checkout-form-summary-split-columns {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-checkout-form-summary-split-form {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-checkout-form-summary-split-form;\n}\n\
[data-blocks-checkout-form-summary-split-summary] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-form-summary-split-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-form-summary-split-name-row {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-form-summary-split-city-row {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-form-summary-split-express {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-checkout-form-summary-split-discount-badge-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-checkout-form-summary-split-product-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-checkout-form-summary-split-product-row {\n  display: grid;\n  grid-template-columns: 4rem 1fr auto;\n  gap: var(--fandhe-space-3);\n  align-items: center;\n}\n\
[data-blocks-checkout-form-summary-split-product-image] {\n  inline-size: 4rem;\n  block-size: 4rem;\n}\n\
.blocks-checkout-form-summary-split-product-detail {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1, 0.25rem);\n  min-inline-size: 0;\n}\n\
[data-blocks-checkout-form-summary-split-product-price] {\n  white-space: nowrap;\n}\n\
[data-blocks-checkout-form-summary-split-confirm] {\n  inline-size: 100%;\n}\n\
[data-blocks-checkout-form-summary-split-summary][data-blocks-checkout-form-summary-split-tone=\"inverted\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  padding: var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-md);\n  --fandhe-color-fg-muted: var(--fandhe-color-bg);\n  --fandhe-data-list-label-color: var(--fandhe-color-bg);\n  --fandhe-data-list-value-color: var(--fandhe-color-bg);\n}\n\
[data-blocks-checkout-form-summary-split-tone=\"inverted\"] [data-scope=\"separator\"][data-part=\"root\"] {\n  border-color: var(--fandhe-color-bg);\n  opacity: 0.3;\n}\n\
@container blocks-checkout-form-summary-split-form (min-width: 40rem) {\n  \
.blocks-checkout-form-summary-split-name-row {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-checkout-form-summary-split-city-row {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-checkout-form-summary-split (min-width: 64rem) {\n  \
.blocks-checkout-form-summary-split-columns {\n    grid-template-columns: minmax(0, 24rem) minmax(0, 1fr);\n    align-items: start;\n  }\n  \
.blocks-checkout-form-summary-split-columns[data-blocks-checkout-form-summary-split-summary-position=\"end\"] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 24rem);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, variant_layout, EMAIL_ID, LAYOUT_CSS, PAYMENT_METHOD_LABEL_ID,
        SHIPPING_METHOD_LABEL_ID, VARIANT_A, VARIANT_B, VARIANT_C,
    };
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"data-list\"",
            "data-scope=\"heading\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert!(html.contains("data-scope=\"field\" data-part=\"select\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn radio_items_are_natively_disabled_and_one_is_checked_per_group() {
        let html = demo_html();
        // 版 A: 配送方法 2 件 + 支払い方法 3 件 = 5 件。
        // 版 B: 配送方法 2 件 + 支払い方法 3 件 + 請求先住所（radio_group）
        // 2 件 = 7 件。版 C: 支払い方法 3 件のみ（配送方法なし）。
        // 計 5+7+3=15 件、いずれも disabled。
        // + お知らせ配信 checkbox 3 版分（各版 1 件、静的表示、#3462
        // レビュー指摘対応でネイティブ disabled 化済み）で disabled 計 18 件
        // （15 radio + 3 checkbox）。
        assert_eq!(html.matches(r#"type="radio""#).count(), 15);
        assert_eq!(html.matches(" disabled=\"\"").count(), 18);
        // checked: 版 A（配送方法 1 + 支払い方法 1）+ 版 B（同 2 + 請求先
        // 住所 1）+ 版 C（支払い方法 1）= 2+3+1=6 件。
        assert_eq!(html.matches(" checked").count(), 6);
        // aria-disabled: 版 A 2（配送方法根 + 支払い方法根）+ 版 B 3（同 2 +
        // 請求先住所根）+ 版 C 1（支払い方法根のみ）= 6 件。
        assert_eq!(html.matches(r#"aria-disabled="true""#).count(), 6);
        assert!(html.contains("現在の選択: 通常配送"));
        assert!(html.contains("現在の選択: カード払い"));
        assert!(html.contains("現在の選択: 配送先と同じ"));
    }

    #[test]
    fn newsletter_checkbox_is_natively_disabled() {
        // 静的 Demo の表示契約: JS ハイドレーションなしでネイティブ input が
        // 操作可能だと、クリック後にカスタム indicator（未チェック表示）と
        // ネイティブ checked 状態が食い違う（#3462 レビュー指摘対応）。
        let html = demo_html();
        assert!(html.contains(
            r#"data-scope="checkbox" data-part="hidden-input" data-state="unchecked" data-disabled="""#
        ));
    }

    #[test]
    fn ids_are_unique_per_variant() {
        // email id・newsletter checkbox name が版接尾辞で一意化され衝突
        // しないこと（`product_overview_gallery_split` の
        // `ids_are_unique_per_variant` と同型）。
        let html = demo_html();
        for suffix in ["a", "b", "c"] {
            // `field::root` の id は派生 id（`"{id}-control"`）でのみ出力
            // される（headless field 契約、モジュール doc「id / ARIA の
            // 方針」節）ため、派生先で一意性を確認する。
            let control_id = format!("{EMAIL_ID}-{suffix}-control");
            assert_eq!(
                html.matches(&format!("id=\"{control_id}\"")).count(),
                1,
                "html={html}"
            );
        }
    }

    #[test]
    fn confirm_button_appears_exactly_three_times() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-checkout-form-summary-split-confirm")
                .count(),
            3
        );
    }

    #[test]
    fn variant_b_has_express_row_badge_fieldset_and_radio_group() {
        let html = render(&variant_layout(&VARIANT_B));
        assert!(html.contains("blocks-checkout-form-summary-split-express"));
        assert!(html.contains("または"));
        assert!(html.contains("data-scope=\"badge\""));
        assert!(html.contains("WELCOME10"));
        assert!(html.contains("data-scope=\"fieldset\""));
        assert!(html.contains("data-scope=\"radio-group\""));
        assert!(html.contains("割引"));
        assert!(html.contains("-¥1,280"));
    }

    #[test]
    fn variant_c_inverted_summary_has_no_input_group_or_confirm() {
        let html = render(&variant_layout(&VARIANT_C));
        assert!(html.contains(r#"data-blocks-checkout-form-summary-split-tone="inverted""#));
        // 操作要素（割引コード入力・確定ボタン）を持たない閲覧専用サマリ
        // （モジュール doc「反転配色の面に入力・ボタンを置かない理由」節）。
        assert!(!html.contains("data-scope=\"input-group\""));
        // 配送方法の節を持たない。
        assert!(!html.contains("配送方法を選択"));
        // 確定ボタンはちょうど 1 個（フォーム列末尾）。
        assert_eq!(
            html.matches("data-blocks-checkout-form-summary-split-confirm")
                .count(),
            1
        );
    }

    #[test]
    fn demo_dom_order_matches_variant_contract() {
        // 版 A/C: サマリ → フォーム（`confirm` はサマリ内のため email より
        // 前）。版 B: フォーム → サマリ（`summary_end`、`confirm` はサマリ
        // 内のため email より後）。版 C: `confirm` はフォーム列末尾のため
        // email より後（モジュール doc「DOM 順を視覚順へ一致させる理由」節）。
        // `data-blocks-checkout-form-summary-split-summary=""` と前方一致
        // する `...-summary-position="end"`（版 B の列属性）と混同しない
        // よう、閉じクォートまで含めた完全な属性文字列で検索する。
        const SUMMARY_MARKER: &str = "data-blocks-checkout-form-summary-split-summary=\"\"";
        const CONFIRM_MARKER: &str = "data-blocks-checkout-form-summary-split-confirm=\"\"";
        for (variant, email_before_summary, confirm_before_email) in
            [(&VARIANT_A, false, true), (&VARIANT_C, false, false)]
        {
            let html = render(&variant_layout(variant));
            let summary_pos = html
                .find(SUMMARY_MARKER)
                .expect("summary marker should exist");
            let confirm_pos = html
                .find(CONFIRM_MARKER)
                .expect("confirm marker should exist");
            let email_id = format!("{EMAIL_ID}-{}-control", variant.suffix);
            let email_pos = html.find(&email_id).expect("email marker should exist");
            assert_eq!(
                email_pos < summary_pos,
                email_before_summary,
                "suffix={} html={html}",
                variant.suffix
            );
            assert_eq!(
                confirm_pos < email_pos,
                confirm_before_email,
                "suffix={} html={html}",
                variant.suffix
            );
        }
        let html = render(&variant_layout(&VARIANT_B));
        let summary_pos = html
            .find(SUMMARY_MARKER)
            .expect("summary marker should exist");
        let confirm_pos = html
            .find(CONFIRM_MARKER)
            .expect("confirm marker should exist");
        let email_id = format!("{EMAIL_ID}-b-control");
        let email_pos = html.find(&email_id).expect("email marker should exist");
        assert!(email_pos < summary_pos, "form should precede summary");
        assert!(summary_pos < confirm_pos, "confirm is inside summary");
    }

    #[test]
    fn disabled_appearance_is_not_neutralized() {
        // 無 JS の固定表示（disabled）を通常の選択項目に見せない（#3462
        // codex 指摘）。disabled の opacity/cursor を打ち消す規則を持たない。
        assert!(!LAYOUT_CSS.contains("opacity: 1;"));
        assert!(!LAYOUT_CSS.contains("[data-disabled]"));
    }

    #[test]
    fn container_query_target_is_not_the_container_itself() {
        // codex(P1) 是正: container-type/name を持つ要素（`-layout`）と
        // @container セレクタの対象（`-columns`）を分離する。
        assert!(LAYOUT_CSS.contains(
            ".blocks-checkout-form-summary-split-layout {\n  container-type: inline-size;\n  container-name: blocks-checkout-form-summary-split;\n}\n"
        ));
        assert!(LAYOUT_CSS.contains(
            "@container blocks-checkout-form-summary-split (min-width: 64rem) {\n  \
.blocks-checkout-form-summary-split-columns {\n    grid-template-columns: minmax(0, 24rem) minmax(0, 1fr);"
        ));
        let html = demo_html();
        assert!(html.contains(
            "class=\"blocks-checkout-form-summary-split-layout\" data-blocks-checkout-form-summary-split-variant=\"a\"><div class=\"blocks-checkout-form-summary-split-columns\">"
        ));
    }

    #[test]
    fn radio_group_labels_resolve() {
        let html = demo_html();
        for suffix in ["a", "b"] {
            let shipping_label_id = format!("{SHIPPING_METHOD_LABEL_ID}-{suffix}");
            assert!(html.contains(&format!("id=\"{shipping_label_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{shipping_label_id}\"")));
        }
        for suffix in ["a", "b", "c"] {
            let payment_label_id = format!("{PAYMENT_METHOD_LABEL_ID}-{suffix}");
            assert!(html.contains(&format!("id=\"{payment_label_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{payment_label_id}\"")));
        }
    }

    #[test]
    fn layout_css_stacks_summary_first_on_narrow() {
        // DOM 順（サマリ → フォーム）と視覚順を一致させるため `order` は
        // 一切使わない（#3462 レビュー指摘対応、モジュール doc「DOM 順を
        // 視覚順へ一致させる理由」節）。
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("order:"));
        // 2 カラム切り替えは Demo の実コンテナ幅で判定する（ビューポート
        // 幅と一致しないため `@media` ではなく `@container` を使う、
        // モジュール doc「2 カラム切り替え・行内 2 列化はコンテナクエリで
        // 判定する理由」節、#3462 Bugbot 指摘対応）。
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-checkout-form-summary-split (min-width: 64rem)")
        );
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 24rem) minmax(0, 1fr);"));
    }

    #[test]
    fn layout_css_form_row_wrapping_uses_form_container_not_page_breakpoint() {
        // 姓名・市区町村/都道府県の行内 2 列化はフォーム列自体の幅に依存する
        // ため、レイアウト全体の 2 カラム切り替えとは別のネストした
        // コンテナ（`blocks-checkout-form-summary-split-form`）で判定する
        // （#3462 Bugbot 指摘対応、モジュール doc 同節）。
        assert!(LAYOUT_CSS
            .contains("@container blocks-checkout-form-summary-split-form (min-width: 40rem)"));
    }

    #[test]
    fn layout_css_does_not_use_sticky_summary() {
        // `.blocks-demo` は overflow-x: auto の横スクロールコンテナのため
        // `position: sticky` が意図どおり機能しない
        // （`cart_two_column_summary` と同型の判断、#3462 Bugbot 指摘対応、
        // モジュール doc「サマリの `position: sticky` は使わない」節）。
        assert!(!LAYOUT_CSS.contains("sticky"));
    }

    #[test]
    fn layout_css_product_image_has_explicit_size() {
        // グリッド列（4rem）に対し画像の実寸が明示されていないと
        // `min-width: auto` の既定でダミー SVG の実寸がはみ出し summary が
        // 崩れうる（#3462 Bugbot 指摘対応、姉妹ブロック
        // `cart_two_column_summary` の `[data-...-thumb]` と同型の判断）。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-checkout-form-summary-split-product-image] {\n  inline-size: 4rem;\n  block-size: 4rem;\n}"
        ));
    }

    #[test]
    fn layout_css_declares_summary_position_end_column_template() {
        // 版 B（フォーム左・サマリ右、`summary_end`）の列幅配分は
        // `data-blocks-checkout-form-summary-split-summary-position="end"`
        // の有無で切り替え、`order` は使わない（モジュール doc「DOM 順を
        // 視覚順へ一致させる理由」節）。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-checkout-form-summary-split-summary-position=\"end\"] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 24rem);\n  }"
        ));
    }

    #[test]
    fn layout_css_declares_inverted_tone_rules() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-checkout-form-summary-split-summary][data-blocks-checkout-form-summary-split-tone=\"inverted\"]"
        ));
        assert!(LAYOUT_CSS.contains("--fandhe-color-fg-muted: var(--fandhe-color-bg);"));
    }

    #[test]
    fn confirm_button_full_width_does_not_leak_to_discount_apply_button() {
        // discount の「適用」ボタンは summary スコープ配下にあるが、確定
        // ボタン専用の data 属性でのみ全幅化する（#3462 Bugbot 指摘対応）。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-checkout-form-summary-split-confirm] {\n  inline-size: 100%;\n}"
        ));
        assert!(!LAYOUT_CSS
            .contains("[data-blocks-checkout-form-summary-split-summary] [data-scope=\"button\"]"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-checkout-form-summary-split-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-checkout-form-summary-split-layout"
        );
    }
}
