//! Primitives（`fandhe-frontend-headless-ui`）Data Display / Utilities 系
//! 15 部品ページの原稿データ（イシュー #1029、親トラッキング #1035
//! Phase 5。イシュー #2114 で `marker`・イシュー #2111 で `attachment`・
//! イシュー #2108 で `bubble`・
//! イシュー #2105 で `message`・
//! イシュー #2065 で `item` を追加、当初 10 部品）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] から参照される
//! [`crate::component_page::ComponentPageSpec`] 定数群を保持する専用
//! モジュール。本モジュール自体は生成物へ直接寄与しない
//! （`component_page::render_component_page` が `spec_for` 経由で読み取り、
//! Demo〔[`crate::primitive_showcase::data_display_utilities`]〕・Anatomy・
//! `data-*` 属性表（いずれも機械導出、Primitives 層は CSS 変数表を持たない）
//! と合成して 6 節ページを組み立てる）。
//!
//! 対象は attachment・avatar・bubble・carousel・item・json-tree-view・
//! marker・message・scroll-area・skip-nav・splitter・steps・tour・
//! tree-view・visually-hidden の 15 部品
//! （`crates/docs-site/src/primitives_catalog.rs` の
//! `PrimitiveCategory::DataDisplayUtilities` 並び順と一致させる）。
//!
//! # Arguments 抽出規約（`/themes/` 側 #946 規約・`primitive_specs::overlay_disclosure`
//! （#1027）からの層固有の変更点を踏襲）
//!
//! `fandhe-frontend-headless-ui` の当該モジュールが公開する関数・
//! （`Carousel`/`Splitter`/`Steps`/`Tour` のような状態機械については）
//! メソッドの**型付き引数**を抽出元とする。`attrs: Vec<(&str, &str)>`/
//! `children: Vec<Node>` は全部品共通の定型引数のため原則除外するが、
//! [`visually_hidden::root`] のように公開関数がこの 2 引数のみを持つ場合は
//! 例外的にそのまま記載する（それ以外に本モジュールの API 面が存在しない
//! ため、除外すると `arguments` が空になり受け入れ条件 1 に反する）。
//! [`ArgRow`] には part 列が無いため、`<関数/メソッド名>: <引数名>` 形式で
//! `name` 列へ埋め込む（10 部品で表記を統一する、#1027 と同型）。
//!
//! # `keyboard` を 4/10 件で空にする理由（6 件は非空、イシュー #1667 で改訂）
//!
//! 本カテゴリ 10 モジュールのうち `tabindex` を出力するのは
//! `scroll_area`（`viewport`、`crates/headless-ui/src/scroll_area.rs:68-71`）・
//! `skip_nav`（`content`、`crates/headless-ui/src/skip_nav.rs:100-107`）・
//! `splitter`（`resize_trigger`）の 3 件のみ（非テストソースの grep
//! 結果、`tree_view`/`json_tree_view` の roving tabindex は
//! `fandhe-frontend-wasm-full` `keynav.rs` が実行時に付与するため headless
//! 側 SSR 出力には現れない）。残り 4 件（avatar/carousel/tour/
//! visually_hidden）は `tabindex` を一切出力せず、クリック・矢印キー等の実
//! DOM 配線は各モジュール doc の out-of-scope 節が
//! `fandhe-frontend-wasm-full` 後続イシューの責務と明示している
//! （`carousel.rs:71`/`tour.rs:89-91` 参照。avatar/visually_hidden はそもそも
//! キー操作の対象になる要素を持たない）。実装が焦点制御に関与しない部品へ
//! 「ArrowRight で次へ進む」のような未実装の対話を書くと利用者へ誤った
//! 安心を与えるため、該当 4 件は `keyboard: &[]` を採用し、非空である
//! `aria` 表のみで Accessibility 節を成立させる（`component_page.rs` の
//! Accessibility 節省略規則参照。scroll_area/skip_nav の 2 件は `tabindex`
//! の**属性事実のみ**を `KeyRow` に記載し、対話そのものは記載しない）。
//! `splitter`/`json_tree_view`/`tree_view` は上記 2 件と異なり
//! Arrow/Home/End 等のキーボード配線が `fandhe-frontend-wasm-full`
//! に実装済み（`splitter` はイシュー #1074、`tree_view`（`json_tree_view`
//! が再利用する構造部も含む）はイシュー #1072）のため、`keyboard` へ
//! 実装済みの対話を記載する。`splitter` はさらに未実装の対話
//! （Shift+Arrow、`SplitterAction::IncrementLarge`/`DecrementLarge`
//! の状態機械のみ）を区別して記載する（イシュー #1664 参照突合）。
//! `tree_view` の既知ギャップ・非追随（`*` 一括展開・Shift+Arrow/Ctrl+A・
//! F2）はキーボード表ではなく `TREE_VIEW.features`/`site/primitives/
//! tree-view.md` 側の「参考サイトとの差分」節に記載する（イシュー #1667）。
//!
//! # `avatar`/`visually_hidden` の Accessibility 節が空にならない理由
//!
//! `avatar`（`crates/headless-ui/src/avatar.rs`）・
//! `visually_hidden`（`crates/headless-ui/src/visually_hidden.rs`）は
//! いずれも `role`/`aria-*` を一切出力しない（`avatar.rs`/
//! `visually_hidden.rs` 全文で `role`/`aria-` grep 0 件、非テスト行のみで
//! 確認）。`aria` を空のままにすると Accessibility 節ごと省略されてしまう
//! ため、非付与の事実そのものを `AriaRow { attribute: "(該当なし)", .. }`
//! として明示する（`crate::component_specs_nav_data::AVATAR`/`BADGE` と同型の
//! 先例）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`ArgRow`]/[`AriaRow`]/
//! [`KeyRow`] を構築し、`raw_html()` や HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切行わない。実際のエスケープは
//! `component_page.rs` 側の `table`/`td`/`text` ノード木経由で `render()`
//! が行う。`examples` のレンダラは `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`
//! （イシュー #693 方針、`hui` エイリアス）経由の headless-ui パート関数
//! のみで組み立て、`fandhe_frontend_pre_styled_ui::` の部品関数（styled
//! 層）は一切呼ばない（受け入れ条件 3）。ダミー文字列は無害なもの
//! （`example.com` 等の予約ドメイン、架空の名前）に限る。

use fandhe_frontend_core::{button, code, div, img, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::attachment::{self, AttachmentRootProps, AttachmentState, AttachmentVariant};
use hui::avatar::{self, ImageStatus};
use hui::bubble::{self, BubbleGroupPosition, BubbleRootProps, BubbleVariant};
use hui::data_attrs::Orientation;
use hui::fandhe_frontend_interactive::Component;
use hui::item::{self, ItemMediaVariant, ItemRootProps, ItemVariant};
use hui::json_tree_view::{self, JsonValue};
use hui::marker::{self, MarkerRootProps, MarkerTone, MarkerVariant};
use hui::message::{self, MessageAlign, MessageRole, MessageRootProps};
use hui::positioning::{Align, Placement, Side};
use hui::progress::Progress;
use hui::scroll_area;
use hui::skip_nav;
use hui::splitter;
use hui::steps::Steps;
use hui::tour::{Tour, TourAction, TourStep, TourTriggerKind};
use hui::tree_view;
use hui::visually_hidden;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Examples 用の枠組み（`primitive_specs/forms_a.rs::wrap_example` /
/// `primitive_specs/forms_c_date_status.rs::wrap_example` と同型。
/// [`crate::primitive_showcase`] のデモ本体と同じ `primitives-demo-frame`/
/// `primitives-demo-note` class のみを使い、`h2`/`h3` は出さない）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

// ---------------------------------------------------------------------
// Attachment（/primitives/attachment/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/attachment.rs`（モジュール doc、
/// イシュータイトルとの差分・会話系 4 部品の共通語彙への不追随・
/// `progress` は attachment scope のスロット・shadcn/ui との意図的差分）、
/// `root`/`media`/`content`/`name`/`meta`/`progress`/`actions`/`action`
/// シグネチャ。非テスト行で `role`/`aria-*` の出力は `action` の
/// `aria-label` のみ。
fn ex_attachment_uploading_with_progress() -> Node {
    let progress = Progress::new(0.0, 100.0, Some(64.0), Orientation::Horizontal);
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::Image,
            state: AttachmentState::Uploading,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(
                vec![],
                vec![img(
                    vec![
                        ("src", crate::showcase::IMAGE_DEMO_SRC),
                        ("alt", "photo.png のプレビュー"),
                    ],
                    vec![],
                )],
            ),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("photo.png")]),
                    attachment::meta(vec![], vec![text("PNG · 2.4 MB · 64%")]),
                ],
            ),
            attachment::progress(
                vec![],
                vec![progress.root(
                    Some("64%"),
                    vec![],
                    vec![progress.track(vec![], vec![progress.range(vec![], vec![])])],
                )],
            ),
        ],
    )
}

/// 削除アクション（`action`、`aria-label` + ネイティブ `disabled`）を持つ
/// エラー状態の例。
fn ex_attachment_error_with_delete_action() -> Node {
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Error,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("📄")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("archive.zip")]),
                    attachment::meta(vec![], vec![text("Upload failed")]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Delete archive.zip",
                    false,
                    vec![],
                    vec![text("✕")],
                )],
            ),
        ],
    )
}

