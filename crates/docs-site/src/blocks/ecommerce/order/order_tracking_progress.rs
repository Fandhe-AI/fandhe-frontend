//! `order-tracking-progress` block（イシュー #3060/#3061、親 #3059。
//! Ecommerce / Order カテゴリ、区分初の block）。注文ヘッダ・商品カード列
//! （配送状況の進捗バー + 到達段階表示）・サマリの 3 領域を合成する。
//! 主参照 R1122（代表構成）を軸に、集約元 R1123（大画像・枠なし版）・
//! R0585（商品ごと配送先強調）・R0588（`steps` によるアイコン付き
//! タイムライン版）の差分は本ファイルと原稿の差分メモ節で扱う。
//! `_/blocks-intake/` の対応ファイルは着手時点で本 worktree に存在しない
//! ため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `progress` / `steps` / `data-list` /
//! `button` / `link` / `card` / `separator` / `badge` の 11 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 構成
//!
//! ヘッダ（注文番号・注文日の見出し + 請求書リンク・確認ボタン）の下に、
//! 商品カード 3 件（画像・名称・価格・配送先 + 配送状況テキスト・
//! 進捗バー・到達段階表示）、区切り線を挟んでサマリ 3 カラム（請求先・
//! 支払い情報・集計）を積む。3 商品は到達段階と段階表示の形式を違える:
//! 1・2 件目は [`StageView::Labels`]（4 段階のラベル列、R1122 主参照）で
//! 「配送中」「発送準備中」を、3 件目は [`StageView::Timeline`]（R0588、
//! `steps` によるアイコン付きタイムライン）で「配達完了」（全段階完了）
//! を示す。
//!
//! # 進捗バーと段階表示を同じ値から導く（[`shipment_progress`]/
//! [`shipment_steps`]）
//!
//! `Progress` の `value`（0〜100）・段階ラベル列（[`stage_list`]）の
//! `data-reached`・`steps` タイムライン（[`stage_timeline`]）の
//! `current`/`complete` 状態が別々の定数から食い違って書かれることを
//! 構造で防ぐため、到達段階 index（0〜3、`reached`）だけを呼び出し側で
//! 決め、3 表現すべてが同じ `reached` から導出される。[`shipment_steps`]
//! は `reached + 1 >= STAGES.len()`（最終段階到達）のときのみ
//! `Steps::step` を `count` に写像し、`steps` の「全 step 完了」
//! （`is_completed`）を表す。
//!
//! # `steps` タイムラインの縦横切替（R0588、[`stage_timeline`]）
//!
//! `fandhe_frontend_headless_ui::steps::Steps` の `orientation` は構築時
//! 固定で CSS だけでは切り替えられないため（`crates/pre-styled-ui/src/
//! steps.rs` モジュール doc「`item`/`separator` のレイアウト契約」節
//! 参照）、3 件目の商品カードは横向き・縦向きの [`stage_timeline`] を
//! それぞれ 1 インスタンスずつ常に両方描画し、[`LAYOUT_CSS`] の
//! `@container` 規則が幅に応じてどちらか一方を `display: none` で隠す
//! （`changelog_timeline` と同型のパターン、`crate::blocks::marketing::
//! changelog::changelog_timeline` 参照）。`display: none` 側は支援技術
//! ツリーからも除外される。
//!
//! # アイコン（[`stage_icon`]）
//!
//! 完了段階は自作の幾何学的なチェックマーク SVG（`aria-hidden="true"`）、
//! 未完了段階は段階番号のテキストを表示する（参照元の絵柄アイコンは
//! 持ち込まない。インライン SVG の前例は `crate::blocks::marketing::
//! hero::hero_email_signup::play_icon` と同型）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `card::root`/`button::button`/`image::image`/`progress::root`/
//! `link::root`/`badge`/`heading`/`text`/`data_list::root`/`steps::root`
//! はいずれも `drop_class_attr` で呼び出し側 `class` を除去してから内部
//! variant クラスと合成するため、これらへの CSS フックは `data-*` 属性で
//! 渡す（`data-blocks-order-tracking-progress-*`）。素の `div`/`ul`/`li` と
//! `card::body`/`card::footer`（variant を持たず `attrs` をそのまま連結
//! する）は `class="blocks-order-tracking-progress-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。レイアウト root の class は
//! `demo_class`（`blocks-order-tracking-progress`）と別名（`-layout`）に
//! する（既存 block の Bugbot 教訓、`profile_detail_datalist` 系と同型）。
//!
//! # 狭い幅では段階ラベル・タイムラインを縦並び・商品グリッド/サマリを
//! 1 カラムにする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のレイアウト root へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のときヘッダ操作群を
//! 下段へ折り返し、商品グリッドとサマリを 1 カラム化し、4 段階の段階
//! ラベル列を 1 カラム（縦並び）へ、`steps` タイムラインを横向きから
//! 縦向きへ切り替える（親仕様「狭い幅では段階ラベルを縦並び」節）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、注文処理・決済・送信先を一切持たない。
//! ボタンは `button::button` の既定 `type="button"` のまま用いる。`steps`
//! の `trigger`（実 `<button>`）も無 JS の no-op で、既存の「注文内容を
//! 確認」ボタンと同じ扱いとする。
//!
//! # `href` は外部絶対 URL のダミー値
//!
//! 請求書リンクは `href="#"` を避け、ダミーでも意味の通る URL
//! （`https://example.com/invoices/…`）を使う。サイト内相対パス
//! （`/invoices/…`）は `linkcheck::check_links`（`crates/docs-site/
//! tests/support/shared_site.rs`）が「実在しないページ」として
//! fail-closed に検知するため使えない（`footer_newsletter_band` の
//! `REPO` 定数と同型の判断、`crate::blocks` モジュール doc
//! 「`href` に絶対 URL を使う理由」節参照）。
//!
//! # ダミー素材について
//!
//! 注文番号・住所・氏名・決済情報はすべて架空（`crate::blocks::
//! dummy_assets` の人名・社名を流用しつつ、注文番号・住所・カード末尾 4
//! 桁・商品価格は本 block 独自の架空値）。メールアドレスは
//! `example.com` ドメイン、電話番号・住所は架空パターンとし、実在の
//! 人物・企業・PII・実クレデンシャルは含まない。カード番号は末尾 4 桁の
//! 伏字表現のみとし、実在パターンは使わない。商品画像はビルド時生成の
//! 同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI
//! は使わない）。参照元の商品名・ブランド・アイコンは持ち込まない。商品
//! 価格は円建てで固定し（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
//! はドル建てのため使わない）、小計・送料・税・合計と通貨・金額を一致
//! させる（3 商品の価格合計 ¥21,600 + 送料 ¥600 + 税 ¥2,200 = 合計
//! ¥24,400）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! R1123（大画像・枠なし版）・R0585（商品ごと配送先強調）は、本 block が
//! 既定として採用する表現（`card` の枠付き・`8rem` 幅の画像列、商品ごと
//! に「配送先: {destination}」行を明示する構成）が両方とも満たしている
//! ため、原稿の差分メモ節でその旨を記す扱いとし、枠なし版・大画像版を
//! Demo に別枠で並べることはしない（1 block 内に枠あり/なしが混在すると
//! 差分の主眼が読み取れなくなるため）。`blocks_contract.rs` への block
//! 固有ページテストの追加も見送る（共通契約テストが `<form>`・`data:`・
//! `raw_html`・重複 id を全 block で検証済みで、本 block が追加する CSS
//! フックは素の `div` の `class` のみのため「フックが黙って効かない」
//! リスクがない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 配送の 4 段階（表示ラベル）。
const STAGES: [&str; 4] = ["注文受付", "発送準備", "配送中", "配達完了"];

