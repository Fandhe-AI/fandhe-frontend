//! Themes（`fandhe-frontend-pre-styled-ui`）部品ページ台帳（イシュー #3617）。
//!
//! # 役割・呼び出し文脈
//!
//! `/themes/<kebab>/` 123 ページの「どの部品が・どの URL に・どの表示名で・
//! どのカテゴリに属し・何を 1 行で説明するか」を持つ定数台帳。Primitives 側の
//! [`crate::primitives_catalog`] に相当するコード側の正であり、これまで Themes
//! には Rust の台帳がなく、`site/themes.md` の手書きリンク集と `site/nav.toml`
//! が二重管理でずれていた（5 部品が原稿から欠落）。
//! [`crate::component_index`] が索引カードグリッドの生成元として使い、
//! `tests/component_index_nav.rs` が `site/nav.toml` の Themes グループ（題名・
//! 順序・path・title）との完全一致を fail-closed で固定する。
//!
//! # 説明文の規約
//!
//! 1 行（改行なし）・80 文字以内・バッククォートや内部番号を含まない。
//! Themes は見た目の観点で書く（構造・ARIA は Primitives 側の説明が担う）。
//!
//! # セキュリティ上の不変条件
//!
//! 値はすべて `&'static str` の定数で外部入力を含まない。描画は
//! [`crate::component_index`] がノード木 API（既定エスケープ）で行い、
//! 本モジュールは `std::fs` も HTML 組み立ても持たない（REQ-1）。

/// nav の Themes セクションのグループ（宣言順＝サイドバー・索引の並び）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThemeCategory {
    /// Typography（12 件）。
    Typography,
    /// Forms（37 件）。
    Forms,
    /// Interactive（27 件）。
    Interactive,
    /// Data Display（29 件）。
    DataDisplay,
    /// Utilities（6 件）。
    Utilities,
    /// Charts（12 件）。
    Charts,
}

impl ThemeCategory {
    /// `site/nav.toml` の `[[section.group]]` title および索引の見出しに使う表示名。
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            ThemeCategory::Typography => "Typography",
            ThemeCategory::Forms => "Forms",
            ThemeCategory::Interactive => "Interactive",
            ThemeCategory::DataDisplay => "Data Display",
            ThemeCategory::Utilities => "Utilities",
            ThemeCategory::Charts => "Charts",
        }
    }

    /// 宣言順（nav のグループ順）で全カテゴリを返す。
    #[must_use]
    pub const fn all() -> &'static [ThemeCategory] {
        &[
            ThemeCategory::Typography,
            ThemeCategory::Forms,
            ThemeCategory::Interactive,
            ThemeCategory::DataDisplay,
            ThemeCategory::Utilities,
            ThemeCategory::Charts,
        ]
    }
}

/// 台帳 1 件。`path` / `title` は `site/nav.toml` の `page.path` / `page.title`
/// と一致させる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeEntry {
    /// サイト上のページ path（`"/themes/<kebab>/"`）。
    pub path: &'static str,
    /// サイト上の表示名。
    pub title: &'static str,
    /// 所属カテゴリ。
    pub category: ThemeCategory,
    /// 索引カードに載せる 1 行説明（見た目の観点）。
    pub description: &'static str,
}

