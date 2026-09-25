//! `feature-vertical-tabs` block（親 #2774「Blocks 目的別パーツ拡充ツリー」
//! 配下、規模 L のため #2775（骨格と主要領域）/ #2776（残り・仕上げ）の
//! 2 sub-issue へ分割済み。いずれも完了）。左列に縦に並んだ feature タブ、
//! 右列に選択中 feature の詳細（見出し・チェック付き機能一覧・画像）を置く
//! 合成例。対応表 ID は R0480（主参照。他に集約元はなく「集約元の差分」は
//! 該当しない）のみを記す（取得手段・ファイル名・内部コンポーネント識別子
//! は記載しない）。
//!
//! # #2775・#2776・実物の `tabs::tabs` 排除ラウンド・recipe 分離ラウンドの
//! 分担（完了記録）
//!
//! #2775 では骨格と主要領域を実装した: レイアウト root・セクション見出し
//! （`heading` H3 + `text` リード文）・4 タブの [`vertical_tabs`]・各 trigger
//! のタイトル・短い説明・各パネルの見出し（`heading` H4）・チェック付き
//! 機能一覧（`icon` とタイトルと説明）・画像（`image`）。#2776 では次を
//! 仕上げた: trigger 先頭のアイコン（[`FeatureTab::icon`]）・画像主体の別
//! パネル形（[`PanelLayout::ImageFirst`]）・[`FEATURES`] の 4 タブそれぞれを
//! 選択済みにした 4 インスタンスへの拡張（下記「全パネルを静的に読める
//! ようにする」節）。さらに後続ラウンドで、[`vertical_tabs`] が実物の
//! `tabs::tabs` を一切使わない非対話構造へ置き換わった（下記「無 JS での
//! 扱い（実物の `tabs::tabs` は一切使わない）」節）。直後のラウンド（本
//! コミット）では、その非対話構造が `data-scope="tabs"`/`data-part="..."`
//! という pre-styled-ui の `tabs` recipe と同一のセレクタを再利用していた
//! ため、recipe の `cursor: pointer`/hover 面/フォーカスリング等の
//! インタラクティブ向けスタイルまで意図せず継承してしまい、無 JS で実際
//! には切り替わらないのに見た目だけ「クリックできそう」に見える不整合が
//! 残っていた（Bugbot Medium 指摘）。是正として recipe のセレクタと
//! 完全に切り離した独自 class 群へ置き換えた（下記「recipe セレクタから
//! 完全に切り離す理由」節）。
//!
//! # 全パネルを静的に読めるようにする
//!
//! [`demo`] は [`FEATURES`] の 4 タブそれぞれを選択済みにした 4 インスタンス
//! を縦に並べる（`build`/`observe` は [`PanelLayout::ListFirst`]、
//! `deploy`/`secure` は [`PanelLayout::ImageFirst`]、2 つのパネル形を交互に
//! 見せる構成）。これにより 4 パネルすべてがページ上のいずれかのインス
//! タンスで可視状態として存在する（`sidebar_07` の expanded/collapsed
//! 2 インスタンス併記・`pricing_tiers_morph` の月額/年額 2 インスタンス
//! 併記と同型の対処で、対象タブ数が 4 のため 4 インスタンスへ拡張した点
//! のみが異なる）。`FEATURES`・`dummy_assets` の既存画像定数はそのまま
//! 再利用し、新規データは追加しない。
//!
//! # trigger 先頭のアイコン
//!
//! 参照元 R0480 にはない装飾として、4 タブそれぞれに 1 意匠の自作幾何
//! アイコンを trigger 先頭へ追加した（下記「自作幾何アイコン」節参照）。
//! 隣に可視のタイトル・説明があるため装飾扱い（[`IconProps::label`] は
//! `None` のまま、`aria-hidden="true"`）とする。
//!
//! # 使用部品
//!
//! `heading` / `text` / `tabs` / `image` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。`tabs` は下記「無 JS
//! での扱い」節・「recipe セレクタから完全に切り離す理由」節のとおり
//! `fandhe_frontend_pre_styled_ui::tabs::tabs` を実際には呼ばず、独自の
//! block 固有 class（`data-scope="tabs"`/`data-part="..."` 等 recipe の
//! セレクタとは一致しない）だけで見た目を独自に再現する。新規 UI 部品は
//! 追加しない。
//!
//! # 無 JS での扱い（実物の `tabs::tabs` は一切使わない）
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` の
//! trigger（`role="tab"`/`type="button"`/`tabindex`）はクリック・キーボード
//! 操作をしても実際には一切切り替わらないにもかかわらず操作可能に見える。
//! 当初は非選択タブ 3 件を `disabled`/`aria-disabled="true"` にして操作
//! できない見た目にしていたが、各インスタンスで唯一有効な選択中タブが
//! `role="tab"`・押せるボタンのまま残り、`role="tablist"` と合わせて
//! キーボード利用者・支援技術には「切り替え可能な UI」と伝わる不整合が
//! 残った（codex-review 指摘、`docs/design/docs-site-blocks-section.md`
//! §19 参照。`feature_tabs_panel::static_tab_list` が先に解決した課題と
//! 同型）。無 JS の docs サイトで実際に切り替えられるようにする経路は
//! ないため、本 block は実物の `tabs::tabs` を**一切使わない**。タブ列は
//! [`static_tab_list`]（`role`/`tabindex`/`<button>`/`data-scope`/
//! `data-part`/`data-state`/`data-orientation` のいずれも持たない `div`
//! のみで [`LAYOUT_CSS`] の独自 class（下記「recipe セレクタから完全に
//! 切り離す理由」節）で見た目を再現する非対話表示）で視覚上だけ模し、
//! 選択中パネルの本文は [`vertical_tabs`] が直接（`panel_body` 経由で）
//! 描画する。残り 3 パネルは同 block の別インスタンスでそれぞれ選択済み
//! として可視になる（上記「全パネルを静的に読めるようにする」節）ため、
//! `feature_tabs_panel` のようなプレビュー併記は不要。
//!
//! # id 規約
//!
//! 基底 id は `blocks-feature-vertical-tabs-<接尾辞>` とする。接尾辞は
//! 選択済みタブの `value`（`build`/`deploy`/`observe`/`secure`）と同じ
//! 文字列にする（[`vertical_tabs`] が組み立てる
//! `<id_prefix>-trigger-<value>`/`<id_prefix>-content-<value>` が全域で
//! 一意になるよう、接尾辞をインスタンス間で必ず変える）。実物の
//! `tabs::tabs` を使わないため `aria-controls`/`aria-labelledby` による
//! 相互参照は発生せず、id は一意性の確保のみを目的とする。
//!
//! # recipe セレクタから完全に切り離す理由（Bugbot Medium 是正）
//!
//! 当初案は [`static_tab_list`]/[`vertical_tabs`] が `data-scope="tabs"`/
//! `data-part="list"`/`"trigger"`/`"content"`/`data-orientation="vertical"`
//! という `fandhe_frontend_pre_styled_ui::tabs` の recipe（イシュー
//! #1542/#2039）と全く同じセレクタを `<div>` へ与え、`pre-styled-ui.css`
//! （全ページ共通で読み込まれる）の recipe CSS を実物のコンポーネントを
//! 介さずに「間借り」する設計だった。しかし recipe の `trigger` base 規則
//! は `cursor: pointer` と `@media (hover: hover)` のホバー面（trigger の
//! `--fandhe-hover-bg`）を持つ（`fandhe_frontend_pre_styled_ui::tabs::
//! recipe` 参照）ため、セレクタが一致するだけで実物の `<button>` と区別
//! なくホバー時にクリック可能な見た目になってしまっていた（Bugbot
//! Medium 指摘: 無 JS で実際には切り替わらないのに 16 個の trigger が
//! 押せるように見える）。`pointer-events: none` での抑止は、trigger 内の
//! 子要素（アイコン・span）経由でなお hover が当たってしまうため根本
//! 対処にならない（同型の指摘が `feature_tabs_panel` 側にもあった）。
//! 是正として、[`LAYOUT_CSS`] は recipe のセレクタと文字通り**一致する
//! セレクタを一切持たない**設計へ変更した: `static_tab_list`/
//! `vertical_tabs` が組み立てる `<div>` は `data-scope`/`data-part`/
//! `data-state`/`data-orientation` のいずれも出力せず、
//! `blocks-feature-vertical-tabs-tabs-root`/`-tablist`/`-tab`/
//! `-tab-active`/`-panel` という block 固有 class だけで縦並び・区切り線・
//! 選択中の強調を独自に定義する（`cursor`/hover/フォーカスリングは一切
//! 定義しない）。これにより `pre-styled-ui.css` の tabs recipe は本 block
//! の出力へ一切当たらず、見た目が recipe の改修と無関係に安定する副次
//! 効果もある。
//!
//! # レスポンシブ（64rem をブレークポイントとする理由）
//!
//! `< 64rem`（lg 未満）は `.blocks-feature-vertical-tabs-tabs-root` を
//! `flex-direction: column` にしてタブ列をパネルの上へ積む。`>= 64rem` で
//! 左の縦タブ列 + 右のパネルの 2 列へ切り替える。テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できない（CSS custom property は
//! 宣言側でのみ有効）ため、[`fandhe_frontend_pre_styled_ui::recipe::
//! Breakpoint`] の `Lg`（1024px = 64rem）と一致するリテラル値
//! `63.99rem`/`64rem` を [`LAYOUT_CSS`] へ直書きする（`feature_expand`/
//! `feature_split_list_image` と同じ判断）。lg 未満でも `.blocks-feature-
//! vertical-tabs-tablist` の軸（縦積み）は変えず `max-width` 制約だけを
//! 外すため、`role`/`aria-*` を出力しない本 block では「見た目と意味論の
//! 食い違い」自体が構造的に発生しない。
//!
//! # trigger は phrasing content だけで組む
//!
//! [`static_tab_list`] の trigger は `<button>` ではなく `<div>` だが、
//! 見た目を独自 class で整えるうえで単純さを保つため引き続き phrasing
//! content のみで構成する（`heading::heading` の `<h*>` や `styled_text::
//! text` の `<p>` は使わない）。[`trigger_body`] は
//! `span[data-blocks-feature-vertical-tabs-trigger-body]` の中に、タイトル
//! 用の `span[data-blocks-feature-vertical-tabs-trigger-title]` と説明用の
//! `span[data-blocks-feature-vertical-tabs-trigger-desc]` を置き、core の
//! `span`/`text` のみで組む。lg 未満でもタブ列は横に伸びず縦積みのまま
//! （上記「レスポンシブ」節）のため、説明文（`trigger-desc`）を隠す必要は
//! なく常に表示する。
//!
//! # CSS フックの選び方
//!
//! `heading`/`text`/`image`/`icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-feature-vertical-tabs-*` 属性で
//! 渡す。`tabs` の見た目（[`static_tab_list`]/[`vertical_tabs`]）は上記
//! 「recipe セレクタから完全に切り離す理由」節のとおり block 固有 class
//! （`.blocks-feature-vertical-tabs-tabs-root`/`-tablist`/`-tab`/
//! `-tab-active`/`-panel`）で完結させ、`pre-styled-ui.css` の recipe とは
//! 詳細度で競合しない（recipe のセレクタと文字通り一致しないため、そもそも
//! 詳細度勝負が発生しない）。`image`（recipe 詳細度 (0,2,0)）への上書きは
//! 引き続き実物の `image::image` を呼ぶため `[data-scope="image"]
//! [data-part="root"][data-blocks-feature-vertical-tabs-image]` の 3
//! セレクタ構成（(0,3,0)）で行う（`feature_split_list_image`/
//! `feature_image_cards` と同型の判断。`image`/`icon`/`heading`/`text` は
//! 実物のコンポーネントであり非対話要素のため、recipe 継承それ自体は
//! 問題にならない）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `feature_split_list_image::geo_icon` と同型の自作ヘルパ [`geo_icon`]
//! （`path` へ `fill="none"` + `stroke="currentColor"` を明示し `icon::icon`
//! の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を上書きして
//! 線画として描画する）で描く。機能一覧のチェックマーク（[`check_icon`]）と
//! 4 タブの trigger アイコン（[`FeatureTab::icon`]、歯車/稲妻/円/盾の単純
//! 図形）はいずれもこのヘルパ 1 つに統一する（重複コード回避）。隣に可視
//! テキストがあるため装飾扱い（[`IconProps::label`] は `None` のまま、
//! `aria-hidden="true"`）とする。
//!
//! # trigger アイコンの CSS
//!
//! 上記「recipe セレクタから完全に切り離す理由」節のとおり `tabs` の
//! recipe は一切継承しないため、アイコンとタイトル列の横並び
//! （`display: inline-flex; gap: var(--fandhe-space-2);`）も
//! `.blocks-feature-vertical-tabs-tab` 自身で定義する。加えて
//! `flex-shrink: 0`（縦積みタイトル/説明列に押し潰されないようにする）を
//! trigger アイコンへ与える。
//!
//! # alt を空文字列にする理由
//!
//! いずれの画像も隣接する見出し・機能一覧で内容が伝わる装飾用途のため、
//! `alt=""` にする（`feature_alternating_rows`/`feature_split_list_image`
//! の前例と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。trigger は `<button>` ではなく `<div>` のため送信先を持たない
//! （上記「無 JS での扱い」節）。文言はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。画像は [`crate::blocks::dummy_assets`]
//! の各定数（いずれもビルド時生成のプレースホルダー SVG）を使い分け、
//! `alt=""` で出力する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el_owned, span, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を複製
/// しないための単純図形、`feature_split_list_image::geo_icon` と同型の判断。
/// `Size::Sm` 固定。チェックマーク・trigger アイコンの双方をこの 1 つの
/// ヘルパへ統一する）。`attrs` は呼び出し側の CSS フック注入用
/// （trigger アイコンは `data-blocks-feature-vertical-tabs-trigger-icon`
/// を渡す）。
fn geo_icon(path_d: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        attrs,
        vec![fandhe_frontend_core::el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// チェックマークの幾何アイコン（機能一覧の各項目に添える）。
fn check_icon() -> Node {
    geo_icon("M5 12l4 4L19 7", vec![])
}

/// trigger 先頭アイコン共通の CSS フック属性。
const TRIGGER_ICON_ATTRS: [(&str, &str); 1] =
    [("data-blocks-feature-vertical-tabs-trigger-icon", "")];

/// 歯車の幾何アイコン（`build` タブの trigger に添える）。
fn gear_icon() -> Node {
    geo_icon(
        "M12 8a4 4 0 100 8 4 4 0 000-8z M12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 稲妻の幾何アイコン（`deploy` タブの trigger に添える）。
fn bolt_icon() -> Node {
    geo_icon("M13 3L5 14h5l-1 7 8-11h-5z", TRIGGER_ICON_ATTRS.to_vec())
}

/// 円の幾何アイコン（`observe` タブの trigger に添える）。
fn circle_icon() -> Node {
    geo_icon(
        "M12 3a9 9 0 100 18 9 9 0 000-18z",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 盾の幾何アイコン（`secure` タブの trigger に添える）。
fn shield_icon() -> Node {
    geo_icon(
        "M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6z",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 機能一覧 1 項目分の架空データ。
struct DetailPoint {
    title: &'static str,
    body: &'static str,
}

/// タブ 1 枚分の架空データ（trigger のタイトル/説明 + パネルの見出し/機能
/// 一覧 3 件/画像）。
struct FeatureTab {
    /// タブ識別 value（ASCII kebab-case）。
    value: &'static str,
    /// trigger 先頭に添える自作幾何アイコン（`Block.demo: fn() -> Node` と
    /// 同型の const 互換フィールド）。
    icon: fn() -> Node,
    /// trigger タイトル。
    title: &'static str,
    /// trigger の短い説明。
    summary: &'static str,
    /// パネル見出し。
    panel_title: &'static str,
    points: [DetailPoint; 3],
    image_src: &'static str,
}

/// 4 タブ分のデータ（画像はタブごとに異なる `dummy_assets` 定数を割り当てる）。
const FEATURES: [FeatureTab; 4] = [
    FeatureTab {
        value: "build",
        icon: gear_icon,
        title: "ビルド",
        summary: "型で表現された構造から静的ファイルを組み立てます。",
        panel_title: "決定的なビルド",
        points: [
            DetailPoint {
                title: "外部依存ゼロの描画コア",
                body: "描画コアは外部クレートに依存しません。",
            },
            DetailPoint {
                title: "型で表現するスロットと props",
                body: "コンポーネントの構造は Rust の型で表現されます。",
            },
            DetailPoint {
                title: "決定的な出力",
                body: "同じ入力からは常に同じ静的ファイルを生成します。",
            },
        ],
        image_src: dummy_assets::PRODUCT_SRC,
    },
    FeatureTab {
        value: "deploy",
        icon: bolt_icon,
        title: "デプロイ",
        summary: "単一実行ファイルへまとめて配布できます。",
        panel_title: "単一バイナリ配布",
        points: [
            DetailPoint {
                title: "SSR/SSG を単一実行ファイルへ",
                body: "サーバー機能を単一バイナリへまとめられます。",
            },
            DetailPoint {
                title: "Docker 想定の配布形態",
                body: "コンテナイメージへそのまま組み込めます。",
            },
            DetailPoint {
                title: "オフライン決定性",
                body: "配布物は同一構成から常に同じ内容になります。",
            },
        ],
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureTab {
        value: "observe",
        icon: circle_icon,
        title: "観測",
        summary: "機械検証可能な構成で挙動を追跡します。",
        panel_title: "機械検証可能な構成",
        points: [
            DetailPoint {
                title: "構造マニフェストによる検証",
                body: "依存関係を構造マニフェストが機械検証します。",
            },
            DetailPoint {
                title: "無 JS の静的表示",
                body: "JS ハイドレーションを行わない決定的な表示です。",
            },
            DetailPoint {
                title: "依存グラフ上限の遵守",
                body: "標準構成の依存パッケージ数は上限内に収めます。",
            },
        ],
        image_src: dummy_assets::BACKGROUND_SRC,
    },
    FeatureTab {
        value: "secure",
        icon: shield_icon,
        title: "保護",
        summary: "既定エスケープと限定された unsafe 境界で守ります。",
        panel_title: "既定エスケープと安全な境界",
        points: [
            DetailPoint {
                title: "既定エスケープ",
                body: "テキスト補間は既定でエスケープされます。",
            },
            DetailPoint {
                title: "限定された unsafe 境界",
                body: "描画コア・状態管理コアでは unsafe を使用しません。",
            },
            DetailPoint {
                title: "明示的なオプトイン API",
                body: "エスケープの迂回経路は明示的な API に限られます。",
            },
        ],
        image_src: dummy_assets::LOGO_SRC,
    },
];

/// セクション見出し（heading H3 + リード文）を組み立てる。ページ側が
/// `## Demo` として `h2` を出すため H3 にする（`feature_split_list_image`
/// と同じ判断）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("4 つの機能を確認する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-vertical-tabs-lead", "")],
                vec![core_text(
                    "機能ごとにタブを選択した状態のパネルを並べて掲載しています。",
                )],
            ),
        ],
    )
}

/// trigger の中身（アイコン + span、phrasing content 制約に従う。モジュール
/// doc「trigger 内は phrasing content だけで組む」節参照。`<svg>` は
/// phrasing content に該当するため制約に抵触しない）。
fn trigger_body(tab: &FeatureTab) -> Vec<Node> {
    vec![
        (tab.icon)(),
        span(
            vec![("data-blocks-feature-vertical-tabs-trigger-body", "")],
            vec![
                span(
                    vec![("data-blocks-feature-vertical-tabs-trigger-title", "")],
                    vec![core_text(tab.title)],
                ),
                span(
                    vec![("data-blocks-feature-vertical-tabs-trigger-desc", "")],
                    vec![core_text(tab.summary)],
                ),
            ],
        ),
    ]
}

/// 機能一覧 1 項目（チェックアイコン + タイトル + 説明）。
fn detail_point(point: &DetailPoint) -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-point")],
        vec![
            check_icon(),
            div(
                vec![("class", "blocks-feature-vertical-tabs-point-text")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Bold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.body)],
                    ),
                ],
            ),
        ],
    )
}

