use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use zellij_tile::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct State {
    username: String,
    tabs: Vec<Tab>,
    tab_hit_rows: Vec<(usize, usize)>,
}

#[cfg(target_arch = "wasm32")]
register_plugin!(State);

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

#[cfg(target_arch = "wasm32")]
impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.username = username_from_configuration_or_environment(&configuration);
        eprintln!("vertical-sidebar loaded username={:?}", self.username);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::TabUpdate, EventType::Mouse]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::TabUpdate(tab_infos) => {
                self.tabs = tab_infos.into_iter().map(Tab::from).collect();
                true
            }
            Event::PermissionRequestResult(status) => {
                eprintln!("vertical-sidebar permission result={status:?}");
                false
            }
            Event::Mouse(Mouse::LeftClick(line, _column)) => {
                if let Some(tab_index) = tab_index_for_row(line, &self.tab_hit_rows) {
                    eprintln!("vertical-sidebar clicked line={line} tab_index={tab_index}");
                    switch_tab_to(tab_index as u32);
                } else {
                    eprintln!("vertical-sidebar ignored click line={line}");
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        eprintln!(
            "vertical-sidebar render rows={} cols={} username={:?} tabs={}",
            rows,
            cols,
            self.username,
            self.tabs.len()
        );

        if rows == 0 || cols == 0 {
            return;
        }

        let sidebar = sidebar_render(&self.username, &self.tabs, rows, cols);
        self.tab_hit_rows = sidebar.tab_hit_rows;

        for line in sidebar.lines {
            println!("{}", line.text);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tab {
    position: usize,
    name: String,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderLine {
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidebarRender {
    lines: Vec<RenderLine>,
    tab_hit_rows: Vec<(usize, usize)>,
}

#[cfg(target_arch = "wasm32")]
impl From<TabInfo> for Tab {
    fn from(tab_info: TabInfo) -> Self {
        Self {
            position: tab_info.position,
            name: tab_info.name,
            active: tab_info.active,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn username_from_configuration_or_environment(configuration: &BTreeMap<String, String>) -> String {
    configuration
        .get("username")
        .cloned()
        .or_else(|| std::env::var("USER").ok())
        .or_else(|| std::env::var("LOGNAME").ok())
        .or_else(|| std::env::var("USERNAME").ok())
        .filter(|username| !username.trim().is_empty())
        .unwrap_or_else(|| "user".to_owned())
}

fn clipped_line(text: &str, cols: usize) -> String {
    if text.chars().count() > cols {
        return text.chars().take(cols).collect();
    }

    text.to_owned()
}

fn sidebar_render(username: &str, tabs: &[Tab], rows: usize, cols: usize) -> SidebarRender {
    let mut lines = Vec::new();
    let mut tab_hit_rows = Vec::new();

    push_line(&mut lines, styled_header(username, cols), rows);
    push_line(&mut lines, divider(cols), rows);
    push_line(&mut lines, String::new(), rows);
    push_line(&mut lines, muted_line("tabs", cols), rows);

    if tabs.is_empty() {
        let row = lines.len();
        if push_line(&mut lines, tab_line(0, ">", "1", cols), rows) {
            tab_hit_rows.push((row, 1));
        }
        return SidebarRender {
            lines,
            tab_hit_rows,
        };
    }

    for (index, tab) in tabs.iter().enumerate() {
        if index > 0 {
            let spacer_row = lines.len();
            if push_line(&mut lines, String::new(), rows) {
                tab_hit_rows.push((spacer_row, tabs[index - 1].position + 1));
            }

            let divider_row = lines.len();
            if push_line(&mut lines, item_divider(cols), rows) {
                tab_hit_rows.push((divider_row, tab.position + 1));
            }
        }

        let row = lines.len();
        if push_line(&mut lines, tab_render_line(tab, cols), rows) {
            tab_hit_rows.push((row, tab.position + 1));
        }
    }

    SidebarRender {
        lines,
        tab_hit_rows,
    }
}

fn push_line(lines: &mut Vec<RenderLine>, text: String, max_rows: usize) -> bool {
    if lines.len() >= max_rows {
        return false;
    }
    lines.push(RenderLine { text });
    true
}

fn styled_header(username: &str, cols: usize) -> String {
    ansi_bold(&clipped_line(username, cols))
}

fn muted_line(text: &str, cols: usize) -> String {
    ansi_dim(&clipped_line(text, cols))
}

fn divider(cols: usize) -> String {
    ansi_dim(&"-".repeat(cols))
}

fn item_divider(cols: usize) -> String {
    if cols <= 2 {
        return ansi_dim(&"⎺".repeat(cols));
    }

    ansi_dim(&format!(" {} ", "⎺".repeat(cols - 2)))
}

fn tab_render_line(tab: &Tab, cols: usize) -> String {
    let marker = if tab.active { ">>" } else { "  " };
    let fallback_name = (tab.position + 1).to_string();
    let name = if tab.name.trim().is_empty() {
        fallback_name.as_str()
    } else {
        tab.name.as_str()
    };
    let display_name = if tab.active {
        name.to_uppercase()
    } else {
        name.to_owned()
    };
    let line = tab_line(tab.position, marker, &display_name, cols);
    if tab.active {
        ansi_reverse(&ansi_bold(&line))
    } else {
        ansi_bold(&line)
    }
}

fn tab_line(position: usize, marker: &str, name: &str, cols: usize) -> String {
    let padded_position = (position + 1).to_string();
    clipped_line(&format!("{padded_position} {marker} {name}"), cols)
}

fn tab_index_for_row(row: isize, tab_hit_rows: &[(usize, usize)]) -> Option<usize> {
    let row = usize::try_from(row).ok()?;
    tab_hit_rows
        .iter()
        .find_map(|(hit_row, tab_index)| (*hit_row == row).then_some(*tab_index))
}

fn ansi_bold(text: &str) -> String {
    format!("\u{1b}[1m{text}\u{1b}[0m")
}

fn ansi_dim(text: &str) -> String {
    format!("\u{1b}[2m{text}\u{1b}[0m")
}

fn ansi_reverse(text: &str) -> String {
    format!("\u{1b}[7m{text}\u{1b}[0m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_prefers_configuration() {
        let mut configuration = BTreeMap::new();
        configuration.insert("username".to_owned(), "configured".to_owned());

        assert_eq!(
            username_from_configuration_or_environment(&configuration),
            "configured"
        );
    }

    #[test]
    fn clipped_line_keeps_text_that_fits() {
        assert_eq!(clipped_line("SIDEBAR", 10), "SIDEBAR");
    }

    #[test]
    fn clipped_line_truncates_when_needed() {
        assert_eq!(clipped_line("SIDEBAR", 3), "SID");
    }

    #[test]
    fn sidebar_render_shows_sections_and_tabs() {
        let tabs = vec![
            Tab {
                position: 0,
                name: "one".to_owned(),
                active: true,
            },
            Tab {
                position: 1,
                name: "two".to_owned(),
                active: false,
            },
        ];

        let sidebar = sidebar_render("kmert", &tabs, 8, 12);
        let lines: Vec<_> = sidebar
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect();

        assert_eq!(lines[0], "\u{1b}[1mkmert\u{1b}[0m");
        assert_eq!(lines[1], "\u{1b}[2m------------\u{1b}[0m");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "\u{1b}[2mtabs\u{1b}[0m");
        assert_eq!(lines[4], "\u{1b}[7m\u{1b}[1m1 >> ONE\u{1b}[0m\u{1b}[0m");
        assert_eq!(lines[5], "");
        assert_eq!(lines[6], "\u{1b}[2m ⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺ \u{1b}[0m");
        assert_eq!(lines[7], "\u{1b}[1m2    two\u{1b}[0m");
        assert_eq!(sidebar.tab_hit_rows, vec![(4, 1), (5, 1), (6, 2), (7, 2)]);
    }

    #[test]
    fn sidebar_render_shows_fallback_tab_before_tab_update() {
        let sidebar = sidebar_render("kmert", &[], 5, 12);
        let lines: Vec<_> = sidebar
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect();

        assert_eq!(lines[4], "1 > 1");
        assert_eq!(sidebar.tab_hit_rows, vec![(4, 1)]);
    }

    #[test]
    fn tab_line_uses_position_when_name_is_empty() {
        let tab = Tab {
            position: 2,
            name: String::new(),
            active: true,
        };

        assert_eq!(
            tab_render_line(&tab, 10),
            "\u{1b}[7m\u{1b}[1m3 >> 3\u{1b}[0m\u{1b}[0m"
        );
    }

    #[test]
    fn tab_index_for_row_maps_rendered_tab_rows_to_one_based_tab_indexes() {
        let tab_hit_rows = vec![(3, 1), (4, 3)];

        assert_eq!(tab_index_for_row(3, &tab_hit_rows), Some(1));
        assert_eq!(tab_index_for_row(4, &tab_hit_rows), Some(3));
    }

    #[test]
    fn tab_index_for_row_ignores_non_tab_rows() {
        let tab_hit_rows = vec![(3, 1)];

        assert_eq!(tab_index_for_row(2, &tab_hit_rows), None);
        assert_eq!(tab_index_for_row(-1, &tab_hit_rows), None);
    }
}
