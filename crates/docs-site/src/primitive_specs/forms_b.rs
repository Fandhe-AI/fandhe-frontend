//! Primitives（`fandhe-frontend-headless-ui`）Forms B カテゴリ（入力系 11
//! 部品）原稿データ（イシュー #1025、親 #1030、トラッキング #1035）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] が集約する `path ->
//! ComponentPageSpec` テーブルの 1 本。`crate::component_page::spec_for` が
//! `Layer::Primitives` のときにこのテーブルを線形探索し、Features / API
//! Reference の Arguments 表 / Examples / Accessibility の 4 節を合成する
//! （Demo・Anatomy・`data-*` 属性表は
//! [`crate::primitive_showcase::forms_b`] とその機械導出経路が別途供給する。
//! 本ファイルの `demo` フィールドは 11 件すべて `None` であり、
//! `component_page::generated_content` は `primitive_showcase` 側の Demo を
//! 常に優先照会するため、`Some(...)` を置いても到達しないデッドコードに
//! なる）。
//!
//! # 一次情報・非捏造の方針
//!
//! 各定数の doc コメントに一次情報の `file:line` を付す。原稿データは
//! `crates/headless-ui/src/{number_input,password_input,pin_input,
//! radio_group,rating_group,segment_group,select,signature_pad,slider,
//! switch,tags_input}.rs` のパート関数シグネチャ・rustdoc・
//! `# セキュリティ不変条件` 節のみから採り、ソースで確認できない挙動
//! （矢印キーによるフォーカス移動等の JS 状態機械前提の操作）は記載しない。
//!
//! # Arguments 表の記法規約（本イシューで新規に定める。他カテゴリへの波及）
//!
//! 行名は `<関数名>(<引数名>)` 形式（例: `root(checked)`）とする。全パート
//! 関数が末尾に共通で取る `attrs: Vec<(&str, &str)>` / `children: Vec<Node>`
//! は毎行繰り返さず本 doc で一度だけ説明し、`ArgRow` には出さない。
//! **複数の兄弟パート関数が同一引数名・同一意味を持つ場合**（例:
//! `root`/`label`/`control` がいずれも `disabled: bool` を同じ意味で持つ）は
//! `<fn1>/<fn2>/…(<引数名>)` の連結形でまとめ、行数の水増しを避ける
//! （個別の意味を持つ引数は連結しない）。`default` 列は当該引数の型が
//! `Default` を実装している場合のみ記載し、単なる位置引数（既定値の概念が
//! ない）は空文字列のままとする。この記法は Forms A・Forms C・日付・状態
//! 表示・Overlay/Disclosure・Navigation・Data Display の兄弟カテゴリ
//! （#1024/#1026〜#1029）も揃えることを想定する。
//!
//! # `keyboard`（Accessibility 節）を原則空にする理由
//!
//! `crate::component_specs::forms`（Themes 層、`crates/docs-site/src/
//! component_specs/forms.rs` doc 参照）と同じ既定方針を継承する。本 docs
//! サイトは `crate::script`（テーマトグル + 目次スクロールスパイ）以外の
//! JS を出力せず、Primitives の Demo も静的初期状態のみを描画する
//! （`crate::primitive_showcase` 執筆規約）。矢印キーによる候補移動等、
//! JS 状態機械前提のキー操作は「対応済み」と書かない。**例外**は該当パート
//! 関数が native なフォーム要素（`<input type="checkbox">`/
//! `<input type="radio">` 等）を出力していることが file:line で確認できる
//! 場合に限る（switch の `hidden_input`・radio_group/segment_group の
//! `item_hidden_input` のみが該当する。number_input の `input` は
//! `type="text"`（`role="spinbutton"` の ARIA spinbutton パターン）であり
//! ネイティブ `<input type="number">` ではないため、上下キー操作を
//! 「ブラウザ標準」とは書かない。slider の `hidden_input` も `type="hidden"`
//! でありフォーカス・キー操作の対象にならないため同様に書かない）。
//!
//! # Examples の import 制約（判断 C）
//!
//! 本ファイルの Examples レンダラは [`fandhe_frontend_core`] と
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`（`hui`
//! エイリアス）のみを import する。`fandhe_frontend_pre_styled_ui::{button,
//! input, ...}` 等の styled 部品モジュールは import しない
//! （`grep -n 'fandhe_frontend_pre_styled_ui::' crates/docs-site/src/
//! primitive_specs/forms_b.rs | grep -v fandhe_frontend_headless_ui` が
//! 何も出力しないことで機械検証できる）。視覚的な統一のため Examples 本体を
//! `div(vec![("class", "primitives-demo-frame")], …)` で包むことのみ許容し
//! （同 class は `crate::primitive_showcase::mod` の `LAYOUT_CSS` に実在する
//! ため `tests/site_css_contract.rs` の class 契約を通る）、それ以外の新規
//! class は追加しない。見出しタグ（`h2`/`h3`）は Examples 本体に含めない
//! （`component_page::examples_section` が `h3` を付与する。右カラム目次の
//! 汚染を避ける、過去事故 #980）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API（[`fandhe_frontend_core::el`]/
//! [`fandhe_frontend_core::text`] とそのタグヘルパー、および headless-ui の
//! パート関数）のみで組み立てる。`raw_html()` および HTML 文字列の直接
//! 組み立て（`format!("<td>{}</td>", …)`）は使わない。全データは
//! `&'static str` リテラルであり、`component_page.rs` 側が [`text`] 経由
//! （既定エスケープ）でのみ出力する。

use fandhe_frontend_core::{code, div, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::data_attrs::Orientation;
use hui::number_input::{self, NumberInputFlags};
use hui::password_input::{self, PasswordAutocomplete, PasswordInputProps};
use hui::pin_input::{self, PinInputKind};
use hui::radio_group;
use hui::rating_group::{self, RatingItemFlags};
use hui::segment_group;
use hui::select;
use hui::signature_pad;
use hui::slider;
use hui::switch;
use hui::tags_input;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Forms B 11 ページの `path -> ComponentPageSpec` テーブル。順序は
/// `crate::primitives_catalog` の `PrimitiveCategory::FormsB` 台帳順
/// （`crates/docs-site/src/primitives_catalog.rs` 参照）に合わせる。
/// [`crate::primitive_specs::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/number-input/", NUMBER_INPUT),
    ("/primitives/password-input/", PASSWORD_INPUT),
    ("/primitives/pin-input/", PIN_INPUT),
    ("/primitives/radio-group/", RADIO_GROUP),
    ("/primitives/rating-group/", RATING_GROUP),
    ("/primitives/segment-group/", SEGMENT_GROUP),
    ("/primitives/select/", SELECT),
    ("/primitives/signature-pad/", SIGNATURE_PAD),
    ("/primitives/slider/", SLIDER),
    ("/primitives/switch/", SWITCH),
    ("/primitives/tags-input/", TAGS_INPUT),
];

/// 一次情報: `crates/headless-ui/src/number_input.rs`
/// （モジュール doc 1-124、`NumberInputFlags` 201-214、
/// `root`/`label`/`control` 218-281、`input` 297-350、`value_text` 353-368、
/// `increment_trigger`/`decrement_trigger` 387-432。7 パーツ・ValueText・
/// `data-readonly`/`data-required`・`role="group"`・`"home"`/`"end"` dispatch
/// はイシュー #1613 で ark-ui/zag.js の number-input machine と突合して追加）。
const NUMBER_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Input / IncrementTrigger / DecrementTrigger / ValueText の 7 anatomy パーツを提供する（`data-state` は持たない。連続量のため境界到達の 2 値のみを各トリガーの `disabled`/`data-disabled` で表現する。ValueText はイシュー #1613 で追加）。",
        "`min`/`max`/`step`/`value` を fail-closed に正規化し、`step` の小数桁数へ丸めてから `[min, max]` へ clamp する（浮動小数点ドリフト対策）。",
        "`input` は `type=\"text\"` + `role=\"spinbutton\"`（WAI-ARIA spinbutton パターン）で意味論を担い、`aria-valuemin`/`aria-valuemax` を常時、`aria-valuenow` は値が確定しているときのみ出力する。`autocomplete=\"off\"`/`autocorrect=\"off\"`/`spellcheck=\"false\"`/`aria-roledescription=\"numberfield\"`（イシュー #1613）も常時出力する。",
        "`control` は `role=\"group\"` を持ち、`disabled`/`invalid` に応じて `aria-disabled`/`aria-invalid` を出力する（イシュー #1613）。`root`/`control` の `data-readonly`、`label` の `data-required`（label のみ、zag.js に倣う）も同イシューで追加した。",
        "dispatch（`\"increment\"`/`\"decrement\"`/`\"set\"`/`\"clear\"`/`\"home\"`/`\"end\"`）と hydration（`data-hydrate-value`/`-min`/`-max`/`-step`）は fail-closed（パース不能・非有限値は `HydrateError` で拒否）。`\"home\"`/`\"end\"`（イシュー #1613）は値を `min`/`max` へ設定する。",
    ],
    arguments: &[
        ArgRow { name: "root/label/control/value_text(flags)", kind: "NumberInputFlags", default: "NumberInputFlags::default()", description: "`disabled`/`readonly`/`required`/`invalid` の 4 bool をまとめた薄い構造体（clippy `too_many_arguments` 回避、イシュー #1613 で `input` 専用から全パーツ共通化）。`required` は `label` のみが `data-required` へ反映する。" },
        ArgRow { name: "label(input_id)", kind: "Option<&str>", default: "", description: "`Some` のとき `input` の id と対で `for` 属性を出力する。" },
        ArgRow { name: "input(name)", kind: "&str", default: "", description: "ネイティブ `name` 属性。" },
        ArgRow { name: "input(id)", kind: "Option<&str>", default: "", description: "`Some` のとき `id` 属性を出力する（`label(input_id)` の関連付け先）。" },
        ArgRow { name: "input(value)", kind: "Option<&str>", default: "", description: "現在値の整形済み文字列。`None`（未入力）なら `aria-valuenow`/`value` 属性ごと出力しない。" },
        ArgRow { name: "input(min)", kind: "&str", default: "", description: "`aria-valuemin` として常時出力する整形済み文字列。" },
        ArgRow { name: "input(max)", kind: "&str", default: "", description: "`aria-valuemax` として常時出力する整形済み文字列。" },
        ArgRow { name: "increment_trigger/decrement_trigger(input_id)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-controls` で `input` と関連付ける。" },
        ArgRow { name: "increment_trigger/decrement_trigger(disabled)", kind: "bool", default: "", description: "境界到達（`can_increment`/`can_decrement` が `false`）と全体無効化を合成した最終値。`true` でネイティブ `disabled` + `data-disabled` を出力する。" },
    ],
    examples: &[
        ExampleEntry {
            title: "Disabled",
            description: "全体を無効化した NumberInput。両トリガーがネイティブ `disabled` を持ち、`input` にも `data-disabled` が付く。",
            render: ex_number_input_disabled,
        },
        ExampleEntry {
            title: "Read-only",
            description: "`readonly` を立てた NumberInput。`root`/`control`/`input` に `data-readonly` が付き、両トリガーは境界到達で `disabled` になる（イシュー #1613）。",
            render: ex_number_input_readonly,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "利用者が data-scope / data-part / data-disabled / data-invalid / data-readonly 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_number_input_custom_css,
        },
    ],
    keyboard: &[
        KeyRow { key: "ArrowUp", description: "`step` 分だけ増加する（`\"increment\"` dispatch）。DOM 配線はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務であり、本モジュールは dispatch 契約のみを提供する。" },
        KeyRow { key: "ArrowDown", description: "`step` 分だけ減少する（`\"decrement\"` dispatch）。DOM 配線はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務であり、本モジュールは dispatch 契約のみを提供する。" },
        KeyRow { key: "Home", description: "値を `min` に設定する（`\"home\"` dispatch、イシュー #1613）。DOM 配線はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務であり、本モジュールは dispatch 契約のみを提供する。" },
        KeyRow { key: "End", description: "値を `max` に設定する（`\"end\"` dispatch、イシュー #1613）。DOM 配線はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務であり、本モジュールは dispatch 契約のみを提供する。" },
        KeyRow { key: "Enter", description: "入力中の値を確定する（`\"set\"` dispatch）。DOM 配線はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務であり、本モジュールは dispatch 契約のみを提供する。" },
    ],
    aria: &[
        AriaRow { attribute: "role=\"spinbutton\" (input)", description: "WAI-ARIA spinbutton パターン。ネイティブ `<input type=\"number\">` ではなく `type=\"text\"` を使うため、上下キーによる native な増減は成立しない（増減はクライアントランタイムの配線責務、モジュール doc「out-of-scope」参照）。" },
        AriaRow { attribute: "aria-roledescription=\"numberfield\" (input)", description: "常時出力する（イシュー #1613）。" },
        AriaRow { attribute: "aria-valuemin / aria-valuemax (input)", description: "正規化済みの `min`/`max` を常時出力する。" },
        AriaRow { attribute: "aria-valuenow (input)", description: "現在値が確定している（`value` が `Some`）ときのみ出力する。" },
        AriaRow { attribute: "role=\"group\" (control)", description: "呼び出し側 `attrs` に同名キーがなければ常時出力する（イシュー #1613）。" },
        AriaRow { attribute: "aria-disabled / aria-invalid (control)", description: "`disabled`/`invalid` が `true` のときのみ出力する（イシュー #1613）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/password_input.rs`
/// （モジュール doc 1-120、`root`/`label`/`control` 拡張版、
/// `input`（`readonly`/`data-state`/`autocapitalize`/`spellcheck` 追加）、
/// `visibility_trigger`/`indicator`、`PasswordInputProps`（`readonly` 追加）。
/// イシュー #1614 で ark-ui docs（Data Attributes 表）・zag
/// `password-input.connect.ts`・Radix `Password Toggle Field` docs と突合し、
/// `readonly` 対応とパーツ別 `data-*` 分布の是正を行った（差分表は PR 本文
/// 参照）。
const PASSWORD_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Input / VisibilityTrigger / Indicator の 6 anatomy パーツを提供する。表示切替（`visible`）状態機械のみを持ち、パスワード値そのものは一切保持しない（`input` は `value` 引数自体を持たない）。",
        "`input` は `visible` の bool から `type=\"password\"`/`type=\"text\"` を決定的に導出する（呼び出し側文字列を type スロットへ通す経路がない）。`data-state`（`root`/`control` と同じ visible/hidden 語彙）・`autocapitalize=\"off\"`・`spellcheck=\"false\"` も固定付与する（ark-ui/zag 準拠、イシュー #1614）。",
        "`readonly`（`PasswordInputProps`）で `input` にネイティブ `readonly` を、6 パーツ全てに `data-readonly` を付与する。`visibility_trigger` へはネイティブ `disabled` を合成しない（表示切替は値を変更しない操作のため、readonly でもキーボード操作での表示確認を封じない）。",
        "パーツ別 `data-*` 分布は ark-ui/zag に合わせる: `root`/`control`/`indicator` は `disabled`/`invalid`/`readonly`、`label` は `disabled`/`invalid`/`readonly`/`required`、`input` は `disabled`/`invalid`/`readonly`/`required`（`required` は当方の superset。`root`/`control` の `data-state` も同様に superset として維持し、後方互換を壊さない）。",
        "`visibility_trigger` は `button type=\"button\"` + `aria-pressed`（トグルボタンパターン）+ `aria-controls` で意味論を担う。`aria-label` は呼び出し側が付与する（国際化はアプリ側の責務）。zag は `aria-expanded`/`tabIndex: -1` を採用するが、本コンポーネントは意図的に追随しない（`aria-pressed` は input が展開/折り畳み対象領域でないため、tab 順序は Radix の「キーボード操作時はトリガーへフォーカス維持」と整合させ tabindex を付与しない。詳細はモジュール doc「参考サイトとの意図的な差分」節参照）。",
        "`PasswordInputProps` から `input` の id（`\"{id}-input\"`）を決定的に導出し、`label`/`visibility_trigger` へも同じ値を一貫伝播する。`name` 属性が必要な場合は `input` の `attrs` へ呼び出し側が付与する（reserved キーではない）。",
        "`Default` は Hidden（パスワードを隠す方が安全側の既定）。hydration 属性 `data-hydrate-visible` は fail-closed（未知値を `HydrateError` で拒否）。",
        "ark-ui/zag の `ignorePasswordManagers` 相当（`data-1p-ignore` 等のパスワードマネージャ連携抑止属性）は本コンポーネントでは提供しない（機能プロップでありデータ属性の状態語彙ではないため、イシュー #1614 でも見送り）。",
    ],
    arguments: &[
        ArgRow { name: "root/control/input/visibility_trigger/indicator(visible)", kind: "bool", default: "", description: "表示中かどうか（`data-state`/`type`/`aria-pressed` を決める）。" },
        ArgRow { name: "PasswordInputProps.id", kind: "&str", default: "", description: "ベース id。`input` の id（`\"{id}-input\"`）の導出元。" },
        ArgRow { name: "PasswordInputProps.disabled", kind: "bool", default: "", description: "`input`/`visibility_trigger` にネイティブ `disabled` + `data-disabled` を付与する。" },
        ArgRow { name: "PasswordInputProps.readonly", kind: "bool", default: "", description: "`input` にネイティブ `readonly` + `data-readonly` を、6 パーツ全てに `data-readonly` を付与する。`visibility_trigger` にはネイティブ `disabled` を合成しない。" },
        ArgRow { name: "PasswordInputProps.invalid", kind: "bool", default: "", description: "`root`/`control`/`input`/`label`/`indicator` に `data-invalid` を、`input` に `aria-invalid=\"true\"` を付与する。" },
        ArgRow { name: "PasswordInputProps.required", kind: "bool", default: "", description: "`label`/`input` にネイティブ `required`（`input` のみ）+ `data-required` を付与する。" },
        ArgRow { name: "PasswordInputProps.autocomplete", kind: "PasswordAutocomplete", default: "", description: "`CurrentPassword`（`autocomplete=\"current-password\"`）または `NewPassword`（`\"new-password\"`）。" },
        ArgRow { name: "indicator(props)", kind: "&PasswordInputProps", default: "", description: "`disabled`/`invalid`/`readonly` の状態反映（`aria-hidden=\"true\"` 固定、意味論は `visibility_trigger` が担う）。" },
    ],
    examples: &[
        ExampleEntry {
            title: "Invalid + new-password",
            description: "登録フォーム向けに `autocomplete=\"new-password\"` を指定し、`invalid`/`required` を立てた状態。`aria-invalid=\"true\"` が `input` に付与される。",
            render: ex_password_input_invalid,
        },
        ExampleEntry {
            title: "Read-only",
            description: "`readonly: true` の状態。`input` にネイティブ `readonly` が、6 パーツ全てに `data-readonly` が付く。表示切替トリガーはキーボード操作のまま無効化されない。",
            render: ex_password_input_readonly,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "headless-ui はスタイルレスであるため、`[data-scope]`/`[data-part]`/`[data-state]`/`data-*` 状態属性セレクタを利用者が自前 CSS で装飾する最小例です。",
            render: ex_password_input_custom_css,
        },
    ],
    keyboard: &[
        KeyRow { key: "Enter / Space", description: "`visibility_trigger` はネイティブ `<button type=\"button\">` のため、フォーカス時の Enter/Space によるクリック相当の発火はブラウザ標準操作として成立する。クリックから表示切替への dispatch 配線（`\"toggle\"`）は `fandhe-frontend-wasm-full` 側の責務。" },
        KeyRow { key: "Tab", description: "`input` → `visibility_trigger` の順にフォーカスが移動する。トリガーには `tabindex` を付与しないため tab 順序から除外されない（モジュール doc「参考サイトとの意図的な差分」節参照）。" },
    ],
    aria: &[
        AriaRow { attribute: "aria-pressed (visibility_trigger)", description: "トグルボタンパターン。表示中（`visible == true`）で `\"true\"`。" },
        AriaRow { attribute: "aria-controls (visibility_trigger)", description: "`input` の id を指す。" },
        AriaRow { attribute: "aria-label (visibility_trigger)", description: "呼び出し側が `attrs` へ付与する（固定文言は持たない）。推奨文言は表示中 `\"Hide password\"`、非表示中 `\"Show password\"`（ark-ui/zag の既定翻訳に準拠）。" },
        AriaRow { attribute: "aria-invalid (input)", description: "`invalid` が `true` のときのみ `\"true\"` を出力する。" },
        AriaRow { attribute: "aria-hidden (indicator)", description: "常に `\"true\"`（装飾専用、意味論は `visibility_trigger` の `aria-pressed` が担うため重複読み上げを防ぐ）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/pin_input.rs`
/// （モジュール doc 1-63、`root`/`label`/`control` 141-172、
/// `input` 174-229、`hidden_input` 231-251、`PinInputKind` 80-139）。
const PIN_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Input（桁ごと）/ HiddenInput の 5 anatomy パーツと、桁ごとの値・フォーカス位置・complete 判定を担う独自状態機械を提供する。",
        "`data-complete` は全桁充足時のみの存在属性として本モジュールが一元管理する（パーツ間で語彙を分裂させない）。",
        "各桁 `input` は `aria-label`（例: `\"PIN digit 1 of 6\"`）を必ず付与し、スクリーンリーダー利用者が桁位置を把握できるようにする。`kind`（`Numeric`/`Alphanumeric`/`Alphabetic`）が文字種検証と `inputmode` の両方を決める。",
        "秘密値の SSR プレフィルは非推奨（`hidden_input` の連結値・各桁 `value` は HTML ソースに平文で現れるため、実際の OTP を初期値として埋め込む用途には使わないこと）。",
    ],
    arguments: &[
        ArgRow { name: "root/label(complete)", kind: "bool", default: "", description: "`data-complete` 存在属性を反映する。" },
        ArgRow { name: "root(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "input(index)", kind: "usize", default: "", description: "0-origin の桁インデックス（`aria-label` の桁位置表示に使う）。" },
        ArgRow { name: "input(count)", kind: "usize", default: "", description: "全桁数（`aria-label` の分母に使う）。" },
        ArgRow { name: "input(value)", kind: "&str", default: "", description: "当該桁の値（空文字列 = 未入力、1 文字 = 入力済み）。" },
        ArgRow { name: "input(kind)", kind: "PinInputKind", default: "PinInputKind::Numeric", description: "文字種別。`inputmode` 属性と文字検証の両方を決める。" },
        ArgRow { name: "input(mask)", kind: "bool", default: "", description: "`true` で `type=\"password\"`（表示マスク）、`false` で `type=\"text\"`。" },
        ArgRow { name: "input(otp)", kind: "bool", default: "", description: "`true` のとき `autocomplete=\"one-time-code\"`（WebOTP/SMS 自動入力連携）を付与する。" },
        ArgRow { name: "hidden_input(name)", kind: "&str", default: "", description: "フォーム送信名。各桁 `input` は `name` を持たないため、実送信値はこのパーツのみが担う。" },
        ArgRow { name: "hidden_input(value)", kind: "&str", default: "", description: "全桁の連結値。" },
    ],
    examples: &[ExampleEntry {
        title: "Alphanumeric, unmasked",
        description: "`PinInputKind::Alphanumeric` + `mask: false` の招待コード用途。`Numeric` 既定と異なり `inputmode` 属性は出力されない。",
        render: ex_pin_input_alphanumeric,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "aria-label (input)", description: "`\"PIN digit {index+1} of {count}\"` を各桁ごとに動的生成する（`render()` の既定エスケープを経由するため注入経路にはならない）。" },
        AriaRow { attribute: "data-complete", description: "全桁充足時のみの存在属性。`root`/`label`/`input` が共有する。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/radio_group.rs`
/// （モジュール doc 1-111、`root`/`label` 132-172、
/// `item`/`item_control`/`item_text` 174-228、`item_hidden_input` 230-260）。
const RADIO_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Item / ItemControl / ItemText / ItemHiddenInput の 6 anatomy パーツと、「高々 1 項目が選択される」状態機械（`crate::state::SingleSelect` 埋め込み）を提供する。",
        "`item_hidden_input` が生成するネイティブ `<input type=\"radio\">` がチェック状態・フォーム送信・グループ内排他選択を担うため、`item_control` には `role=\"radio\"`/`aria-checked` を重複付与しない（二重読み上げ防止）。",
        "`item` は `<label>` 要素であり、内包する `item_hidden_input` へのクリック委譲が JS なしで機能する。",
        "`data-state` 値語彙（`\"checked\"`/`\"unchecked\"`）は Checkbox / Switch と共有する共通機械由来であり、本モジュール独自の値を作らない。",
    ],
    arguments: &[
        ArgRow { name: "root(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "root(orientation)", kind: "Option<Orientation>", default: "", description: "`Some` のときのみ `data-orientation`/`aria-orientation` を付与する。" },
        ArgRow { name: "root(labelled_by)", kind: "Option<&str>", default: "", description: "`Some` のときのみ `aria-labelledby` を付与する（`label(id)` と対で使う）。" },
        ArgRow { name: "label(id)", kind: "Option<&str>", default: "", description: "`root(labelled_by)` の参照先 id。" },
        ArgRow { name: "item/item_control/item_text/item_hidden_input(checked)", kind: "bool", default: "", description: "`data-state` の checked/unchecked を決める。" },
        ArgRow { name: "item/item_control/item_text/item_hidden_input(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "item(value)", kind: "&str", default: "", description: "選択肢の値。`data-value` として動的値のまま出力する（クリック → dispatch payload の源）。" },
        ArgRow { name: "item_hidden_input(name)", kind: "Option<&str>", default: "", description: "`Some` のとき `name` 属性を出力する（グループ内排他選択の同一 name グループ）。" },
        ArgRow { name: "item_hidden_input(value)", kind: "&str", default: "", description: "ネイティブ `value` 属性。" },
    ],
    examples: &[ExampleEntry {
        title: "Vertical, with disabled item",
        description: "`orientation: Some(Orientation::Vertical)` + 2 番目の項目を `disabled` にした例。",
        render: ex_radio_group_vertical_disabled,
    }],
    keyboard: &[KeyRow {
        key: "ArrowUp / ArrowDown / ArrowLeft / ArrowRight",
        description: "`item_hidden_input` はネイティブ `<input type=\"radio\">` であり、同一 `name` グループ内の矢印キー移動・選択はブラウザ標準操作として成立する（JS 配線不要）。",
    }],
    aria: &[
        AriaRow { attribute: "role=\"radiogroup\" (root)", description: "固定付与。" },
        AriaRow { attribute: "role=\"radio\" / aria-checked", description: "`item_control` へは明示付与しない。意味論は `item_hidden_input` のネイティブ `<input type=\"radio\">` が担う（二重読み上げ防止）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/rating_group.rs`
/// （モジュール doc 1-76、`root`/`label`/`control` 89-135、
/// `item` 137-192、`hidden_input` 194-216）。
const RATING_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Item / HiddenInput の 5 anatomy パーツと、`1..=count` の数値評価値（未評価は `None`）+ hover プレビューを持つ状態機械を提供する。",
        "`role=\"radiogroup\"` + `item` の `role=\"radio\"`/`aria-checked` で WAI-ARIA radio パターンを表現するが、ネイティブ `<input type=\"radio\">` の組ではなく単一の `hidden_input`（`type=\"hidden\"`）でフォーム送信値を送る。",
        "`hover`（ポインタが指している星）は transient な CSR 挙動のため SSR 静的マークアップには現れず、hydration でも直列化しない（常に `None` から開始）。",
        "`readonly` が `true` のとき値の変更操作は no-op になる（他ユーザーの平均評価等、表示専用の評価を安全に描画する用途）。",
    ],
    arguments: &[
        ArgRow { name: "root(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "root(readonly)", kind: "bool", default: "", description: "`data-readonly` を反映する。" },
        ArgRow { name: "label(id)", kind: "Option<&str>", default: "", description: "`control(labelled_by)` の参照先 id。" },
        ArgRow { name: "control(labelled_by)", kind: "Option<&str>", default: "", description: "`Some` のときのみ `aria-labelledby` を付与する。" },
        ArgRow { name: "item(index)", kind: "u32", default: "", description: "1-origin の星番号。`data-value` として動的値のまま出力する。" },
        ArgRow { name: "item(flags).checked", kind: "bool", default: "RatingItemFlags::default()", description: "確定選択中（`index == value`）かどうか。`aria-checked`/`data-checked` へ反映する。" },
        ArgRow { name: "item(flags).highlighted", kind: "bool", default: "", description: "塗り表示対象（`index <= display_value`）かどうか。`data-highlighted` へ反映する（確定選択とは独立の軸）。" },
        ArgRow { name: "item(flags).readonly", kind: "bool", default: "", description: "`data-readonly` を反映する。" },
        ArgRow { name: "item(aria_label)", kind: "&str", default: "", description: "呼び出し側が必須で与える国際化可能なラベル（例: `\"1 star\"`）。フレームワーク側でハードコード生成しない。" },
        ArgRow { name: "hidden_input(name)", kind: "Option<&str>", default: "", description: "`Some` のとき `name` 属性を出力する。" },
        ArgRow { name: "hidden_input(value_text)", kind: "&str", default: "", description: "フォーム送信用の現在値 1 個（星群ではなく単一値）。" },
    ],
    examples: &[ExampleEntry {
        title: "Read-only average rating",
        description: "`readonly: true` + `RatingItemFlags { readonly: true, .. }` の他ユーザー平均評価表示例。",
        render: ex_rating_group_readonly,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "role=\"radiogroup\" (control)", description: "固定付与。" },
        AriaRow { attribute: "role=\"radio\" / aria-checked (item)", description: "`item` 自身が `span[role=\"radio\"]`（ネイティブ input の組ではない）。`aria-checked` は `flags.checked` を反映する。" },
        AriaRow { attribute: "aria-label (item)", description: "呼び出し側が必須で与える（例: `\"1 star\"`）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/segment_group.rs`
/// （モジュール doc 1-84、`root` 103-125、`indicator` 127-167、
/// `item`/`item_control`/`item_text` 169-212、`item_hidden_input` 214-243）。
const SEGMENT_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Indicator / Item / ItemText / ItemControl / ItemHiddenInput の 6 anatomy パーツを提供する。状態機械・dispatch・hydration は `crate::radio_group::RadioGroup` へ全委譲し、独自の状態機械を新設しない（WAI-ARIA 上 segmented control は radio パターンそのものであるため）。",
        "`indicator` は SSR 決定的な位置表現を持つ: `(index, item_count)` から `--fandhe-segment-group-index`/`--fandhe-segment-group-count` の 2 CSS カスタムプロパティのみを `style` 属性へ出力する（JS 計測は行わない）。",
        "`item_hidden_input` が生成するネイティブ `<input type=\"radio\">` がチェック状態・フォーム送信・グループ内排他選択を担う（`radio_group` と同型）。",
    ],
    arguments: &[
        ArgRow { name: "root(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "root(orientation)", kind: "Option<Orientation>", default: "", description: "`Some` のときのみ `data-orientation`/`aria-orientation` を付与する。" },
        ArgRow { name: "root(labelled_by)", kind: "Option<&str>", default: "", description: "`Some` のときのみ `aria-labelledby` を付与する。" },
        ArgRow { name: "indicator(position)", kind: "Option<(usize, usize)>", default: "", description: "`Some((index, count))` のとき `data-state=\"checked\"` + 位置 CSS 変数 2 種を出力する。`None`（未選択）は `data-state=\"unchecked\"` のみ。" },
        ArgRow { name: "indicator(orientation)", kind: "Option<Orientation>", default: "", description: "`Some` のとき `data-orientation` を出力する（styled 層が `translateX`/`translateY` を切り替える判断材料）。" },
        ArgRow { name: "item/item_control/item_text/item_hidden_input(checked)", kind: "bool", default: "", description: "`data-state` の checked/unchecked を決める。" },
        ArgRow { name: "item/item_control/item_text/item_hidden_input(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "item(value)", kind: "&str", default: "", description: "選択肢の値。`data-value` として動的値のまま出力する。" },
    ],
    examples: &[ExampleEntry {
        title: "Vertical orientation",
        description: "`orientation: Some(Orientation::Vertical)` で縦並びにした例。`indicator` の `data-orientation` が styled 層の `translateY` 切替判断材料になる。",
        render: ex_segment_group_vertical,
    }],
    keyboard: &[KeyRow {
        key: "ArrowUp / ArrowDown / ArrowLeft / ArrowRight",
        description: "`item_hidden_input` はネイティブ `<input type=\"radio\">` であり、同一 `name` グループ内の矢印キー移動・選択はブラウザ標準操作として成立する（`radio_group` と同型）。",
    }],
    aria: &[
        AriaRow { attribute: "role=\"radiogroup\" (root)", description: "固定付与。" },
        AriaRow { attribute: "aria-hidden=\"true\" (indicator)", description: "装飾専用パーツとして固定付与する。" },
        AriaRow { attribute: "role=\"radio\" / aria-checked", description: "`item_control` へは明示付与しない（`radio_group` と同じ二重読み上げ防止の最小主義）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/select.rs`
/// （モジュール doc 1-76、`root`/`label`/`control` 103-130、
/// `trigger` 132-167、`value_text` 169-198、`clear_trigger`/`indicator`
/// 200-223、`positioner`/`content` 225-286、`item_group`/`item_group_label`
/// 288-322、`item`/`item_text`/`item_indicator` 324-404、
/// `hidden_select` 406-457）。
const SELECT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Trigger / ValueText / ClearTrigger / Indicator / Positioner / Content / ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator / HiddenSelect の 15 anatomy パーツを提供する。",
        "listbox の開閉（`Disclosure`）+ 選択値（`SingleSelect`、高々 1 個）を合成した状態機械を持つ。`data-state` 値語彙は `crate::state::OpenState` の `\"open\"`/`\"closed\"` に一元化し、選択有無の表現にも同じ語彙を再利用する（ark-ui の `checked`/`unchecked` は不採用）。",
        "`hidden_select` はフォーム統合専用のネイティブ `<select>` であり `aria-hidden=\"true\"` + `tabindex=\"-1\"` を固定付与して視覚 UI（`trigger`/`content`）との二重露出を防ぐ。未選択時は非表示 placeholder option を自動挿入し、ブラウザの「先頭 option 自動選択」による誤送信を防ぐ。",
        "位置決め（`positioner` の `style`/`data-side`/`data-align`）は `crate::positioning`（#590）が算出した値を呼び出し側が渡す。Select は arrow を持たない。",
        "highlight 移動・typeahead・キーボードナビゲーション自体は CSR 挙動層のスコープであり、本モジュールは `item(highlighted)`/`content(activedescendant)` の SSR 静的表現のみを提供する。",
    ],
    arguments: &[
        ArgRow { name: "root/control/trigger/indicator/positioner/content/item/item_indicator(state)", kind: "OpenState", default: "OpenState::Closed", description: "開閉（または選択有無）状態。`data-state` へ反映する。" },
        ArgRow { name: "label/item_group_label(id)", kind: "Option<&str>", default: "", description: "`Some` のとき `id` 属性を出力し、対応する `labelledby`/`labelled_by` 引数の関連付け先になる。" },
        ArgRow { name: "trigger(disabled)", kind: "bool", default: "", description: "ネイティブ `disabled` + `data-disabled` を反映する。" },
        ArgRow { name: "trigger(controls)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-controls` で `content` の id と関連付ける。" },
        ArgRow { name: "trigger(labelledby)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-labelledby` で `label` と関連付ける。" },
        ArgRow { name: "value_text(placeholder_shown)", kind: "bool", default: "", description: "`true` のとき `data-placeholder-shown` 存在属性を付与する（未選択時のプレースホルダー表示）。" },
        ArgRow { name: "content(id)", kind: "Option<&str>", default: "", description: "`trigger(controls)` の参照先 id。" },
        ArgRow { name: "content(labelledby)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-labelledby` を付与する。" },
        ArgRow { name: "content(activedescendant)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-activedescendant` を付与する（現在ハイライト中の `item` の id を指す。移動自体は CSR 挙動層の責務）。" },
        ArgRow { name: "item_group(labelledby)", kind: "Option<&str>", default: "", description: "`Some` のときのみ `role=\"group\"` + `aria-labelledby` を対で付与する（名前なし group を作らない）。" },
        ArgRow { name: "item(disabled)", kind: "bool", default: "", description: "`true` のとき `aria-disabled=\"true\"` + `data-disabled` を対で付与する（`div[role=\"option\"]` はネイティブ `disabled` を持たない）。" },
        ArgRow { name: "item(highlighted)", kind: "bool", default: "", description: "`data-highlighted` へ反映する（キーボードナビゲーション等によるフォーカス位置の SSR 静的表現）。" },
        ArgRow { name: "item(value)", kind: "&str", default: "", description: "選択肢の値。`data-value` として動的値のまま出力する。" },
        ArgRow { name: "item(id)", kind: "Option<&str>", default: "", description: "`Some` のとき、`content(activedescendant)` の参照先識別子になる。" },
        ArgRow { name: "hidden_select(selected)", kind: "Option<&str>", default: "", description: "現在選択中の値。`None` のとき非表示 placeholder option を先頭へ自動挿入する。" },
        ArgRow { name: "hidden_select(name)", kind: "Option<&str>", default: "", description: "`Some` のとき `name` 属性を出力する。" },
        ArgRow { name: "hidden_select(options)", kind: "Vec<(&str, &str)>", default: "", description: "`(value, label)` の列。各要素を `<option>` として組み立てる。" },
    ],
    examples: &[ExampleEntry {
        title: "Disabled trigger, unselected",
        description: "`trigger(disabled: true)` + `hidden_select(selected: None)` の未選択・無効化状態。非表示 placeholder option が自動挿入される。",
        render: ex_select_disabled_unselected,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "aria-haspopup=\"listbox\" (trigger)", description: "固定付与。" },
        AriaRow { attribute: "aria-expanded (trigger)", description: "`state.is_open()` を反映する。" },
        AriaRow { attribute: "role=\"listbox\" (content)", description: "固定付与。`aria-activedescendant` の配線先（`trigger` は素の `button` のため付与しない）。" },
        AriaRow { attribute: "role=\"option\" / aria-selected (item)", description: "固定付与 + `selected_state.is_open()` を反映する。" },
        AriaRow { attribute: "aria-hidden=\"true\" / tabindex=\"-1\" (hidden_select)", description: "視覚 UI（`trigger`/`content`）との二重露出・二重フォーカスを防ぐため固定付与する。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/signature_pad.rs`
/// （モジュール doc 1-79、`root`/`label`/`control` 278-307、
/// `segment`/`segment_path` 309-342、`guide`/`clear_trigger` 344-365、
/// `hidden_input` 367-386、`stroke_path_d` 196-230）。
const SIGNATURE_PAD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Segment（`svg`）/ SegmentPath（ストロークごとの `path`）/ Guide / ClearTrigger / HiddenInput の 8 anatomy パーツを提供する。canvas を一切使わず、ストローク（座標列）を SVG path 文字列へ変換する決定的な純粋関数（`stroke_path_d`）が中核である。",
        "`stroke_path_d` は同一座標列から常に同一の `d` 属性値を生成する（固定小数点 2 桁・round half away from zero・指数表記なし）。ポインタイベントの収集自体は本モジュールの責務外（`fandhe-frontend-wasm-full` が座標列を正規化して dispatch する）。",
        "`hidden_input` は全ストロークの `d` 文字列を `;` 結合した値をフォーム送信する。points 数・ストローク数には上限（`MAX_POINTS_PER_STROKE`/`MAX_STROKES`）があり、改ざんされた dispatch payload・hydration 属性による無制限メモリ確保を防ぐ。",
    ],
    arguments: &[
        ArgRow { name: "root(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "root(empty)", kind: "bool", default: "", description: "strokes が空かどうか。`data-empty` として反映する。" },
        ArgRow { name: "control(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "segment(width, height)", kind: "u32, u32", default: "", description: "`viewBox=\"0 0 {width} {height}\"` として描画領域寸法を出力する。" },
        ArgRow { name: "segment(aria_label_text)", kind: "Option<&str>", default: "", description: "`Some` のとき `aria-label` を付与する。未指定時は `role=\"img\"` のみ（偽の説明文を捏造しない fail-closed 方針）。" },
        ArgRow { name: "segment_path(stroke)", kind: "&Stroke", default: "", description: "`d` 属性値は `stroke_path_d(stroke)` の内部生成文字列のみ（`fill`/`stroke` は付与せず headless 中立を保つ）。" },
        ArgRow { name: "clear_trigger(disabled)", kind: "bool", default: "", description: "ネイティブ `disabled` + `data-disabled` を反映する。" },
        ArgRow { name: "hidden_input(name, value)", kind: "&str, &str", default: "", description: "フォーム送信名と、全ストロークを `;` 結合した値。" },
    ],
    examples: &[ExampleEntry {
        title: "Empty, disabled",
        description: "ストロークが 1 本もない無効化状態（`root(empty: true, disabled: true)`）。`guide` のみが描画される。",
        render: ex_signature_pad_empty_disabled,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "role=\"img\" (segment)", description: "`aria_label_text` が `None` のときのみ `role=\"img\"` のみを出力し、偽の説明文を作らない（`aria_label_text` が `Some` のときは `aria-label` を併せて付与する）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/slider.rs`
/// （モジュール doc 1-83、`root`/`label`/`control`/`track`/`range`
/// 179-244、`thumb` 246-284、`hidden_input`/`value_text` 286-310）。
const SLIDER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Track / Range / Thumb / HiddenInput / ValueText の 8 anatomy パーツを提供する。単一値・連続量スライダーであり `data-state` は持たない（境界区分がないため）。",
        "`value` は常に `min` 起点の `step` 単位へスナップしてから `[min, max]` へ clamp する。`max`/`min` ちょうどの値は step グリッドに乗らない場合でも常に到達可能（`snap_to_step_and_clamp`）。",
        "`thumb`（`div role=\"slider\"`）が WAI-ARIA slider パターンの `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-orientation` を常時出力する。ネイティブ `<input type=\"range\">` ではないため、矢印キー操作はブラウザ標準では成立しない（クライアントランタイム側の後続責務）。",
        "range slider（複数 thumb）・pointer ドラッグ・キーボード操作の DOM 配線はスコープ外（単一値スライダーのみ）。",
    ],
    arguments: &[
        ArgRow { name: "root/control/track/range(orientation)", kind: "Orientation", default: "", description: "`data-orientation`（+ `control`/`track`/`range` は同名引数）を反映する。" },
        ArgRow { name: "root/control/track/range(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "thumb(orientation)", kind: "Orientation", default: "", description: "`aria-orientation`/`data-orientation` を反映する。" },
        ArgRow { name: "thumb(min, max, now)", kind: "&str, &str, &str", default: "", description: "`aria-valuemin`/`aria-valuemax`/`aria-valuenow` として常時出力する整形済み文字列。" },
        ArgRow { name: "thumb(aria_valuetext)", kind: "Option<&str>", default: "", description: "`Some` のときのみ `aria-valuetext` を追加する。" },
        ArgRow { name: "thumb(disabled)", kind: "bool", default: "", description: "`true` で `tabindex=\"-1\"` + `aria-disabled`、`false` で `tabindex=\"0\"`。" },
        ArgRow { name: "hidden_input(name, value)", kind: "&str, &str", default: "", description: "フォーム送信専用（意味論は `thumb` の `role=\"slider\"` が担う）。" },
    ],
    examples: &[ExampleEntry {
        title: "Vertical, disabled",
        description: "`Orientation::Vertical` + `disabled: true` の例。`thumb` は `tabindex=\"-1\"` + `aria-disabled=\"true\"` を持つ。",
        render: ex_slider_vertical_disabled,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "role=\"slider\" (thumb)", description: "WAI-ARIA slider パターン。ネイティブ `<input type=\"range\">` ではなくカスタム `div` であり、矢印キー操作は本モジュール単体では成立しない。" },
        AriaRow { attribute: "aria-valuemin / aria-valuemax / aria-valuenow (thumb)", description: "常時出力する。" },
        AriaRow { attribute: "aria-valuetext (thumb)", description: "`Some` のときのみ出力する。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/switch.rs`
/// （モジュール doc 1-71、`root`/`control`/`thumb`/`label` 80-136、
/// `hidden_input` 138-172）。
const SWITCH: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（`<label>`）/ Control / Thumb / Label / HiddenInput の 5 anatomy パーツと、`crate::state::Checkable` を埋め込んだチェック状態機械を提供する。",
        "`data-state` 値語彙は `\"checked\"`/`\"unchecked\"`（`crate::state::Checkable` が一元管理、Checkbox/RadioGroup と共有）。",
        "`hidden_input` は `<input type=\"checkbox\" role=\"switch\">`（WAI-ARIA APG 準拠）。native の `checked` がブラウザによって `aria-checked` へ自動マップされるため、`aria-checked` を明示付与しない（二重読み上げ防止）。",
        "`root` が `<label>` のため、内包する `hidden_input` との暗黙のラベル関連付けが JS なしで成立する（`for`/`id` の配線が不要）。",
    ],
    arguments: &[
        ArgRow { name: "root/control/thumb/label(checked)", kind: "bool", default: "", description: "`data-state` の checked/unchecked を決める。" },
        ArgRow { name: "root/control(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "hidden_input(name, value)", kind: "&str, &str", default: "", description: "ネイティブ `name`/`value` 属性。" },
        ArgRow { name: "hidden_input(checked)", kind: "bool", default: "", description: "`true` のときのみネイティブ `checked` 存在属性を出力する。" },
        ArgRow { name: "hidden_input(disabled)", kind: "bool", default: "", description: "`true` のときのみネイティブ `disabled` 存在属性を出力する。" },
        ArgRow { name: "hidden_input(required)", kind: "bool", default: "", description: "`true` のときネイティブ `required` + `data-required` を出力する。" },
    ],
    examples: &[ExampleEntry {
        title: "Disabled, checked",
        description: "`checked: true` + `disabled: true` の組み合わせ。`hidden_input` にネイティブ `checked`/`disabled` の両方が存在属性として出力される。",
        render: ex_switch_disabled_checked,
    }],
    keyboard: &[KeyRow {
        key: "Space",
        description: "`hidden_input` はネイティブ `<input type=\"checkbox\">` であり、Space キーでのトグルはブラウザ標準操作として成立する（JS 配線不要）。",
    }],
    aria: &[
        AriaRow { attribute: "aria-hidden=\"true\" (control)", description: "装飾専用パーツとして固定付与する（意味論は `hidden_input` が担う）。" },
        AriaRow { attribute: "aria-checked", description: "`hidden_input` へ明示付与しない。native の `<input type=\"checkbox\" role=\"switch\">` の `checked` 状態がブラウザにより自動マップされるため（二重読み上げ防止）。" },
    ],
    demo: None,
};

/// 一次情報: `crates/headless-ui/src/tags_input.rs`
/// （モジュール doc 1-89、`root`/`label` 107-122、`control` 124-144、
/// `item`/`item_preview`/`item_text`/`item_input` 146-195、
/// `item_delete_trigger`/`clear_trigger` 197-240、`input` 242-262、
/// `hidden_input` 264-281）。
const TAGS_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Control / Input / Item / ItemPreview / ItemText / ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput / LiveRegion の 12 anatomy パーツと、タグ文字列の可変長リスト + 重複拒否 + 上限 + 編集中インデックスを持つ値状態機械を提供する。",
        "不変条件「重複タグなし・`len() <= max`・カンマを含まない・空文字列を含まない」を破る入力は dispatch・hydration・コンストラクタのすべての入口で一貫して拒否する（カンマ・空文字列はフォーム送信値のカンマ結合時に曖昧さを生むため、Cursor Bugbot 指摘を踏まえ全入口で禁止）。",
        "`control` は `role=\"listbox\"` + `aria-orientation=\"horizontal\"` を持ち、`item_preview` は `role=\"option\"` + `aria-selected=\"true\"` 固定（常に選択済みタグを表示するため）。",
        "`editing`（編集中インデックス）は ephemeral な DOM 状態のため hydration では運ばない。",
        "`live_region` はタグ数の変化を通知する live region（`role=\"status\"` + `aria-live=\"polite\"` 固定、`root` の直接の子で `control` の兄弟として配置する。テキスト更新の実配線は `fandhe-frontend-wasm-full` の後続責務、イシュー #1069）。",
    ],
    arguments: &[
        ArgRow { name: "root/item(disabled)", kind: "bool", default: "", description: "`data-disabled` を反映する。" },
        ArgRow { name: "control(invalid)", kind: "bool", default: "", description: "`data-invalid` を反映する。" },
        ArgRow { name: "control(label_text)", kind: "&str", default: "", description: "`aria-label` として付与する（listbox 相当の ARIA）。" },
        ArgRow { name: "item(editing)", kind: "bool", default: "", description: "編集モード中かどうか。`data-editing` 存在属性へ反映する。" },
        ArgRow { name: "item_preview(highlighted)", kind: "bool", default: "", description: "`data-highlighted` へ反映する（`aria-selected=\"true\"` 固定とは独立の軸）。" },
        ArgRow { name: "item_input(value)", kind: "&str", default: "", description: "編集中の暫定値。" },
        ArgRow { name: "item_delete_trigger(tag)", kind: "&str", default: "", description: "`aria-label` を `\"Delete {tag}\"` として動的生成する（`children` の視覚的内容とは独立に常時付与）。" },
        ArgRow { name: "input(value)", kind: "&str", default: "", description: "新規タグ入力用のネイティブ入力欄の値。" },
        ArgRow { name: "input(at_max)", kind: "bool", default: "", description: "上限到達時に `data-invalid` + `aria-invalid=\"true\"` を出力する。" },
        ArgRow { name: "hidden_input(name, value)", kind: "&str, &str", default: "", description: "フォーム送信名と、全タグのカンマ結合値。" },
        ArgRow { name: "live_region(children)", kind: "Vec<Node>", default: "", description: "role=\"status\"/aria-live=\"polite\"/aria-atomic=\"true\" を固定付与する live region。通知文言は children として呼び出し側が渡す（イシュー #1069）。" },
    ],
    examples: &[ExampleEntry {
        title: "At max (invalid input)",
        description: "上限に達した状態の `input(at_max: true)`。`data-invalid` + `aria-invalid=\"true\"` が新規タグ入力欄に出力される。",
        render: ex_tags_input_at_max,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "role=\"listbox\" / aria-orientation (control)", description: "固定付与（`aria-orientation=\"horizontal\"`）。" },
        AriaRow { attribute: "role=\"option\" / aria-selected (item_preview)", description: "固定付与。常に選択済みタグを表示するため `aria-selected=\"true\"` 固定。" },
        AriaRow { attribute: "aria-invalid (input)", description: "上限到達（`at_max`）時のみ `\"true\"` を出力する。" },
        AriaRow { attribute: "aria-label (item_delete_trigger)", description: "`\"Delete {tag}\"` を動的生成する（`render()` の既定エスケープ経由）。" },
        AriaRow { attribute: "role=\"status\" / aria-live=\"polite\" / aria-atomic=\"true\" (live_region)", description: "タグ数の変化を通知する live region に固定付与する（イシュー #1069）。" },
    ],
    demo: None,
};

// ---- Examples レンダラ（headless-ui のパート関数のみで組み立てる） ----

fn ex_number_input_disabled() -> Node {
    let flags = NumberInputFlags {
        disabled: true,
        ..NumberInputFlags::default()
    };
    let body = vec![number_input::root(
        flags,
        vec![],
        vec![
            number_input::label(
                flags,
                Some("ni-disabled-input"),
                vec![],
                vec![text("Seats")],
            ),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-disabled-input"),
                        true,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "seats",
                        Some("ni-disabled-input"),
                        Some("2"),
                        "0",
                        "10",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-disabled-input"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

/// `readonly` を立てた NumberInput の Example（イシュー #1613）。
/// `root`/`control`/`input` に `data-readonly` が付き、両トリガーは境界
/// 到達（`readonly` 時は増減しても値が変わらないため常に境界扱い）で
/// `disabled` になる契約は `crate::primitive_showcase::forms_b` の
/// readonly インスタンスと同じ判断。
fn ex_number_input_readonly() -> Node {
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let body = vec![number_input::root(
        flags,
        vec![],
        vec![
            number_input::label(
                flags,
                Some("ni-readonly-input"),
                vec![],
                vec![text("Seats (locked)")],
            ),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-readonly-input"),
                        true,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "seats-readonly",
                        Some("ni-readonly-input"),
                        Some("4"),
                        "0",
                        "10",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-readonly-input"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

/// Examples 用の枠組み（[`crate::primitive_specs::forms_a::wrap_example`] と
/// 同型・同じ class 契約。private のためモジュールをまたいで再利用できず、
/// 本ファイルにも複製する）。`h2`/`h3` は出さない（`examples_section` が
/// `h3` を供給済み）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

/// Examples 用の枠組み。`primitive_showcase::forms_b` のデモ本体と同じ
/// `primitives-demo-frame`/`primitives-demo-note` class のみを使い、
/// `h2`/`h3` は出さない（`forms_a::wrap_example` は private のためこの
/// ファイルへ同型のヘルパを複製する、判断根拠は同ファイル参照）。
fn wrap_password_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-disabled`/`data-invalid`/`data-readonly`
/// 属性セレクタで見た目を組み立てる例を示す（イシュー #1613）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const NUMBER_INPUT_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"number-input\"][data-part=\"control\"] {\n  \
  display: inline-flex;\n  align-items: center;\n  border: 1px solid #888;\n  border-radius: 4px;\n\
}\n\
[data-scope=\"number-input\"][data-part=\"input\"] {\n  \
  border: none;\n  padding: 0.25rem 0.5rem;\n  width: 4rem;\n\
}\n\
[data-scope=\"number-input\"][data-part=\"input\"][data-invalid] {\n  \
  outline: 2px solid #dc2626;\n\
}\n\
[data-scope=\"number-input\"][data-part=\"control\"][data-readonly] {\n  \
  background: #f3f4f6;\n\
}\n\
[data-scope=\"number-input\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n";

fn ex_number_input_custom_css() -> Node {
    let flags = NumberInputFlags::default();
    let markup = number_input::root(
        flags,
        vec![],
        vec![
            number_input::label(flags, Some("ni-css-input"), vec![], vec![text("Quantity")]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-css-input"),
                        false,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity-css",
                        Some("ni-css-input"),
                        Some("5"),
                        "0",
                        "10",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-css-input"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-disabled / data-invalid / data-readonly 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(
                vec![],
                vec![code(vec![], vec![text(NUMBER_INPUT_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

fn password_input_demo_node(props: &PasswordInputProps<'_>, label_text: &str) -> Node {
    password_input::root(
        false,
        props,
        vec![],
        vec![
            password_input::label(props, vec![], vec![text(label_text)]),
            password_input::control(
                false,
                props,
                vec![],
                vec![
                    password_input::input(false, props, vec![]),
                    password_input::visibility_trigger(
                        false,
                        props,
                        vec![("aria-label", "Show password")],
                        vec![password_input::indicator(
                            false,
                            props,
                            vec![],
                            vec![text("👁")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

fn ex_password_input_invalid() -> Node {
    let props = PasswordInputProps {
        id: "pw-invalid-example",
        disabled: false,
        readonly: false,
        invalid: true,
        required: true,
        autocomplete: PasswordAutocomplete::NewPassword,
    };
    let body = vec![password_input_demo_node(&props, "New password")];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_password_input_readonly() -> Node {
    let props = PasswordInputProps {
        id: "pw-readonly-example",
        disabled: false,
        readonly: true,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let body = vec![password_input_demo_node(&props, "Password")];
    div(vec![("class", "primitives-demo-frame")], body)
}

/// 利用者が自前 CSS で `password-input` を装飾する最小例のスニペット
/// （`[data-scope]`/`[data-part]`/`[data-state]`/`data-*` 状態属性セレクタ）。
/// `assets/primitives-showcase.css` には一切追加しない（`[data-scope=`/
/// `[data-part=` 不在契約、`tests/site_css_contract.rs` 参照）。テキストは
/// `text()` 経由（既定エスケープ）で `pre`/`code` に出力するのみで、CSS を
/// 実行・適用する経路は持たない。
const PASSWORD_INPUT_CUSTOM_CSS_SNIPPET: &str = r#"[data-scope="password-input"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  border: 1px solid #ccc;
}

[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: #0a7;
}

[data-scope="password-input"][data-part="input"][data-invalid] {
  border-color: #d33;
}

[data-scope="password-input"][data-part="root"][data-disabled],
[data-scope="password-input"] [data-readonly] {
  opacity: 0.6;
}"#;

fn ex_password_input_custom_css() -> Node {
    let props = PasswordInputProps {
        id: "pw-custom-css",
        disabled: false,
        readonly: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let demo = password_input_demo_node(&props, "Password");
    let snippet = pre(
        vec![],
        vec![code(vec![], vec![text(PASSWORD_INPUT_CUSTOM_CSS_SNIPPET)])],
    );
    wrap_password_example(
        "headless-ui はスタイルレスです。data-scope/data-part/data-state/data-* をセレクタに使い、以下のような CSS を自前で当てられます。",
        vec![demo, snippet],
    )
}

fn ex_pin_input_alphanumeric() -> Node {
    let body = vec![pin_input::root(
        false,
        false,
        vec![],
        vec![
            pin_input::label(false, vec![], vec![text("Invite code")]),
            pin_input::control(
                vec![],
                vec![
                    pin_input::input(
                        0,
                        4,
                        "A",
                        PinInputKind::Alphanumeric,
                        false,
                        false,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        1,
                        4,
                        "B",
                        PinInputKind::Alphanumeric,
                        false,
                        false,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        2,
                        4,
                        "",
                        PinInputKind::Alphanumeric,
                        false,
                        false,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        3,
                        4,
                        "",
                        PinInputKind::Alphanumeric,
                        false,
                        false,
                        false,
                        false,
                        vec![],
                    ),
                ],
            ),
            pin_input::hidden_input("invite", "AB", false, vec![]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_radio_group_vertical_disabled() -> Node {
    let body = vec![radio_group::root(
        false,
        Some(Orientation::Vertical),
        None,
        vec![],
        vec![
            radio_group::label(None, vec![], vec![text("Delivery")]),
            radio_group::item(
                true,
                false,
                "standard",
                vec![],
                vec![
                    radio_group::item_control(true, false, vec![]),
                    radio_group::item_text(true, false, vec![], vec![text("Standard")]),
                    radio_group::item_hidden_input(
                        true,
                        false,
                        Some("delivery"),
                        "standard",
                        vec![],
                    ),
                ],
            ),
            radio_group::item(
                false,
                true,
                "same-day",
                vec![],
                vec![
                    radio_group::item_control(false, true, vec![]),
                    radio_group::item_text(
                        false,
                        true,
                        vec![],
                        vec![text("Same day (unavailable)")],
                    ),
                    radio_group::item_hidden_input(
                        false,
                        true,
                        Some("delivery"),
                        "same-day",
                        vec![],
                    ),
                ],
            ),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_rating_group_readonly() -> Node {
    let mk = |index: u32, checked: bool, highlighted: bool| {
        rating_group::item(
            index,
            RatingItemFlags {
                checked,
                highlighted,
                disabled: false,
                readonly: true,
            },
            &format!("{index} star"),
            vec![],
            vec![text("★")],
        )
    };
    let body = vec![rating_group::root(
        false,
        true,
        vec![],
        vec![
            rating_group::label(None, vec![], vec![text("Average rating")]),
            rating_group::control(
                None,
                vec![],
                vec![
                    mk(1, false, true),
                    mk(2, false, true),
                    mk(3, false, true),
                    mk(4, true, true),
                    mk(5, false, false),
                ],
            ),
            rating_group::hidden_input(Some("avg-rating"), "4", false, vec![]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_segment_group_vertical() -> Node {
    let orientation = Orientation::Vertical;
    let body = vec![segment_group::root(
        false,
        Some(orientation),
        None,
        vec![],
        vec![
            segment_group::indicator(Some((1, 3)), Some(orientation), vec![]),
            segment_group::item(
                false,
                false,
                "day",
                vec![],
                vec![
                    segment_group::item_control(false, false, vec![]),
                    segment_group::item_text(false, false, vec![], vec![text("Day")]),
                    segment_group::item_hidden_input(false, false, Some("range"), "day", vec![]),
                ],
            ),
            segment_group::item(
                true,
                false,
                "week",
                vec![],
                vec![
                    segment_group::item_control(true, false, vec![]),
                    segment_group::item_text(true, false, vec![], vec![text("Week")]),
                    segment_group::item_hidden_input(true, false, Some("range"), "week", vec![]),
                ],
            ),
            segment_group::item(
                false,
                false,
                "month",
                vec![],
                vec![
                    segment_group::item_control(false, false, vec![]),
                    segment_group::item_text(false, false, vec![], vec![text("Month")]),
                    segment_group::item_hidden_input(false, false, Some("range"), "month", vec![]),
                ],
            ),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_select_disabled_unselected() -> Node {
    let state = OpenState::Closed;
    let body = vec![select::root(
        state,
        vec![],
        vec![
            select::label(Some("sel2-label"), vec![], vec![text("Country")]),
            select::control(
                state,
                vec![],
                vec![
                    select::trigger(
                        state,
                        true,
                        Some("sel2-content"),
                        Some("sel2-label"),
                        vec![],
                        vec![
                            select::value_text(true, vec![], vec![text("Select a country")]),
                            select::indicator(state, vec![], vec![text("▾")]),
                        ],
                    ),
                    select::clear_trigger(vec![], vec![text("×")]),
                ],
            ),
            select::positioner(
                state,
                vec![],
                vec![select::content(
                    state,
                    Some("sel2-content"),
                    Some("sel2-label"),
                    None,
                    vec![],
                    vec![select::item_group(
                        None,
                        vec![],
                        vec![select::item(
                            OpenState::Closed,
                            false,
                            false,
                            "jp",
                            None,
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("Japan")]),
                                select::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        )],
                    )],
                )],
            ),
            select::hidden_select(
                None,
                Some("country"),
                true,
                vec![],
                vec![("jp", "Japan"), ("us", "United States")],
            ),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_signature_pad_empty_disabled() -> Node {
    let body = vec![signature_pad::root(
        true,
        true,
        vec![],
        vec![
            signature_pad::label(vec![], vec![text("Signature (disabled)")]),
            signature_pad::control(
                true,
                vec![],
                vec![
                    signature_pad::guide(vec![], vec![]),
                    signature_pad::segment(160, 60, Some("Empty signature area"), vec![], vec![]),
                ],
            ),
            signature_pad::clear_trigger(true, vec![], vec![text("Clear")]),
            signature_pad::hidden_input("signature", "", true, vec![]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_slider_vertical_disabled() -> Node {
    let orientation = Orientation::Vertical;
    let body = vec![slider::root(
        orientation,
        true,
        vec![],
        vec![
            slider::label(vec![], vec![text("Brightness")]),
            slider::control(
                orientation,
                true,
                vec![],
                vec![
                    slider::track(
                        orientation,
                        true,
                        vec![],
                        vec![slider::range(orientation, true, vec![], vec![])],
                    ),
                    slider::thumb(
                        orientation,
                        "0",
                        "100",
                        "70",
                        Some("70%"),
                        true,
                        vec![],
                        vec![],
                    ),
                ],
            ),
            slider::hidden_input("brightness", "70", true, vec![]),
            slider::value_text(vec![], vec![text("70%")]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_switch_disabled_checked() -> Node {
    let body = vec![switch::root(
        true,
        true,
        vec![],
        vec![
            switch::control(
                true,
                true,
                vec![],
                vec![switch::thumb(true, vec![], vec![])],
            ),
            switch::label(true, vec![], vec![text("Dark mode (locked on)")]),
            switch::hidden_input("dark-mode", "on", true, true, false, vec![]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}

fn ex_tags_input_at_max() -> Node {
    let body = vec![tags_input::root(
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Skills (max 2)")]),
            tags_input::control(
                false,
                true,
                "Skills",
                vec![],
                vec![
                    tags_input::item(
                        false,
                        false,
                        vec![],
                        vec![tags_input::item_preview(
                            false,
                            vec![],
                            vec![
                                tags_input::item_text(vec![], vec![text("rust")]),
                                tags_input::item_delete_trigger(
                                    "rust",
                                    false,
                                    vec![],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                    tags_input::item(
                        false,
                        false,
                        vec![],
                        vec![tags_input::item_preview(
                            false,
                            vec![],
                            vec![
                                tags_input::item_text(vec![], vec![text("wasm")]),
                                tags_input::item_delete_trigger(
                                    "wasm",
                                    false,
                                    vec![],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                    tags_input::input("", false, true, vec![("aria-label", "Add a skill")]),
                ],
            ),
            tags_input::clear_trigger(false, vec![], vec![text("Clear")]),
            tags_input::hidden_input("skills", "rust,wasm", false, vec![]),
        ],
    )];
    div(vec![("class", "primitives-demo-frame")], body)
}