/// 自前 CSS の最小例（`AVATAR_CUSTOM_CSS_SNIPPET` と同型のパターン）。
/// CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const ATTACHMENT_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"attachment\"][data-part=\"root\"] {\n  \
  display: flex;\n  gap: 0.5rem;\n  padding: 0.5rem;\n  border: 1px solid #d1d5db;\n  border-radius: 0.375rem;\n\
}\n\
[data-scope=\"attachment\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"attachment\"][data-part=\"root\"][data-state=\"error\"] {\n  \
  border-color: #dc2626;\n\
}\n\
[data-scope=\"attachment\"][data-part=\"content\"] {\n  \
  display: flex;\n  flex-direction: column;\n\
}\n";

fn ex_attachment_custom_css() -> Node {
    let demo = attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Idle,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("📄")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("report.pdf")]),
                    attachment::meta(vec![], vec![text("PDF · 128 KB")]),
                ],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-disabled 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            demo,
            pre(
                vec![],
                vec![code(vec![], vec![text(ATTACHMENT_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const ATTACHMENT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "添付ファイル 1 件の表示を表現する 8 anatomy パーツ（root/media/content/name/meta/progress/actions/action）を提供する（attachment.rs）。イシュータイトルの 7 パーツに加え、削除等の個別操作を担う action を追加した（attachment.rs「イシュータイトルとの差分」節）。状態機械を持たない静的部品であり wasm-full 側の配線は不要。",
        "root へ data-variant（file/image）・data-state（idle/uploading/error）・data-disabled（存在属性）を付与する。会話系 4 部品（message/bubble/attachment/marker）の共通語彙（data-role/data-align）は意図的に持たない（attachment.rs「会話系 4 部品の共通語彙への不追随」節）。",
        "progress は attachment scope の単純なスロットであり、呼び出し側が crate::progress::Progress のパーツ（root/track/range 等）を入れ子にする契約とする（attachment.rs「progress は attachment scope のスロット」節。data-scope=\"progress\" は attachment scope を上書きしない）。",
        "name/meta は整形済み文字列を children で受け取るだけのスロットであり、byte → KB 変換等の数値・単位整形は行わない（docs/policy/intentional-non-adoption.md §3.23 と同じ判断軸、attachment.rs参照）。参照実体は shadcn/ui の Attachment のみ（ark-ui・chakra-ui・Radix に対応部品なし、参照軸 #2001）。size/orientation・trigger（カード全面クリックオーバーレイ）・group（横スクロールコンテナ）は意図的に非採用（attachment.rs「shadcn/ui 実 API との意図的差分」節）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: props.variant",
            kind: "AttachmentVariant",
            default: "AttachmentVariant::File",
            description: "表示形態（file/image）。data-variant へ出力する（attachment.rs）。",
        },
        ArgRow {
            name: "root: props.state",
            kind: "AttachmentState",
            default: "AttachmentState::Idle",
            description: "アップロード状態（idle/uploading/error）。data-state へ出力する（attachment.rs）。",
        },
        ArgRow {
            name: "root: props.disabled",
            kind: "bool",
            default: "false",
            description: "true のとき data-disabled 存在属性を付与する（attachment.rs）。",
        },
        ArgRow {
            name: "action: label",
            kind: "&str",
            default: "\"\"",
            description: "空でなければ aria-label へ出力する（attachment.rs）。",
        },
        ArgRow {
            name: "action: disabled",
            kind: "bool",
            default: "false",
            description: "true のときネイティブ disabled と data-disabled の両方を出力する（attachment.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Uploading with progress",
            description: "画像プレビュー + アップロード進捗（Progress の入れ子）を表示する例です。",
            render: ex_attachment_uploading_with_progress,
        },
        ExampleEntry {
            title: "Error with delete action",
            description: "アップロード失敗状態と削除アクション（aria-label 付き button）の例です。",
            render: ex_attachment_error_with_delete_action,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-state/data-disabled 属性セレクタでスタイルを当てる例です。",
            render: ex_attachment_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "(なし)",
        description: "root/media/content/name/meta/progress/actions はいずれもキー操作を提供しない静的コンテナである。action はネイティブ button のため Space/Enter で押下できる（attachment.rs）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-label（action）",
            description: "label が空でなければ action パーツへ出力する（attachment.rs「actions/action」節）。",
        },
        AriaRow {
            attribute: "disabled（action、ネイティブ）",
            description: "disabled 引数が true のとき action パーツへネイティブ disabled と data-disabled の両方を出力する（attachment.rs）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Avatar（/primitives/avatar/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/avatar.rs:1-120`（モジュール doc、
/// `data-state` 語彙・ARIA について・スコープ外・参考サイトとの突合）、
/// `:193-248`（`root`/`image`/`fallback` シグネチャ）、`:262-282`
/// （`Avatar::new`）。非テスト行で `role`/`aria-` の出力は 0 件。
fn ex_avatar_error_fallback() -> Node {
    let status = ImageStatus::Error;
    avatar::root(
        vec![],
        vec![
            avatar::image(
                status,
                "https://example.com/broken-avatar.png",
                "Ada Lovelace",
                vec![],
            ),
            avatar::fallback(status, vec![], vec![text("AL")]),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1659、`CALENDAR_CUSTOM_CSS_SNIPPET`
/// 〔`primitive_specs/forms_c_date_status.rs`〕と同型のパターン）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const AVATAR_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"avatar\"][data-part=\"root\"] {\n  \
  display: inline-block;\n  width: 3rem;\n  height: 3rem;\n  border-radius: 50%;\n  overflow: hidden;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"image\"] {\n  \
  width: 100%;\n  height: 100%;\n  object-fit: cover;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"fallback\"] {\n  \
  display: flex;\n  align-items: center;\n  justify-content: center;\n  \
  width: 100%;\n  height: 100%;\n  background: #e5e7eb;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"image\"][data-state=\"hidden\"],\n\
[data-scope=\"avatar\"][data-part=\"fallback\"][data-state=\"hidden\"] {\n  \
  display: none;\n\
}\n";

/// Radix Primitives の `delayMs`（フォールバック表示遅延）を非採用とした
/// 代わりに、利用者が自前 CSS でどう円形アバターを組み立てるかを示す例
/// （イシュー #1659 差分メモ参照）。`Loaded`（実在アセット）と `Error`
/// （イニシャルフォールバック）の 2 インスタンスを並べ、
/// `data-state="hidden"` の多層防御（属性セレクタ + `hidden` 存在属性）を
/// 実演する。
fn ex_avatar_custom_css() -> Node {
    let loaded = ImageStatus::Loaded;
    let loaded_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                loaded,
                crate::showcase::IMAGE_DEMO_SRC,
                "Naledi Khumalo",
                vec![],
            ),
            avatar::fallback(loaded, vec![], vec![text("NK")]),
        ],
    );
    let error = ImageStatus::Error;
    let error_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                error,
                "https://example.com/missing-avatar.png",
                "Priya Das",
                vec![],
            ),
            avatar::fallback(error, vec![], vec![text("PD")]),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            loaded_avatar,
            error_avatar,
            pre(
                vec![],
                vec![code(vec![], vec![text(AVATAR_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const AVATAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "画像読み込みステータス 3 値（ImageStatus::Loading/Loaded/Error）を管理する状態機械 Avatar を提供する（avatar.rs:135-182）。",
        "Root / Image / Fallback の 3 anatomy パーツで構成し、Loaded のときのみ image を表示、それ以外は fallback を表示する安全側の既定（avatar.rs:173-182,193-248）。",
        "image パーツは alt テキストを必須引数として要求することが実質的なアクセシビリティ担保であり、専用の role/aria-* は付与しない（avatar.rs:37-43）。",
        "ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui の 4 参照サイトと突合済み（イシュー #1659）。anatomy/data-*/ARIA は一致し是正なし。Radix の delayMs（表示遅延）・各パーツへの dir/id・イニシャル自動導出は意図的に非採用（avatar.rs:81-120）。",
    ],
    arguments: &[
        ArgRow {
            name: "image: src",
            kind: "&str",
            default: "",
            description: "画像 URL（avatar.rs:211-229、必須）。",
        },
        ArgRow {
            name: "image: alt",
            kind: "&str",
            default: "",
            description: "代替テキスト（avatar.rs:211-229、必須。実質的なアクセシビリティ担保）。",
        },
        ArgRow {
            name: "image/fallback: status",
            kind: "ImageStatus",
            default: "ImageStatus::Loading",
            description: "画像読み込みステータス。Loaded のときのみ image が可視（avatar.rs:140-148,173-182）。",
        },
        ArgRow {
            name: "Avatar::new: initial",
            kind: "ImageStatus",
            default: "ImageStatus::Loading",
            description: "状態機械の初期ステータス（avatar.rs:262-282）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Error fallback",
            description: "画像読み込み失敗（ImageStatus::Error）時のイニシャル表示例です。",
            render: ex_avatar_error_fallback,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-state 属性セレクタで円形アバターのスタイルを当てる例です。",
            render: ex_avatar_custom_css,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root/image/fallback は固有の role/aria-* を出力しない（avatar.rs 全文の非テスト行で role/aria- grep 0 件）。image パーツの alt テキストのみが代替情報を提供する。参照 4 サイト（ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui）とも role/aria-* を付与しない点で一致する（イシュー #1659 突合）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Bubble（/primitives/bubble/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/bubble.rs`（モジュール doc「会話系
/// 4 部品の共通語彙への追随」「`data-variant`（shadcn の 7 色調を 3 形態
/// へ縮約）」「`data-group-position`（算出は利用者責務）」
/// 「`reactions`/`reaction`」「折りたたみは bubble scope のまま
/// `collapsible` の属性契約を再利用」参照）。基本の吹き出し 1 対を実演する。
fn ex_bubble_basic_pair() -> Node {
    div(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Outline,
                    align: MessageAlign::End,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(
                    vec![],
                    vec![text("How do I center a div?")],
                )],
            ),
            bubble::root(
                BubbleRootProps::default(),
                vec![],
                vec![bubble::content(
                    vec![],
                    vec![text("Use display: flex + place-items: center.")],
                )],
            ),
        ],
    )
}

/// 連続発言（同じ発言者の 3 個の吹き出し）で `data-group-position`
/// （`first`/`middle`/`last`）の使い分けを実演する（モジュール doc
/// 「`data-group-position`（算出は利用者責務）」参照。算出そのものは
/// 呼び出し側が行い、本例では固定値として渡す）。
fn ex_bubble_group_position() -> Node {
    div(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    group_position: BubbleGroupPosition::First,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(
                    vec![],
                    vec![text("First, install the crate.")],
                )],
            ),
            bubble::root(
                BubbleRootProps {
                    group_position: BubbleGroupPosition::Middle,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(
                    vec![],
                    vec![text("Then add it to your Cargo.toml.")],
                )],
            ),
            bubble::root(
                BubbleRootProps {
                    group_position: BubbleGroupPosition::Last,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(vec![], vec![text("Then call render().")])],
            ),
        ],
    )
}

/// 自前 CSS の最小例（`data-scope`/`data-part`/`data-variant`/`data-align`/
/// `data-group-position` 属性セレクタのみを使う。headless-ui 自体は
/// スタイルを持たない）。
const BUBBLE_CUSTOM_CSS_SNIPPET: &str = "[data-scope=\"bubble\"][data-part=\"root\"] {\n  display: inline-block;\n  max-width: 32rem;\n  padding: 0.5rem 0.75rem;\n  border-radius: 0.75rem;\n}\n[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"solid\"] {\n  background: #e2e8f0;\n}\n[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"outline\"] {\n  border: 1px solid #cbd5e1;\n}\n[data-scope=\"bubble\"][data-part=\"root\"][data-group-position=\"first\"] {\n  border-radius: 0.75rem 0.75rem 0.75rem 0.25rem;\n}\n[data-scope=\"bubble\"][data-part=\"root\"][data-group-position=\"last\"] {\n  border-radius: 0.75rem 0.75rem 0.25rem 0.75rem;\n}\n";

fn ex_bubble_custom_css() -> Node {
    let node = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Solid,
            group_position: BubbleGroupPosition::First,
            ..Default::default()
        },
        vec![],
        vec![bubble::content(
            vec![],
            vec![text("Styled with plain CSS.")],
        )],
    );
    wrap_example(
        "data-scope / data-part / data-variant / data-align / data-group-position 属性セレクタで塗り・角丸連結を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            node,
            pre(
                vec![],
                vec![code(vec![], vec![text(BUBBLE_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const BUBBLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "チャット吹き出し 1 個を表現する 6 anatomy パーツ（root/content/reactions/reaction/collapse-trigger/collapse-content）を提供する（bubble.rs）。状態機械を持たない静的部品。",
        "data-align は会話系 4 部品（message/bubble/attachment/marker）の共通語彙（正は message.rs）をそのまま再利用する。data-variant（solid/outline/plain）は shadcn の 7 色調を「塗り・枠線・無装飾」の 3 形態へ縮約し、実際の色調選択は fandhe-frontend-pre-styled-ui の ColorPalette 軸へ委ねる（bubble.rs「data-variant」節）。",
        "data-group-position（single/first/middle/last）は連続発言の角丸連結用の表示状態のみを持ち、算出（何番目かの判定）は利用者責務である（bubble.rs「data-group-position」節）。",
        "reactions/reaction は role=\"group\" + 任意 aria-label（role=\"img\" は不採用）を採用し、reaction は data-selected 存在属性のみを持つ非インタラクティブな表示用パーツである。押下・集計・トグルは内包しない（bubble.rs「reactions/reaction」節）。",
        "collapse-trigger/collapse-content は collapsible.rs の属性契約（OpenState・aria-expanded・aria-controls）を bubble scope のまま再利用する。fandhe-frontend-wasm-full の折りたたみクリック配線は現時点で未整備（bubble.rs「wasm-full 未配線」節）。参照実体は shadcn/ui の Bubble のみ（ark-ui・chakra-ui・Radix に対応部品なし、参照軸 #2001）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: props.variant",
            kind: "BubbleVariant",
            default: "BubbleVariant::Solid",
            description: "見た目の形態（solid/outline/plain）。data-variant へ出力する（bubble.rs）。",
        },
        ArgRow {
            name: "root: props.align",
            kind: "MessageAlign",
            default: "MessageAlign::Start",
            description: "水平整列（start/end）。message::MessageAlign を再利用し data-align へ出力する（bubble.rs）。",
        },
        ArgRow {
            name: "root: props.group_position",
            kind: "BubbleGroupPosition",
            default: "BubbleGroupPosition::Single",
            description: "連続発言中の位置（single/first/middle/last）。算出は利用者責務。data-group-position へ出力する（bubble.rs）。",
        },
        ArgRow {
            name: "reactions: label",
            kind: "&str",
            default: "\"\"",
            description: "空でなければ aria-label へ出力する（bubble.rs）。",
        },
        ArgRow {
            name: "reaction: selected",
            kind: "bool",
            default: "false",
            description: "true のとき data-selected 存在属性を付与する（bubble.rs）。",
        },
        ArgRow {
            name: "collapse-trigger: state",
            kind: "OpenState",
            default: "(呼び出し側が指定)",
            description: "aria-expanded・data-state を同期させる開閉状態（collapsible.rs の属性契約を再利用、bubble.rs）。",
        },
        ArgRow {
            name: "collapse-trigger: controls",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき aria-controls で collapse-content と関連付ける（bubble.rs）。",
        },
        ArgRow {
            name: "collapse-content: state",
            kind: "OpenState",
            default: "(呼び出し側が指定)",
            description: "closed のとき hidden 存在属性を付与する（bubble.rs）。",
        },
        ArgRow {
            name: "collapse-content: id",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき id 属性を出力し、collapse-trigger の controls と対で関連付ける（bubble.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Basic bubble pair",
            description: "outline（自分側、右寄せ）と solid（相手側、左寄せ）の 1 往復です。",
            render: ex_bubble_basic_pair,
        },
        ExampleEntry {
            title: "Consecutive bubbles with group position",
            description: "同じ発言者の連続発言で data-group-position（first/middle/last）を使い分ける例です。",
            render: ex_bubble_group_position,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-variant/data-align/data-group-position 属性セレクタでスタイルを当てる例です。",
            render: ex_bubble_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "Space / Enter",
        description: "collapse-trigger はネイティブ <button type=\"button\"> として描画され、ブラウザ標準の Space/Enter → click 発火に従う（bubble.rs。fandhe-frontend-wasm-full の click → \"toggle\" dispatch 配線は現時点で未整備のため、CSR での実際の開閉には呼び出し側の配線が必要）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"group\" / aria-label",
            description: "reactions パーツに固定付与する。label が空でなければ aria-label を出力する（bubble.rs「reactions/reaction」節）。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "collapse-trigger パーツに付与する。controls が Some のとき aria-controls で collapse-content と関連付ける（collapsible.rs の属性契約を再利用、bubble.rs）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Carousel（/primitives/carousel/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/carousel.rs`（モジュール doc「参考
/// サイトとの突合（イシュー #1660）」節・決定的な遷移規則・スコープ外）、
/// `root`/`prev_trigger`/`next_trigger`/`item_group`/`item`/`indicator` の
/// シグネチャと role/aria-*/`data-*` 出力、`Carousel::new`。
fn ex_carousel_vertical_loop() -> Node {
    // 状態機械 `Carousel` の利便メソッドで組み立てる（イシュー #1660、
    // `crate::primitive_showcase::data_display_utilities::carousel_section`
    // と同型の理由: `prev_disabled`/`next_disabled` の自動注入で `data-*`
    // 出力の不整合を構造的に防ぐ）。index=2（末尾）・slide_count=3・
    // loop=true・縦方向で、循環により prev/next とも disabled にならない
    // 構成を示す。
    let carousel = hui::carousel::Carousel::new(2, 3, true, Orientation::Vertical);
    carousel.root(
        "Featured photos (vertical, looping)",
        vec![],
        vec![
            carousel.item_group(
                vec![],
                vec![
                    carousel.item(0, vec![], vec![text("Slide 1")]),
                    carousel.item(1, vec![], vec![text("Slide 2")]),
                    carousel.item(2, vec![], vec![text("Slide 3")]),
                ],
            ),
            carousel.control(
                vec![],
                vec![
                    carousel.prev_trigger("Previous slide", vec![], vec![text("‹")]),
                    carousel.indicator_group(
                        vec![],
                        vec![
                            carousel.indicator(0, vec![]),
                            carousel.indicator(1, vec![]),
                            carousel.indicator(2, vec![]),
                        ],
                    ),
                    carousel.next_trigger("Next slide", vec![], vec![text("›")]),
                ],
            ),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1660、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定
/// エスケープを経由し、`crate::primitive_showcase` の専用スタイルシート
/// （`[data-scope=`/`[data-part=` を持たない契約、
/// `tests/site_css_contract.rs`）へは追加しない。イシュー #1660 で追加した
/// `data-index`（トラック水平移動）・`data-orientation="vertical"`（縦方向
/// 切替）・`data-current`（indicator の強調表示）・`data-disabled`
/// （trigger の減光）の 4 セレクタを実演する。`root` を `overflow: hidden`
/// の表示領域（クリッパー）、`item-group` を実際に動く軌道（トラック）に
/// 分離し、`item-group` の `transform` が `Carousel::item_group` の
/// `style` 出力する `--fandhe-carousel-index` を `calc()` で参照して現在
/// 位置に応じた `translateX`（水平）/`translateY`（垂直、
/// `data-orientation="vertical"` 側で上書き）を反映する
/// （`crates/pre-styled-ui/src/carousel.rs` の `recipe()` と同じ
/// root=クリッパー/item-group=トラックの分離構造）。codex-review 指摘
/// #1925 是正: 是正前は `overflow: hidden` と `transform` を同じ
/// `item-group` へ重ねており、トラックを動かすとクリップ領域ごと
/// 一緒にずれて index=1 でも Slide 1 が表示されたままだった（クリッパーと
/// トラックが同一要素だとクリップ座標系ごと移動してしまうため）。
/// PR #1925 codex-review 指摘 是正（1 回目）: 縦方向 `item-group` の
/// `flex-direction: column` 上書きに合わせ `height: 100%` を追加した
/// （`root` の `height: 12rem` を継承しないと主軸サイズが不定のまま残り、
/// `item` の `flex: 0 0 100%` が正しい高さへ解決できず `translateY` の
/// 百分率計算が崩れて Slide 2 以降が表示されなかったため）。
///
/// PR #1925 codex-review 指摘 是正（2 回目・P1 指摘）: 上記の是正のまま
/// `root` に固定高さ `height: 12rem` を持たせ続けると、`root` 自身の
/// ボックスが 12rem に固定され、その下に通常のブロックフローで並ぶ兄弟
/// パーツ `control`（前後ボタン・indicator）が `root` の
/// `overflow: hidden` に完全にクリップされ表示されなくなる不具合があった
/// （`item-group` が `height: 100%` で `root` の 12rem を丸ごと占有し、
/// `control` の描画開始位置が `root` の境界と一致してしまうため）。
/// `root` からは固定高さを外し（`control` を隠さないよう auto のまま
/// 子要素合計に追随させる）、代わりに **`item-group` 自身**へ固定高さ
/// `height: 12rem` と `overflow: hidden` を持たせることで、スタックした
/// スライドのクリップを `item-group` 自身の内部で完結させる
/// （`crates/pre-styled-ui/src/carousel.rs` の `recipe()` における
/// 同型の是正と同じ判断、モジュール rustdoc「transform ベースのスライド
/// 位置表現」節参照）。
///
/// PR #1925 codex-review 指摘 是正（3 回目・P1 1 件 + Cursor Bugbot 指摘）:
/// 上記（2 回目）の是正は `item-group` 自身に固定高さ・`overflow: hidden`・
/// `translateY` の 3 つを同時に持たせてしまい、クリップ領域（固定高さ +
/// `overflow: hidden` を持つ静止した表示領域）と移動対象（`translateY` で
/// 動くトラック）が同一要素になっていた。この状態で `item-group` を
/// 動かすとクリップ座標系ごと一緒に移動してしまい、index=1 以降で次の
/// スライドがクリップ領域の外（表示領域外）に出てしまう不具合があった。
/// `item-group[data-orientation="vertical"]` は固定高さ・
/// `overflow: hidden` を保持したまま `transform: none`（`item-group` 既定
/// の横方向 `translateX` を打ち消す）で**静止した表示領域**に徹し、
/// 代わりに `item[data-orientation="vertical"]` へ `translateY` を移す
/// （`item` は `flex: 0 0 100%` で `item-group` と同じ高さを持つため、
/// 全 `item` へ同じ量の `translateY` を適用すると `item-group` 自体を
/// 動かすのと幾何学的に等価になる。`crates/pre-styled-ui/src/carousel.rs`
/// の `recipe()` における同型の是正と同じ判断）。control を表示領域の外
/// （`item-group` の後続の通常フロー）に置く構造自体は既に満たしている
/// ため変更しない。
///
/// PR #1925 codex-review 指摘 是正（4 回目・P1 1 件 + Cursor Bugbot 指摘）:
/// `item[data-orientation="vertical"]` は `flex: 0 0 100%` のみで既定
/// `min-height: auto`（CSS Flexbox 仕様の automatic minimum size、
/// https://www.w3.org/TR/css-flexbox-1/#min-size-auto）が残っており、
/// 背の高い画像などを含む `item` は `item-group` の固定高さ `12rem` を
/// 超えて伸びてしまう。`translateY` は各 `item` 自身の border box 高さを
/// 基準にした百分率のため、`item` ごとに高さが揃わないと index=1 以降で
/// 移動量が食い違い、次のスライドが表示領域外にずれる。`min-height: 0`
/// で自動最小サイズを打ち消し、`overflow: hidden` で内容の高さに関わらず
/// `item` 自身の境界を確定させる（`crates/pre-styled-ui/src/carousel.rs`
/// の `recipe()` における同型の是正と同じ判断）。
///
/// PR #1925 codex-review 指摘 是正（5 回目・P1 2 件）: 横方向 `item` は
/// 内容を境界でクリップしていなかった。`min-width: 0` のみで `overflow`
/// が既定の `visible` のままだったため、幅を超える内容（大きい画像等）が
/// 隣のスライドへはみ出していた。orientation によらない `item` セレクタへ
/// `min-width: 0`/`min-height: 0`/`overflow: hidden` をまとめ（縦方向
/// セレクタ側の重複宣言は削除）、横縦で対称な寸法固定・内容クリップに
/// した（`crates/pre-styled-ui/src/carousel.rs` の `.base("item", ...)`
/// における同型の是正と同じ判断）。
const CAROUSEL_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"carousel\"][data-part=\"root\"] {\n  \
  overflow: hidden;\n  width: 100%;\n\
}\n\
[data-scope=\"carousel\"][data-part=\"item-group\"] {\n  \
  display: flex;\n  \
  transform: translateX(calc(var(--fandhe-carousel-index) * -100%));\n  \
  transition: transform 0.2s;\n\
}\n\
[data-scope=\"carousel\"][data-part=\"item\"] {\n  \
  flex: 0 0 100%;\n  \
  min-width: 0;\n  \
  min-height: 0;\n  \
  overflow: hidden;\n\
}\n\
[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"] {\n  \
  flex-direction: column;\n  \
  height: 12rem;\n  \
  overflow: hidden;\n  \
  transform: none;\n\
}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-orientation=\"vertical\"] {\n  \
  transition: transform 0.2s;\n  \
  transform: translateY(calc(var(--fandhe-carousel-index) * -100%));\n\
}\n\
[data-scope=\"carousel\"][data-part=\"indicator\"][data-current] {\n  \
  background: #2563eb;\n\
}\n\
[data-scope=\"carousel\"][data-part=\"prev-trigger\"][data-disabled],\n\
[data-scope=\"carousel\"][data-part=\"next-trigger\"][data-disabled] {\n  \
  opacity: 0.4;\n  cursor: not-allowed;\n\
}\n";

/// index を用いて `translateX(calc(var(--fandhe-carousel-index) * -100%))`
/// を組み立てる呼び出し側の実演（headless 自体はこの `style` を出力しない、
/// `fandhe-frontend-pre-styled-ui` の `item_group` recipe と同じ計算式）。
/// `data-current`/`data-disabled` は状態機械が自動出力する（headless の
/// 既定契約）ため、本例では CSS セレクタ側の当て方のみを示す。
fn ex_carousel_custom_css() -> Node {
    let carousel = hui::carousel::Carousel::new(1, 3, false, Orientation::Horizontal);
    let node = carousel.root(
        "Featured photos",
        vec![],
        vec![
            carousel.item_group(
                vec![],
                vec![
                    carousel.item(0, vec![], vec![text("Slide 1")]),
                    carousel.item(1, vec![], vec![text("Slide 2")]),
                    carousel.item(2, vec![], vec![text("Slide 3")]),
                ],
            ),
            carousel.control(
                vec![],
                vec![
                    carousel.prev_trigger("Previous slide", vec![], vec![text("‹")]),
                    carousel.indicator_group(
                        vec![],
                        vec![
                            carousel.indicator(0, vec![]),
                            carousel.indicator(1, vec![]),
                            carousel.indicator(2, vec![]),
                        ],
                    ),
                    carousel.next_trigger("Next slide", vec![], vec![text("›")]),
                ],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-orientation / data-current / data-disabled 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません（root を表示領域〔overflow: hidden〕、item-group をトラックとして分離し、item_group の style=\"--fandhe-carousel-index: N\" を calc() で参照する transform〔水平は translateX、縦方向は translateY〕で現在位置までスライドを移動します）。",
        vec![
            node,
            pre(
                vec![],
                vec![code(vec![], vec![text(CAROUSEL_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const CAROUSEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Control / PrevTrigger / NextTrigger / ItemGroup / Item / IndicatorGroup / Indicator の 8 anatomy パーツを提供する（carousel.rs 冒頭）。",
        "root は role=\"region\" + aria-roledescription=\"carousel\" + 呼び出し側指定の aria-label を固定出力する（carousel.rs::root）。",
        "item は role=\"group\" + aria-roledescription=\"slide\" + \"{n} of {m}\" 形式の aria-label を自動生成する（carousel.rs::item）。",
        "autoplay 非対応の初期実装のため item_group の aria-live は常に \"polite\" 固定（carousel.rs::item_group）。",
        "index==slide_count-1 かつ loop=true のとき Next は先頭へ循環し、prev/next とも disabled にならない決定的な遷移規則を持つ（carousel.rs モジュール doc「決定的な遷移規則」節）。",
        "zag.js との参照突合（イシュー #1660）により data-orientation を全 8 パーツへ拡張し、item/indicator に data-index（0-origin）、item に data-inview を追加した（carousel.rs モジュール doc「参考サイトとの突合」節）。",
        "dispatch に Home/End キー相当の \"first\"/\"last\"（CarouselAction::First/Last）を追加した（carousel.rs::CarouselAction）。DOM keydown 配線自体は wasm-full 側の後続責務のためスコープ外（下記キーボード操作参照）。",
    ],
    arguments: &[
        ArgRow {
            name: "root/control/prev_trigger/next_trigger/item_group/item/indicator_group/indicator: orientation",
            kind: "Orientation",
            default: "",
            description: "carousel のレイアウト方向。全 8 パーツが data-orientation を出力する（carousel.rs、イシュー #1660）。",
        },
        ArgRow {
            name: "root: label",
            kind: "&str",
            default: "",
            description: "aria-label に出力する説明文（carousel.rs::root、必須）。",
        },
        ArgRow {
            name: "prev_trigger/next_trigger: disabled",
            kind: "bool",
            default: "false",
            description: "端かつ loop=false のとき true（native disabled + data-disabled、carousel.rs::prev_trigger/next_trigger）。",
        },
        ArgRow {
            name: "item: index, count, current",
            kind: "usize, usize, bool",
            default: "",
            description: "スライド位置・総数・現在表示中かどうか（aria-label/data-index/data-inview/data-current 生成元、carousel.rs::item）。",
        },
        ArgRow {
            name: "indicator: index, current",
            kind: "usize, bool",
            default: "",
            description: "インジケータの位置・現在位置かどうか（aria-label/data-index/aria-current/data-current 生成元、carousel.rs::indicator）。",
        },
        ArgRow {
            name: "Carousel::new: index, slide_count, loop_, orientation",
            kind: "usize, usize, bool, Orientation",
            default: "",
            description: "状態機械の初期値（carousel.rs::Carousel::new）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Vertical looping carousel",
            description: "縦方向レイアウト・loop 有効で末尾スライドを表示した例です（Demo は水平・非 loop）。",
            render: ex_carousel_vertical_loop,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-orientation/data-current/data-disabled 属性セレクタでトラック移動・インジケータ強調・trigger 減光のスタイルを当てる例です。",
            render: ex_carousel_custom_css,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"region\" / aria-roledescription=\"carousel\" / aria-label",
            description: "root に固定出力する（carousel.rs::root）。",
        },
        AriaRow {
            attribute: "role=\"group\" / aria-roledescription=\"slide\" / aria-label",
            description: "item に固定出力し、aria-label は \"{n} of {m}\" 形式（carousel.rs::item）。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "item_group に固定出力する（carousel.rs::item_group）。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "prev_trigger/next_trigger/indicator に呼び出し側指定または自動生成の説明文を出力する（carousel.rs::prev_trigger/next_trigger/indicator）。",
        },
        AriaRow {
            attribute: "aria-current=\"true\"",
            description: "indicator が current のときのみ出力する（carousel.rs::indicator。zag.js には存在しない超集合）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Item（/primitives/item/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/item.rs`（モジュール doc「`root` を
/// `a` として描画する経路」節）、`item::root`/`ItemRootProps` のシグネチャ。
fn ex_item_link() -> Node {
    item::root(
        ItemRootProps {
            href: Some("https://example.com/items/42"),
            external: false,
            variant: ItemVariant::Outline,
            ..Default::default()
        },
        vec![],
        vec![
            item::media(ItemMediaVariant::Icon, vec![], vec![text("→")]),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text("Browse Primitives")]),
                    item::description(vec![], vec![text("href 指定時は a として描画されます。")]),
                ],
            ),
        ],
    )
}

/// 一次情報: `crates/headless-ui/src/item.rs`（モジュール doc「`group` は
/// `role="group"`」節・「`separator` は `group` 内専用の水平固定パーツ」
/// 節）、`item::group`/`item::separator` のシグネチャ。
fn ex_item_group_with_separator() -> Node {
    item::group(
        "Recent items",
        vec![],
        vec![
            item::root(
                ItemRootProps::default(),
                vec![],
                vec![
                    item::media(ItemMediaVariant::Icon, vec![], vec![text("🔔")]),
                    item::content(
                        vec![],
                        vec![
                            item::title(vec![], vec![text("First item")]),
                            item::description(vec![], vec![text("A short description.")]),
                        ],
                    ),
                ],
            ),
            item::separator(vec![], vec![]),
            item::root(
                ItemRootProps::default(),
                vec![],
                vec![
                    item::media(ItemMediaVariant::Icon, vec![], vec![text("🔔")]),
                    item::content(vec![], vec![item::title(vec![], vec![text("Second item")])]),
                ],
            ),
        ],
    )
}

/// [`ex_item_group_with_separator`] の自前 CSS 実演（`data-scope`/
/// `data-part`/`data-variant`/`data-size` 属性セレクタのみを使う最小例。
/// headless-ui 自体はスタイルを持たない）。
const ITEM_CUSTOM_CSS_SNIPPET: &str = "[data-scope=\"item\"][data-part=\"root\"] {\n  display: flex;\n  gap: 0.75rem;\n  padding: 0.75rem;\n  border-radius: 0.5rem;\n}\n[data-scope=\"item\"][data-part=\"root\"][data-variant=\"outline\"] {\n  border: 1px solid currentColor;\n}\n[data-scope=\"item\"][data-part=\"separator\"] {\n  border-top: 1px solid currentColor;\n}\n";

fn ex_item_custom_css() -> Node {
    let node = item::group(
        "",
        vec![],
        vec![
            item::root(
                ItemRootProps {
                    variant: ItemVariant::Outline,
                    ..Default::default()
                },
                vec![],
                vec![item::content(
                    vec![],
                    vec![item::title(vec![], vec![text("First item")])],
                )],
            ),
            item::separator(vec![], vec![]),
            item::root(
                ItemRootProps::default(),
                vec![],
                vec![item::content(
                    vec![],
                    vec![item::title(vec![], vec![text("Second item")])],
                )],
            ),
        ],
    );
    wrap_example(
        "data-scope / data-part / data-variant / data-size 属性セレクタで行間の区切り・角丸・境界線を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            node,
            pre(vec![], vec![code(vec![], vec![text(ITEM_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

pub const ITEM: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "media（アイコン・画像・アバター）+ title/description + actions からなる汎用リスト行を表現する 10 anatomy パーツ（root/media/content/title/description/actions/header/footer/group/separator）を提供する（item.rs）。",
        "root は href 指定時に div ではなく a として描画し、external=true で target=\"_blank\"/rel=\"noopener noreferrer\" を不可分に付与する（reverse tabnabbing 対策、mod@link と同型の判断、item.rs「root を a として描画する経路」節）。",
        "group は role=\"group\"（shadcn/ui の role=\"list\" から意図的に差分化。a[href] が listitem ロールを持てないため list/listitem 対を成立させられない、item.rs「group は role=\"group\"」節）。",
        "参照実体は shadcn/ui の Item のみ（ark-ui・chakra-ui・Radix に対応部品なし、docs/design/component-coverage-map.md、参照軸 #2001）。状態機械を持たない静的部品であり wasm-full 側の配線は不要。",
    ],
    arguments: &[
        ArgRow {
            name: "root: props.href",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき a として描画し href へ固定付与する（item.rs）。",
        },
        ArgRow {
            name: "root: props.external",
            kind: "bool",
            default: "false",
            description: "href が Some のときのみ意味を持つ。true で target/rel を不可分に付与する（item.rs）。",
        },
        ArgRow {
            name: "root: props.variant",
            kind: "ItemVariant",
            default: "ItemVariant::Default",
            description: "見た目バリアント（default/outline/muted）。data-variant へ出力する（item.rs）。",
        },
        ArgRow {
            name: "root: props.size",
            kind: "ItemSize",
            default: "ItemSize::Default",
            description: "サイズバリアント（default/sm）。data-size へ出力する（item.rs）。",
        },
        ArgRow {
            name: "media: variant",
            kind: "ItemMediaVariant",
            default: "ItemMediaVariant::Default",
            description: "media パーツの見た目バリアント（default/icon/image）。data-variant へ出力する（item.rs）。",
        },
        ArgRow {
            name: "group: label",
            kind: "&str",
            default: "\"\"",
            description: "空でなければ aria-label へ出力する（item.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Link item",
            description: "href 指定で root が a として描画される例です。",
            render: ex_item_link,
        },
        ExampleEntry {
            title: "Group with separator",
            description: "group（role=\"group\"）の中に root と separator を並べる例です。",
            render: ex_item_group_with_separator,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-variant/data-size 属性セレクタでスタイルを当てる例です。",
            render: ex_item_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab / Enter",
        description: "root が a として描画されたときのみ、ネイティブ a[href] のフォーカス移動・起動に委ねる（item.rs「root を a として描画する経路」節）。div のときはキー操作を提供しない。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"group\" / aria-label",
            description: "group パーツに固定付与する。label が空でなければ aria-label を出力する（item.rs）。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-orientation=\"horizontal\"",
            description: "separator パーツに固定付与する（item.rs「separator は group 内専用の水平固定パーツ」節）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// JSON Tree View（/primitives/json-tree-view/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/json_tree_view.rs:1-93`（モジュール
/// doc、`tree_view`（#753）の派生であることの位置づけ・データモデル・
/// out-of-scope）、`:194-223`（`key`/`value`/`render_json` シグネチャ）、
/// `:347`（`expanded_to_depth`）。role/aria-* は `tree_view` 側
/// （`branch`/`item`/`tree`/`branch_content`）が出力し、本モジュール固有の
/// `key`/`value` パーツは role/aria-* を持たない。
fn ex_json_tree_view_collapsed_array() -> Node {
    let value = JsonValue::Array(vec![
        JsonValue::Number(1.0),
        JsonValue::Number(2.0),
        JsonValue::String("three".to_string()),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 0);
    json_tree_view::render_json(&tree, &value)
}

/// 自前 CSS の最小例（イシュー #1661、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定
/// エスケープを経由し、`crate::primitive_showcase` の専用スタイルシート
/// （`[data-scope=`/`[data-part=` を持たない契約、`tests/site_css_contract.rs`）
/// へは追加しない。json-tree-view スコープ（`key`/`colon`/`value`）と
/// tree-view スコープ（構造部）の 2 スコープが併存する事実を正直に示す
/// （両方に触れなければ見た目が完成しないことを明示する）。
const JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"tree-view\"][data-part=\"branch-content\"][hidden] {\n  \
  display: none;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-indent-guide\"] {\n  \
  width: 1rem;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"key\"] {\n  \
  color: #7c3aed;\n  font-family: monospace;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"colon\"] {\n  \
  color: #6b7280;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"string\"] {\n  \
  color: #059669;\n  font-family: monospace;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"number\"] {\n  \
  color: #2563eb;\n  font-family: monospace;\n\
}\n";

/// [`JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET`] を実演する例（Collapsed array と
/// 同じ `depth=0` の配列を使い回す）。
fn ex_json_tree_view_custom_css() -> Node {
    let value = JsonValue::Array(vec![
        JsonValue::Number(1.0),
        JsonValue::String("two".to_string()),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 1);
    wrap_example(
        "json-tree-view スコープ（key/colon/value）と tree-view スコープ（構造部）の両方に data-scope / data-part / data-kind 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            json_tree_view::render_json(&tree, &value),
            pre(
                vec![],
                vec![code(
                    vec![],
                    vec![text(JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET)],
                )],
            ),
        ],
    )
}

pub const JSON_TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "実装済み crate::tree_view（イシュー #753）の 12 anatomy パーツ・状態機械 TreeView を再利用し、JSON 固有の key/colon/value 3 パーツを branch-text/item-text の内側へ入れ子にして追加する（json_tree_view.rs:1-35、colon はイシュー #1661 で ark-ui/zag 突合により追加）。",
        "Object は HashMap ではなく挿入順を保持する Vec<(String, JsonValue)> で表現し、render_json の出力が決定的（バイト単位で一致）である（json_tree_view.rs:31-37,118-136）。",
        "ノード識別子に RFC 6901 JSON Pointer を用い、key に `/`/`~` を含むデータでも一意性が壊れない（json_tree_view.rs:39-46,180-192）。",
        "role/aria-*（role=\"tree\"/\"treeitem\"/\"group\"、aria-expanded/aria-selected/aria-level/aria-posinset/aria-setsize）はすべて crate::tree_view のパーツ関数から継承する（json_tree_view.rs:16-24）。",
    ],
    arguments: &[
        ArgRow {
            name: "render_json: tree",
            kind: "&TreeView",
            default: "",
            description: "展開・選択状態（json_tree_view.rs:221-223）。",
        },
        ArgRow {
            name: "render_json: root",
            kind: "&JsonValue",
            default: "",
            description: "描画対象の JSON 風データ木（json_tree_view.rs:221-223）。",
        },
        ArgRow {
            name: "expanded_to_depth: depth",
            kind: "usize",
            default: "",
            description: "この深さまでのブランチを展開済みにした TreeView を生成する（json_tree_view.rs:347）。",
        },
        ArgRow {
            name: "value: kind",
            kind: "&'static str",
            default: "",
            description: "JsonValue::kind() が返す固定語彙（\"null\"/\"boolean\"/\"number\"/\"string\"/\"array\"/\"object\"）のみを受け取る data-kind 属性値（イシュー #1661 で \"bool\" から \"boolean\" へ変更、json_tree_view.rs:138-153,201-209）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Collapsed array",
            description: "depth=0（全ブランチ折りたたみ）で描画した配列の例です（Demo は depth=2 まで展開済みのオブジェクト）。",
            render: ex_json_tree_view_collapsed_array,
        },
        ExampleEntry {
            title: "Custom CSS for key/colon/value",
            description: "本モジュールが型別配色を提供しない headless 部品のため、キー・区切り・値の 3 パーツへ自前 CSS を当てる最小例です（json-tree-view スコープのみを対象とし、構造部の tree-view スコープには触れません）。",
            render: ex_json_tree_view_custom_css,
        },
    ],
    // イシュー #1661: tree-view 構造部（branch/branch-control/item 等）は
    // json_tree_view でも data-scope="tree-view" のまま出力されるため、
    // `fandhe-frontend-wasm-full` の `keynav.rs::handle_tree_view_keydown`
    // （`TREE_VIEW_TREE_SELECTOR`/`TREE_VIEW_ITEM_SELECTOR` で
    // `[data-scope="tree-view"]` を探索）がそのまま適用される（継承元:
    // `crates/wasm-full/src/keynav.rs`、イシュー #1072）。
    keyboard: &[
        KeyRow {
            key: "ArrowDown / ArrowUp",
            description: "可視かつ disabled でない treeitem 間で 1 件ずつフォーカス移動する（折りたたみ中の子孫はスキップ、非循環）。",
        },
        KeyRow {
            key: "ArrowRight",
            description: "閉じたブランチは展開する。開いたブランチは最初の子へフォーカス移動する。葉ノードでは no-op。",
        },
        KeyRow {
            key: "ArrowLeft",
            description: "開いたブランチは折りたたむ。それ以外（葉ノード・閉じたブランチ）は親ブランチへフォーカス移動する（ルート直下では no-op）。",
        },
        KeyRow {
            key: "Home / End",
            description: "可視かつ disabled でない最初/最後の treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "葉ノードは選択（select）、ブランチは開閉（toggle）を発火する。",
        },
        KeyRow {
            key: "印字可能文字",
            description: "typeahead: 直近の入力から一致する treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Escape",
            description: "typeahead バッファをリセットするのみ（TreeView は常時展開のツリーであり閉じる操作を持たない）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"tree\" / role=\"treeitem\" / role=\"group\"",
            description: "crate::tree_view の tree/branch・item/branch_content から継承する（tree_view.rs:125-259）。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-selected / aria-level / aria-posinset / aria-setsize",
            description: "crate::tree_view の branch/item から継承する（tree_view.rs:159-303）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Marker（/primitives/marker/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/marker.rs`（モジュール doc、区切り線は
/// headless で描かない・`data-variant`/`data-tone` 語彙・会話系 4 部品の
/// 共通語彙への不追随・アクセシビリティ）、`root`/`icon`/`content`
/// シグネチャ。非テスト行で `role`/`aria-*` の出力は `icon` の
/// `aria-hidden="true"` のみ。
fn ex_marker_note_with_icon() -> Node {
    marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Note,
            tone: MarkerTone::Info,
        },
        vec![],
        vec![
            marker::icon(vec![], vec![text("i")]),
            marker::content(vec![], vec![text("Explored 4 files")]),
        ],
    )
}

/// 中央ラベル + 左右の線（shadcn `separator` variant 相当）の区切りを表す
/// 例。線要素は headless 層が出力しない（モジュール doc「区切り線は
/// headless で描かない」参照）ため、Demo でも線を描かない。
fn ex_marker_label_separator() -> Node {
    marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Label,
            tone: MarkerTone::Neutral,
        },
        vec![],
        vec![marker::content(vec![], vec![text("Today")])],
    )
}

/// `data-tone="warning"` の行境界線（shadcn `border` variant 相当）を表す
/// 例。
fn ex_marker_divider_tone_warning() -> Node {
    marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Divider,
            tone: MarkerTone::Warning,
        },
        vec![],
        vec![marker::content(
            vec![],
            vec![text("Context window 80% full")],
        )],
    )
}

/// 自前 CSS の最小例（`ATTACHMENT_CUSTOM_CSS_SNIPPET` と同型のパターン）。
/// CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const MARKER_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"marker\"][data-part=\"root\"] {\n  \
  display: flex;\n  align-items: center;\n  gap: 0.375rem;\n  font-size: 0.8125rem;\n  color: #6b7280;\n\
}\n\
[data-scope=\"marker\"][data-part=\"root\"][data-tone=\"warning\"] {\n  \
  color: #b45309;\n\
}\n\
[data-scope=\"marker\"][data-part=\"root\"][data-tone=\"danger\"] {\n  \
  color: #b91c1c;\n\
}\n\
[data-scope=\"marker\"][data-part=\"root\"][data-variant=\"label\"] {\n  \
  justify-content: center;\n\
}\n";

fn ex_marker_custom_css() -> Node {
    let demo = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Note,
            tone: MarkerTone::Neutral,
        },
        vec![],
        vec![marker::content(
            vec![],
            vec![text("Conversation compacted")],
        )],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-variant / data-tone 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。区切り線（divider/label variant）の描画自体は本サンプルの範囲外です。",
        vec![
            demo,
            pre(vec![], vec![code(vec![], vec![text(MARKER_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

pub const MARKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "会話スレッド内のインライン注記行（システム注記・日付等の区切り・ラベル付きセパレータ）を表現する 3 anatomy パーツ（root/icon/content）を提供する（marker.rs）。状態機械を持たない静的部品であり wasm-full 側の配線は不要。",
        "root へ data-variant（note/divider/label）・data-tone（neutral/info/warning/danger）を付与する。会話系 4 部品（message/bubble/attachment/marker）の共通語彙（data-role/data-align）は意図的に持たない（marker.rs「会話系 4 部品の共通語彙への不追随」節）。",
        "divider/label variant の区切り線（水平線・左右の線）は headless 層で一切出力しない。fandhe-frontend-pre-styled-ui 側が separator パーツの再利用または CSS で描く契約とする（marker.rs「区切り線は headless で描かない」節）。data-tone の値語彙は fandhe-frontend-pre-styled-ui recipe::ColorPalette の同名 4 値の部分集合であり新語彙を作らない。",
    ],
    arguments: &[
        ArgRow {
            name: "root: props.variant",
            kind: "MarkerVariant",
            default: "MarkerVariant::Note",
            description: "表示形態（note/divider/label）。data-variant へ出力する（marker.rs）。",
        },
        ArgRow {
            name: "root: props.tone",
            kind: "MarkerTone",
            default: "MarkerTone::Neutral",
            description: "色調（neutral/info/warning/danger）。data-tone へ出力する（marker.rs）。",
        },
        ArgRow {
            name: "root/icon/content: attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "呼び出し側の追加属性。予約キー（data-variant/data-tone/aria-hidden）は大小文字無視でなりすまし除去される（marker.rs）。",
        },
        ArgRow {
            name: "root/icon/content: children",
            kind: "Vec<Node>",
            default: "",
            description: "子ノード。content は注記の本文テキストを受け取るスロット（marker.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Note with icon",
            description: "インライン注記（アイコン + 本文）の例です。",
            render: ex_marker_note_with_icon,
        },
        ExampleEntry {
            title: "Label separator",
            description: "中央ラベル形式の区切りを表す例です（線自体は範囲外）。",
            render: ex_marker_label_separator,
        },
        ExampleEntry {
            title: "Divider, tone warning",
            description: "警告色の行境界線を表す例です（線自体は範囲外）。",
            render: ex_marker_divider_tone_warning,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-variant/data-tone 属性セレクタでスタイルを当てる例です。",
            render: ex_marker_custom_css,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-hidden=\"true\"（icon、固定）",
            description: "装飾スロットとして常に付与し、呼び出し側の aria-hidden=\"false\" 偽装は除去する（marker.rs「アクセシビリティ」節）。",
        },
        AriaRow {
            attribute: "role（root、非固定）",
            description: "静的注記に割り込み通知は不要なため root へ role を固定付与しない。ストリーミング中の注記へ role=\"status\" を付与する運用は呼び出し側の attrs で可能（marker.rs「アクセシビリティ」節）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Message（/primitives/message/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/message.rs`（モジュール doc「会話系
/// 4 部品の共通語彙」「`role="listitem"`/`role="list"`」「`aria-live`/
/// `aria-busy` を付けない理由」）。基本の user/assistant 対を実演する。
fn ex_message_basic_pair() -> Node {
    message::group(
        "Conversation",
        vec![],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::User,
                    align: MessageAlign::End,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("How do I center a div?")],
                )],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    align: MessageAlign::Start,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("Use display: flex + place-items: center.")],
                )],
            ),
        ],
    )
}

/// 同じ発言者（assistant）の連続発言を 1 つの [`message::group`] へまとめる
/// 例（モジュール doc「`group` は連続発言のまとめ」参照。先頭以外の avatar
/// 省略は CSS 側の責務のため、本例では avatar パーツ自体を省略する）。
fn ex_message_consecutive_group() -> Node {
    message::group(
        "Assistant reply (2 parts)",
        vec![],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("First, install the crate.")],
                )],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    ..Default::default()
                },
                vec![],
                vec![message::content(vec![], vec![text("Then call render().")])],
            ),
        ],
    )
}

/// 自前 CSS の最小例（`data-scope`/`data-part`/`data-role`/`data-align`
/// 属性セレクタのみを使う。headless-ui 自体はスタイルを持たない）。
const MESSAGE_CUSTOM_CSS_SNIPPET: &str = "[data-scope=\"message\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  max-width: 32rem;\n}\n[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] {\n  margin-left: auto;\n}\n[data-scope=\"message\"][data-part=\"content\"] {\n  padding: 0.5rem 0.75rem;\n  border-radius: 0.75rem;\n}\n";

fn ex_message_custom_css() -> Node {
    let node = message::group(
        "Conversation",
        vec![],
        vec![message::root(
            MessageRootProps {
                role: MessageRole::User,
                align: MessageAlign::End,
                ..Default::default()
            },
            vec![],
            vec![message::content(
                vec![],
                vec![text("Styled with plain CSS.")],
            )],
        )],
    );
    wrap_example(
        "data-scope / data-part / data-role / data-align 属性セレクタで吹き出しの位置・余白を当てる最小例です。headless-ui 自体はスタイルを持ちません。root（role=\"listitem\"）は message::group（role=\"list\"）でラップし required context を満たします。",
        vec![
            node,
            pre(
                vec![],
                vec![code(vec![], vec![text(MESSAGE_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const MESSAGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AI チャット UI の「会話 1 発言」を表現する 6 anatomy パーツ（root/avatar/header/content/footer/group）を提供する（message.rs）。状態機械を持たない静的部品であり wasm-full 側の配線は不要。",
        "会話系 4 部品（message/bubble/attachment/marker）が共有する data-role（user/assistant/system）・data-align（start/end、data-role から独立した軸）・data-loading/data-error（存在属性）を本モジュールが最初に確定する（message.rs「会話系 4 部品の共通語彙」節）。",
        "root は role=\"listitem\" を固定付与し、group は required context を満たす role=\"list\" + 任意 aria-label を固定付与する（message.rs「role=\"listitem\"/role=\"list\"」節）。",
        "aria-live/aria-busy は付けない（ストリーミング通知・応答待ちの読み上げはアプリ責務、message.rs「aria-live/aria-busy を付けない理由」節）。参照実体は shadcn/ui の Message のみ（ark-ui・chakra-ui・Radix に対応部品なし、docs/design/component-coverage-map.md §12.1、参照軸 #2001）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: props.role",
            kind: "MessageRole",
            default: "MessageRole::User",
            description: "発言者の役割（user/assistant/system）。data-role へ出力する（message.rs）。",
        },
        ArgRow {
            name: "root: props.align",
            kind: "MessageAlign",
            default: "MessageAlign::Start",
            description: "root の水平整列（start/end）。data-role から独立した軸。data-align へ出力する（message.rs）。",
        },
        ArgRow {
            name: "root: props.loading",
            kind: "bool",
            default: "false",
            description: "true のとき data-loading 存在属性を付与する（応答待ちの表示のみ、message.rs）。",
        },
        ArgRow {
            name: "root: props.error",
            kind: "bool",
            default: "false",
            description: "true のとき data-error 存在属性を付与する（送信失敗の表示のみ、message.rs）。",
        },
        ArgRow {
            name: "group: label",
            kind: "&str",
            default: "\"\"",
            description: "空でなければ aria-label へ出力する（message.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Basic user/assistant pair",
            description: "user（右寄せ）と assistant（左寄せ）の 1 往復を group でまとめる基本例です。",
            render: ex_message_basic_pair,
        },
        ExampleEntry {
            title: "Consecutive messages in one group",
            description: "同じ発言者（assistant）の連続発言を 1 つの group へまとめる例です。",
            render: ex_message_consecutive_group,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-role/data-align 属性セレクタでスタイルを当てる例です。",
            render: ex_message_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "(なし)",
        description: "root/avatar/header/content/footer/group はいずれもキー操作を提供しない静的コンテナである（message.rs。子要素として置くインタラクティブ要素のキー操作はその要素自体に委ねる）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role=\"listitem\"",
            description: "root パーツに固定付与する（message.rs「role=\"listitem\"/role=\"list\"」節）。",
        },
        AriaRow {
            attribute: "role=\"list\" / aria-label",
            description: "group パーツに固定付与する。label が空でなければ aria-label を出力する（message.rs）。",
        },
        AriaRow {
            attribute: "(aria-live / aria-busy は付与しない)",
            description: "ストリーミング通知・応答待ちの読み上げはアプリ責務のため本モジュールは付与しない（message.rs「aria-live/aria-busy を付けない理由」節）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Scroll Area（/primitives/scroll-area/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/scroll_area.rs:1-90`（モジュール doc、
/// スコープ外・参考サイトとの突合〔イシュー #1662〕）、`:95-183`
/// （予約キー定数・`drop_reserved`・`root`/`viewport`/`content`/
/// `scrollbar`/`thumb`/`corner` シグネチャと `tabindex`/`aria-hidden` 出力）。
fn ex_scroll_area_horizontal() -> Node {
    scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(
                    vec![],
                    vec![text("Wide scrollable content…")],
                )],
            ),
            scroll_area::scrollbar(
                Orientation::Horizontal,
                vec![],
                vec![scroll_area::thumb(Orientation::Horizontal, vec![], vec![])],
            ),
            scroll_area::corner(vec![], vec![]),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1662、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープ
/// を経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。
///
/// `scrollbar`/`thumb`/`corner` パーツは JS によるスクロール位置追従・
/// drag 操作を実装していない静的マークアップ（`crates/headless-ui/
/// src/scroll_area.rs` モジュール doc「スコープ外」節）であり、これらを
/// 表示したままネイティブスクロールバーを隠すと、スクロール位置表示・
/// マウス操作の両方を失う（codex-review P1 指摘、イシュー #1662）。
/// `crates/pre-styled-ui/src/scroll_area.rs` の実装契約と同じく、
/// `scrollbar`/`thumb`/`corner` は非表示のまま維持し、ネイティブ
/// スクロールバー自体を標準プロパティ（`scrollbar-width`/
/// `scrollbar-color`）+ `::-webkit-scrollbar` 系疑似要素で装飾する
/// 最小構成を示す。
const SCROLL_AREA_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  \
  position: relative;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"] {\n  \
  overflow: auto;\n  height: 8rem;\n  \
  scrollbar-width: thin;\n  scrollbar-color: #9ca3af transparent;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar {\n  \
  width: 0.5rem;\n  height: 0.5rem;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-track {\n  \
  background: transparent;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-thumb {\n  \
  background: #9ca3af;\n  border-radius: 9999px;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"scrollbar\"],\n\
[data-scope=\"scroll-area\"][data-part=\"thumb\"],\n\
[data-scope=\"scroll-area\"][data-part=\"corner\"] {\n  \
  display: none;\n\
}\n";

fn ex_scroll_area_custom_css() -> Node {
    let items: Vec<Node> = (1..=6)
        .map(|n| fandhe_frontend_core::li(vec![], vec![text(format!("Item {n}"))]))
        .collect();
    let demo = scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(
                    vec![],
                    vec![fandhe_frontend_core::ul(vec![], items)],
                )],
            ),
            scroll_area::scrollbar(
                Orientation::Vertical,
                vec![],
                vec![scroll_area::thumb(Orientation::Vertical, vec![], vec![])],
            ),
            scroll_area::corner(vec![], vec![]),
        ],
    );
    wrap_example(
        "利用者が data-scope/data-part 属性セレクタでネイティブスクロールバーを装飾する最小例です（scrollbar/thumb/corner パーツは静的マークアップのため非表示のまま維持します）。headless-ui 自体はスタイルを持ちません。",
        vec![
            demo,
            pre(
                vec![],
                vec![code(vec![], vec![text(SCROLL_AREA_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const SCROLL_AREA: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Viewport / Content / Scrollbar / Thumb / Corner の 6 anatomy パーツを提供し、CSS overflow を主体とするスクロール領域を表現する（scroll_area.rs:1-10）。ark-ui/Zag.js の anatomy（6 パーツ）と完全一致（イシュー #1662 突合）。",
        "viewport は WAI 慣行（矢印キー/Page キーでフォーカス済み要素をスクロールできる）に従い tabindex=\"0\" を固定出力する（scroll_area.rs:139-146）。SSR では overflow の有無を判定できないため常時付与する安全側の設計（WCAG 2.1.1）。",
        "scrollbar/corner はネイティブスクロールバーと意味が重複する装飾要素のため aria-hidden=\"true\" を固定出力する（scroll_area.rs:157-183）。",
        "呼び出し側 attrs による固定属性（tabindex/aria-hidden/data-orientation）のなりすまし・重複出力を drop_reserved で除去する（scroll_area.rs:116-127、イシュー #1662）。",
        "ark-ui/Zag.js・Radix Primitives・chakra-ui・Radix Themes の 4 参照サイトと突合済み（イシュー #1662）。anatomy は増減なし。参照側の data-overflow-*/data-at-*/data-hover/data-scrolling/data-dragging・Radix の data-state（いずれも DOM 計測・ポインタ操作由来）は SSR で真の値を決定できないため意図的に非採用（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。Zag.js の role=\"presentation\" も、viewport が tabindex=\"0\" でフォーカス可能なため WAI-ARIA 1.2 §5.4 により無視される値として非採用。",
    ],
    arguments: &[
        ArgRow {
            name: "scrollbar/thumb: orientation",
            kind: "Orientation",
            default: "",
            description: "data-orientation 属性値（scroll_area.rs:157-176）。値語彙は vertical/horizontal の 2 値のみ。",
        },
        ArgRow {
            name: "viewport: attrs, children",
            kind: "Vec<(&str, &str)>, Vec<Node>",
            default: "vec![], vec![]",
            description: "tabindex=\"0\" は固定出力のため呼び出し側から指定する引数ではない（drop_reserved が除去、scroll_area.rs:139-146）。読み上げ名が必要な場合は aria-label/aria-labelledby を attrs へ付与することを推奨する。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Horizontal scroll area",
            description: "水平方向のスクロールバー配置例です（Demo は垂直・水平の両軸 + corner）。",
            render: ex_scroll_area_horizontal,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-orientation 属性セレクタでスクロールバー表現を当てる例です。",
            render: ex_scroll_area_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "viewport は tabindex=\"0\" を固定出力するため通常の Tab 順序に含まれる（scroll_area.rs:139-146）。",
        },
        KeyRow {
            key: "↑ / ↓ / ← / →",
            description: "フォーカス済み viewport 上でのネイティブスクロール（本モジュールは独自のキーイベントリスナを持たず、ブラウザ既定動作に委ねる。Radix Primitives docs も同じ方針を明記、イシュー #1662 突合）。",
        },
        KeyRow {
            key: "PageUp / PageDown / Home / End",
            description: "ネイティブスクロールのページ単位・端への移動（ブラウザ既定動作、独自リスナなし）。",
        },
        KeyRow {
            key: "Space / Shift+Space",
            description: "ネイティブスクロールの前方/後方ページ送り（ブラウザ既定動作、独自リスナなし）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "scrollbar/corner に固定出力する（scroll_area.rs:157-183）。ネイティブスクロールバーとの意味重複を明示する目的（両参照サイトにはない本実装独自の付与、イシュー #1662）。",
        },
        AriaRow {
            attribute: "(該当なし)",
            description: "root/viewport/content は role/aria-* を出力しない。Zag.js の role=\"presentation\" は、viewport が tabindex=\"0\" でフォーカス可能なため WAI-ARIA 1.2 §5.4 により UA に無視される値であり、Radix Primitives（role 非付与）に整合する形で追加していない（イシュー #1662 突合）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Skip Nav（/primitives/skip-nav/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/skip_nav.rs:1-63`（モジュール doc、
/// href の構成・契約属性の除去）、`:75-108`（`DEFAULT_ID`/`link`/`content`
/// シグネチャ）。role/aria-* の出力は非テスト行で 0 件。
fn ex_skip_nav_custom_id() -> Node {
    div(
        vec![],
        vec![
            skip_nav::link(
                "primitives-main",
                vec![],
                vec![text("Skip to primitives content")],
            ),
            skip_nav::content(
                "primitives-main",
                vec![],
                vec![text("Primitives content starts here.")],
            ),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1663、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープ
/// を経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。headless-ui 自体はスタイルを持たないため、chakra-ui の
/// `SkipNavContent` inline `style={{ outline: 0 }}`（headless 層では
/// `docs/policy/intentional-non-adoption.md` §3.25 規則 2 により非採用、
/// イシュー #1663 突合結果）に相当する調整も含め、利用者側 CSS で示す。
const SKIP_NAV_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"skip-nav\"][data-part=\"link\"] {\n  \
  position: absolute;\n  width: 1px;\n  height: 1px;\n  margin: -1px;\n  \
  overflow: hidden;\n  clip: rect(0 0 0 0);\n  border: 0;\n\
}\n\
[data-scope=\"skip-nav\"][data-part=\"link\"]:focus-visible {\n  \
  clip: auto;\n  width: auto;\n  height: auto;\n  position: fixed;\n  \
  top: 1rem;\n  left: 1rem;\n  padding: 0.5rem 0.75rem;\n  \
  background: #fff;\n  outline: 2px solid #2563eb;\n\
}\n\
[data-scope=\"skip-nav\"][data-part=\"content\"] {\n  \
  outline: none;\n\
}\n";

/// [`SKIP_NAV_CUSTOM_CSS_SNIPPET`] を実演する例（`AVATAR_CUSTOM_CSS_SNIPPET`
/// の実演関数と同型）。id はページ骨格の `DEFAULT_ID`（`"fandhe-skip-nav"`）
/// や他の例の `"primitives-main"` と衝突しない第 3 の値
/// （`"primitives-custom-css-target"`）を使い、`href` の解決先が曖昧に
/// ならないようにする。
fn ex_skip_nav_custom_css() -> Node {
    div(
        vec![],
        vec![
            skip_nav::link(
                "primitives-custom-css-target",
                vec![],
                vec![text("Skip to content (custom CSS)")],
            ),
            skip_nav::content(
                "primitives-custom-css-target",
                vec![],
                vec![text("Content reachable via the styled skip link.")],
            ),
            pre(
                vec![],
                vec![code(vec![], vec![text(SKIP_NAV_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const SKIP_NAV: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "link / content の 2 anatomy パーツで WCAG 2.1 SC 2.4.1 Bypass Blocks を実現する（skip_nav.rs:1-14）。",
        "link は任意の URL を呼び出し側から受け取らず、常に `#<id>`（フラグメントのみ）を内部で組み立てるためスキーム注入経路を持たない（skip_nav.rs:16-22,77-91）。",
        "content は id/tabindex を、link は href を、それぞれ呼び出し側 attrs に同名キー（大文字小文字を無視）があっても fail-closed に除去してから合成する（skip_nav.rs:24-32,82-108）。",
        "DEFAULT_ID 定数（\"fandhe-skip-nav\"）を提供し、ページ全体に 1 個だけ配置する典型利用を想定する（skip_nav.rs:71-75）。",
        "chakra-ui（唯一の参照軸、Ark UI は該当ページ 404・Radix Primitives / Radix Themes に該当部品なし）と突合済み（イシュー #1663）。anatomy / data-* / ARIA は増減なし。chakra-ui の SkipNavContent が出力する inline outline: 0 は装飾のため headless 層では非採用（`docs/policy/intentional-non-adoption.md` §3.25 規則 2、Themes 版が CSS で担当）。",
    ],
    arguments: &[
        ArgRow {
            name: "link: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "スキップ先 id。href=\"#<id>\" として出力する（skip_nav.rs:81-91）。",
        },
        ArgRow {
            name: "content: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "id 属性値。link の href と対にする（skip_nav.rs:100-108）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Custom id",
            description: "DEFAULT_ID ではなく呼び出し側指定の id（\"primitives-main\"）を使う例です。",
            render: ex_skip_nav_custom_id,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "利用者が data-scope/data-part 属性セレクタで見た目を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_skip_nav_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab (link)",
            description: "独自のキーリスナを持たず、link はページ先頭に置かれた通常の <a> としてブラウザの既定 Tab 順序に従いフォーカス可能になる（chakra-ui docs の「できるだけ DOM 先頭に置く」前提と一致、イシュー #1663 突合結果）。",
        },
        KeyRow {
            key: "Enter (link, focused)",
            description: "ネイティブのフラグメント遷移で href=\"#<id>\" へジャンプする。独自のキーイベントリスナは持たずブラウザ既定動作に委ねる（skip_nav.rs:16-22、イシュー #1663 突合結果）。",
        },
        KeyRow {
            key: "Tab (content, after link activation)",
            description: "content は tabindex=\"-1\" を固定出力するため通常の Tab 順序には含まれず、link クリック後のプログラム的フォーカス移動のみを許可する（skip_nav.rs:93-108）。",
        },
    ],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "link/content は固有の role/aria-* を出力しない（skip_nav.rs 全文の非テスト行で role/aria- grep 0 件）。chakra-ui も role/aria-* を付与せず整合する（イシュー #1663 突合結果）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Splitter（/primitives/splitter/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/splitter.rs:1-76`（モジュール doc、
/// `aria-orientation` の向き・決定的な正規化・スコープ外）、`:210-287`
/// （`root`/`panel`/`resize_trigger`/`resize_trigger_indicator`
/// シグネチャと role/aria-*/tabindex 出力）、`:352`（`Splitter::new`）。
fn ex_splitter_vertical_three_panels() -> Node {
    let orientation = Orientation::Vertical;
    splitter::root(
        orientation,
        false,
        vec![],
        vec![
            splitter::panel("sp-top", 0, orientation, vec![], vec![text("Top")]),
            splitter::resize_trigger(
                orientation,
                "0",
                "100",
                "33",
                "sp-top",
                "sp-middle",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel("sp-middle", 1, orientation, vec![], vec![text("Middle")]),
            splitter::resize_trigger(
                orientation,
                "0",
                "100",
                "66",
                "sp-middle",
                "sp-bottom",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel("sp-bottom", 2, orientation, vec![], vec![text("Bottom")]),
        ],
    )
}

pub const SPLITTER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Panel / ResizeTrigger / ResizeTriggerIndicator の 4 anatomy パーツと、パネルサイズ状態機械 Splitter を提供する（splitter.rs:1-11）。",
        "resize_trigger は role=\"separator\" + aria-valuemin/aria-valuemax/aria-valuenow + aria-orientation（パネルレイアウトと逆向き）+ aria-controls（隣接 2 パネルの id）+ data-id（\"<leading>:<trailing>\"）を常に出力する。",
        "panel は data-index（パネル序数）・data-id（id の写し）を出力する（イシュー #1664 で ark-ui docs の data-* 表と突合し追加）。",
        "disabled=true のとき tabindex=\"-1\" + aria-disabled、false のとき tabindex=\"0\" を出力する。",
        "SplitterAction::IncrementLarge/DecrementLarge（zag.js keyboardResizeBy 既定値 ×10 相当）を状態機械として提供する（イシュー #1664、DOM 配線は未実装）。",
        "呼び出し側 attrs からの role/aria-*/tabindex/data-*/id のなりすまし・重複出力を drop_reserved で除去する（イシュー #1664）。",
        "Splitter::new はパネル数 2 未満・非有限値・制約矛盾等の実現不能構成を既定（2 パネル 50/50）へ fail-closed にフォールバックする（splitter.rs:33-45,194-208）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: orientation, disabled",
            kind: "Orientation, bool",
            default: "",
            description: "パネルレイアウトの向きと無効状態。",
        },
        ArgRow {
            name: "panel: id, index",
            kind: "&str, usize",
            default: "",
            description: "resize_trigger の aria-controls 先となる id（必須）と data-index として出力するパネル序数（イシュー #1664 で index を追加、破壊的変更）。",
        },
        ArgRow {
            name: "resize_trigger: min, max, now, leading_id, trailing_id, disabled",
            kind: "&str, &str, &str, &str, &str, bool",
            default: "",
            description: "aria-valuemin/aria-valuemax/aria-valuenow と、隣接 2 パネルの id（aria-controls/data-id へ出力）・tabindex 切替の元（イシュー #1664 で controls 単一引数を leading_id/trailing_id へ置換、破壊的変更）。",
        },
        ArgRow {
            name: "Splitter::new: panels, orientation",
            kind: "&[PanelSpec], Orientation",
            default: "",
            description: "パネル構成（size/min/max）と向き。fail-closed に正規化する（splitter.rs:352）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical 3-panel splitter",
        description: "縦方向レイアウトで 3 パネル・2 セパレータを組んだ例です（Demo は水平 2 パネル）。",
        render: ex_splitter_vertical_three_panels,
    }],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "resize_trigger は disabled=false のとき tabindex=\"0\" で通常の Tab 順序に入り、disabled=true のとき tabindex=\"-1\" で除外される。",
        },
        KeyRow {
            key: "Arrow（軸別）",
            description: "SplitterAction::Increment/Decrement（ステップ 1%）として状態遷移する。イシュー #1074 で fandhe-frontend-wasm-full の splitter モジュールが DOM keydown 配線済み。",
        },
        KeyRow {
            key: "Home / End",
            description: "SplitterAction::SetToMin/SetToMax として状態遷移する。DOM 配線済み（イシュー #1074）。",
        },
        KeyRow {
            key: "Shift+Arrow",
            description: "SplitterAction::IncrementLarge/DecrementLarge（ステップ 10%）として状態機械のみ提供する。fandhe-frontend-wasm-full の DOM 配線は未実装（イシュー #1664 時点、別 Issue 起票対象）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"separator\"",
            description: "resize_trigger に固定出力する。",
        },
        AriaRow {
            attribute: "aria-valuemin / aria-valuemax / aria-valuenow",
            description: "先行パネルのサイズ%（有限性検証・クランプ済み）を出力する。",
        },
        AriaRow {
            attribute: "aria-orientation",
            description: "セパレータ自体の向き（パネルレイアウトと逆向き、WAI-ARIA APG 準拠。zag.js の非反転出力とは非同値、イシュー #1664 参照突合）。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "隣接 2 パネルの id（\"<leading> <trailing>\"、イシュー #1664 で先行パネルのみから拡張）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "disabled=true のときのみ出力する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Steps（/primitives/steps/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/steps.rs:1-76`（モジュール doc、
/// 状態モデル・out-of-scope）、`:153-388`（`Steps::new`/`root`/`list`/
/// `item`/`trigger`/`indicator`/`separator`/`content`/`completed_content`/
/// `prev_trigger`/`next_trigger` シグネチャと role/aria-* 出力）。
fn ex_steps_completed() -> Node {
    let steps = Steps::new(2, 2, Orientation::Horizontal);
    steps.root(
        vec![],
        vec![
            steps.list(
                vec![],
                vec![
                    steps.item(
                        0,
                        vec![],
                        vec![steps.trigger(
                            0,
                            vec![],
                            vec![steps.indicator(0, vec![], vec![text("1")])],
                        )],
                    ),
                    steps.separator(0, vec![], vec![]),
                    steps.item(
                        1,
                        vec![],
                        vec![steps.trigger(
                            1,
                            vec![],
                            vec![steps.indicator(1, vec![], vec![text("2")])],
                        )],
                    ),
                ],
            ),
            steps.completed_content(vec![], vec![text("All steps completed.")]),
        ],
    )
}

/// 一次情報: `crates/headless-ui/src/steps.rs`（`Steps::trigger`/
/// `Steps::content`/`Steps::completed_content` の `data-orientation` 加算、
/// イシュー #1665 参照突合）。
fn ex_steps_vertical() -> Node {
    let steps = Steps::new(3, 1, Orientation::Vertical);
    steps.root(
        vec![],
        vec![
            steps.list(
                vec![],
                vec![
                    steps.item(
                        0,
                        vec![],
                        vec![steps.trigger(
                            0,
                            vec![],
                            vec![steps.indicator(0, vec![], vec![text("1")])],
                        )],
                    ),
                    steps.separator(0, vec![], vec![]),
                    steps.item(
                        1,
                        vec![],
                        vec![steps.trigger(
                            1,
                            vec![],
                            vec![steps.indicator(1, vec![], vec![text("2")])],
                        )],
                    ),
                    steps.separator(1, vec![], vec![]),
                    steps.item(
                        2,
                        vec![],
                        vec![steps.trigger(
                            2,
                            vec![],
                            vec![steps.indicator(2, vec![], vec![text("3")])],
                        )],
                    ),
                ],
            ),
            steps.progress(vec![("aria-label", "Steps progress")], vec![]),
            steps.content(1, vec![], vec![text("Step 2 content")]),
        ],
    )
}

pub const STEPS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / List / Item / Trigger / Indicator / Separator / Content / CompletedContent / PrevTrigger / NextTrigger / Progress の 11 anatomy パーツを提供する（steps.rs:1-11。Progress はイシュー #1665 の参照突合で新設）。",
        "count/step から complete/current/incomplete の 3 状態を導出し、data-state・data-complete/-current/-incomplete へ一元反映する（steps.rs:24-34,89-121）。",
        "current な item の trigger のみ aria-current=\"step\" を付与する（steps.rs:58-61,249-269）。",
        "trigger/content/completed-content は data-orientation を出力する（イシュー #1665 参照突合で加算。呼び出し側 CSS が単独パートのみでレイアウト条件化できるようにする）。",
        "separator は role=\"separator\" + aria-hidden=\"true\" で装飾要素として a11y ツリーから除外する（steps.rs:288-310）。",
        "prev_trigger/next_trigger は境界（step==0/step==count）で disabled + data-disabled 属性を付与する（イシュー #1665 で data-disabled を加算。本リポジトリの disabled 語彙統一）。",
        "progress は role=\"progressbar\" + aria-valuemin/aria-valuemax/aria-valuenow/aria-valuetext を出力し、percent==100（全 step 完了）のときのみ data-complete を付与する（イシュー #1665 新設）。",
        "呼び出し側 attrs からの role/aria-*/data-*/type/hidden のなりすまし・重複出力を drop_reserved で除去する（イシュー #1665、toolbar/splitter と同型）。",
    ],
    arguments: &[
        ArgRow {
            name: "Steps::new: count, step, orientation",
            kind: "usize, usize, Orientation",
            default: "",
            description: "全 step 数・現在位置・向き。fail-closed に正規化する（steps.rs:123-133,164-171）。",
        },
        ArgRow {
            name: "item/trigger/indicator/separator/content: index",
            kind: "usize",
            default: "",
            description: "0..count の step インデックス（3 状態・aria-current 判定の元、steps.rs:224-334）。",
        },
        ArgRow {
            name: "progress: attrs, children",
            kind: "Vec<(&str, &str)>, Vec<Node>",
            default: "",
            description: "percent（step * 100 / count）を aria-valuenow/aria-valuetext へ出力する progressbar パーツ（イシュー #1665 新設）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "All steps completed",
            description: "count==step（全 step 完了）で completed_content が表示される例です（Demo は 2 番目の step が current）。",
            render: ex_steps_completed,
        },
        ExampleEntry {
            title: "Vertical orientation",
            description: "Orientation::Vertical で trigger/content/progress の data-orientation=\"vertical\" が反映される例です。",
            render: ex_steps_vertical,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "trigger/prev-trigger/next-trigger はネイティブ button のためフォーカス順序に入る（disabled 境界では native disabled によりフォーカス順序から除外される）。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "ネイティブ button の既定挙動（click イベント発火）のみ機能する。`fandhe-frontend-wasm-full` の headless::MAPPING_TABLE に \"steps\" scope は登録されておらず、trigger/prev-trigger/next-trigger の click から dispatch（\"goto\"/\"prev\"/\"next\"）への実配線は未実装（イシュー #1665 時点、別 Issue 起票対象）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-current=\"step\"",
            description: "current な item の trigger のみに出力する（steps.rs:249-269）。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-hidden=\"true\"",
            description: "separator に固定出力する（装飾要素、steps.rs:288-310）。",
        },
        AriaRow {
            attribute: "data-disabled",
            description: "prev-trigger/next-trigger の境界（step==0/step==count）で native disabled と併せて出力する（イシュー #1665 加算）。",
        },
        AriaRow {
            attribute: "role=\"progressbar\" / aria-valuemin / aria-valuemax / aria-valuenow / aria-valuetext",
            description: "progress パーツに固定出力する（イシュー #1665 新設）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Tour（/primitives/tour/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/tour.rs:1-94`（モジュール doc、
/// スコープ・状態モデル・out-of-scope）、`:220-458`（`Tour::new`/`content`/
/// `title`/`description`/`progress_text` シグネチャと role/aria-* 出力）。
fn ex_tour_second_step() -> Node {
    let mut tour = Tour::new(vec![
        TourStep {
            id: "step-1".to_string(),
            target: Some("#docs-toc-heading".to_string()),
            title: "Page navigation".to_string(),
            description: "Use this menu to jump between sections.".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
        TourStep {
            id: "step-2".to_string(),
            target: Some("#docs-search".to_string()),
            title: "Full-text search".to_string(),
            description: "Use this box to search the whole site.".to_string(),
            placement: Placement::new(Side::Top, Align::Start),
        },
    ]);
    tour.update(TourAction::Start);
    tour.update(TourAction::Next);
    tour.root(
        vec![],
        vec![
            tour.backdrop(vec![], vec![]),
            tour.spotlight(vec![], vec![]),
            tour.positioner(
                vec![],
                vec![
                    tour.arrow(vec![], vec![tour.arrow_tip(vec![], vec![])]),
                    tour.content(
                        hui::tour::ContentIds {
                            id: Some("tour-content-2"),
                            labelledby: Some("tour-title-2"),
                            describedby: Some("tour-desc-2"),
                        },
                        vec![],
                        vec![
                            tour.title(
                                Some("tour-title-2"),
                                vec![],
                                vec![text("Full-text search")],
                            ),
                            tour.description(
                                Some("tour-desc-2"),
                                vec![],
                                vec![text("Use this box to search the whole site.")],
                            ),
                            tour.progress_text(vec![], vec![text("Step 2 of 2")]),
                            tour.control(
                                vec![],
                                vec![
                                    tour.action_trigger(
                                        TourTriggerKind::Prev,
                                        vec![],
                                        vec![text("Prev")],
                                    ),
                                    tour.action_trigger(
                                        TourTriggerKind::Complete,
                                        vec![],
                                        vec![text("Finish")],
                                    ),
                                ],
                            ),
                            tour.close_trigger(vec![], vec![text("×")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

pub const TOUR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description / ProgressText / Control / CloseTrigger / ActionTrigger の 13 anatomy パーツを提供する（イシュー #1666 で Control を追加、tour.rs:1-13）。",
        "Idle/Active{step}/Skipped/Completed の決定的な状態機械を持ち、終端状態からのいずれのアクションも no-op（一度終了したツアーは暗黙に再開しない、tour.rs:34-52）。",
        "content は role=\"dialog\" と tabindex=\"-1\" を固定出力し、ContentIds が Some のときのみ aria-labelledby/aria-describedby を、Active 時のみ現在ステップの id を data-step として出力する（イシュー #1666、tour.rs 参照）。",
        "action_trigger は TourTriggerKind（Next/Prev/Skip/Complete/Custom）に応じて data-type を出力し、Prev は dispatch が no-op になる境界（先頭ステップ）でのみ disabled/data-disabled を付与する（イシュー #1666）。",
        "progress_text は aria-live=\"polite\" を固定出力し、ステップ遷移を支援技術へ読み上げさせる。",
        "対象要素の実座標追従・スポットライト実測値注入・target セレクタの実解決は fandhe-frontend-wasm-full の後続イシューのスコープ（本モジュールは data-target 出力と静的な data-side/data-align のみ）。",
    ],
    arguments: &[
        ArgRow {
            name: "Tour::new: steps",
            kind: "Vec<TourStep>",
            default: "",
            description: "ツアーの全ステップ（初期状態は常に Idle、tour.rs:238-243）。",
        },
        ArgRow {
            name: "content: ids",
            kind: "ContentIds",
            default: "ContentIds::default()",
            description: "id/aria-labelledby/aria-describedby の関連付け先。",
        },
        ArgRow {
            name: "title/description: id",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき content の labelledby/describedby と対にする。",
        },
        ArgRow {
            name: "action_trigger: kind",
            kind: "TourTriggerKind",
            default: "",
            description: "Next/Prev/Skip/Complete/Custom のいずれか。data-type 出力と Prev の disabled 判定に使う（イシュー #1666）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Second step (2-step tour)",
        description: "2 ステップ構成で 2 番目の step まで進めた例です（Demo は 1 ステップのみ）。",
        render: ex_tour_second_step,
    }],
    keyboard: &[
        KeyRow {
            key: "Escape",
            description: "zag.js は closeOnEscape 既定で dismiss へ写像する（本状態機械では \"skip\" dispatch 相当）。DOM keydown 配線は fandhe-frontend-wasm-full の後続イシューのスコープで未実装（イシュー #1666）。",
        },
        KeyRow {
            key: "ArrowRight / ArrowLeft",
            description: "zag.js は keyboardNavigation 既定で \"next\"/\"prev\" dispatch へ写像する。DOM keydown 配線は同様に未実装（イシュー #1666）。",
        },
        KeyRow {
            key: "Space / Enter",
            description: "close_trigger/action_trigger はネイティブ <button type=\"button\"> のため、ブラウザ既定動作としてすでに機能する。",
        },
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "zag.js は trapFocus([content, target]) で content と対象要素の間で巡回させる。本実装はフォーカストラップを配線しておらず未対応（イシュー #1666）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定出力する。alertdialog（即時応答を要する警告向け）へは非追随（オンボーディング案内は該当しないため、イシュー #1666）。",
        },
        AriaRow {
            attribute: "tabindex=\"-1\"",
            description: "content に固定出力する（フォーカス移動の受け皿、イシュー #1666）。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "ContentIds が Some のときのみ content に出力する。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "progress_text に固定出力する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Tree View（/primitives/tree-view/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/tree_view.rs:1-137`（モジュール doc、
/// out-of-scope・参照突合（イシュー #1667）節）、`:274-519`（`root`/`label`/
/// `tree`/`branch`/`branch_control`/`branch_indicator`/`branch_text`/
/// `branch_content`/`branch_indent_guide`/`item`/`item_text`/
/// `item_indicator` シグネチャと role/aria-*/data-* 出力）、`:681-683`
/// （`TreeView::render_nodes`）。
fn ex_tree_view_closed_branch() -> Node {
    let closed = OpenState::Closed;
    let props = tree_view::TreeItemProps {
        value: "src",
        selected: false,
        disabled: false,
        level: "1",
        posinset: "1",
        setsize: "1",
        depth: "0",
    };
    let lib_rs_props = tree_view::TreeItemProps {
        value: "lib.rs",
        selected: false,
        disabled: false,
        level: "2",
        posinset: "1",
        setsize: "1",
        depth: "1",
    };
    tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(
                Some("Project files"),
                None,
                vec![],
                vec![tree_view::branch(
                    closed,
                    props,
                    vec![],
                    vec![
                        tree_view::branch_control(
                            closed,
                            props,
                            vec![],
                            vec![
                                tree_view::branch_indicator(closed, props, vec![], vec![text("▸")]),
                                tree_view::branch_text(closed, props, vec![], vec![text("src")]),
                            ],
                        ),
                        tree_view::branch_content(
                            closed,
                            props,
                            vec![],
                            vec![
                                tree_view::branch_indent_guide(props, vec![], vec![]),
                                tree_view::item(
                                    lib_rs_props,
                                    vec![],
                                    vec![
                                        tree_view::item_indicator(lib_rs_props, vec![], vec![]),
                                        tree_view::item_text(
                                            lib_rs_props,
                                            vec![],
                                            vec![text("lib.rs")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// headless-ui は data-scope="tree-view"/data-part="*"/data-* のみを出力し
/// 外観を持たない（`JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET` と同型の最小例）。
/// branch-content の hidden 上書き（`pre_styled_ui::tree_view::stylesheet`
/// と同型の対応、UA 既定の `[hidden] { display: none }` を base 規則の
/// `display` 宣言が上書きしてしまう不具合の回避）・branch-indent-guide の
/// 幅・選択/無効状態の強調を自前 CSS で当てる。
const TREE_VIEW_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"tree-view\"][data-part=\"branch-content\"][hidden] {\n  \
  display: none;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-indent-guide\"] {\n  \
  width: 1rem;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-control\"][data-selected],\n\
[data-scope=\"tree-view\"][data-part=\"item\"][data-selected] {\n  \
  background: #eff6ff;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-control\"][data-disabled],\n\
[data-scope=\"tree-view\"][data-part=\"item\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-control\"]:focus-visible,\n\
[data-scope=\"tree-view\"][data-part=\"item\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n\
}\n";

/// [`TREE_VIEW_CUSTOM_CSS_SNIPPET`] を実演する例（[`ex_tree_view_closed_branch`]
/// と同じ木を使い回す）。
fn ex_tree_view_custom_css() -> Node {
    wrap_example(
        "headless-ui はスタイルを持たないため、data-scope=\"tree-view\"/data-part 属性セレクタで自前 CSS を当てる最小例です。",
        vec![
            ex_tree_view_closed_branch(),
            pre(
                vec![],
                vec![code(vec![], vec![text(TREE_VIEW_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Tree / Branch / BranchControl / BranchIndicator / BranchText / BranchContent / BranchIndentGuide / Item / ItemText / ItemIndicator の 12 anatomy パーツを提供する（tree_view.rs:1-10）。",
        "展開集合（MultiSelect）+ 選択値（SingleSelect）を合成した状態機械 TreeView を提供し、TreeView::render_nodes が深さ・aria-posinset/aria-setsize を再帰的に計算する（tree_view.rs:636-758）。",
        "tree は role=\"tree\"、branch/item は role=\"treeitem\"、branch_content は role=\"group\" を固定出力する WAI-ARIA APG Tree パターン準拠（tree_view.rs:291-487）。",
        "branch/item は disabled=true のとき aria-disabled=\"true\" を対で付与する（ネイティブ disabled を持たない role=\"treeitem\" のための代替、tree_view.rs:340-344,476-480）。",
        "ark-ui docs / zag.js との参照突合（イシュー #1667）で data-branch（branch）・data-value/data-depth（branch-control）・data-selected/data-disabled/data-state（インジケータ・テキスト系）・非選択時 hidden（item-indicator）を追加した（tree_view.rs 冒頭「参照突合（イシュー #1667）」節）。",
        "矢印キー・Home/End・Enter/Space・typeahead の DOM 配線は fandhe-frontend-wasm-full keynav.rs §TreeView（イシュー #1072）が担う。* による兄弟一括展開・Shift+Arrow/Ctrl+A（複数選択前提）・F2（rename）は非追随（tree_view.rs 冒頭「out-of-scope」節）。",
    ],
    arguments: &[
        ArgRow {
            name: "tree: aria_label_text, aria_labelledby_id",
            kind: "Option<&str>, Option<&str>",
            default: "None, None",
            description: "tree のアクセシブルな名前（いずれか片方が Some を推奨、tree_view.rs:291-306）。",
        },
        ArgRow {
            name: "TreeItemProps: value, selected, disabled, level, posinset, setsize, depth",
            kind: "&str, bool, bool, &str, &str, &str, &str",
            default: "",
            description: "branch/branch_control/branch_indicator/branch_text/branch_content/branch_indent_guide/item/item_text/item_indicator へ一括で通すノード共通プロパティ（level/posinset/setsize/depth は呼び出し側が usize から文字列化した aria-level/aria-posinset/aria-setsize/data-depth、イシュー #1667 で新設、tree_view.rs 内 TreeItemProps 定義）。",
        },
        ArgRow {
            name: "TreeView::render_nodes: nodes",
            kind: "&[TreeNode]",
            default: "",
            description: "決定的な静的木を現在の展開・選択状態で再帰描画する（tree_view.rs:681-683）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Closed branch",
            description: "branch が折りたたまれ、branch_content が hidden の例です（Demo は展開済み）。",
            render: ex_tree_view_closed_branch,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "headless 層は data-scope/data-part/data-* のみを出力し外観を持たないため、branch-content の hidden 上書き・branch-indent-guide の幅・選択状態の強調を自前 CSS で当てる最小例です。",
            render: ex_tree_view_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "ArrowDown / ArrowUp",
            description: "可視かつ disabled でない treeitem 間で 1 件ずつフォーカス移動する（折りたたみ中の子孫はスキップ、非循環）。",
        },
        KeyRow {
            key: "ArrowRight",
            description: "閉じたブランチは展開する。開いたブランチは最初の子へフォーカス移動する。葉ノードでは no-op。",
        },
        KeyRow {
            key: "ArrowLeft",
            description: "開いたブランチは折りたたむ。それ以外（葉ノード・閉じたブランチ）は親ブランチへフォーカス移動する（ルート直下では no-op）。",
        },
        KeyRow {
            key: "Home / End",
            description: "可視かつ disabled でない最初/最後の treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "葉ノードは選択（select）、ブランチは開閉（toggle）を発火する（ブランチ自体は選択できない）。",
        },
        KeyRow {
            key: "印字可能文字",
            description: "typeahead: 直近の入力から一致する treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Escape",
            description: "typeahead バッファをリセットする。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"tree\" / role=\"treeitem\" / role=\"group\"",
            description: "tree/branch・item/branch_content に固定出力する（tree_view.rs:291-487）。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "branch のみ（open/closed を反映、tree_view.rs:322-325）。",
        },
        AriaRow {
            attribute: "aria-selected / aria-level / aria-posinset / aria-setsize",
            description: "branch/item 双方に出力する（disabled 時も aria-selected=\"false\" を省略しない APG superset、tree_view.rs:322-329,459-466）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "disabled=true のときのみ branch/item に出力する（tree_view.rs:340-344,476-480）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "branch-indicator/item-indicator（装飾アイコン）に固定出力する（イシュー #1667 で追加、tree_view.rs 内 branch_indicator/item_indicator 定義）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Visually Hidden（/primitives/visually-hidden/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/visually_hidden.rs:1-46`（モジュール
/// doc、`aria-hidden` を付けない不変条件）、`:94-102`（`root` シグネチャ）、
/// `:47-87`（イシュー #1668 参考サイトとの突合節）。非テスト行で
/// `role`/`aria-` の出力は 0 件。
fn ex_visually_hidden_status_label() -> Node {
    div(
        vec![],
        vec![
            visually_hidden::root(vec![], vec![text("Build status: ")]),
            text("✓ Passing"),
        ],
    )
}

/// Radix Primitives / chakra-ui のデモ（アイコンのみボタン + 隠しラベル）
/// と同型の利用パターン（イシュー #1668 突合結果）。アイコン記号は
/// 視覚的な表示のみを担い、実際の操作名は [`visually_hidden::root`] が
/// 支援技術へ読ませる。
fn ex_visually_hidden_icon_button() -> Node {
    button(
        vec![("type", "button")],
        vec![
            text("🔔"),
            visually_hidden::root(vec![], vec![text("Notifications")]),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1668、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープ
/// を経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。clip 手法は `fandhe-frontend-pre-styled-ui::visually_hidden` の
/// `clip_declarations()`（`skip_nav` と共有）に整合させる。
const VISUALLY_HIDDEN_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"visually-hidden\"][data-part=\"root\"] {\n  \
  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  \
  overflow: hidden;\n  clip: rect(0 0 0 0);\n  white-space: nowrap;\n  overflow-wrap: normal;\n  \
  border-width: 0;\n\
}\n";

/// [`VISUALLY_HIDDEN_CUSTOM_CSS_SNIPPET`] を実演する例（`AVATAR_CUSTOM_CSS_SNIPPET`
/// の `ex_avatar_custom_css` と同型のパターン）。Radix のインライン
/// `style` による clip 手法（headless-ui へは持ち込まない装飾、イシュー
/// #1668 突合結果）を利用者が自前 CSS でどう再現するかを示す。
fn ex_visually_hidden_custom_css() -> Node {
    wrap_example(
        "利用者が data-scope / data-part 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            ex_visually_hidden_icon_button(),
            pre(
                vec![],
                vec![code(
                    vec![],
                    vec![text(VISUALLY_HIDDEN_CUSTOM_CSS_SNIPPET)],
                )],
            ),
        ],
    )
}

pub const VISUALLY_HIDDEN: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（span）の 1 anatomy パーツのみで構成する最小の状態非依存パーツ（visually_hidden.rs:94-102）。",
        "視覚的には隠すが支援技術（スクリーンリーダー）には読ませ続けるテキストコンテナであり、装飾要素とは逆に aria-hidden を意図的に付与しない（visually_hidden.rs:11-21）。",
        "styled 層（fandhe-frontend-pre-styled-ui::visually_hidden）は本モジュールが出力する data-scope=\"visually-hidden\"/data-part=\"root\" セレクタを前提に clip 手法の CSS を当てる（visually_hidden.rs:23-29）。",
        "Radix Primitives / Radix Themes / chakra-ui と突合済み（イシュー #1668）。anatomy（1 パーツ）・data-*・role/aria-*・キーボード操作のいずれも一致し是正なし。asChild/as（要素差し替え API）・視覚的に隠した input パートは意図的に非採用（checkbox/switch/radio_group/select の hidden input 系パーツが同用途を担う、visually_hidden.rs:47-87）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: attrs",
            kind: "Vec<(&str, &str)>",
            default: "vec![]",
            description: "呼び出し側が追加する属性。data-scope/data-part は本パーツが固定出力するため上書きされない（visually_hidden.rs:94-102）。root が唯一の公開関数であり attrs/children 以外の型付き引数を持たない。",
        },
        ArgRow {
            name: "root: children",
            kind: "Vec<Node>",
            default: "vec![]",
            description: "視覚的には隠すがスクリーンリーダーには読み上げさせるテキスト・ノード（visually_hidden.rs:94-102）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Status label prefix",
            description: "視覚的には記号のみを見せつつ、スクリーンリーダーには前置ラベルを読ませる例です。",
            render: ex_visually_hidden_status_label,
        },
        ExampleEntry {
            title: "Icon-only button label",
            description: "アイコンのみのボタンに、支援技術のためのラベルを追加する例です（Radix Primitives / chakra-ui のデモと同型のパターン、イシュー #1668）。",
            render: ex_visually_hidden_icon_button,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "headless 層は data-scope/data-part のみを出力し外観を持たないため、clip 手法を自前 CSS で当てる最小例です。",
            render: ex_visually_hidden_custom_css,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root は固有の role/aria-* を出力しない（visually_hidden.rs 全文の非テスト行で role/aria- grep 0 件）。装飾要素の aria-hidden=\"true\" 固定付与パターンとは逆に、本コンポーネントは aria-hidden を意図的に付与しない（visually_hidden.rs:11-21）。Radix Primitives / Radix Themes / chakra-ui のいずれも role/aria-* を自ら付与しない点と一致する（イシュー #1668 突合結果）。",
    }],
    demo: None,
};

/// 本カテゴリ 15 部品の `path -> ComponentPageSpec` テーブル
/// （`crate::primitive_specs::SPEC_TABLES` へ集約される、#1027 と同型。
/// イシュー #2114 で `marker` 追加、旧 14。イシュー #2111 で `attachment`
/// 追加、旧 13。イシュー #2108 で `bubble`
/// 追加、旧 12。イシュー #2105 で `message` 追加、
/// 旧 11）。
/// 並び順は `crate::primitives_catalog::PrimitiveCategory::DataDisplayUtilities`
/// のカタログ順と一致させる。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/attachment/", ATTACHMENT),
    ("/primitives/avatar/", AVATAR),
    ("/primitives/bubble/", BUBBLE),
    ("/primitives/carousel/", CAROUSEL),
    ("/primitives/item/", ITEM),
    ("/primitives/json-tree-view/", JSON_TREE_VIEW),
    ("/primitives/marker/", MARKER),
    ("/primitives/message/", MESSAGE),
    ("/primitives/scroll-area/", SCROLL_AREA),
    ("/primitives/skip-nav/", SKIP_NAV),
    ("/primitives/splitter/", SPLITTER),
    ("/primitives/steps/", STEPS),
    ("/primitives/tour/", TOUR),
    ("/primitives/tree-view/", TREE_VIEW),
    ("/primitives/visually-hidden/", VISUALLY_HIDDEN),
];
