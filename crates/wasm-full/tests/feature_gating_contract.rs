//! イシュー #2327 の scope feature（`headless::MAPPING_TABLE` 行・
//! `keynav::wire_keynav` の scope 別 `match scope` arm を cfg ゲートする
//! feature 群）が、ソース中で実際に一貫して宣言・配線されていることを
//! 機械検知する契約テスト（native、`route_shared_static.rs` と同型の
//! ソース静的解析）。
//!
//! 固定する不変条件:
//!
//! 1. `headless.rs` の `MAPPING_TABLE` 内で `MappingRow { scope: "X", ... }`
//!    が現れる各要素は、直前の非コメント・非空行が
//!    `#[cfg(feature = "X")]` であること（X はその行が持つ `scope` 値と
//!    一致する）。
//! 2. `keynav.rs` の `match scope { ... }` ブロック内の各 `"Y" => ...` arm
//!    は、直前の非コメント・非空行が期待 feature（`radio` → `radio-group`、
//!    `navigation-menu-trigger`/`navigation-menu-link` → `navigation-menu`、
//!    それ以外は同名）へ cfg ゲートされていること。
//! 3. 1・2 で現れるすべての feature 名が `Cargo.toml` の `[features]` 節に
//!    定義され、かつ `default` 配列に列挙されていること（既定は全 on の
//!    不変条件、`.claude/rules/coding-rust.md` #638 系の semver 方針とも
//!    整合）。
//! 4. `lib.rs` の `keynav::wire_readonly_click_guard(root.clone())?;` 呼び
//!    出しが `mount`/`hydrate` で 2 回現れ、いずれも直前行が
//!    `#[cfg(` で始まらないこと（readonly RadioGroup の click capture
//!    保護がいかなる feature にも依存しない常時配線であるという、
//!    イシュー #2333 由来の不変条件を後退させない）。`keynav.rs` の
//!    `pub fn wire_readonly_click_guard` 定義自体にも `#[cfg(feature`
//!    が付与されていないこと。

use std::fs;
use std::path::PathBuf;

/// `crates/wasm-full/` の絶対パス（本クレート自身のルート）。
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `headless.rs`/`keynav.rs`/`lib.rs`/`Cargo.toml` が、コメント・空行を
/// 読み飛ばして「直前の意味のある行」を判定できるよう、非コメント・
/// 非空行だけを残した `(元の行番号, 行内容)` のベクトルへ変換する。
/// ブロックコメント（`/* */`）はこのクレートのソースでは使用されて
/// いないため（コーディング規約が行コメントのみを想定、`code-comment-
/// style.md`）、行コメント（`//`）の除去のみを行う軽量フィルタで足りる。
fn meaningful_lines(src: &str) -> Vec<(usize, &str)> {
    src.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                None
            } else {
                Some((i, line))
            }
        })
        .collect()
}

/// (scope, part) → feature 名の対応（keynav の match arm 用）。
/// MAPPING_TABLE 側は `scope` フィールドの文字列がそのまま feature 名と
/// 一致するため専用の変換は不要。
fn keynav_arm_feature(arm_literal: &str) -> &'static str {
    match arm_literal {
        "radio" => "radio-group",
        "navigation-menu-trigger" | "navigation-menu-link" => "navigation-menu",
        "tabs" => "tabs",
        "accordion" => "accordion",
        "menu" => "menu",
        "select" => "select",
        "menubar" => "menubar",
        "combobox" => "combobox",
        "listbox" => "listbox",
        "toggle-group" => "toggle-group",
        "tree-view" => "tree-view",
        "calendar" => "calendar",
        other => panic!(
            "feature_gating_contract: keynav.rs の match scope に未知の \
             arm リテラル \"{other}\" が追加された。keynav_arm_feature() \
             へ対応を追記すること（イシュー #2327 規約 (a')〜(d') 参照）"
        ),
    }
}

