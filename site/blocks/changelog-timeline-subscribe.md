# changelog-timeline-subscribe

`heading` / `text` / `field` / `input` / `button` / `badge` / `timeline` / `list` / `visually_hidden` の 9 部品を合成した、購読フォーム付きのタイムライン型 changelog レイアウトです。中央寄せの見出し・リード文の下にメールアドレス入力欄とボタンを横に連結した購読フォームを配置し、その下に「日付列 / コネクタ（indicator + 縦線）/ 本文列」の 3 列タイムラインでリリースを並べます。

- 静的表示です。`<form>` は使わず、購読ボタンは `type="button"` のまま送信処理・バリデーションを持ちません。実際の購読処理は利用側の Rust/JS コードで実装します。
- 可視ラベルは出さず、`visually_hidden` で包んだラベルと `<label for>` の関連付けでメール入力欄のアクセシブル名を確保しています（イシュー本文の部品一覧にはない部品ですが、この目的のために追加しました）。
- 各リリースの日付は日付列と本文列の 2 箇所に出力し、CSS でどちらか一方だけを常に表示します（`display: none` は支援技術からも隠れるため、読み上げの重複は起きません）。
- 狭い画面（`< 48rem`）では日付列を隠して本文列内の日付表示へ切り替えますが、購読フォームは横並びのまま維持します（縦積みにはしません）。
- 文言・バージョン番号・日付・変更点はすべて架空のものです。
- 集約元は 1 件のみです（対応表 ID R0042）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    changes: &'static [&'static str],
}

/// リリース一覧（架空、3 件）。
const RELEASES: [Release; 3] = [
    Release {
        version: "v3.6.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "購読フォーム付きタイムライン表示を追加",
        tags: &["新機能"],
        changes: &[
            "更新履歴を時系列のタイムラインとして表示するレイアウトを追加",
            "メールアドレスで新着リリースを購読できる導線を追加",
        ],
    },
    Release {
        version: "v3.5.2",
        date_iso: "2026-09-08",
        date_label: "2026年9月8日",
        title: "狭い画面での表示崩れを修正",
        tags: &["修正"],
        changes: &["狭い幅で日付列が本文と重なる表示崩れを修正"],
    },
    Release {
        version: "v3.5.0",
        date_iso: "2026-08-25",
        date_label: "2026年8月25日",
        title: "変更点の種別タグ表示に対応",
        tags: &["改善", "内部"],
        changes: &[
            "変更点ごとに新機能・改善・修正の種別タグを表示",
            "リリース一覧の内部データ構造を整理",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`changelog_accordion` と同じ判断）。
fn release_date(class: &'static str, iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", class), ("datetime", iso)],
        vec![text(label)],
    )
}

/// 中央寄せの見出し + リード文 + 購読フォーム（`header` 領域）。
fn header() -> Node {
    const EMAIL_FIELD_ID: &str = "blocks-changelog-timeline-subscribe-email";
    let email_field = FieldProps {
        id: EMAIL_FIELD_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    div(
        vec![("class", "blocks-changelog-timeline-subscribe-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("更新履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "新しいリリースをメールでお届けします。いつでも購読解除できます。",
                )],
            ),
            div(
                vec![("class", "blocks-changelog-timeline-subscribe-form")],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![("data-blocks-changelog-timeline-subscribe-field", "")],
                        vec![
                            visually_hidden::root(
                                vec![],
                                vec![field::label(
                                    &email_field,
                                    vec![],
                                    vec![text("メールアドレス")],
                                )],
                            ),
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![
                                    ("data-blocks-changelog-timeline-subscribe-input", ""),
                                    ("type", "email"),
                                    ("autocomplete", "email"),
                                    ("placeholder", "you@example.com"),
                                ],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-changelog-timeline-subscribe-submit", "")],
                        vec![text("購読する")],
                    ),
                ],
            ),
        ],
    )
}

/// リリース 1 件分のタイトル行（version + 種別 badge 群）。
fn release_head(release: &Release) -> Node {
    let mut children = vec![span(
        vec![("class", "blocks-changelog-timeline-subscribe-version")],
        vec![text(release.version)],
    )];
    children.extend(release.tags.iter().map(|tag| {
        badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-changelog-timeline-subscribe-tag", "")],
            vec![text(*tag)],
        )
    }));
    div(
        vec![("class", "blocks-changelog-timeline-subscribe-release-head")],
        children,
    )
}

/// リリース 1 件分の本文（`timeline::content`。狭幅用インライン日付 +
/// バージョン行 + タイトル + 変更点リスト）。
fn release_body(release: &Release) -> Node {
    let changes = list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-timeline-subscribe-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    );

    timeline::content(
        vec![],
        vec![
            release_date(
                "blocks-changelog-timeline-subscribe-inline-date",
                release.date_iso,
                release.date_label,
            ),
            release_head(release),
            timeline::title(vec![], vec![text(release.title)]),
            changes,
        ],
    )
}

/// リリース 1 件分の item（日付列 + connector + 本文列）。
fn release_item(index: usize, release: &Release) -> Node {
    let is_last = index + 1 == RELEASES.len();

    let date_col = timeline::content(
        vec![("data-blocks-changelog-timeline-subscribe-date-col", "")],
        vec![release_date(
            "blocks-changelog-timeline-subscribe-date",
            release.date_iso,
            release.date_label,
        )],
    );

    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", "complete")],
        vec![],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }

    timeline::item(
        vec![("data-blocks-changelog-timeline-subscribe-item", "")],
        vec![
            date_col,
            timeline::connector(vec![], connector_children),
            release_body(release),
        ],
    )
}

/// `changelog-timeline-subscribe` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| release_item(index, release))
        .collect();

    let timeline_node = div(
        vec![("class", "blocks-changelog-timeline-subscribe-timeline")],
        vec![timeline::root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![("data-blocks-changelog-timeline-subscribe-root", "")],
            items,
        )],
    );

    div(
        vec![("class", "blocks-changelog-timeline-subscribe-layout")],
        vec![header(), timeline_node],
    )
}
```

## 原案差分メモ

- 見出しレベルを 1 段下げました（ページ側の `## Demo` に合わせるため、セクション見出しは `h3`）。
- `<form>`/submit を持たず、購読ボタンは `type="button"` の静的表示にしました。
- 可視ラベルの代わりに `visually_hidden` + `field::label` を追加し、`<label for>` の関連付けでアクセシブル名を確保しました。
- 変更ごとの画像は使わず、タイトル + 変更点リスト + 種別 badge のみに簡略化しました。
- 日付の二重出力を、CSS で片方のみを常に表示する形で再現しました（狭い幅では日付列を隠し本文列内へ切り替え）。
- 購読フォームの入力欄とボタンは、隣接辺の角丸を打ち消して 1 本の連結コントロールに見えるようにしました。
- 文言をすべて独自に書き直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- 日付には機械可読な `datetime` 属性を付けています。