/// パネルの構成違い（#2776「パネルを画像主体にした別の形」）。DOM 順を
/// 直接入れ替える（`order` は使わない、`feature_split_list_image` と同じ
/// 判断）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelLayout {
    /// 既定: 見出し → 機能一覧 → 画像（#2775 と同じ、画像は下寄せ）。
    ListFirst,
    /// 画像主体: 見出し → 画像 → 機能一覧（画像は見出しと機能一覧の両側に
    /// 余白を持つ。見出しは `margin: 0` のため上マージンを省くと見出しに
    /// 密着してしまう、#2776 codex-review Medium 是正）。
    ImageFirst,
}

/// パネル画像（レイアウトごとに別の CSS フックを使うため、見出しの直後・
/// 一覧の直前いずれの位置でも margin が競合しない）。
fn panel_image(tab: &FeatureTab, layout: PanelLayout) -> Node {
    let hook = match layout {
        PanelLayout::ListFirst => "data-blocks-feature-vertical-tabs-image",
        PanelLayout::ImageFirst => "data-blocks-feature-vertical-tabs-image-primary",
    };
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Landscape,
            shape: ImageShape::Rounded,
            ..ImageProps::new(tab.image_src, "")
        },
        vec![(hook, "")],
    )
}

/// パネル（content）の中身（見出し H4 + `layout` に応じた一覧/画像の順序）。
fn panel_body(tab: &FeatureTab, layout: PanelLayout) -> Vec<Node> {
    let heading_node = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![core_text(tab.panel_title)],
    );
    let points = div(
        vec![("class", "blocks-feature-vertical-tabs-points")],
        tab.points.iter().map(detail_point).collect(),
    );
    let image_node = panel_image(tab, layout);
    match layout {
        PanelLayout::ListFirst => vec![heading_node, points, image_node],
        PanelLayout::ImageFirst => vec![heading_node, image_node, points],
    }
}

