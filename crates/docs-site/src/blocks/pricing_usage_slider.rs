//! `pricing-usage-slider` block（イシュー #2547。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/pricing-sections` 由来の合成例で、
//! `crate::blocks` モジュール doc の契約を `pricing_tiers_morph` に続いて
//! 9 件目に実装する）。
//!
//! # 使用部品
//!
//! `slider`（利用量の目盛り）+ `stat`（連動する価格表示）を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 静的な組であることの明示（実行時計算・ライブ連動を持たない）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、`slider` は他の
//! block と同様に常に固定値で描画される（`fandhe-frontend-wasm-full` を
//! ハイドレートしない限り操作できない）。本 block は「slider の値と stat
//! の価格表示が同じ値を指し示す静的な組」を SSR で描画するのみであり、
//! 「slider の値を実行時に stat へ反映する」ライブ連動そのものは実装しない。
//! これは `docs/policy/intentional-non-adoption.md` §3.25 の責務境界（UI
//! コンポーネント層はバリデーション・データ整形等のアプリケーションロジック
//! を内包しない。数値・日時整形は UI コンポーネント層の責務外という判断軸
//! の一般化）に従う判断であり、**実運用でスライダーの `input`/`change` を
//! 価格表示へ反映する処理は、それを組み込む利用者アプリケーションの実装
//! 範囲**である（例: `fandhe-frontend-wasm-full` の dispatch を購読して
//! `stat::value_text` のテキストを書き換えるアプリ側コード）。
//!
//! 価格テーブル（[`price_for`]）は決定的な固定関数（外部入力・ユーザー入力
//! を一切受け取らない `&'static str` の範囲テーブル）であり、数値のカウント
//! アップアニメーション（別 issue #2539 の担当）とは無関係である。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。プラン名・価格・利用量の目盛りはすべて架空のものであり、
//! 実企業名・実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `slider::root`/`stat::root` は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-pricing-usage-slider-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節参照）。一方
//! `stat::label`/`value_text`/`help_text`・素の `div` には呼び出し側
//! `attrs` がそのまま連結されるため、それらは従来どおりクラスセレクタを
//! 使う。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::slider::{self, SliderProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 利用量（月間リクエスト数、架空の目盛り）から価格表示を導く決定的な固定
/// テーブル。範囲は 4 段のプリセット目盛り（10/50/100/500）を跨ぐしきい値
/// で区切っており、外部入力・ユーザー入力を一切受け取らない。
#[must_use]
fn price_for(units: u32) -> &'static str {
    match units {
        0..=10 => "$9",
        11..=50 => "$29",
        51..=100 => "$49",
        _ => "$199",
    }
}

/// `pricing-usage-slider` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。スライダーは 4 段のプリセット目盛り（10/50/100/500）の中間
/// （50）に固定した初期値で描画し、対応する価格を [`price_for`] から
/// 求めて `stat` へ表示する。
pub fn demo() -> Node {
    let props = SliderProps::default();
    let selected_units = 50.0_f64;
    let state = Slider::new(0.0, 500.0, 10.0, selected_units, Orientation::Horizontal);

    let slider_node = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        &props,
        vec![("data-blocks-pricing-usage-slider-slider", "")],
        vec![
            slider::label(&props, vec![], vec![text("月間リクエスト数（千件）")]),
            slider::control(
                Orientation::Horizontal,
                &props,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        &props,
                        vec![],
                        vec![slider::range(&state, &props, vec![])],
                    ),
                    slider::thumb_styled(&state, Some("50 千件"), &props, vec![]),
                    slider::marker_group(
                        vec![],
                        vec![
                            slider::marker(&state, 10.0, false, vec![], vec![]),
                            slider::marker(&state, 50.0, false, vec![], vec![]),
                            slider::marker(&state, 100.0, false, vec![], vec![]),
                            slider::marker(&state, 500.0, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            slider::hidden_input("usage-units", "50", false, vec![]),
        ],
    );

    let stat_node = stat::root(
        Size::Lg,
        vec![("data-blocks-pricing-usage-slider-stat", "")],
        vec![
            stat::label(vec![], vec![text("想定コスト")]),
            stat::value_text(
                vec![],
                vec![text(price_for(selected_units as u32)), text(" / 月")],
            ),
            stat::help_text(
                vec![],
                vec![text(
                    "スライダーの選択位置（50 千件/月）に対応する固定表示です",
                )],
            ),
        ],
    );

    div(
        vec![("data-blocks-pricing-usage-slider-layout", "")],
        vec![slider_node, stat_node],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-usage-slider/",
    title: "pricing-usage-slider",
    rust_source: "crates/docs-site/src/blocks/pricing_usage_slider.rs",
    demo_class: "blocks-pricing-usage-slider",
    parts: &[
        Part {
            label: "Slider",
            path: "/themes/slider/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
    ],
    demo,
};

/// `pricing_usage_slider` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
[data-blocks-pricing-usage-slider-layout] {\n  display: flex;\n  flex-direction: column;\n  gap: 2rem;\n  align-items: stretch;\n  max-width: 28rem;\n}\n\
[data-blocks-pricing-usage-slider-slider] {\n  width: 100%;\n}\n\
[data-blocks-pricing-usage-slider-stat] {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  padding: 1rem 1.25rem;\n}\n";