#[test]
fn headless_mapping_table_rows_are_cfg_gated_by_their_scope_feature() {
    let path = crate_root().join("src/headless.rs");
    let src = fs::read_to_string(&path).expect("src/headless.rs must be readable");
    let lines = meaningful_lines(&src);

    let mut checked = 0usize;
    for (idx, (_, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed != "scope: \"" && !trimmed.starts_with("scope: \"") {
            continue;
        }
        // `scope: "X",` から X を抽出する。
        let Some(scope) = trimmed
            .strip_prefix("scope: \"")
            .and_then(|rest| rest.split('"').next())
        else {
            continue;
        };
        // この `scope:` 行を含む `MappingRow {` を、直前方向へ遡って探す
        // （`scope` は各 MappingRow の最初のフィールドという既存の書式
        // 規約に依存しない、より頑健な後方探索）。
        let mut row_start = None;
        for back in (0..idx).rev() {
            let (_, candidate) = lines[back];
            if candidate.trim() == "MappingRow {" {
                row_start = Some(back);
                break;
            }
            // 別の要素の終端（`},`）に達したら、この `scope:` 行を含む
            // `MappingRow {` は見つからない（構造が壊れている）。
            if candidate.trim() == "}," {
                break;
            }
        }
        let Some(row_start) = row_start else {
            continue;
        };
        // `MappingRow {` の直前の意味のある行が期待する cfg か確認する。
        assert!(
            row_start > 0,
            "feature_gating_contract: MAPPING_TABLE の先頭要素に \
             #[cfg(feature = \"...\")] が付与されていない（scope=\"{scope}\"）"
        );
        let (_, prev_line) = lines[row_start - 1];
        let expected = format!("#[cfg(feature = \"{scope}\")]");
        assert_eq!(
            prev_line.trim(),
            expected,
            "feature_gating_contract: headless.rs の MappingRow \
             (scope=\"{scope}\") の直前行が期待する cfg \
             (\"{expected}\") と一致しない（実際: \"{}\"）。\
             イシュー #2327 規約 (a')〜(d') に従い cfg を付与すること",
            prev_line.trim()
        );
        checked += 1;
    }

    assert_eq!(
        checked, 32,
        "feature_gating_contract: MAPPING_TABLE の行数が想定 32 件から \
         変化した。新規行を追加した場合は本テストの期待値と \
         Cargo.toml の scope feature 対応表を追随更新すること"
    );
}

#[test]
fn keynav_match_scope_arms_are_cfg_gated_by_their_scope_feature() {
    let path = crate_root().join("src/keynav.rs");
    let src = fs::read_to_string(&path).expect("src/keynav.rs must be readable");
    let lines = meaningful_lines(&src);

    // `match scope {` ブロックの範囲（次の `keydown_root.add_event_listener...`
    // 呼び出し手前の `});` まで）を特定する。
    let match_start = lines
        .iter()
        .position(|(_, l)| l.trim() == "match scope {")
        .expect("wire_keynav must contain `match scope {`");

    let mut checked_arms: Vec<&str> = Vec::new();
    let mut idx = match_start + 1;
    let mut depth = 1i32;
    while idx < lines.len() && depth > 0 {
        let (_, line) = lines[idx];
        let trimmed = line.trim();
        depth += trimmed.matches('{').count() as i32;
        depth -= trimmed.matches('}').count() as i32;
        if depth <= 0 {
            break;
        }
        // arm パターン行: `"literal" =>` で始まる（複数リテラルの or は
        // 本 match では使われていないため単一リテラルのみを想定する）。
        if let Some(rest) = trimmed.strip_prefix('"') {
            if let Some(literal) = rest.split('"').next() {
                if trimmed[1 + literal.len() + 1..]
                    .trim_start()
                    .starts_with("=>")
                {
                    let expected_feature = keynav_arm_feature(literal);
                    let (_, prev_line) = lines[idx - 1];
                    let expected = format!("#[cfg(feature = \"{expected_feature}\")]");
                    assert_eq!(
                        prev_line.trim(),
                        expected,
                        "feature_gating_contract: keynav.rs の match scope \
                         arm \"{literal}\" の直前行が期待する cfg \
                         (\"{expected}\") と一致しない（実際: \"{}\"）。\
                         イシュー #2327 規約 (a')〜(d') に従い cfg を \
                         付与すること",
                        prev_line.trim()
                    );
                    checked_arms.push(literal);
                }
            }
        }
        idx += 1;
    }

    let expected_arms = [
        "tabs",
        "accordion",
        "menu",
        "select",
        "radio",
        "menubar",
        "combobox",
        "listbox",
        "navigation-menu-trigger",
        "navigation-menu-link",
        "toggle-group",
        "tree-view",
        "calendar",
    ];
    assert_eq!(
        checked_arms, expected_arms,
        "feature_gating_contract: keynav.rs の match scope arm 集合が \
         想定と異なる。新規 arm を追加した場合は本テストの \
         expected_arms・keynav_arm_feature() を追随更新すること"
    );
}

#[test]
fn scope_features_referenced_in_source_are_declared_and_defaulted_in_cargo_toml() {
    let cargo_toml_path = crate_root().join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path).expect("Cargo.toml must be readable");

    // headless.rs の 32 scope（重複あり）から一意集合を、keynav.rs の
    // arm から導出した feature 名の一意集合と合成する。
    let headless_src = fs::read_to_string(crate_root().join("src/headless.rs")).unwrap();
    let mut scope_features: Vec<String> = Vec::new();
    for line in headless_src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("scope: \"") {
            if let Some(scope) = rest.split('"').next() {
                let owned = scope.to_string();
                if !scope_features.contains(&owned) {
                    scope_features.push(owned);
                }
            }
        }
    }
    for arm in [
        "tabs",
        "accordion",
        "menu",
        "select",
        "radio",
        "menubar",
        "combobox",
        "listbox",
        "navigation-menu-trigger",
        "navigation-menu-link",
        "toggle-group",
        "tree-view",
        "calendar",
    ] {
        let feature = keynav_arm_feature(arm).to_string();
        if !scope_features.contains(&feature) {
            scope_features.push(feature);
        }
    }

    // Cargo.toml の `[features]` 節本文（`default = [...]` を含む）を
    // 単純な部分文字列一致で確認する（このクレートの Cargo.toml は
    // xtask 外部依存ゼロ方針〔REQ-3〕の対象外だが、他の xtask 契約
    // テストと同様に軽量な文字列検査で足りる範囲に留める）。
    let features_section_start = cargo_toml
        .find("[features]")
        .expect("Cargo.toml must have a [features] section");
    let features_section = &cargo_toml[features_section_start..];

    for feature in &scope_features {
        let decl = format!("\n{feature} = []\n");
        assert!(
            features_section.contains(&decl),
            "feature_gating_contract: Cargo.toml の [features] に \
             \"{feature} = []\" 宣言が見つからない"
        );
        let default_entry = format!("\"{feature}\",\n");
        assert!(
            features_section.contains(&default_entry),
            "feature_gating_contract: Cargo.toml の default 配列に \
             \"{feature}\" が列挙されていない（scope feature は既定 on \
             が不変条件）"
        );
    }

    assert_eq!(
        scope_features.len(),
        18,
        "feature_gating_contract: headless.rs/keynav.rs から導出される \
         scope feature の一意集合が想定 18 件（MAPPING_TABLE の 17 scope \
         〔accordion, calendar, collapsible, combobox, dialog, menu, \
         menubar, navigation-menu, popover, radio-group, select, \
         sidebar, signature-pad, tabs, toggle-group, tooltip, \
         tree-view〕+ keynav 専用の listbox）から変化した。\
         想定件数の見直しが必要"
    );
}

