# empty-state-card-header

`fandhe-frontend-pre-styled-ui` の `card` / `empty-state` / `button` /
`dialog` / `field` / `input` / `native-select` 部品を合成した、ヘッダー付き
カード内の空状態の合成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R0375、集約元は R0234 の 2 件です。
出典の固有名・ファイル名は記載しません）。

カードは一覧が入る領域と同じ幅（デモ枠全幅）で表示し、ヘッダーに題名と
「新規作成」ボタン、本文に空状態（アイコン・見出し・説明・「最初の
プロジェクトを作成」ボタン）を中央寄せで配置します。「新規作成」を
押した後のダイアログの見た目は、無 JS の docs サイトでは開閉トグルを
表現できないため、カードの下にもう 1 インスタンスとして静的併記します
（1 つ目がカード単体 = R0375、2 つ目が作成ダイアログの開いた状態 =
R0234）。

本 Demo は静的な表示例であり、開閉・フォーカストラップ・Escape キー等の
挙動は一切扱いません。`<form>` 要素は出力せず、ボタンはすべて
`type="button"` のままで、送信先・入力値検証・状態管理は一切持ちません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI
コンポーネント層はアプリケーションロジックを内包しません。実際に送信
処理を実装する場合は、利用者自身の Rust/JS コードで実装してください）。

画面幅が狭いときはダイアログのフッターボタンを縦積みにします。

## Rust コード

```rust
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::empty_state::{
    self, EmptyStateIndicatorVariant, EmptyStateProps,
};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（フォルダに「+」を重ねた単純な折れ線・矩形。
/// `hero_email_signup::play_icon` と同型の装飾用 SVG、`aria-hidden="true"`）。
/// 実在ブランドのロゴ・商標は模さない。
fn folder_plus_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 24 24"),
            ("width", "1em"),
            ("height", "1em"),
            ("aria-hidden", "true"),
        ],
        vec![
            el(
                "path",
                vec![
                    (
                        "d",
                        "M3 6a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6z",
                    ),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M12 11v5M9.5 13.5h5"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// カード単体インスタンス（R0375・主参照）。ヘッダーに題名 + 作成ボタン、
/// 本文に空状態（アイコン・見出し・説明・作成ボタン）を中央寄せで置く。
fn card_instance() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-empty-state-card-header-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("プロジェクト")]),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("新規作成")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![empty_state::root(
                    &EmptyStateProps::default(),
                    vec![("data-blocks-empty-state-card-header-empty", "")],
                    vec![empty_state::content(
                        vec![],
                        vec![
                            empty_state::indicator_with(
                                EmptyStateIndicatorVariant::Boxed,
                                vec![],
                                vec![folder_plus_icon()],
                            ),
                            empty_state::title(vec![], vec![text("プロジェクトがありません")]),
                            empty_state::description(
                                vec![],
                                vec![text(
                                    "最初のプロジェクトを作成すると、ここに一覧が表示されます。",
                                )],
                            ),
                            empty_state::actions(
                                vec![],
                                vec![button::button(
                                    &ButtonProps::default(),
                                    vec![],
                                    vec![text("最初のプロジェクトを作成")],
                                )],
                            ),
                        ],
                    )],
                )],
            ),
        ],
    )
}

/// 作成ダイアログインスタンス（R0234・集約元）。`contact-dialog-form` と
/// 同型の「既に開いた静的な初期状態のみを描く」構成。
fn dialog_instance() -> Node {
    let title_id = "blocks-empty-state-card-header-dialog-title";
    let description_id = "blocks-empty-state-card-header-dialog-description";

    let name_field = FieldProps {
        id: "blocks-empty-state-card-header-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let visibility_field = FieldProps {
        id: "blocks-empty-state-card-header-visibility",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-empty-state-card-header-dialog-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側にカードインスタンス・
                    // 説明・コードがあるため、表示実態と一致させ aria-modal
                    // は false にする（`contact-dialog-form` と同じ判断）。
                    false,
                    ContentIds {
                        id: Some("blocks-empty-state-card-header-dialog-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-empty-state-card-header-dialog-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("プロジェクトを作成")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text("プロジェクト名と公開範囲を指定してください。")],
                        ),
                        dialog::body(
                            vec![("data-blocks-empty-state-card-header-dialog-body", "")],
                            vec![field::group(
                                vec![],
                                vec![
                                    field::root(
                                        &orientation,
                                        &name_field,
                                        vec![],
                                        vec![
                                            field::label(
                                                &name_field,
                                                vec![],
                                                vec![text("プロジェクト名")],
                                            ),
                                            input::input(
                                                &InputProps::default(),
                                                &name_field,
                                                vec![
                                                    ("type", "text"),
                                                    ("placeholder", "新しいプロジェクト"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &visibility_field,
                                        vec![],
                                        vec![
                                            field::label(
                                                &visibility_field,
                                                vec![],
                                                vec![text("公開範囲")],
                                            ),
                                            native_select::native_select(
                                                &NativeSelectProps::default(),
                                                &visibility_field,
                                                vec![],
                                                vec![
                                                    el(
                                                        "option",
                                                        vec![
                                                            ("value", "private"),
                                                            ("selected", "selected"),
                                                        ],
                                                        vec![text("非公開")],
                                                    ),
                                                    el(
                                                        "option",
                                                        vec![("value", "team")],
                                                        vec![text("チームのみ")],
                                                    ),
                                                    el(
                                                        "option",
                                                        vec![("value", "public")],
                                                        vec![text("公開")],
                                                    ),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            )],
                        ),
                        dialog::footer(
                            vec![("data-blocks-empty-state-card-header-dialog-footer", "")],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("キャンセル")],
                                ),
                                button::button(&ButtonProps::default(), vec![], vec![text("作成")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `empty-state-card-header` の Demo 本体。カード単体（R0375）と作成
/// ダイアログ（R0234）を縦積みで静的併記する（モジュール doc「2 インス
/// タンスの静的併記」節参照）。
pub fn demo() -> Node {
    el(
        "div",
        vec![("class", "blocks-empty-state-card-header-layout")],
        vec![card_instance(), dialog_instance()],
    )
}
```