/// Themes 台帳（123 件、nav のグループ内宣言順）。
pub const THEMES: &[ThemeEntry] = &[
    ThemeEntry {
        path: "/themes/blockquote/",
        title: "Blockquote",
        category: ThemeCategory::Typography,
        description: "引用ブロックを、罫線と余白で整えて表示する見た目。",
    },
    ThemeEntry {
        path: "/themes/code/",
        title: "Code",
        category: ThemeCategory::Typography,
        description: "インラインのコード片を、等幅フォントと背景で示す見た目。",
    },
    ThemeEntry {
        path: "/themes/em/",
        title: "Em",
        category: ThemeCategory::Typography,
        description: "強調したい語句を、斜体で示すスタイル済み部品。",
    },
    ThemeEntry {
        path: "/themes/heading/",
        title: "Heading",
        category: ThemeCategory::Typography,
        description: "見出しを、レベルとサイズの組み合わせで整えて表示する見た目。",
    },
    ThemeEntry {
        path: "/themes/highlight/",
        title: "Highlight",
        category: ThemeCategory::Typography,
        description: "検索語などの一部分を、背景色で目立たせるスタイル。",
    },
    ThemeEntry {
        path: "/themes/kbd/",
        title: "Kbd",
        category: ThemeCategory::Typography,
        description: "キーボードのキー表記を、枠付きの見た目で示す部品。",
    },
    ThemeEntry {
        path: "/themes/link/",
        title: "Link",
        category: ThemeCategory::Typography,
        description: "本文中のリンクを、下線や色で示すスタイル済み部品。",
    },
    ThemeEntry {
        path: "/themes/list/",
        title: "List",
        category: ThemeCategory::Typography,
        description: "箇条書きと番号付きリストを、余白を整えて表示する見た目。",
    },
    ThemeEntry {
        path: "/themes/mark/",
        title: "Mark",
        category: ThemeCategory::Typography,
        description: "文中の語句を、マーカー風の背景で強調するスタイル。",
    },
    ThemeEntry {
        path: "/themes/quote/",
        title: "Quote",
        category: ThemeCategory::Typography,
        description: "文中の短い引用を、引用符つきで示すスタイル済み部品。",
    },
    ThemeEntry {
        path: "/themes/strong/",
        title: "Strong",
        category: ThemeCategory::Typography,
        description: "重要な語句を、太字で示すスタイル済み部品。",
    },
    ThemeEntry {
        path: "/themes/text/",
        title: "Text",
        category: ThemeCategory::Typography,
        description: "本文テキストを、サイズと色調のバリエーションで表示する部品。",
    },
    ThemeEntry {
        path: "/themes/angle-slider/",
        title: "Angle Slider",
        category: ThemeCategory::Forms,
        description: "角度を円環状のコントロールで選ばせる、スタイル済み入力。",
    },
    ThemeEntry {
        path: "/themes/button/",
        title: "Button",
        category: ThemeCategory::Forms,
        description: "サイズ・色・形状を選べる、スタイル済みのボタン。",
    },
    ThemeEntry {
        path: "/themes/calendar/",
        title: "Calendar",
        category: ThemeCategory::Forms,
        description: "月表示のカレンダーを、選択日と今日を強調して表示する見た目。",
    },
    ThemeEntry {
        path: "/themes/checkbox/",
        title: "Checkbox",
        category: ThemeCategory::Forms,
        description: "チェック状態ごとの見た目を整えた、スタイル済みチェックボックス。",
    },
    ThemeEntry {
        path: "/themes/checkbox-card/",
        title: "Checkbox Card",
        category: ThemeCategory::Forms,
        description: "カード全体をクリックして選ぶ、チェックボックス付きカード。",
    },
    ThemeEntry {
        path: "/themes/checkbox-group/",
        title: "Checkbox Group",
        category: ThemeCategory::Forms,
        description: "複数選択の項目を縦横に並べる、スタイル済みグループ。",
    },
    ThemeEntry {
        path: "/themes/color-picker/",
        title: "Color Picker",
        category: ThemeCategory::Forms,
        description: "色領域とスライダーで色を選ぶ、スタイル済みの色選択入力。",
    },
    ThemeEntry {
        path: "/themes/combobox/",
        title: "Combobox",
        category: ThemeCategory::Forms,
        description: "入力しながら候補を絞り込む、スタイル済みのコンボボックス。",
    },
    ThemeEntry {
        path: "/themes/command/",
        title: "Command",
        category: ThemeCategory::Forms,
        description: "検索と候補リストを備えた、スタイル済みのコマンドパレット。",
    },
    ThemeEntry {
        path: "/themes/date-input/",
        title: "Date Input",
        category: ThemeCategory::Forms,
        description: "年月日を区切って入力する、スタイル済みの日付入力。",
    },
    ThemeEntry {
        path: "/themes/date-picker/",
        title: "Date Picker",
        category: ThemeCategory::Forms,
        description: "カレンダーを開いて日付を選ぶ、スタイル済みの日付選択。",
    },
    ThemeEntry {
        path: "/themes/download-trigger/",
        title: "Download Trigger",
        category: ThemeCategory::Forms,
        description: "ファイル保存を促す、スタイル済みのダウンロードリンク。",
    },
    ThemeEntry {
        path: "/themes/editable/",
        title: "Editable",
        category: ThemeCategory::Forms,
        description: "表示と編集を切り替える、スタイル済みのインライン編集。",
    },
    ThemeEntry {
        path: "/themes/field/",
        title: "Field",
        category: ThemeCategory::Forms,
        description: "ラベル・補助文・エラー文と入力欄を整えて並べる、入力ラッパー。",
    },
    ThemeEntry {
        path: "/themes/fieldset/",
        title: "Fieldset",
        category: ThemeCategory::Forms,
        description: "関連する入力をまとめて囲む、スタイル済みのグループ枠。",
    },
    ThemeEntry {
        path: "/themes/file-upload/",
        title: "File Upload",
        category: ThemeCategory::Forms,
        description: "ドロップゾーンと選択済みファイル一覧を備えた、アップロード部品。",
    },
    ThemeEntry {
        path: "/themes/image-cropper/",
        title: "Image Cropper",
        category: ThemeCategory::Forms,
        description: "画像上で切り抜き範囲を選ばせる、スタイル済みの選択枠。",
    },
    ThemeEntry {
        path: "/themes/input/",
        title: "Input",
        category: ThemeCategory::Forms,
        description: "サイズとバリアントを選べる、スタイル済みのテキスト入力。",
    },
    ThemeEntry {
        path: "/themes/input-group/",
        title: "Input Group",
        category: ThemeCategory::Forms,
        description: "入力欄の前後に文字やボタンを添える、スタイル済みの入力グループ。",
    },
    ThemeEntry {
        path: "/themes/listbox/",
        title: "Listbox",
        category: ThemeCategory::Forms,
        description: "常時展開の一覧から選ぶ、スタイル済みのリストボックス。",
    },
    ThemeEntry {
        path: "/themes/native-select/",
        title: "Native Select",
        category: ThemeCategory::Forms,
        description: "ブラウザ標準の select を、既定スタイルで整えた選択入力。",
    },
    ThemeEntry {
        path: "/themes/number-input/",
        title: "Number Input",
        category: ThemeCategory::Forms,
        description: "増減ボタン付きの、スタイル済み数値入力。",
    },
    ThemeEntry {
        path: "/themes/password-input/",
        title: "Password Input",
        category: ThemeCategory::Forms,
        description: "表示切替ボタン付きの、スタイル済みパスワード入力。",
    },
    ThemeEntry {
        path: "/themes/pin-input/",
        title: "Pin Input",
        category: ThemeCategory::Forms,
        description: "桁ごとに分かれた、スタイル済みの PIN と OTP 入力。",
    },
    ThemeEntry {
        path: "/themes/questionnaire/",
        title: "Questionnaire",
        category: ThemeCategory::Forms,
        description: "多段の質問を順に提示する、スタイル済みのアンケート部品。",
    },
    ThemeEntry {
        path: "/themes/radio-card/",
        title: "Radio Card",
        category: ThemeCategory::Forms,
        description: "カード全体をクリックして選ぶ、ラジオボタン付きカード。",
    },
    ThemeEntry {
        path: "/themes/radio-group/",
        title: "Radio Group",
        category: ThemeCategory::Forms,
        description: "1 つだけ選ぶ項目を並べる、スタイル済みのラジオグループ。",
    },
    ThemeEntry {
        path: "/themes/rating-group/",
        title: "Rating Group",
        category: ThemeCategory::Forms,
        description: "星の数で評価を選ばせる、スタイル済みの評価入力。",
    },
    ThemeEntry {
        path: "/themes/segment-group/",
        title: "Segment Group",
        category: ThemeCategory::Forms,
        description: "切り替えボタンを横に並べる、スタイル済みのセグメントコントロール。",
    },
    ThemeEntry {
        path: "/themes/select/",
        title: "Select",
        category: ThemeCategory::Forms,
        description: "リストから選ぶ、スタイル済みのセレクトボックス。",
    },
    ThemeEntry {
        path: "/themes/signature-pad/",
        title: "Signature Pad",
        category: ThemeCategory::Forms,
        description: "手書き署名を描かせる、スタイル済みの署名入力。",
    },
    ThemeEntry {
        path: "/themes/slider/",
        title: "Slider",
        category: ThemeCategory::Forms,
        description: "つまみをドラッグして値を選ぶ、スタイル済みのスライダー。",
    },
    ThemeEntry {
        path: "/themes/switch/",
        title: "Switch",
        category: ThemeCategory::Forms,
        description: "オンとオフを切り替える、スタイル済みのスイッチ。",
    },
    ThemeEntry {
        path: "/themes/tags-input/",
        title: "Tags Input",
        category: ThemeCategory::Forms,
        description: "タグを追加・削除しながら入力する、スタイル済みのタグ入力。",
    },
    ThemeEntry {
        path: "/themes/textarea/",
        title: "Textarea",
        category: ThemeCategory::Forms,
        description: "複数行のテキストを入力する、スタイル済みのテキストエリア。",
    },
    ThemeEntry {
        path: "/themes/toggle/",
        title: "Toggle",
        category: ThemeCategory::Forms,
        description: "押下状態を切り替える、スタイル済みのトグルボタン。",
    },
    ThemeEntry {
        path: "/themes/toggle-group/",
        title: "Toggle Group",
        category: ThemeCategory::Forms,
        description: "複数のトグルボタンをまとめる、スタイル済みのグループ。",
    },
    ThemeEntry {
        path: "/themes/accordion/",
        title: "Accordion",
        category: ThemeCategory::Interactive,
        description: "項目を開閉して内容を出し入れする、スタイル済みのアコーディオン。",
    },
    ThemeEntry {
        path: "/themes/action-bar/",
        title: "Action Bar",
        category: ThemeCategory::Interactive,
        description: "選択項目への一括操作を並べる、画面下部のスタイル済みバー。",
    },
    ThemeEntry {
        path: "/themes/breadcrumb/",
        title: "Breadcrumb",
        category: ThemeCategory::Interactive,
        description: "現在地までの階層を区切り付きで示す、スタイル済みのパンくず。",
    },
    ThemeEntry {
        path: "/themes/button-group/",
        title: "Button Group",
        category: ThemeCategory::Interactive,
        description: "関連ボタンを連結して 1 つに見せる、スタイル済みのグループ。",
    },
    ThemeEntry {
        path: "/themes/carousel/",
        title: "Carousel",
        category: ThemeCategory::Interactive,
        description: "スライドを送りながら表示する、スタイル済みのカルーセル。",
    },
    ThemeEntry {
        path: "/themes/clipboard/",
        title: "Clipboard",
        category: ThemeCategory::Interactive,
        description: "値のコピー操作とコピー済み表示を整えた、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/collapsible/",
        title: "Collapsible",
        category: ThemeCategory::Interactive,
        description: "1 つのパネルを開閉する、スタイル済みの折りたたみ領域。",
    },
    ThemeEntry {
        path: "/themes/dialog/",
        title: "Dialog",
        category: ThemeCategory::Interactive,
        description: "画面を覆って表示する、スタイル済みのモーダルダイアログ。",
    },
    ThemeEntry {
        path: "/themes/drawer/",
        title: "Drawer",
        category: ThemeCategory::Interactive,
        description: "画面端からスライドして現れる、スタイル済みのパネル。",
    },
    ThemeEntry {
        path: "/themes/floating-panel/",
        title: "Floating Panel",
        category: ThemeCategory::Interactive,
        description: "ドラッグとリサイズができる、スタイル済みの浮遊パネル。",
    },
    ThemeEntry {
        path: "/themes/hover-card/",
        title: "Hover Card",
        category: ThemeCategory::Interactive,
        description: "hover で現れる、リンク先プレビュー用のスタイル済みカード。",
    },
    ThemeEntry {
        path: "/themes/menu/",
        title: "Menu",
        category: ThemeCategory::Interactive,
        description: "トリガーから開く、スタイル済みのドロップダウンメニュー。",
    },
    ThemeEntry {
        path: "/themes/menubar/",
        title: "Menubar",
        category: ThemeCategory::Interactive,
        description: "メニューを横に並べる、スタイル済みのメニューバー。",
    },
    ThemeEntry {
        path: "/themes/nav-list/",
        title: "Nav List",
        category: ThemeCategory::Interactive,
        description: "見出しとリンクで組む、スタイル済みの文書ナビ。",
    },
    ThemeEntry {
        path: "/themes/navigation-menu/",
        title: "Navigation Menu",
        category: ThemeCategory::Interactive,
        description: "トリガーで開くナビゲーションパネルの、スタイル済みメニュー。",
    },
    ThemeEntry {
        path: "/themes/pagination/",
        title: "Pagination",
        category: ThemeCategory::Interactive,
        description: "ページ番号と前後ボタンを並べる、スタイル済みのページ送り。",
    },
    ThemeEntry {
        path: "/themes/popover/",
        title: "Popover",
        category: ThemeCategory::Interactive,
        description: "トリガー付近に重ねて表示する、スタイル済みのポップオーバー。",
    },
    ThemeEntry {
        path: "/themes/sidebar/",
        title: "Sidebar",
        category: ThemeCategory::Interactive,
        description: "アプリシェル向けの、スタイル済みサイドバー。",
    },
    ThemeEntry {
        path: "/themes/splitter/",
        title: "Splitter",
        category: ThemeCategory::Interactive,
        description: "ドラッグで幅を変えられる、スタイル済みの分割パネル。",
    },
    ThemeEntry {
        path: "/themes/steps/",
        title: "Steps",
        category: ThemeCategory::Interactive,
        description: "段階の進み具合を示す、スタイル済みのステップ表示。",
    },
    ThemeEntry {
        path: "/themes/tab-nav/",
        title: "Tab Nav",
        category: ThemeCategory::Interactive,
        description: "リンクで画面を切り替える、スタイル済みのタブ型ナビ。",
    },
    ThemeEntry {
        path: "/themes/tabs/",
        title: "Tabs",
        category: ThemeCategory::Interactive,
        description: "タブで表示内容を切り替える、スタイル済みのタブ部品。",
    },
    ThemeEntry {
        path: "/themes/toast/",
        title: "Toast",
        category: ThemeCategory::Interactive,
        description: "一時的な通知を積んで表示する、スタイル済みのトースト。",
    },
    ThemeEntry {
        path: "/themes/toggle-tip/",
        title: "Toggle Tip",
        category: ThemeCategory::Interactive,
        description: "クリックで開閉する、スタイル済みの小型ヒント。",
    },
    ThemeEntry {
        path: "/themes/toolbar/",
        title: "Toolbar",
        category: ThemeCategory::Interactive,
        description: "ボタンやリンクを束ねる、スタイル済みのツールバー。",
    },
    ThemeEntry {
        path: "/themes/tooltip/",
        title: "Tooltip",
        category: ThemeCategory::Interactive,
        description: "hover や focus で現れる、スタイル済みの吹き出しヒント。",
    },
    ThemeEntry {
        path: "/themes/tour/",
        title: "Tour",
        category: ThemeCategory::Interactive,
        description: "画面上でステップ順に案内する、スタイル済みのガイド。",
    },
    ThemeEntry {
        path: "/themes/alert/",
        title: "Alert",
        category: ThemeCategory::DataDisplay,
        description: "状況をアイコンと色で伝える、スタイル済みの通知ブロック。",
    },
    ThemeEntry {
        path: "/themes/attachment/",
        title: "Attachment",
        category: ThemeCategory::DataDisplay,
        description: "添付ファイル 1 件を示す、スタイル済みの表示部品。",
    },
    ThemeEntry {
        path: "/themes/avatar/",
        title: "Avatar",
        category: ThemeCategory::DataDisplay,
        description: "画像とイニシャルのフォールバックを備えた、スタイル済みアバター。",
    },
    ThemeEntry {
        path: "/themes/badge/",
        title: "Badge",
        category: ThemeCategory::DataDisplay,
        description: "状態や件数を短く示す、スタイル済みのバッジ。",
    },
    ThemeEntry {
        path: "/themes/bubble/",
        title: "Bubble",
        category: ThemeCategory::DataDisplay,
        description: "チャットの吹き出しを、送信側と受信側で描き分ける見た目。",
    },
    ThemeEntry {
        path: "/themes/callout/",
        title: "Callout",
        category: ThemeCategory::DataDisplay,
        description: "補足や注意を目立たせて示す、スタイル済みの囲み枠。",
    },
    ThemeEntry {
        path: "/themes/card/",
        title: "Card",
        category: ThemeCategory::DataDisplay,
        description: "ヘッダー・本文・フッターで内容をまとめる、スタイル済みのカード。",
    },
    ThemeEntry {
        path: "/themes/color-swatch/",
        title: "Color Swatch",
        category: ThemeCategory::DataDisplay,
        description: "色の見本を小さな四角で示す、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/data-list/",
        title: "Data List",
        category: ThemeCategory::DataDisplay,
        description: "項目名と値の組を並べる、スタイル済みの定義リスト。",
    },
    ThemeEntry {
        path: "/themes/data-table/",
        title: "Data Table",
        category: ThemeCategory::DataDisplay,
        description: "ソートや行選択の操作を備えた、スタイル済みのデータ表。",
    },
    ThemeEntry {
        path: "/themes/empty-state/",
        title: "Empty State",
        category: ThemeCategory::DataDisplay,
        description: "データがない状態を案内する、スタイル済みの空状態表示。",
    },
    ThemeEntry {
        path: "/themes/icon/",
        title: "Icon",
        category: ThemeCategory::DataDisplay,
        description: "サイズと色を揃えて並べられる、スタイル済みのアイコン枠。",
    },
    ThemeEntry {
        path: "/themes/image/",
        title: "Image",
        category: ThemeCategory::DataDisplay,
        description: "画像を比率と角丸で整えて表示する、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/item/",
        title: "Item",
        category: ThemeCategory::DataDisplay,
        description: "アイコンとタイトル、操作を並べる、スタイル済みのリスト行。",
    },
    ThemeEntry {
        path: "/themes/json-tree-view/",
        title: "JSON Tree View",
        category: ThemeCategory::DataDisplay,
        description: "JSON 風データを、キーと値で色分けして示すツリー表示。",
    },
    ThemeEntry {
        path: "/themes/marker/",
        title: "Marker",
        category: ThemeCategory::DataDisplay,
        description: "会話内の注記や区切りを示す、スタイル済みの行。",
    },
    ThemeEntry {
        path: "/themes/message/",
        title: "Message",
        category: ThemeCategory::DataDisplay,
        description: "AI チャットの 1 発言を、役割ごとに描き分ける見た目。",
    },
    ThemeEntry {
        path: "/themes/message-scroller/",
        title: "Message Scroller",
        category: ThemeCategory::DataDisplay,
        description: "会話ログを収める、スタイル済みのスクロール領域。",
    },
    ThemeEntry {
        path: "/themes/progress/",
        title: "Progress",
        category: ThemeCategory::DataDisplay,
        description: "進捗を帯で示す、スタイル済みのプログレス表示。",
    },
    ThemeEntry {
        path: "/themes/qr-code/",
        title: "QR Code",
        category: ThemeCategory::DataDisplay,
        description: "文字列から生成した QR コードを、余白付きで表示する部品。",
    },
    ThemeEntry {
        path: "/themes/skeleton/",
        title: "Skeleton",
        category: ThemeCategory::DataDisplay,
        description: "読み込み中の領域を、仮の形で示すスタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/spinner/",
        title: "Spinner",
        category: ThemeCategory::DataDisplay,
        description: "処理中を回転で示す、スタイル済みのスピナー。",
    },
    ThemeEntry {
        path: "/themes/stat/",
        title: "Stat",
        category: ThemeCategory::DataDisplay,
        description: "指標の名前・値・増減を並べる、スタイル済みの統計表示。",
    },
    ThemeEntry {
        path: "/themes/status/",
        title: "Status",
        category: ThemeCategory::DataDisplay,
        description: "状態を色付きの点とラベルで示す、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/table/",
        title: "Table",
        category: ThemeCategory::DataDisplay,
        description: "行と列でデータを並べる、スタイル済みの表。",
    },
    ThemeEntry {
        path: "/themes/tag/",
        title: "Tag",
        category: ThemeCategory::DataDisplay,
        description: "分類やキーワードを示す、削除ボタン付きにもできるタグ。",
    },
    ThemeEntry {
        path: "/themes/timeline/",
        title: "Timeline",
        category: ThemeCategory::DataDisplay,
        description: "出来事を時系列に並べる、スタイル済みのタイムライン。",
    },
    ThemeEntry {
        path: "/themes/timer/",
        title: "Timer",
        category: ThemeCategory::DataDisplay,
        description: "カウントダウンとカウントアップを示す、スタイル済みの時間表示。",
    },
    ThemeEntry {
        path: "/themes/tree-view/",
        title: "Tree View",
        category: ThemeCategory::DataDisplay,
        description: "階層を展開・折りたたみで辿る、スタイル済みのツリー表示。",
    },
    ThemeEntry {
        path: "/themes/link-overlay/",
        title: "Link Overlay",
        category: ThemeCategory::Utilities,
        description: "カード全体をクリック可能にする、リンクオーバーレイ。",
    },
    ThemeEntry {
        path: "/themes/marquee/",
        title: "Marquee",
        category: ThemeCategory::Utilities,
        description: "要素を横に流し続ける、スタイル済みのマーキー。",
    },
    ThemeEntry {
        path: "/themes/scroll-area/",
        title: "Scroll Area",
        category: ThemeCategory::Utilities,
        description: "スクロールバーを整えた、スタイル済みのスクロール領域。",
    },
    ThemeEntry {
        path: "/themes/separator/",
        title: "Separator",
        category: ThemeCategory::Utilities,
        description: "区切り線を、向きと太さを選んで引くスタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/skip-nav/",
        title: "Skip Nav",
        category: ThemeCategory::Utilities,
        description: "キーボード操作時だけ現れる、本文へのスキップリンク。",
    },
    ThemeEntry {
        path: "/themes/visually-hidden/",
        title: "Visually Hidden",
        category: ThemeCategory::Utilities,
        description: "視覚的には隠し、支援技術には読ませ続けるテキスト。",
    },
    ThemeEntry {
        path: "/themes/area-chart/",
        title: "Area Chart",
        category: ThemeCategory::Charts,
        description: "推移を塗りつぶした面で示す、スタイル済みの面グラフ。",
    },
    ThemeEntry {
        path: "/themes/bar-chart/",
        title: "Bar Chart",
        category: ThemeCategory::Charts,
        description: "値の大小を棒の長さで比べる、スタイル済みの棒グラフ。",
    },
    ThemeEntry {
        path: "/themes/bar-list/",
        title: "Bar List",
        category: ThemeCategory::Charts,
        description: "項目ごとの値を横棒つきの一覧で示す、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/bar-segment/",
        title: "Bar Segment",
        category: ThemeCategory::Charts,
        description: "構成比を 1 本の分割バーで示す、スタイル済みの部品。",
    },
    ThemeEntry {
        path: "/themes/charts/",
        title: "Charts（共通 API）",
        category: ThemeCategory::Charts,
        description: "各グラフが共有する、軸・凡例・色の共通 API。",
    },
    ThemeEntry {
        path: "/themes/donut-chart/",
        title: "Donut Chart",
        category: ThemeCategory::Charts,
        description: "構成比を輪の分割で示す、スタイル済みのドーナツグラフ。",
    },
    ThemeEntry {
        path: "/themes/line-chart/",
        title: "Line Chart",
        category: ThemeCategory::Charts,
        description: "時間などによる変化を線で示す、スタイル済みの折れ線グラフ。",
    },
    ThemeEntry {
        path: "/themes/pie-chart/",
        title: "Pie Chart",
        category: ThemeCategory::Charts,
        description: "全体に対する割合を扇形で示す、スタイル済みの円グラフ。",
    },
    ThemeEntry {
        path: "/themes/radar-chart/",
        title: "Radar Chart",
        category: ThemeCategory::Charts,
        description: "複数の指標を多角形で比べる、スタイル済みのレーダーチャート。",
    },
    ThemeEntry {
        path: "/themes/radial-chart/",
        title: "Radial Chart",
        category: ThemeCategory::Charts,
        description: "値を円弧の長さで示す、スタイル済みの放射状グラフ。",
    },
    ThemeEntry {
        path: "/themes/scatter-chart/",
        title: "Scatter Chart",
        category: ThemeCategory::Charts,
        description: "2 つの値の分布を点で示す、スタイル済みの散布図。",
    },
    ThemeEntry {
        path: "/themes/sparkline/",
        title: "Sparkline",
        category: ThemeCategory::Charts,
        description: "推移を小さな線で添える、スタイル済みのスパークライン。",
    },
];

