//! Blocks（`/blocks/`）の区分・カテゴリ型（イシュー #2733）。
//!
//! # 役割・呼び出し文脈
//!
//! [`super::Block`] は本モジュールの [`BlockCategory`] を必須フィールドと
//! して持ち、[`super::index_generated_sections`]（`/blocks/` 索引ページの
//! 生成）が「区分 → カテゴリ」の階層見出しを組み立てる際の唯一の分類源に
//! 使う。親トラッキング #2730（目的別パーツ拡充ツリー、新規約 300 block）
//! で追加される block はすべて本モジュールの [`BlockCategory`] のいずれか
//! 1 件を持つことが要求される。カテゴリ一覧・区分割当はイシュー #2733 の
//! 本文記載を正とする（`_/blocks-intake/` はローカル専用のため参照しない）。
//!
//! # 全域性による未知カテゴリの排除
//!
//! [`BlockCategory::section`]・[`BlockCategory::label`]・
//! [`BlockCategory::kebab`] はいずれも `_ =>` を使わない全 variant 明示の
//! `match` であり、新規カテゴリを [`BlockCategory`] へ追加してこれらの
//! 実装を更新し忘れるとコンパイルエラーになる（fail-closed）。

/// block の大区分（イシュー #2733 本文の 4 区分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSection {
    Marketing,
    Application,
    Ecommerce,
    Docs,
}

impl BlockSection {
    /// 索引ページでの表示順（イシュー #2733 本文の記載順）。
    pub const ALL: [BlockSection; 4] = [
        BlockSection::Marketing,
        BlockSection::Application,
        BlockSection::Ecommerce,
        BlockSection::Docs,
    ];

    /// 索引ページの `h2` 見出しに使う表示ラベル。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            BlockSection::Marketing => "Marketing",
            BlockSection::Application => "Application",
            BlockSection::Ecommerce => "Ecommerce",
            BlockSection::Docs => "Docs",
        }
    }
}

/// block のカテゴリ（イシュー #2733 本文の 66 種、区分内は本文記載順で宣言）。
///
/// # `Ecommerce::CategoryListing` の命名（イシュー本文の `category` カテゴリ）
///
/// イシュー本文の ecommerce 区分にはカテゴリ名そのものが `category`
/// （商品カテゴリ一覧ページを指す）として列挙されている。この enum 自体の
/// 概念名（カテゴリ）と variant 名が衝突すると読み手を混乱させるため、
/// variant 名は `CategoryListing` とし、[`BlockCategory::kebab`] が厳密な
/// 文字列 `"category"` を返すことで実際の分類名とのズレを吸収する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCategory {
    // marketing
    Hero,
    Feature,
    Cta,
    Pricing,
    Testimonial,
    LogoCloud,
    Stats,
    Team,
    Faq,
    Contact,
    Newsletter,
    Blog,
    Content,
    Header,
    Footer,
    Banner,
    Bento,
    Comparison,
    Careers,
    Changelog,
    ErrorPage,
    Gallery,
    SectionHeading,
    // application
    AppShell,
    Sidebar,
    Navbar,
    PageHeading,
    CardHeading,
    List,
    Table,
    GridList,
    DescriptionList,
    Calendar,
    Feed,
    FormLayout,
    Auth,
    Settings,
    ActionPanel,
    EmptyState,
    Notification,
    Dialog,
    Drawer,
    CommandPalette,
    Onboarding,
    Card,
    Profile,
    Chart,
    Dashboard,
    AiChat,
    HelpCenter,
    // ecommerce
    ProductOverview,
    ProductList,
    Quickview,
    CategoryListing,
    Filter,
    Cart,
    Checkout,
    Order,
    Reviews,
    StoreNav,
    Incentives,
    Promo,
    // docs
    DocsLayout,
    CodeBlock,
    ExamplePreview,
    ApiReference,
}

