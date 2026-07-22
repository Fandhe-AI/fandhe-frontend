//! slot recipe（chakra-ui の slot recipe 相当）: 複数 anatomy パーツ（slot）を
//! 横断する variant 定義から、クラス名と静的 CSS を決定的に生成する。
//!
//! [`crate::css`] の低レベル宣言・検証を使い、`fandhe-frontend-headless-ui`
//! の `data-scope` / `data-part` セレクタ（`crates/headless-ui/src/anatomy.rs`）
//! と接続する CSS 規則を組み立てる。イシュー #550/#551 の styled 部品
//! （Button・Dialog ラッパー等）はここで定義した [`SlotRecipe`] を通じて
//! 「どの HTML 要素にどのクラスを付けるか」を決定する契約になる。
//!
//! # 順序規約（決定性の根拠）
//!
//! 内部ストレージは `Vec` のみを使い、`HashMap`/`HashSet` は使わない
//! （反復順序がプロセスごとに変わりうる型を持ち込まない）。[`SlotRecipe::css`]
//! の出力順は「base（`slots` の宣言順）→ variants（登録順）→ compound variants
//! （登録順、イシュー #604）」に固定し、同一 slot・同一 axis/value への複数回
//! 登録は「後に登録された規則が CSS 中で後に出力される」（CSS のカスケードに
//! おいて後勝ちになる）という素直な規約に従う。この規約より複雑な優先順位
//! 判定は行わない。
//!
//! compound variant（[`SlotRecipe::compound_variant`]）は複数軸の条件を
//! `.fd-<scope>--<a1>-<v1>.fd-<scope>--<a2>-<v2>...` のように連結したセレクタ
//! として出力する。条件 2 個以上なら詳細度が単一 variant セレクタより必ず
//! 高くなるため記述順に依存せず上書きが決まり、条件 1 個の場合でも compound
//! ブロックを variants ブロックより後に出力するため CSS カスケードの後勝ちで
//! 上書きされる（chakra-ui の「compoundVariants は variants を上書きする」
//! 意味論に対応する 2 段の保証）。
//!
//! # colorPalette 軸（イシュー #606）
//!
//! [`ColorPalette`] は `size` と並ぶ標準 variant 軸で、[`crate::theme`] の
//! セマンティック色（`accent`/`info`/`success`/`warning`/`danger`）と 1:1
//! 対応する。[`palette_declarations`] が返す宣言を palette 値ごとの
//! `SlotRecipe::variant` として root slot へ登録すると、選択された palette に
//! 応じて `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg`
//! （chakra-ui の virtual token 方式に相当するローカル custom property）が
//! 切り替わる。styled 部品側の色宣言は `var(--fandhe-palette)` 等を参照する
//! だけでよく、palette 軸の追加を機に既存の `var(--fandhe-color-accent)` 直書き
//! を書き換える（Button/Badge/Spinner/Alert、`crate` rustdoc 参照）。

use crate::css::{decl, is_valid_identifier, serialize_rule, Declaration};

/// クラス名プレフィックス（ライブラリ固定）。変更用の API は設けない
/// （`fd-{scope}--{axis}-{value}` の形式を全 styled 部品で一貫させるため）。
const CLASS_PREFIX: &str = "fd";

/// variant 軸 1 個の値を表す enum が実装するトレイト。
///
/// `Size::Sm` のような具象値から `axis()`（例: `"size"`）と `value()`（例:
/// `"sm"`）を取り出せることを要求する。[`SlotRecipe::variant`] /
/// [`SlotRecipe::variant_class`] はこのトレイトを通じてのみ variant を
/// 受け取るため、styled 部品側は生の文字列ではなく型安全な enum を渡す
/// （chakra-ui の `variants: { size: { sm: {...} } }` に対する型安全な代替）。
pub trait VariantValue: Copy {
    /// この値が属す variant 軸の名前（例: `"size"`）。
    fn axis(self) -> &'static str;
    /// この軸におけるこの値の名前（例: `"sm"`）。
    fn value(self) -> &'static str;
}

/// 標準の `size` 軸。#550 以降の styled 部品が共用する最初の具象 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    /// 小サイズ。
    Sm,
    /// 中サイズ（既定値として使われることが多い）。
    Md,
    /// 大サイズ。
    Lg,
}