/// 台帳の全件を宣言順に返す。
pub fn entries() -> impl Iterator<Item = &'static ThemeEntry> {
    THEMES.iter()
}

/// 指定カテゴリに属する台帳エントリを宣言順に返す。
pub fn entries_in(category: ThemeCategory) -> impl Iterator<Item = &'static ThemeEntry> {
    THEMES.iter().filter(move |e| e.category == category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_123_entries_with_expected_category_counts() {
        assert_eq!(THEMES.len(), 123);
        let counts: Vec<usize> = ThemeCategory::all()
            .iter()
            .map(|c| entries_in(*c).count())
            .collect();
        assert_eq!(counts, [12, 37, 27, 29, 6, 12]);
    }

    #[test]
    fn paths_are_unique_and_under_themes() {
        let mut paths: Vec<_> = THEMES.iter().map(|e| e.path).collect();
        paths.sort_unstable();
        paths.dedup();
        assert_eq!(paths.len(), THEMES.len());
        assert!(THEMES
            .iter()
            .all(|e| e.path.starts_with("/themes/") && e.path.ends_with('/')));
    }

    #[test]
    fn categories_are_contiguous_in_declaration_order() {
        let cats: Vec<_> = THEMES.iter().map(|e| e.category).collect();
        let mut sorted = cats.clone();
        sorted.sort();
        assert_eq!(cats, sorted);
    }
}