impl BlockCategory {
    /// 索引ページでの表示順（区分内はイシュー #2733 本文の記載順）。
    /// 新規カテゴリを追加する際はここへも追記すること（欠落は
    /// `crates/docs-site/tests/blocks_nav.rs` 等の網羅チェックが検知する）。
    pub const ALL: &'static [BlockCategory] = &[
        // marketing
        BlockCategory::Hero,
        BlockCategory::Feature,
        BlockCategory::Cta,
        BlockCategory::Pricing,
        BlockCategory::Testimonial,
        BlockCategory::LogoCloud,
        BlockCategory::Stats,
        BlockCategory::Team,
        BlockCategory::Faq,
        BlockCategory::Contact,
        BlockCategory::Newsletter,
        BlockCategory::Blog,
        BlockCategory::Content,
        BlockCategory::Header,
        BlockCategory::Footer,
        BlockCategory::Banner,
        BlockCategory::Bento,
        BlockCategory::Comparison,
        BlockCategory::Careers,
        BlockCategory::Changelog,
        BlockCategory::ErrorPage,
        BlockCategory::Gallery,
        BlockCategory::SectionHeading,
        // application
        BlockCategory::AppShell,
        BlockCategory::Sidebar,
        BlockCategory::Navbar,
        BlockCategory::PageHeading,
        BlockCategory::CardHeading,
        BlockCategory::List,
        BlockCategory::Table,
        BlockCategory::GridList,
        BlockCategory::DescriptionList,
        BlockCategory::Calendar,
        BlockCategory::Feed,
        BlockCategory::FormLayout,
        BlockCategory::Auth,
        BlockCategory::Settings,
        BlockCategory::ActionPanel,
        BlockCategory::EmptyState,
        BlockCategory::Notification,
        BlockCategory::Dialog,
        BlockCategory::Drawer,
        BlockCategory::CommandPalette,
        BlockCategory::Onboarding,
        BlockCategory::Card,
        BlockCategory::Profile,
        BlockCategory::Chart,
        BlockCategory::Dashboard,
        BlockCategory::AiChat,
        BlockCategory::HelpCenter,
        // ecommerce
        BlockCategory::ProductOverview,
        BlockCategory::ProductList,
        BlockCategory::Quickview,
        BlockCategory::CategoryListing,
        BlockCategory::Filter,
        BlockCategory::Cart,
        BlockCategory::Checkout,
        BlockCategory::Order,
        BlockCategory::Reviews,
        BlockCategory::StoreNav,
        BlockCategory::Incentives,
        BlockCategory::Promo,
        // docs
        BlockCategory::DocsLayout,
        BlockCategory::CodeBlock,
        BlockCategory::ExamplePreview,
        BlockCategory::ApiReference,
    ];

    /// このカテゴリが属する区分。全 variant を明示する `match`（全域性に
    /// より新規カテゴリの区分未定義をコンパイル時に検知する）。
    #[must_use]
    pub const fn section(self) -> BlockSection {
        match self {
            BlockCategory::Hero
            | BlockCategory::Feature
            | BlockCategory::Cta
            | BlockCategory::Pricing
            | BlockCategory::Testimonial
            | BlockCategory::LogoCloud
            | BlockCategory::Stats
            | BlockCategory::Team
            | BlockCategory::Faq
            | BlockCategory::Contact
            | BlockCategory::Newsletter
            | BlockCategory::Blog
            | BlockCategory::Content
            | BlockCategory::Header
            | BlockCategory::Footer
            | BlockCategory::Banner
            | BlockCategory::Bento
            | BlockCategory::Comparison
            | BlockCategory::Careers
            | BlockCategory::Changelog
            | BlockCategory::ErrorPage
            | BlockCategory::Gallery
            | BlockCategory::SectionHeading => BlockSection::Marketing,
            BlockCategory::AppShell
            | BlockCategory::Sidebar
            | BlockCategory::Navbar
            | BlockCategory::PageHeading
            | BlockCategory::CardHeading
            | BlockCategory::List
            | BlockCategory::Table
            | BlockCategory::GridList
            | BlockCategory::DescriptionList
            | BlockCategory::Calendar
            | BlockCategory::Feed
            | BlockCategory::FormLayout
            | BlockCategory::Auth
            | BlockCategory::Settings
            | BlockCategory::ActionPanel
            | BlockCategory::EmptyState
            | BlockCategory::Notification
            | BlockCategory::Dialog
            | BlockCategory::Drawer
            | BlockCategory::CommandPalette
            | BlockCategory::Onboarding
            | BlockCategory::Card
            | BlockCategory::Profile
            | BlockCategory::Chart
            | BlockCategory::Dashboard
            | BlockCategory::AiChat
            | BlockCategory::HelpCenter => BlockSection::Application,
            BlockCategory::ProductOverview
            | BlockCategory::ProductList
            | BlockCategory::Quickview
            | BlockCategory::CategoryListing
            | BlockCategory::Filter
            | BlockCategory::Cart
            | BlockCategory::Checkout
            | BlockCategory::Order
            | BlockCategory::Reviews
            | BlockCategory::StoreNav
            | BlockCategory::Incentives
            | BlockCategory::Promo => BlockSection::Ecommerce,
            BlockCategory::DocsLayout
            | BlockCategory::CodeBlock
            | BlockCategory::ExamplePreview
            | BlockCategory::ApiReference => BlockSection::Docs,
        }
    }

    /// 索引ページの `h3` 見出しに使う表示ラベル。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            BlockCategory::Hero => "Hero",
            BlockCategory::Feature => "Feature",
            BlockCategory::Cta => "CTA",
            BlockCategory::Pricing => "Pricing",
            BlockCategory::Testimonial => "Testimonial",
            BlockCategory::LogoCloud => "Logo Cloud",
            BlockCategory::Stats => "Stats",
            BlockCategory::Team => "Team",
            BlockCategory::Faq => "FAQ",
            BlockCategory::Contact => "Contact",
            BlockCategory::Newsletter => "Newsletter",
            BlockCategory::Blog => "Blog",
            BlockCategory::Content => "Content",
            BlockCategory::Header => "Header",
            BlockCategory::Footer => "Footer",
            BlockCategory::Banner => "Banner",
            BlockCategory::Bento => "Bento",
            BlockCategory::Comparison => "Comparison",
            BlockCategory::Careers => "Careers",
            BlockCategory::Changelog => "Changelog",
            BlockCategory::ErrorPage => "Error Page",
            BlockCategory::Gallery => "Gallery",
            BlockCategory::SectionHeading => "Section Heading",
            BlockCategory::AppShell => "App Shell",
            BlockCategory::Sidebar => "Sidebar",
            BlockCategory::Navbar => "Navbar",
            BlockCategory::PageHeading => "Page Heading",
            BlockCategory::CardHeading => "Card Heading",
            BlockCategory::List => "List",
            BlockCategory::Table => "Table",
            BlockCategory::GridList => "Grid List",
            BlockCategory::DescriptionList => "Description List",
            BlockCategory::Calendar => "Calendar",
            BlockCategory::Feed => "Feed",
            BlockCategory::FormLayout => "Form Layout",
            BlockCategory::Auth => "Auth",
            BlockCategory::Settings => "Settings",
            BlockCategory::ActionPanel => "Action Panel",
            BlockCategory::EmptyState => "Empty State",
            BlockCategory::Notification => "Notification",
            BlockCategory::Dialog => "Dialog",
            BlockCategory::Drawer => "Drawer",
            BlockCategory::CommandPalette => "Command Palette",
            BlockCategory::Onboarding => "Onboarding",
            BlockCategory::Card => "Card",
            BlockCategory::Profile => "Profile",
            BlockCategory::Chart => "Chart",
            BlockCategory::Dashboard => "Dashboard",
            BlockCategory::AiChat => "AI Chat",
            BlockCategory::HelpCenter => "Help Center",
            BlockCategory::ProductOverview => "Product Overview",
            BlockCategory::ProductList => "Product List",
            BlockCategory::Quickview => "Quickview",
            BlockCategory::CategoryListing => "Category",
            BlockCategory::Filter => "Filter",
            BlockCategory::Cart => "Cart",
            BlockCategory::Checkout => "Checkout",
            BlockCategory::Order => "Order",
            BlockCategory::Reviews => "Reviews",
            BlockCategory::StoreNav => "Store Nav",
            BlockCategory::Incentives => "Incentives",
            BlockCategory::Promo => "Promo",
            BlockCategory::DocsLayout => "Docs Layout",
            BlockCategory::CodeBlock => "Code Block",
            BlockCategory::ExamplePreview => "Example Preview",
            BlockCategory::ApiReference => "API Reference",
        }
    }

    /// イシュー本文・`_/blocks-intake/` 側の catalog slug との対応検証・
    /// デバッグ表示用の kebab-case 文字列（索引ページの URL 生成には
    /// 使わない。block ページ自体の URL は [`super::Block::path`] が持つ）。
    #[must_use]
    pub const fn kebab(self) -> &'static str {
        match self {
            BlockCategory::Hero => "hero",
            BlockCategory::Feature => "feature",
            BlockCategory::Cta => "cta",
            BlockCategory::Pricing => "pricing",
            BlockCategory::Testimonial => "testimonial",
            BlockCategory::LogoCloud => "logo-cloud",
            BlockCategory::Stats => "stats",
            BlockCategory::Team => "team",
            BlockCategory::Faq => "faq",
            BlockCategory::Contact => "contact",
            BlockCategory::Newsletter => "newsletter",
            BlockCategory::Blog => "blog",
            BlockCategory::Content => "content",
            BlockCategory::Header => "header",
            BlockCategory::Footer => "footer",
            BlockCategory::Banner => "banner",
            BlockCategory::Bento => "bento",
            BlockCategory::Comparison => "comparison",
            BlockCategory::Careers => "careers",
            BlockCategory::Changelog => "changelog",
            BlockCategory::ErrorPage => "error-page",
            BlockCategory::Gallery => "gallery",
            BlockCategory::SectionHeading => "section-heading",
            BlockCategory::AppShell => "app-shell",
            BlockCategory::Sidebar => "sidebar",
            BlockCategory::Navbar => "navbar",
            BlockCategory::PageHeading => "page-heading",
            BlockCategory::CardHeading => "card-heading",
            BlockCategory::List => "list",
            BlockCategory::Table => "table",
            BlockCategory::GridList => "grid-list",
            BlockCategory::DescriptionList => "description-list",
            BlockCategory::Calendar => "calendar",
            BlockCategory::Feed => "feed",
            BlockCategory::FormLayout => "form-layout",
            BlockCategory::Auth => "auth",
            BlockCategory::Settings => "settings",
            BlockCategory::ActionPanel => "action-panel",
            BlockCategory::EmptyState => "empty-state",
            BlockCategory::Notification => "notification",
            BlockCategory::Dialog => "dialog",
            BlockCategory::Drawer => "drawer",
            BlockCategory::CommandPalette => "command-palette",
            BlockCategory::Onboarding => "onboarding",
            BlockCategory::Card => "card",
            BlockCategory::Profile => "profile",
            BlockCategory::Chart => "chart",
            BlockCategory::Dashboard => "dashboard",
            BlockCategory::AiChat => "ai-chat",
            BlockCategory::HelpCenter => "help-center",
            BlockCategory::ProductOverview => "product-overview",
            BlockCategory::ProductList => "product-list",
            BlockCategory::Quickview => "quickview",
            BlockCategory::CategoryListing => "category",
            BlockCategory::Filter => "filter",
            BlockCategory::Cart => "cart",
            BlockCategory::Checkout => "checkout",
            BlockCategory::Order => "order",
            BlockCategory::Reviews => "reviews",
            BlockCategory::StoreNav => "store-nav",
            BlockCategory::Incentives => "incentives",
            BlockCategory::Promo => "promo",
            BlockCategory::DocsLayout => "docs-layout",
            BlockCategory::CodeBlock => "code-block",
            BlockCategory::ExamplePreview => "example-preview",
            BlockCategory::ApiReference => "api-reference",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_categories_have_unique_kebab() {
        let mut seen = HashSet::new();
        for category in BlockCategory::ALL {
            assert!(
                seen.insert(category.kebab()),
                "duplicate kebab: {}",
                category.kebab()
            );
        }
        assert_eq!(seen.len(), BlockCategory::ALL.len());
    }

    #[test]
    fn all_categories_resolve_a_section() {
        // 全 66 variant を網羅していることの明示的な固定（コンパイルが通れば
        // 自明だが、ALL の件数と section() が正常終了することを回帰させる）。
        assert_eq!(BlockCategory::ALL.len(), 66);
        for category in BlockCategory::ALL {
            let _ = category.section();
            assert!(!category.label().is_empty());
        }
    }
}