/// 到達段階 index（0〜3）から `Progress`（`value` は 0〜100 の 0〜3 等分）を
/// 導く。段階ラベル列（[`stage_list`]）と進捗バーが同じ `reached` から
/// 導出されるため値の食い違いが構造的に起きない
/// （モジュール冒頭「進捗バーと段階ラベルを同じ値から導く」節参照）。
fn shipment_progress(reached: usize) -> Progress {
    let value = (reached as f64) * 100.0 / ((STAGES.len() - 1) as f64);
    Progress::new(0.0, 100.0, Some(value), Orientation::Horizontal)
}

/// 4 段階の到達ラベル列。`reached` 未満の index は `data-reached` を持つ
/// （狭幅時は [`LAYOUT_CSS`] が 1 カラム＝縦並びへ切り替える）。
fn stage_list(reached: usize) -> Node {
    ul(
        vec![("class", "blocks-order-tracking-progress-stages")],
        STAGES
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let mut attrs = vec![];
                if index <= reached {
                    attrs.push(("data-reached", ""));
                }
                li(attrs, vec![text(*label)])
            })
            .collect(),
    )
}

/// 段階表示の形式（[`product_card`] の `view` 引数）。R0588（`steps` に
/// よるアイコン付きタイムライン版）の差分を、既存のラベル列表示
/// （[`stage_list`]）と並記するために導入した（モジュール冒頭「構成」節
/// 参照）。
enum StageView {
    /// 4 段階を横並びのラベル列で表示する（R1122 主参照、既定）。
    Labels,
    /// `steps` によるアイコン付きタイムラインで表示する（R0588）。狭幅では
    /// 縦向きへ切り替える（[`LAYOUT_CSS`] の `@container` 規則、
    /// [`stage_timeline`] 参照）。
    Timeline,
}