impl VariantValue for Size {
    fn axis(self) -> &'static str {
        "size"
    }

    fn value(self) -> &'static str {
        match self {
            Size::Sm => "sm",
            Size::Md => "md",
            Size::Lg => "lg",
        }
    }
}

/// 標準の `color-palette` 軸（chakra-ui の `colorPalette` 相当、イシュー #606）。
///
/// [`crate::theme`] の既定パレット（`accent`/`info`/`success`/`warning`/
/// `danger`）と 1:1 対応する。Button/Badge/Spinner がこの軸を公開 API の
/// variant として持ち、[`palette_declarations`] が生成する宣言を通じて
/// `--fandhe-palette` 系のローカル custom property を上書きする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorPalette {
    /// 強調色（既定）。
    #[default]
    Accent,
    /// 情報提供色。
    Info,
    /// 成功色。
    Success,
    /// 警告色。
    Warning,
    /// 危険・エラー色。
    Danger,
}

impl VariantValue for ColorPalette {
    fn axis(self) -> &'static str {
        "color-palette"
    }

    fn value(self) -> &'static str {
        match self {
            ColorPalette::Accent => "accent",
            ColorPalette::Info => "info",
            ColorPalette::Success => "success",
            ColorPalette::Warning => "warning",
            ColorPalette::Danger => "danger",
        }
    }
}

/// `palette` が選択されたときに root slot へ登録する宣言列を返す
/// （chakra-ui の virtual token 方式の静的 CSS 版、イシュー #606）。
///
/// [`crate::theme`] が生成する `--fandhe-color-*`（テーマ層の名前空間）とは
/// 別の `--fandhe-palette-*` 名前空間へ、選択された palette に対応する
/// `accent`/`info`/`success`/`warning`/`danger` の 3 役割（base/emphasized/fg）
/// を `var()` 参照として束ねる。styled 部品側は `var(--fandhe-palette)` 等を
/// 参照するだけで、どの colorPalette が選択されたかに関わらず同じ宣言で
/// 色を切り替えられる。名前空間を分離しているため、ユーザーがカスタム
/// テーマへ独自の `palette` という名の色トークンを追加しても
/// （`Theme::push_color("palette", ...)`）本ヘルパの出力とは衝突しない。
#[must_use]
pub fn palette_declarations(p: ColorPalette) -> Vec<Declaration> {
    match p {
        ColorPalette::Accent => vec![
            decl("--fandhe-palette", "var(--fandhe-color-accent)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-accent-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-accent-fg)"),
        ],
        ColorPalette::Info => vec![
            decl("--fandhe-palette", "var(--fandhe-color-info)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-info-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-info-fg)"),
        ],
        ColorPalette::Success => vec![
            decl("--fandhe-palette", "var(--fandhe-color-success)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-success-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-success-fg)"),
        ],
        ColorPalette::Warning => vec![
            decl("--fandhe-palette", "var(--fandhe-color-warning)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-warning-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-warning-fg)"),
        ],
        ColorPalette::Danger => vec![
            decl("--fandhe-palette", "var(--fandhe-color-danger)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-danger-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-danger-fg)"),
        ],
    }
}

