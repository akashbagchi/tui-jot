use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

// ── Theme Struct ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Theme {
    // Backgrounds
    pub bg0: Color,
    pub bg1: Color,
    pub bg2: Color,
    pub bg3: Color,
    pub bg4: Color,
    // Foregrounds
    pub fg0: Color,
    pub fg1: Color,
    pub fg2: Color,
    pub fg3: Color,
    pub fg4: Color,
    // Accents
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub purple: Color,
    pub aqua: Color,
    pub orange: Color,
    // Semantic
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub border_overlay: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub heading_1: Color,
    pub heading_2: Color,
    pub heading_3: Color,
    pub link_fg: Color,
    pub link_selected_fg: Color,
    pub link_selected_bg: Color,
    pub link_broken: Color,
    pub tag_fg: Color,
    pub inline_code: Color,
    pub title_fg: Color,
    pub title_bar_bg: Color,
    pub status_bar_bg: Color,
    pub cursor_blink: Color,
    pub empty_hint: Color,
    pub dir_fg: Color,
    pub file_fg: Color,
    pub backlink_fg: Color,
    pub tag_filter_border: Color,
    pub search_prompt: Color,
    pub finder_prompt: Color,
    pub autocomplete_bg: Color,
    pub autocomplete_sel_bg: Color,
}