/// 到達段階 index（0〜3）から `Steps` 状態機械を導く（[`stage_timeline`]
/// のみが呼ぶ内部ヘルパ）。最終段階（配達完了）到達時のみ `step` を
/// `count` に写像し、`Steps::is_completed` が真になる（モジュール冒頭
/// 「進捗バーと段階表示を同じ値から導く」節参照）。
fn shipment_steps(reached: usize, orientation: Orientation) -> Steps {
    let step = if reached + 1 >= STAGES.len() {
        STAGES.len()
    } else {
        reached
    };
    Steps::new(STAGES.len(), step, orientation)
}

/// `steps` の indicator 内アイコン。完了段階は自作のチェックマーク SVG、
/// 未完了段階は段階番号のテキスト（モジュール冒頭「アイコン」節参照）。
fn stage_icon(complete: bool, index: usize) -> Node {
    if complete {
        el(
            "svg",
            vec![
                ("viewBox", "0 0 24 24"),
                ("width", "16"),
                ("height", "16"),
                ("aria-hidden", "true"),
            ],
            vec![el(
                "path",
                vec![
                    ("d", "M5 12l4 4L19 7"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                ],
                vec![],
            )],
        )
    } else {
        text((index + 1).to_string())
    }
}

/// 4 段階の `steps` タイムライン 1 インスタンス（[`Orientation`] を構築時
/// 固定で受け取る、モジュール冒頭「`steps` タイムラインの縦横切替」節
/// 参照）。呼び出し側（[`product_card`]）が横向き・縦向きの 2 インスタンス
/// を常に両方描画し、`@container` で表示を切り替える。
fn stage_timeline(reached: usize, orientation: Orientation) -> Node {
    let state = shipment_steps(reached, orientation);
    let items = STAGES
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let complete = index < state.step();
            let trigger = steps::trigger(
                &state,
                index,
                vec![],
                vec![
                    steps::indicator(&state, index, vec![], vec![stage_icon(complete, index)]),
                    text(*label),
                ],
            );
            let mut item_children = vec![trigger];
            if index + 1 < STAGES.len() {
                item_children.push(steps::separator(&state, index, vec![], vec![]));
            }
            steps::item(&state, index, vec![], item_children)
        })
        .collect();
    let list = steps::list(&state, vec![], items);
    steps::root(
        Size::Sm,
        ColorPalette::Accent,
        &state,
        vec![("data-blocks-order-tracking-progress-timeline", "")],
        vec![list],
    )
}