/// slot 1 個への base 宣言登録（内部表現）。
struct BaseRule {
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// slot 1 個・variant 値 1 個への宣言登録（内部表現）。
struct VariantRule {
    axis: &'static str,
    value: &'static str,
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// axis ごとの既定 variant 値（内部表現）。
struct DefaultVariant {
    axis: &'static str,
    value: &'static str,
}

/// compound variant の条件 1 件（axis, value の型消去された組）。
///
/// [`when()`] を通じてのみ [`VariantValue`] 実装 enum から構築できる（生の
/// 文字列を受け付けない。既存 `variant()` と同じ enum ベースの型安全性を
/// [`SlotRecipe::compound_variant`] の条件部でも保つ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariantCondition {
    axis: &'static str,
    value: &'static str,
}

/// `VariantValue` の具象値から [`VariantCondition`] を作る（chakra-ui の
/// `compoundVariants: [{ size: "sm", tone: "outline", css: {...} }]` の
/// 条件部 1 軸分に相当）。[`SlotRecipe::compound_variant`] の `conditions`
/// 引数は本関数の戻り値を並べた `Vec` として組み立てる。
#[must_use]
pub fn when<V: VariantValue>(v: V) -> VariantCondition {
    VariantCondition {
        axis: v.axis(),
        value: v.value(),
    }
}

/// 複数 variant 軸の組み合わせ条件が揃ったときの slot への宣言登録
/// （内部表現。chakra-ui の `compoundVariants` 1 件に相当、イシュー #604）。
struct CompoundVariantRule {
    conditions: Vec<VariantCondition>,
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// slot recipe: `scope`（headless anatomy と同一値）・`slots`・base・variants・
/// defaultVariants を保持し、静的 CSS とクラス名を決定的に生成する。
///
/// # 呼び出し文脈
///
/// `scope` は対応する `fandhe-frontend-headless-ui` の
/// `Anatomy::new(scope)`（例: `crates/headless-ui/src/tabs.rs` の
/// `ANATOMY`）と同じ値を渡す契約とする。これにより [`SlotRecipe::css`] が
/// 生成するセレクタ `[data-scope="<scope>"][data-part="<slot>"]` が、
/// headless 層が実際にレンダリングする属性と一致する（本クレートの
/// `tests/recipe_css.rs` が headless 層の実マークアップと照合して固定する）。
pub struct SlotRecipe {
    scope: &'static str,
    slots: &'static [&'static str],
    base: Vec<BaseRule>,
    variants: Vec<VariantRule>,
    default_variants: Vec<DefaultVariant>,
    compound_variants: Vec<CompoundVariantRule>,
}

impl SlotRecipe {
    /// `scope` と、この recipe が扱う `slots`（anatomy の part 名一覧）を
    /// 指定して空の recipe を作る。
    #[must_use]
    pub const fn new(scope: &'static str, slots: &'static [&'static str]) -> Self {
        Self {
            scope,
            slots,
            base: Vec::new(),
            variants: Vec::new(),
            default_variants: Vec::new(),
            compound_variants: Vec::new(),
        }
    }

    /// 指定した `slot` への base 宣言を登録する（builder、自己消費）。
    ///
    /// `slot` が [`SlotRecipe::new`] で宣言した `slots` に含まれない場合、
    /// この登録は [`SlotRecipe::css`] の出力から除外される（fail-closed。
    /// `slots` 未宣言の slot への意図しない CSS 漏出を防ぐ）。
    #[must_use]
    pub fn base(mut self, slot: &'static str, declarations: Vec<Declaration>) -> Self {
        self.base.push(BaseRule { slot, declarations });
        self
    }

    /// 指定した variant 値 `v` が選択されたときの `slot` への宣言を登録する
    /// （builder、自己消費）。
    ///
    /// `slot` が `slots` に含まれない場合、または `v` の `axis()`/`value()`
    /// が識別子として不正な場合は [`SlotRecipe::css`] の出力から除外される。
    #[must_use]
    pub fn variant<V: VariantValue>(
        mut self,
        v: V,
        slot: &'static str,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.variants.push(VariantRule {
            axis: v.axis(),
            value: v.value(),
            slot,
            declarations,
        });
        self
    }

    /// axis `V` の既定 variant 値を登録する（builder、自己消費）。
    ///
    /// [`SlotRecipe::variant_classes`] は選択で指定されなかった axis を
    /// ここで登録した既定値で補完する。
    #[must_use]
    pub fn default_variant<V: VariantValue>(mut self, v: V) -> Self {
        self.default_variants.push(DefaultVariant {
            axis: v.axis(),
            value: v.value(),
        });
        self
    }

