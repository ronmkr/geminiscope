use crate::app::App;
use crate::models::{View, ProjectSort, State};
use ratatui::{
    layout::Rect,
    widgets::ListItem,
    Frame,
};

pub trait ViewHandler {
    fn count(&self, state: &State, search_query: &str) -> usize;
    fn list_items<'a>(&self, state: &'a State, search_query: &str, sort_mode: ProjectSort) -> Vec<ListItem<'a>>;
    fn render_detail(&self, f: &mut Frame, app: &App, area: Rect);
}

pub struct ExplorerHandler {
    pub view: View,
}

impl ViewHandler for ExplorerHandler {
    fn count(&self, state: &State, search_query: &str) -> usize {
        match self.view {
            View::Chats | View::Tools => state.filtered_sessions(search_query).len(),
            View::Timeline => state.timeline.len(),
            _ => 0,
        }
    }

    fn list_items<'a>(&self, state: &'a State, search_query: &str, _sort_mode: ProjectSort) -> Vec<ListItem<'a>> {
        crate::ui::explorer::get_explorer_list_items(state, self.view, search_query)
    }

    fn render_detail(&self, f: &mut Frame, app: &App, area: Rect) {
        crate::ui::explorer::render_explorer_detail(f, app, area);
    }
}

pub struct StatsHandler;

impl ViewHandler for StatsHandler {
    fn count(&self, state: &State, _search_query: &str) -> usize {
        state.projects.len() + 1
    }

    fn list_items<'a>(&self, state: &'a State, _search_query: &str, sort_mode: ProjectSort) -> Vec<ListItem<'a>> {
        crate::ui::stats::get_stats_list_items(state, sort_mode)
    }

    fn render_detail(&self, f: &mut Frame, app: &App, area: Rect) {
        crate::ui::stats::render_stats_detail(f, app, area);
    }
}

pub struct InfraHandler {
    pub view: View,
}

impl ViewHandler for InfraHandler {
    fn count(&self, state: &State, _search_query: &str) -> usize {
        match self.view {
            View::Memory => state.projects.iter().map(|p| p.memory_files.len()).sum(),
            View::Plans => state.projects.iter().map(|p| p.plan_files.len()).sum(),
            View::Health => state.health.len(),
            View::Skills => state.skills.len(),
            View::Mcp => state.mcp_servers.len(),
            View::Settings => {
                let mut count = 0;
                let mut last_section = String::new();
                for (path, _, _) in crate::ui::infrastructure::get_all_settings(state) {
                    let section = path.split('.').next().unwrap_or("").to_uppercase();
                    if section != last_section {
                        count += 1; // Section header
                        last_section = section;
                    }
                    count += 1; // Setting item
                }
                count
            },
            _ => 0,
        }
    }

    fn list_items<'a>(&self, state: &'a State, _search_query: &str, _sort_mode: ProjectSort) -> Vec<ListItem<'a>> {
        crate::ui::infrastructure::get_infra_list_items(state, self.view)
    }

    fn render_detail(&self, f: &mut Frame, app: &App, area: Rect) {
        crate::ui::infrastructure::render_infra_detail(f, app, area);
    }
}

pub struct DiffHandler;

impl ViewHandler for DiffHandler {
    fn count(&self, _state: &State, _search_query: &str) -> usize { 1 }
    fn list_items<'a>(&self, _state: &'a State, _search_query: &str, _sort_mode: ProjectSort) -> Vec<ListItem<'a>> { Vec::new() }
    fn render_detail(&self, f: &mut Frame, app: &App, area: Rect) {
        if let Some((_, _, diff_text)) = &app.diff_results {
            crate::ui::render_markdown(f, app, area, "Session Comparison", diff_text);
        }
    }
}

pub fn get_handler(view: View) -> Box<dyn ViewHandler> {
    match view {
        View::Chats | View::Tools | View::Timeline => Box::new(ExplorerHandler { view }),
        View::Stats => Box::new(StatsHandler),
        View::Diff => Box::new(DiffHandler),
        _ => Box::new(InfraHandler { view }),
    }
}