/// 注文番号・注文日の見出しと、請求書リンク・確認ボタンの操作群を束ねる
/// ヘッダ行。
fn order_header(order_number: &'static str, order_date: &'static str) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-header")],
        vec![
            div(
                vec![("class", "blocks-order-tracking-progress-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(format!("注文 #{order_number}"))],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(format!("注文日 {order_date}"))],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-order-tracking-progress-actions")],
                vec![
                    link::root(
                        "https://example.com/invoices/fd-2048-113",
                        &LinkProps::default(),
                        vec![],
                        vec![text("請求書を表示")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("注文内容を確認")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品カード 1 件（画像・名称・価格・配送先 + 配送状況・進捗バー・
/// 段階ラベル列）。
fn product_card(
    name: &'static str,
    price: &'static str,
    destination: &'static str,
    status_label: &'static str,
    eta: &'static str,
    reached: usize,
    view: StageView,
) -> Node {
    let progress_state = shipment_progress(reached);
    let progress_aria_label = format!("{name} の配送の進捗");
    card::root(
        CardProps::default(),
        vec![("data-blocks-order-tracking-progress-card", "")],
        vec![
            card::body(
                vec![("class", "blocks-order-tracking-progress-product")],
                vec![
                    image(
                        &ImageProps {
                            fit: ImageFit::Cover,
                            shape: ImageShape::Rounded,
                            ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                        },
                        vec![("data-blocks-order-tracking-progress-image", "")],
                    ),
                    div(
                        vec![("class", "blocks-order-tracking-progress-product-info")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(name)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(price)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送先: {destination}"))],
                            ),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-order-tracking-progress-shipping")],
                vec![
                    div(
                        vec![("class", "blocks-order-tracking-progress-status")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送状況: {status_label}"))],
                            ),
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Subtle,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text(eta)],
                            ),
                        ],
                    ),
                    progress::root(
                        &progress_state,
                        &ProgressProps::default(),
                        Some(status_label),
                        vec![
                            ("data-blocks-order-tracking-progress-bar", ""),
                            ("aria-label", progress_aria_label.as_str()),
                        ],
                        vec![progress_state
                            .track(vec![], vec![progress::range(&progress_state, vec![])])],
                    ),
                    match view {
                        StageView::Labels => stage_list(reached),
                        StageView::Timeline => div(
                            vec![("class", "blocks-order-tracking-progress-timeline-group")],
                            vec![
                                div(
                                    vec![(
                                        "class",
                                        "blocks-order-tracking-progress-timeline-horizontal",
                                    )],
                                    vec![stage_timeline(reached, Orientation::Horizontal)],
                                ),
                                div(
                                    vec![(
                                        "class",
                                        "blocks-order-tracking-progress-timeline-vertical",
                                    )],
                                    vec![stage_timeline(reached, Orientation::Vertical)],
                                ),
                            ],
                        ),
                    },
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 見出し + 定義リストの 1 セクション（請求先・支払い情報・集計）。
fn summary_section(title: &'static str, orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-summary-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            data_list::root(
                DataListProps {
                    orientation,
                    ..DataListProps::default()
                },
                vec![],
                rows,
            ),
        ],
    )
}

/// `order-tracking-progress` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-layout")],
        vec![
            order_header("FD-2048-113", "2026-09-24"),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-items")],
                vec![
                    product_card(
                        dummy_assets::COMPANY_NAMES[0],
                        "¥12,800",
                        "東京都渋谷区 1-2-3",
                        "配送中",
                        "9/27 到着予定",
                        2,
                        StageView::Labels,
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[1],
                        "¥5,600",
                        "大阪府大阪市 4-5-6",
                        "発送準備中",
                        "9/29 到着予定",
                        1,
                        StageView::Labels,
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[2],
                        "¥3,200",
                        "福岡県福岡市 7-8-9",
                        "配達完了",
                        "9/24 到着済み",
                        3,
                        StageView::Timeline,
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-summary")],
                vec![
                    summary_section(
                        "請求先",
                        DataListOrientation::Vertical,
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("住所", "東京都渋谷区 1-2-3"),
                            row("メール", "haruto.fujimaki@example.com"),
                        ],
                    ),
                    summary_section(
                        "支払い情報",
                        DataListOrientation::Vertical,
                        vec![
                            row("支払方法", "クレジットカード"),
                            row("カード番号", "**** **** **** 4242"),
                            row("請求日", "2026-09-24"),
                        ],
                    ),
                    summary_section(
                        "集計",
                        DataListOrientation::Horizontal,
                        vec![
                            row("小計", "¥21,600"),
                            row("送料", "¥600"),
                            row("税", "¥2,200"),
                            row("合計", "¥24,400"),
                        ],
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/order-tracking-progress/",
    title: "order-tracking-progress",
    category: BlockCategory::Order,
    rust_source: "crates/docs-site/src/blocks/ecommerce/order/order_tracking_progress.rs",
    demo_class: "blocks-order-tracking-progress",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `order_tracking_progress` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-order-tracking-progress-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-order-tracking-progress;\n}\n\
.blocks-order-tracking-progress-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-tracking-progress-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-tracking-progress-actions {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-tracking-progress-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-order-tracking-progress-product {\n  display: grid;\n  grid-template-columns: 8rem 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-order-tracking-progress-image] {\n  width: 100%;\n  height: 6rem;\n}\n\
.blocks-order-tracking-progress-product-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-order-tracking-progress-shipping {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-order-tracking-progress-status {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-order-tracking-progress-stages {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-order-tracking-progress-stages > li {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  text-align: center;\n}\n\
.blocks-order-tracking-progress-stages > li[data-reached] {\n  color: var(--fandhe-color-fg);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-order-tracking-progress-timeline-group {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-order-tracking-progress-timeline] {\n  width: 100%;\n}\n\
.blocks-order-tracking-progress-timeline-vertical {\n  display: none;\n}\n\
.blocks-order-tracking-progress-summary {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-order-tracking-progress-summary-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-order-tracking-progress (max-width: 40rem) {\n  \
.blocks-order-tracking-progress-actions {\n    width: 100%;\n  }\n  \
.blocks-order-tracking-progress-product {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-tracking-progress-stages {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-order-tracking-progress-timeline-horizontal {\n    display: none;\n  }\n  \
.blocks-order-tracking-progress-timeline-vertical {\n    display: block;\n  }\n  \
.blocks-order-tracking-progress-summary {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"progress\"",
            "data-scope=\"steps\"",
            "data-scope=\"data-list\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"card\"",
            "data-scope=\"separator\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-scope=\"progress\" data-part=\"root\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"root\"")
                .count(),
            2
        );
        assert!(html.contains("data-orientation=\"horizontal\""));
        assert!(html.contains("data-orientation=\"vertical\""));
        // ラベル列（ul>li）2 × 4 段階 + steps タイムライン（ol>li）2 インスタンス
        // （横向き・縦向き）× 4 段階 = 16（モジュール冒頭「構成」節参照）。
        assert_eq!(html.matches("<li").count(), 16);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn progress_value_matches_reached_stage_count() {
        let html = demo_html();
        // 商品 1: reached = 2 → value = 2 * 100 / 3 ≈ 66.67%
        assert!(html.contains("--fandhe-progress-percent: 66.66666666666667%"));
        // 商品 2: reached = 1 → value = 1 * 100 / 3 ≈ 33.33%
        assert!(html.contains("--fandhe-progress-percent: 33.333333333333336%"));
        // 商品 3: reached = 3（全段階到達）→ value = 3 * 100 / 3 = 100%
        assert!(html.contains("--fandhe-progress-percent: 100%"));
        // data-reached はラベル列表示（商品 1・2）のみが持つ属性（商品 3 は
        // steps タイムラインの data-state/data-complete で到達を表す）。
        assert_eq!(html.matches("data-reached=\"\"").count(), 5);
        // 商品 3（reached=3 → Steps::step=count=4、全 4 段階完了）の
        // aria-hidden="true" は、チェックマーク SVG（横向き・縦向きの
        // 2 インスタンス × 4 完了段階 = 8）と `steps::separator`（`role=
        // "separator"` + `aria-hidden`、最後の item を除く 3 本 × 2
        // インスタンス = 6）の合計 14。
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 14);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-order-tracking-progress (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-tracking-progress-stages {\n    grid-template-columns: 1fr;\n  }"
        ));
        assert!(LAYOUT_CSS
            .contains(".blocks-order-tracking-progress-timeline-vertical {\n  display: none;\n}"));
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-tracking-progress-timeline-horizontal {\n    display: none;\n  }"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-order-tracking-progress-timeline-vertical {\n    display: block;\n  }"
        ));
    }
}