    /// 複数 variant 軸の組み合わせ条件が満たされたときの `slot` への宣言を
    /// 登録する（builder、自己消費。chakra-ui の `compoundVariants` 相当、
    /// イシュー #604）。
    ///
    /// `conditions` は [`when()`] で作った [`VariantCondition`] の列（AND
    /// 条件、すべて満たされたときのみ適用される）。以下のいずれかに該当する
    /// 規則は [`SlotRecipe::css`] の出力から除外される（fail-closed。既存
    /// `base`/`variant` と同じ「不正入力は panic せず出力から除外する」方針）:
    ///
    /// - `slot` が `slots` に未宣言、または識別子として不正
    /// - いずれかの条件の `axis`/`value` が識別子として不正
    /// - `conditions` が空（base と同義になる無意味な規則の混入防止）
    /// - `conditions` 内に同一 `axis` が重複（[`SlotRecipe::variant_classes`]
    ///   は 1 軸につき高々 1 クラスしか emit しないため、同一軸に異なる値を
    ///   要求する条件は決して同時に一致しない矛盾条件であり、dead CSS の
    ///   混入を防ぐ）
    /// - 条件の `(axis, value)` の組が [`SlotRecipe::variant`] /
    ///   [`SlotRecipe::default_variant`] のいずれにも未登録（axis/value の
    ///   タイポによる dead CSS の混入を防ぐ。検証は `css()` 呼び出し時に
    ///   行うため builder の呼び出し順には依存しない）
    #[must_use]
    pub fn compound_variant(
        mut self,
        conditions: Vec<VariantCondition>,
        slot: &'static str,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.compound_variants.push(CompoundVariantRule {
            conditions,
            slot,
            declarations,
        });
        self
    }

    /// この slot に属するかどうかを判定する（`slots` 未宣言の slot を
    /// fail-closed で除外するための内部ヘルパ）。
    fn is_declared_slot(&self, slot: &str) -> bool {
        self.slots.contains(&slot)
    }

    /// `(axis, value)` の組が `variant()`（任意 slot）または
    /// `default_variant()` に既に登録済みかどうかを判定する
    /// （[`SlotRecipe::compound_variant`] の fail-closed 検証専用の内部
    /// ヘルパ。axis/value のタイポによる dead CSS 混入を防ぐ）。
    fn is_registered_variant_value(&self, axis: &str, value: &str) -> bool {
        self.variants
            .iter()
            .any(|rule| rule.axis == axis && rule.value == value)
            || self
                .default_variants
                .iter()
                .any(|d| d.axis == axis && d.value == value)
    }

    /// この recipe が生成する静的 CSS 全量を返す（決定的: 同一の `self` に
    /// 対する複数回の呼び出しは常にバイト単位で同一の文字列を返す）。
    ///
    /// 出力順は「base（`slots` の宣言順）→ variants（登録順）→ compound
    /// variants（登録順、イシュー #604）」。セレクタは base が
    /// `[data-scope="<scope>"][data-part="<slot>"]`、variant が
    /// `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<axis>-<value>`
    /// （詳細度 (0,3,0) が base の (0,2,0) に必ず勝つため、CSS 記述順に
    /// 依存しない上書きを保証する）、compound variant は条件クラスを
    /// 登録順に連結した
    /// `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<a1>-<v1>.fd-<scope>--<a2>-<v2>...`
    /// （条件 2 個以上なら詳細度が単一 variant を必ず上回り、1 個の場合でも
    /// 出力順が variants より後のため CSS カスケードの後勝ちで上書きされる）。
    ///
    /// `scope`（[`SlotRecipe::new`] に渡した値）が識別子として不正な場合は
    /// 空文字列を返す（fail-closed。`slot`/`axis`/`value` と同様に `scope` も
    /// セレクタ・クラス名へそのまま埋め込まれるため、ここで検証しないと
    /// `</style>` やセレクタ脱出を許す構造破壊文字が CSS 生成経路に残ってしまう）。
    #[must_use]
    pub fn css(&self) -> String {
        if !is_valid_identifier(self.scope) {
            return String::new();
        }

        let mut out = String::new();

        for slot in self.slots {
            for rule in self.base.iter().filter(|rule| rule.slot == *slot) {
                if !is_valid_identifier(rule.slot) {
                    continue;
                }
                let selector = format!(
                    "[data-scope=\"{}\"][data-part=\"{}\"]",
                    self.scope, rule.slot
                );
                if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                    out.push_str(&css);
                    out.push('\n');
                }
            }
        }

        for rule in &self.variants {
            if !self.is_declared_slot(rule.slot)
                || !is_valid_identifier(rule.slot)
                || !is_valid_identifier(rule.axis)
                || !is_valid_identifier(rule.value)
            {
                continue;
            }
            let class_name = format!(
                "{CLASS_PREFIX}-{}--{}-{}",
                self.scope, rule.axis, rule.value
            );
            let selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"].{class_name}",
                self.scope, rule.slot
            );
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                out.push_str(&css);
                out.push('\n');
            }
        }

        'compound: for rule in &self.compound_variants {
            if rule.conditions.is_empty()
                || !self.is_declared_slot(rule.slot)
                || !is_valid_identifier(rule.slot)
            {
                continue;
            }

            // 同一 axis の重複は矛盾条件（variant_classes は 1 軸につき
            // 高々 1 クラスしか emit しないため決して同時に一致しない）と
            // みなし、dead CSS の混入を防ぐため規則ごと除外する。
            let mut seen_axes: Vec<&str> = Vec::new();
            for cond in &rule.conditions {
                if !is_valid_identifier(cond.axis) || !is_valid_identifier(cond.value) {
                    continue 'compound;
                }
                if seen_axes.contains(&cond.axis) {
                    continue 'compound;
                }
                seen_axes.push(cond.axis);
                if !self.is_registered_variant_value(cond.axis, cond.value) {
                    continue 'compound;
                }
            }

            let mut selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"]",
                self.scope, rule.slot
            );
            for cond in &rule.conditions {
                selector.push_str(&format!(
                    ".{CLASS_PREFIX}-{}--{}-{}",
                    self.scope, cond.axis, cond.value
                ));
            }
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                out.push_str(&css);
                out.push('\n');
            }
        }

        // 末尾の空行は規則ブロック間の区切りとしてのみ入れるため、
        // 最後の 1 つを削って「規則間は空行 1 つ」書式を保つ。
        if out.ends_with("\n\n") {
            out.pop();
        }
        out
    }

    /// variant 値 1 個に対応するクラス名（`fd-<scope>--<axis>-<value>`）を返す。
    ///
    /// `scope`/`axis()`/`value()` のいずれかが識別子として不正な場合は
    /// 空文字列を返す（呼び出し側が不正なクラスを HTML へ書き出すことを防ぐ
    /// fail-closed 動作。`fandhe_frontend_core::render` 経由でエスケープは
    /// されるが、無効なクラス名を出力に混入させないための追加防御）。
    #[must_use]
    pub fn variant_class<V: VariantValue>(&self, v: V) -> String {
        let axis = v.axis();
        let value = v.value();
        if !is_valid_identifier(self.scope)
            || !is_valid_identifier(axis)
            || !is_valid_identifier(value)
        {
            return String::new();
        }
        format!("{CLASS_PREFIX}-{}--{axis}-{value}", self.scope)
    }

    /// axis 名 → value 名の選択列からクラス文字列を組み立てる。
    ///
    /// `selection` で指定されなかった axis は [`SlotRecipe::default_variant`]
    /// で登録した既定値で補完する。戻り値は axis の登録順（`variant`/
    /// `default_variant` で最初に現れた順）で連結したクラス文字列
    /// （スペース区切り、`class="..."` にそのまま渡せる形式）。
    ///
    /// `scope` が識別子として不正な場合は空文字列を返す（[`SlotRecipe::css`]・
    /// [`SlotRecipe::variant_class`] と同じ fail-closed 方針）。
    #[must_use]
    pub fn variant_classes(&self, selection: &[(&str, &str)]) -> String {
        if !is_valid_identifier(self.scope) {
            return String::new();
        }

        let mut axes: Vec<&'static str> = Vec::new();
        for rule in &self.variants {
            if !axes.contains(&rule.axis) {
                axes.push(rule.axis);
            }
        }
        for d in &self.default_variants {
            if !axes.contains(&d.axis) {
                axes.push(d.axis);
            }
        }

        let mut classes: Vec<String> = Vec::new();
        for axis in axes {
            let value = selection
                .iter()
                .find(|(a, _)| *a == axis)
                .map(|(_, v)| *v)
                .or_else(|| {
                    self.default_variants
                        .iter()
                        .find(|d| d.axis == axis)
                        .map(|d| d.value)
                });
            if let Some(value) = value {
                if is_valid_identifier(axis) && is_valid_identifier(value) {
                    classes.push(format!("{CLASS_PREFIX}-{}--{axis}-{value}", self.scope));
                }
            }
        }
        classes.join(" ")
    }
}