/// 状態違い・パネル形違いの並記に使う見出し（`feature_accordion_image::
/// variant_label` と同一実装）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(label)],
    )
}

/// 実物の `tabs::tabs` を一切使わない非対話タブ列（モジュール doc「無 JS
/// での扱い（実物の `tabs::tabs` は一切使わない）」節・「recipe セレクタ
/// から完全に切り離す理由」節、`feature_tabs_panel::static_tab_list` と
/// 同型の判断）。`role`/`tabindex`/`<button>` に加え、`data-scope`/
/// `data-part`/`data-state`/`data-orientation`（`fandhe_frontend_pre_
/// styled_ui::tabs` の recipe が読むセレクタ）もいずれも持たない `div` の
/// みで構成し、block 固有 class（`blocks-feature-vertical-tabs-tablist`/
/// `-tab`/`-tab-active`）だけで見た目を独自に定義する（recipe の
/// `cursor: pointer`/hover 面/フォーカスリング等インタラクティブ向け
/// スタイルを一切継承しないため操作可能に見えない、Bugbot Medium 是正）。
/// ラベルのみを持つ装飾要素として `aria-hidden="true"` を付与し支援技術の
/// ツリーから除外する（本 block の trigger はいずれもタイトル/説明のみで
/// 進捗等の実情報を持たないため常に付与してよい）。
fn static_tab_list(id_prefix: &'static str, selected: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-tablist")],
        FEATURES
            .iter()
            .map(|tab| {
                let class = if tab.value == selected {
                    "blocks-feature-vertical-tabs-tab blocks-feature-vertical-tabs-tab-active"
                } else {
                    "blocks-feature-vertical-tabs-tab"
                };
                el_owned(
                    "div",
                    vec![
                        ("class".to_string(), class.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                        (
                            "id".to_string(),
                            format!("{id_prefix}-trigger-{0}", tab.value),
                        ),
                    ],
                    trigger_body(tab),
                )
            })
            .collect(),
    )
}