#[test]
fn readonly_click_guard_wiring_is_never_feature_gated() {
    let lib_path = crate_root().join("src/lib.rs");
    let lib_src = fs::read_to_string(&lib_path).expect("src/lib.rs must be readable");
    let lines: Vec<&str> = lib_src.lines().collect();

    let mut call_sites = 0usize;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == "keynav::wire_readonly_click_guard(root.clone())?;" {
            call_sites += 1;
            // 直前の非空行が `#[cfg(` で始まっていないこと（コメント行は
            // 許容: readonly click guard の呼び出し直前には既存の
            // 説明コメントが置かれている）。
            let mut back = idx;
            let prev = loop {
                if back == 0 {
                    break None;
                }
                back -= 1;
                let candidate = lines[back].trim();
                if candidate.is_empty() {
                    continue;
                }
                break Some(candidate);
            };
            if let Some(prev_line) = prev {
                assert!(
                    !prev_line.starts_with("#[cfg("),
                    "feature_gating_contract: lib.rs の \
                     wire_readonly_click_guard 呼び出しの直前行が \
                     #[cfg(...)] になっている（イシュー #2333 の常時配線 \
                     不変条件への違反、直前行: \"{prev_line}\"）"
                );
            }
        }
    }
    assert_eq!(
        call_sites, 2,
        "feature_gating_contract: wire_readonly_click_guard の呼び出しが \
         mount/hydrate の 2 箇所から想定外の件数になった"
    );

    let keynav_path = crate_root().join("src/keynav.rs");
    let keynav_src = fs::read_to_string(&keynav_path).expect("src/keynav.rs must be readable");
    let keynav_lines: Vec<&str> = keynav_src.lines().collect();
    let def_idx = keynav_lines
        .iter()
        .position(|l| {
            l.trim() == "pub fn wire_readonly_click_guard(root: Element) -> Result<(), JsValue> {"
        })
        .expect("keynav.rs must define pub fn wire_readonly_click_guard");
    let mut back = def_idx;
    let prev = loop {
        if back == 0 {
            break None;
        }
        back -= 1;
        let candidate = keynav_lines[back].trim();
        if candidate.is_empty() {
            continue;
        }
        break Some(candidate);
    };
    if let Some(prev_line) = prev {
        assert!(
            !prev_line.starts_with("#[cfg(feature"),
            "feature_gating_contract: keynav.rs の \
             pub fn wire_readonly_click_guard 定義自体に feature cfg が \
             付与されている（常時配線の不変条件への違反）"
        );
    }
}