impl Theme {
    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border_unfocused)
        }
    }

    pub fn selection_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn from_name(name: &str) -> Option<Theme> {
        match name {
            "gruvbox-dark" => Some(gruvbox_dark()),
            "gruvbox-light" => Some(gruvbox_light()),
            "catppuccin-mocha" => Some(catppuccin_mocha()),
            "catppuccin-latte" => Some(catppuccin_latte()),
            "tokyo-night" => Some(tokyo_night()),
            "tokyo-night-day" => Some(tokyo_night_day()),
            "nord" => Some(nord()),
            "dracula" => Some(dracula()),
            _ => None,
        }
    }

    pub fn from_config(ui: &crate::config::UiConfig) -> Theme {
        let mut theme = Theme::from_name(&ui.theme).unwrap_or_else(gruvbox_dark);
        theme.apply_overrides(&ui.theme_overrides);
        theme
    }

    pub fn apply_overrides(&mut self, overrides: &HashMap<String, String>) {
        for (key, value) in overrides {
            if let Some(color) = parse_hex_color(value) {
                match key.as_str() {
                    "bg0" => self.bg0 = color,
                    "bg1" => self.bg1 = color,
                    "bg2" => self.bg2 = color,
                    "bg3" => self.bg3 = color,
                    "bg4" => self.bg4 = color,
                    "fg0" => self.fg0 = color,
                    "fg1" => self.fg1 = color,
                    "fg2" => self.fg2 = color,
                    "fg3" => self.fg3 = color,
                    "fg4" => self.fg4 = color,
                    "red" => self.red = color,
                    "green" => self.green = color,
                    "yellow" => self.yellow = color,
                    "blue" => self.blue = color,
                    "purple" => self.purple = color,
                    "aqua" => self.aqua = color,
                    "orange" => self.orange = color,
                    "border_focused" => self.border_focused = color,
                    "border_unfocused" => self.border_unfocused = color,
                    "border_overlay" => self.border_overlay = color,
                    "selected_fg" => self.selected_fg = color,
                    "selected_bg" => self.selected_bg = color,
                    "heading_1" => self.heading_1 = color,
                    "heading_2" => self.heading_2 = color,
                    "heading_3" => self.heading_3 = color,
                    "link_fg" => self.link_fg = color,
                    "link_selected_fg" => self.link_selected_fg = color,
                    "link_selected_bg" => self.link_selected_bg = color,
                    "link_broken" => self.link_broken = color,
                    "tag_fg" => self.tag_fg = color,
                    "inline_code" => self.inline_code = color,
                    "title_fg" => self.title_fg = color,
                    "title_bar_bg" => self.title_bar_bg = color,
                    "status_bar_bg" => self.status_bar_bg = color,
                    "cursor_blink" => self.cursor_blink = color,
                    "empty_hint" => self.empty_hint = color,
                    "dir_fg" => self.dir_fg = color,
                    "file_fg" => self.file_fg = color,
                    "backlink_fg" => self.backlink_fg = color,
                    "tag_filter_border" => self.tag_filter_border = color,
                    "search_prompt" => self.search_prompt = color,
                    "finder_prompt" => self.finder_prompt = color,
                    "autocomplete_bg" => self.autocomplete_bg = color,
                    "autocomplete_sel_bg" => self.autocomplete_sel_bg = color,
                    _ => {}
                }
            }
        }
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

// ── Nerd Font Icons ───────────────────────────────────────────────

pub const ICON_APP: &str = "󰠮 ";
pub const ICON_FILE: &str = "󰈙 ";
pub const ICON_FOLDER_OPEN: &str = " ";
pub const ICON_FOLDER_CLOSED: &str = " ";
pub const ICON_SEARCH: &str = " ";
pub const ICON_TAG: &str = " ";
pub const ICON_LINK: &str = "󰌹 ";
pub const ICON_EDIT: &str = " ";

// ── Style Helpers (non-theme) ───────────────────────────────────

pub fn border_type() -> BorderType {
    BorderType::Rounded
}

// ── Built-in Presets ────────────────────────────────────────────

pub fn gruvbox_dark() -> Theme {
    let bg0 = Color::Rgb(40, 40, 40);
    let bg1 = Color::Rgb(60, 56, 54);
    let bg2 = Color::Rgb(80, 73, 69);
    let bg3 = Color::Rgb(102, 92, 84);
    let bg4 = Color::Rgb(124, 111, 100);
    let fg0 = Color::Rgb(251, 241, 199);
    let fg1 = Color::Rgb(235, 219, 178);
    let fg2 = Color::Rgb(213, 196, 161);
    let fg3 = Color::Rgb(189, 174, 147);
    let fg4 = Color::Rgb(168, 153, 132);
    let red = Color::Rgb(251, 73, 52);
    let green = Color::Rgb(184, 187, 38);
    let yellow = Color::Rgb(250, 189, 47);
    let blue = Color::Rgb(131, 165, 152);
    let purple = Color::Rgb(211, 134, 155);
    let aqua = Color::Rgb(142, 192, 124);
    let orange = Color::Rgb(254, 128, 25);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn gruvbox_light() -> Theme {
    let bg0 = Color::Rgb(251, 241, 199);
    let bg1 = Color::Rgb(235, 219, 178);
    let bg2 = Color::Rgb(213, 196, 161);
    let bg3 = Color::Rgb(189, 174, 147);
    let bg4 = Color::Rgb(168, 153, 132);
    let fg0 = Color::Rgb(40, 40, 40);
    let fg1 = Color::Rgb(60, 56, 54);
    let fg2 = Color::Rgb(80, 73, 69);
    let fg3 = Color::Rgb(102, 92, 84);
    let fg4 = Color::Rgb(124, 111, 100);
    let red = Color::Rgb(204, 36, 29);
    let green = Color::Rgb(152, 151, 26);
    let yellow = Color::Rgb(215, 153, 33);
    let blue = Color::Rgb(69, 133, 136);
    let purple = Color::Rgb(177, 98, 134);
    let aqua = Color::Rgb(104, 157, 106);
    let orange = Color::Rgb(214, 93, 14);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn catppuccin_mocha() -> Theme {
    let bg0 = Color::Rgb(30, 30, 46);
    let bg1 = Color::Rgb(49, 50, 68);
    let bg2 = Color::Rgb(69, 71, 90);
    let bg3 = Color::Rgb(88, 91, 112);
    let bg4 = Color::Rgb(108, 112, 134);
    let fg0 = Color::Rgb(205, 214, 244);
    let fg1 = Color::Rgb(186, 194, 222);
    let fg2 = Color::Rgb(166, 173, 200);
    let fg3 = Color::Rgb(147, 153, 178);
    let fg4 = Color::Rgb(127, 132, 156);
    let red = Color::Rgb(243, 139, 168);
    let green = Color::Rgb(166, 227, 161);
    let yellow = Color::Rgb(249, 226, 175);
    let blue = Color::Rgb(137, 180, 250);
    let purple = Color::Rgb(203, 166, 247);
    let aqua = Color::Rgb(148, 226, 213);
    let orange = Color::Rgb(250, 179, 135);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn catppuccin_latte() -> Theme {
    let bg0 = Color::Rgb(239, 241, 245);
    let bg1 = Color::Rgb(230, 233, 239);
    let bg2 = Color::Rgb(204, 208, 218);
    let bg3 = Color::Rgb(188, 192, 204);
    let bg4 = Color::Rgb(172, 176, 190);
    let fg0 = Color::Rgb(76, 79, 105);
    let fg1 = Color::Rgb(92, 95, 119);
    let fg2 = Color::Rgb(108, 111, 133);
    let fg3 = Color::Rgb(124, 127, 147);
    let fg4 = Color::Rgb(140, 143, 161);
    let red = Color::Rgb(210, 15, 57);
    let green = Color::Rgb(64, 160, 43);
    let yellow = Color::Rgb(223, 142, 29);
    let blue = Color::Rgb(30, 102, 245);
    let purple = Color::Rgb(136, 57, 239);
    let aqua = Color::Rgb(23, 146, 153);
    let orange = Color::Rgb(254, 100, 11);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn tokyo_night() -> Theme {
    let bg0 = Color::Rgb(26, 27, 38);
    let bg1 = Color::Rgb(36, 40, 59);
    let bg2 = Color::Rgb(52, 59, 88);
    let bg3 = Color::Rgb(68, 75, 106);
    let bg4 = Color::Rgb(86, 95, 137);
    let fg0 = Color::Rgb(192, 202, 245);
    let fg1 = Color::Rgb(169, 177, 214);
    let fg2 = Color::Rgb(146, 152, 183);
    let fg3 = Color::Rgb(120, 128, 163);
    let fg4 = Color::Rgb(96, 104, 142);
    let red = Color::Rgb(247, 118, 142);
    let green = Color::Rgb(158, 206, 106);
    let yellow = Color::Rgb(224, 175, 104);
    let blue = Color::Rgb(122, 162, 247);
    let purple = Color::Rgb(187, 154, 247);
    let aqua = Color::Rgb(125, 207, 255);
    let orange = Color::Rgb(255, 158, 100);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn tokyo_night_day() -> Theme {
    let bg0 = Color::Rgb(212, 216, 232);
    let bg1 = Color::Rgb(199, 203, 219);
    let bg2 = Color::Rgb(182, 188, 209);
    let bg3 = Color::Rgb(157, 163, 186);
    let bg4 = Color::Rgb(132, 139, 167);
    let fg0 = Color::Rgb(52, 54, 86);
    let fg1 = Color::Rgb(56, 62, 104);
    let fg2 = Color::Rgb(72, 78, 118);
    let fg3 = Color::Rgb(107, 111, 142);
    let fg4 = Color::Rgb(132, 139, 167);
    let red = Color::Rgb(143, 85, 115);
    let green = Color::Rgb(56, 110, 72);
    let yellow = Color::Rgb(142, 108, 32);
    let blue = Color::Rgb(52, 84, 138);
    let purple = Color::Rgb(92, 72, 138);
    let aqua = Color::Rgb(0, 114, 139);
    let orange = Color::Rgb(150, 96, 47);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn nord() -> Theme {
    let bg0 = Color::Rgb(46, 52, 64);
    let bg1 = Color::Rgb(59, 66, 82);
    let bg2 = Color::Rgb(67, 76, 94);
    let bg3 = Color::Rgb(76, 86, 106);
    let bg4 = Color::Rgb(94, 105, 126);
    let fg0 = Color::Rgb(236, 239, 244);
    let fg1 = Color::Rgb(229, 233, 240);
    let fg2 = Color::Rgb(216, 222, 233);
    let fg3 = Color::Rgb(200, 207, 220);
    let fg4 = Color::Rgb(180, 188, 204);
    let red = Color::Rgb(191, 97, 106);
    let green = Color::Rgb(163, 190, 140);
    let yellow = Color::Rgb(235, 203, 139);
    let blue = Color::Rgb(129, 161, 193);
    let purple = Color::Rgb(180, 142, 173);
    let aqua = Color::Rgb(143, 188, 187);
    let orange = Color::Rgb(208, 135, 112);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: blue,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: blue,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: orange,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}

pub fn dracula() -> Theme {
    let bg0 = Color::Rgb(40, 42, 54);
    let bg1 = Color::Rgb(68, 71, 90);
    let bg2 = Color::Rgb(80, 83, 102);
    let bg3 = Color::Rgb(98, 100, 118);
    let bg4 = Color::Rgb(118, 120, 136);
    let fg0 = Color::Rgb(248, 248, 242);
    let fg1 = Color::Rgb(230, 230, 224);
    let fg2 = Color::Rgb(210, 210, 206);
    let fg3 = Color::Rgb(190, 190, 188);
    let fg4 = Color::Rgb(170, 170, 170);
    let red = Color::Rgb(255, 85, 85);
    let green = Color::Rgb(80, 250, 123);
    let yellow = Color::Rgb(241, 250, 140);
    let blue = Color::Rgb(98, 114, 164);
    let purple = Color::Rgb(189, 147, 249);
    let aqua = Color::Rgb(139, 233, 253);
    let orange = Color::Rgb(255, 184, 108);

    Theme {
        bg0, bg1, bg2, bg3, bg4,
        fg0, fg1, fg2, fg3, fg4,
        red, green, yellow, blue, purple, aqua, orange,
        border_focused: purple,
        border_unfocused: bg3,
        border_overlay: orange,
        selected_fg: fg0,
        selected_bg: bg2,
        heading_1: orange,
        heading_2: yellow,
        heading_3: aqua,
        link_fg: purple,
        link_selected_fg: aqua,
        link_selected_bg: bg2,
        link_broken: red,
        tag_fg: yellow,
        inline_code: orange,
        title_fg: aqua,
        title_bar_bg: bg1,
        status_bar_bg: bg1,
        cursor_blink: green,
        empty_hint: fg4,
        dir_fg: yellow,
        file_fg: fg1,
        backlink_fg: purple,
        tag_filter_border: yellow,
        search_prompt: green,
        finder_prompt: purple,
        autocomplete_bg: bg1,
        autocomplete_sel_bg: bg2,
    }
}