/// 縦並びタブ列（見た目のみ模す）+ 選択中パネル本体を組み立てる（並記
/// インスタンス間で再利用する共通ヘルパ。`id_prefix` は呼び出し側が
/// リテラルで完全指定する）。実物の `tabs::tabs` を使わないため、選択中
/// パネル 1 件のみを直接描画する（他パネルは同 block の別インスタンスで
/// 可視になる、モジュール doc「全パネルを静的に読めるようにする」節）。
fn vertical_tabs(id_prefix: &'static str, selected: &'static str, layout: PanelLayout) -> Node {
    let selected_tab = FEATURES
        .iter()
        .find(|tab| tab.value == selected)
        .unwrap_or_else(|| panic!("unknown tab value: {selected}"));
    let content = el_owned(
        "div",
        vec![
            (
                "class".to_string(),
                "blocks-feature-vertical-tabs-panel".to_string(),
            ),
            (
                "id".to_string(),
                format!("{id_prefix}-content-{0}", selected_tab.value),
            ),
        ],
        panel_body(selected_tab, layout),
    );
    div(
        vec![("class", "blocks-feature-vertical-tabs-tabs-root")],
        vec![static_tab_list(id_prefix, selected), content],
    )
}

/// `feature-vertical-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。[`FEATURES`] の 4 タブ
/// それぞれを選択済みにした 4 インスタンスを縦に並べる（モジュール doc
/// 「全パネルを静的に読めるようにする」節参照。#2776 の codex-review P1
/// 是正）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-layout")],
        vec![
            section_header(),
            variant_label("ビルド（機能一覧が主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-build",
                "build",
                PanelLayout::ListFirst,
            ),
            variant_label("デプロイ（画像主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-deploy",
                "deploy",
                PanelLayout::ImageFirst,
            ),
            variant_label("観測（機能一覧が主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-observe",
                "observe",
                PanelLayout::ListFirst,
            ),
            variant_label("保護（画像主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-secure",
                "secure",
                PanelLayout::ImageFirst,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-vertical-tabs/",
    title: "feature-vertical-tabs",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_vertical_tabs.rs",
    demo_class: "blocks-feature-vertical-tabs",
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
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_vertical_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-vertical-tabs-*` と `[data-blocks-feature-vertical-tabs-*]`
/// のみで完結する。実物のコンポーネントである `image`（`[data-scope="image"]
/// [data-part="root"][data-blocks-feature-vertical-tabs-image*]`）を除き、
/// `[data-scope=...]`/`[data-part=...]` セレクタは一切使わない（モジュール
/// doc「recipe セレクタから完全に切り離す理由」節、Bugbot Medium 是正）。
/// 他 block や部品の素のセレクタへは影響させない。
///
/// `.blocks-feature-vertical-tabs-tabs-root` へ `align-items` を明示しない
/// （`flex-start` を指定すると `.blocks-feature-vertical-tabs-tablist`/
/// `-panel` が root の高さへストレッチされず、`tablist` の
/// `border-inline-end`（区切り線）が `panel` 全体の高さに沿わない。既定
/// `align-items: normal`（flex コンテナでは `stretch` として解決される）を
/// 維持することで両側の高さが揃う。旧・実物の `tabs` recipe 継承時代から
/// 引き継ぐ判断、`fandhe_frontend_pre_styled_ui::tabs` recipe 側コメント
/// 参照）。
const LAYOUT_CSS: &str = "\
.blocks-feature-vertical-tabs-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-vertical-tabs-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-lead] {\n  margin: 0;\n}\n\
.blocks-feature-vertical-tabs-tabs-root {\n  display: flex;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-vertical-tabs-tablist {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 0 0 auto;\n  max-width: 20rem;\n  border-inline-end: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-feature-vertical-tabs-tab {\n  display: inline-flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  margin-inline-end: -1px;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  line-height: var(--fandhe-font-line-height-normal);\n  color: var(--fandhe-color-fg-muted);\n  text-align: start;\n  white-space: normal;\n  border-inline-end: 2px solid transparent;\n}\n\
.blocks-feature-vertical-tabs-tab-active {\n  color: var(--fandhe-color-fg);\n  border-inline-end-color: var(--fandhe-palette, var(--fandhe-color-accent));\n}\n\
@media (forced-colors: active) {\n  \
.blocks-feature-vertical-tabs-tab-active {\n    border-inline-end-color: CanvasText;\n  }\n\
}\n\
.blocks-feature-vertical-tabs-panel {\n  flex: 1;\n  min-width: 0;\n  color: var(--fandhe-color-fg);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-title] {\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-desc] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-icon] {\n  flex-shrink: 0;\n}\n\
.blocks-feature-vertical-tabs-points {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block: var(--fandhe-space-4) 0;\n}\n\
.blocks-feature-vertical-tabs-point {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-vertical-tabs-point-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-image] {\n  display: block;\n  width: 100%;\n  margin-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-image-primary] {\n  display: block;\n  width: 100%;\n  margin: var(--fandhe-space-4) 0;\n}\n\
@media (max-width: 63.99rem) {\n  \
.blocks-feature-vertical-tabs-tabs-root {\n    flex-direction: column;\n  }\n  \
.blocks-feature-vertical-tabs-tablist {\n    max-width: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。実物の `tabs::tabs` を使わないため
    /// `<button>` は出力しない（モジュール doc「無 JS での扱い（実物の
    /// `tabs::tabs` は一切使わない）」節）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
            "class=\"blocks-feature-vertical-tabs-tablist\"",
            "class=\"blocks-feature-vertical-tabs-tab\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<button"));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 実物の `tabs::tabs` を一切使わず、その recipe が読むセレクタ
    /// （`data-scope="tabs"`/`data-part="..."`/`data-state`/
    /// `data-orientation`）も一切出力しないこと（Bugbot Medium 是正の回帰
    /// 固定: セレクタが一致するだけで `pre-styled-ui.css` の `cursor:
    /// pointer`/hover 面を継承し、無 JS で実際には切り替わらないのに
    /// 操作可能に見えてしまっていた。モジュール doc「recipe セレクタから
    /// 完全に切り離す理由」節）。`role="tablist"`/`role="tab"`/
    /// `role="tabpanel"`/`tabindex`/`aria-selected`/`aria-controls`/
    /// `disabled`/`aria-disabled`/`hidden`/`aria-orientation` も一切
    /// 出力しないこと（codex-review 指摘の回帰固定）。
    #[test]
    fn no_tabs_instance_is_interactive() {
        let html = render(&demo());
        for forbidden in [
            "data-scope=\"tabs\"",
            "data-part=\"list\"",
            "data-part=\"trigger\"",
            "data-part=\"content\"",
            "data-state=",
            "data-orientation",
            "role=\"tablist\"",
            "role=\"tab\"",
            "role=\"tabpanel\"",
            "tabindex",
            "aria-selected",
            "aria-controls",
            "aria-labelledby",
            "aria-orientation",
            "disabled",
            "aria-disabled",
            " hidden",
        ] {
            assert!(
                !html.contains(forbidden),
                "demo output should never contain {forbidden}"
            );
        }
    }

    /// [`super::FEATURES`] の全 4 タブが、それぞれ自分専用のインスタンス
    /// のパネル本文として実際に出力されること（全パネルが静的に到達可能
    /// であることの固定、モジュール doc「全パネルを静的に読めるようにする」
    /// 節）。各インスタンスの id 接頭辞が選択タブの `value` と一致する設計
    /// （「id 規約」節）のため、`id="...-<value>-content-<value>"` は全域で
    /// ちょうど 1 回だけ現れる。
    #[test]
    fn demo_makes_every_feature_panel_reachable_without_js() {
        let html = render(&demo());
        for tab in super::FEATURES {
            let panel_id = format!(
                "id=\"blocks-feature-vertical-tabs-{0}-content-{0}\"",
                tab.value
            );
            assert!(
                html.contains(&panel_id),
                "panel for {} should exist and be reachable: {panel_id}",
                tab.value
            );
        }
        // 実物の `tabs::tabs` を使わないため、非選択パネルの本文は同じ
        // インスタンス内には存在しない（`class="blocks-feature-vertical-
        // tabs-panel"` は 4 インスタンス分＝4 件のみ。
        // `no_tabs_instance_is_interactive` が `hidden` 属性の不在を
        // 別途固定する）。
        assert_eq!(
            html.matches("class=\"blocks-feature-vertical-tabs-panel\"")
                .count(),
            4
        );
    }

    /// 4 インスタンスそれぞれの id 接頭辞が選択タブの `value` と一致する
    /// こと（全域で一意になることも兼ねて確認する、モジュール doc「id 規約」
    /// 節）。
    #[test]
    fn demo_uses_expected_id_prefix() {
        let html = render(&demo());
        for tab in super::FEATURES {
            assert!(html.contains(&format!(
                "id=\"blocks-feature-vertical-tabs-{0}-trigger-{0}\"",
                tab.value
            )));
            assert!(html.contains(&format!(
                "id=\"blocks-feature-vertical-tabs-{0}-content-{0}\"",
                tab.value
            )));
        }
    }

    /// trigger（`class="blocks-feature-vertical-tabs-tab"`/`"...tab-active"`
    /// の `<div>...</div>` 区間、実物の `tabs::tabs` を使わないため
    /// `<button>` ではない）が phrasing content のみで構成されること
    /// （モジュール doc「trigger は phrasing content だけで組む」節の
    /// 不変条件）。`blocks-feature-vertical-tabs-tab` は
    /// `blocks-feature-vertical-tabs-tablist`/`-tabs-root` と文字列としては
    /// 前方一致してしまうため、`class="..."` の完全一致リテラル 2 種
    /// （非選択/選択済み）のみを対象にし誤検知を避ける。
    #[test]
    fn trigger_contains_no_block_level_elements() {
        let html = render(&demo());
        let mut checked = 0;
        for needle in [
            "class=\"blocks-feature-vertical-tabs-tab\"",
            "class=\"blocks-feature-vertical-tabs-tab blocks-feature-vertical-tabs-tab-active\"",
        ] {
            let mut start = 0;
            while let Some(attr_rel) = html[start..].find(needle) {
                let attr_pos = start + attr_rel;
                let open = html[..attr_pos]
                    .rfind('<')
                    .expect("class attribute should be inside an opening tag");
                let open_end = html[open..].find('>').map(|i| open + i + 1).unwrap();
                let close = html[open_end..]
                    .find("</div>")
                    .map(|i| open_end + i)
                    .unwrap();
                let inner = &html[open_end..close];
                // `"<p"` 単体だと trigger アイコンの `<path>`（svg 要素、
                // phrasing content として許容される）を誤検知するため
                // `"<p "`（属性付き `<p ...>` の開きタグのみ）で判定する
                // （`styled_text::text` は常に `class`/`data-*` 属性を持つ
                // ため属性なし `<p>` は存在しない）。
                assert!(
                    !inner.contains("<p "),
                    "trigger must not contain <p>: {inner}"
                );
                assert!(
                    !inner.contains("<h"),
                    "trigger must not contain heading tags: {inner}"
                );
                assert!(
                    !inner.contains("<div"),
                    "trigger must not contain nested block elements: {inner}"
                );
                start = close + "</div>".len();
                checked += 1;
            }
        }
        assert_eq!(checked, 16);
    }

    /// trigger アイコンが 4 インスタンス × 4 タブ分（16 件）出力されること
    /// （モジュール doc「trigger 先頭のアイコン」節）。
    #[test]
    fn demo_renders_trigger_icon_for_every_tab_in_every_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-vertical-tabs-trigger-icon=\"\"")
                .count(),
            16
        );
    }

    /// [`super::PanelLayout::ListFirst`] のインスタンス（`build`/`observe`）
    /// は機能一覧が画像より DOM 順で先に現れ、[`super::PanelLayout::
    /// ImageFirst`] のインスタンス（`deploy`/`secure`）は画像が機能一覧
    /// より先に現れること（モジュール doc「全パネルを静的に読めるように
    /// する」節の不変条件）。
    #[test]
    fn demo_orders_panel_image_and_points_per_instance_layout() {
        let html = render(&demo());
        for (value, image_hook) in [
            ("build", "data-blocks-feature-vertical-tabs-image=\"\""),
            ("observe", "data-blocks-feature-vertical-tabs-image=\"\""),
        ] {
            let content_start = html
                .find(&format!(
                    "id=\"blocks-feature-vertical-tabs-{value}-content-{value}\""
                ))
                .unwrap_or_else(|| panic!("{value} content should exist"));
            let panel = &html[content_start..];
            let points_index = panel
                .find("blocks-feature-vertical-tabs-points")
                .expect("ListFirst panel should render the points list");
            let image_index = panel
                .find(image_hook)
                .expect("ListFirst panel should render the image hook");
            assert!(
                points_index < image_index,
                "{value} (ListFirst) should render points before the image"
            );
        }
        for value in ["deploy", "secure"] {
            let content_start = html
                .find(&format!(
                    "id=\"blocks-feature-vertical-tabs-{value}-content-{value}\""
                ))
                .unwrap_or_else(|| panic!("{value} content should exist"));
            let panel = &html[content_start..];
            let image_index = panel
                .find("data-blocks-feature-vertical-tabs-image-primary=\"\"")
                .expect("ImageFirst panel should render the ImageFirst image hook");
            let points_index = panel
                .find("blocks-feature-vertical-tabs-points")
                .expect("ImageFirst panel should render the points list");
            assert!(
                image_index < points_index,
                "{value} (ImageFirst) should render the image before points"
            );
        }
    }

    /// [`LAYOUT_CSS`] が lg 未満で root を縦積みへ切り替える上書きを持ち、
    /// `list` の軸（縦積み）を崩さないこと（モジュール doc「レスポンシブ」
    /// 節）。
    #[test]
    fn layout_css_declares_lg_breakpoint_overrides() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 63.99rem)"));
        assert!(LAYOUT_CSS
            .contains(".blocks-feature-vertical-tabs-tabs-root {\n    flex-direction: column;"));
        assert!(
            LAYOUT_CSS.contains(".blocks-feature-vertical-tabs-tablist {\n    max-width: none;")
        );
        assert!(!LAYOUT_CSS.contains("flex-direction: row;"));
        // recipe セレクタ（`[data-scope=...]`/`[data-part=...]`）は tabs
        // 部分から完全に排除されていること（モジュール doc「recipe
        // セレクタから完全に切り離す理由」節、Bugbot Medium 是正の回帰
        // 固定）。`image` 向けの `[data-scope="image"]...` セレクタは実物の
        // コンポーネントのため対象外とし、`tabs`/`list`/`trigger`/`content`
        // という値のみを対象にする。
        for forbidden in [
            "[data-scope=\"tabs\"]",
            "[data-part=\"list\"]",
            "[data-part=\"trigger\"]",
            "[data-part=\"content\"]",
            "cursor: pointer",
        ] {
            assert!(
                !LAYOUT_CSS.contains(forbidden),
                "LAYOUT_CSS should never contain {forbidden}"
            );
        }
    }

    /// 各インスタンスにつき選択中タブ 1 件のみが `blocks-feature-vertical-
    /// tabs-tab-active` を追加で持ち、残り 3 件は `blocks-feature-vertical-
    /// tabs-tab` のみになること（4 インスタンス分、モジュール doc「無 JS
    /// での扱い（実物の `tabs::tabs` は一切使わない）」節）。強調表示は
    /// この class 差だけで CSS へ伝え、`disabled`/`aria-disabled`（実物の
    /// `tabs::tabs` が持っていた「操作できない見た目」）は
    /// [`no_tabs_instance_is_interactive`] のとおり一切出力しない。
    #[test]
    fn demo_marks_exactly_one_active_trigger_per_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches("blocks-feature-vertical-tabs-tab-active")
                .count(),
            4
        );
        assert_eq!(
            html.matches("class=\"blocks-feature-vertical-tabs-tab\"")
                .count(),
            12
        );
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_split_list_image`/`feature_image_cards` と
    /// 同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-vertical-tabs-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-vertical-tabs-layout"
        );
    }
}