## 原案差分メモ

参照（主参照 R0375・集約元 R0234。出典の固有名・ファイル名は記載しません）
から取り込んだのは構造（領域の配置と部品構成）のみであり、次の点を独自に
設計・変更しています。

- R0375（カード単体）と R0234（作成ダイアログ付き）の差分は、docs サイトが
  JS ハイドレーションを行わない設計のため、開閉トグルではなく 2 インス
  タンスの静的併記で表現しています。
- ダイアログは開閉トリガーを持たず、既に開いた静的な初期状態のみを描き
  ます（原案は開閉可能なインタラクティブなダイアログですが、無 JS の
  ため）。
- `aria-modal` は `false` にしています。静的なデモは閉じる機構を持たず、
  ダイアログの外側にカードインスタンス・説明・コードがあるため、支援
  技術が外側を無視しないよう表示の実態に合わせました。
- ボタンはすべて `<form>` との関連付け（`form=` 属性）を持たず、
  `type="button"` のままにしています。送信処理・入力値検証は一切
  実装していません。
- 空状態のアイコンは参照元のアイコンを転記せず、独自の幾何学的な
  フォルダ + プラス記号の SVG を描いています。
- `dialog::body` の既定 CSS（`max-height: 50vh; overflow-y: auto`）は、
  入力欄 2 件のみの本 Demo では不要な縦スクロールを生むため、
  `max-height: none; overflow: visible;` へ上書きしています。
- 文言（題名・見出し・説明文・ラベル・プレースホルダー・ボタンラベル）は
  すべて独自に書き直しました。実在の人物・企業・実クレデンシャルは一切
  含みません。

関連情報: [Card](../themes/card.md) / [Empty State](../themes/empty-state.md) /
[Button](../themes/button.md) / [Dialog](../themes/dialog.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Native Select](../themes/native-select.md)
