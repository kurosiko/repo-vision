use std::{collections::BTreeSet, ops::Not};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

use crate::apply_changes;

use super::{RepoData, RepoType};

#[derive(Debug, Clone, Copy)]
enum RepoFilter {
    Public,
    Private,
    Star,
    None,
}

impl Not for RepoType {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Public => Self::Private,
            Self::Private => Self::Public,
        }
    }
}

impl RepoFilter {
    fn next(self) -> Self {
        match self {
            Self::Public => Self::Private,
            Self::Private => Self::Star,
            Self::Star => Self::None,
            Self::None => Self::Public,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Private => "Private",
            Self::Star => "With stars",
            Self::None => "All",
        }
    }

    fn matches(self, repo: &RepoData) -> bool {
        match self {
            Self::Public => matches!(repo.visibility, RepoType::Public),
            Self::Private => matches!(repo.visibility, RepoType::Private),
            Self::Star => repo.stargazer_count > 0,
            Self::None => true,
        }
    }
}

pub(crate) fn run(terminal: &mut DefaultTerminal, repos: &mut [RepoData]) -> std::io::Result<()> {
    let mut table_state = TableState::default();
    if !repos.is_empty() {
        table_state.select(Some(0));
    }

    app(terminal, repos, &mut table_state)
}

fn app(
    terminal: &mut DefaultTerminal,
    repos: &mut [RepoData],
    table_state: &mut TableState,
) -> std::io::Result<()> {
    let mut filter = RepoFilter::None;
    let mut original_visibility: Vec<_> = repos.iter().map(|repo| repo.visibility).collect();
    let mut changed_indices: BTreeSet<usize> = BTreeSet::new();
    let mut popup_open = false;

    loop {
        let visible_indices: Vec<usize> = repos
            .iter()
            .enumerate()
            .filter(|(_, repo)| filter.matches(repo))
            .map(|(index, _)| index)
            .collect();

        if table_state
            .selected()
            .is_some_and(|selected| selected >= visible_indices.len())
        {
            table_state.select(visible_indices.len().checked_sub(1));
        }

        terminal.draw(|frame| {
            render(frame, repos, &visible_indices, table_state, filter);
            if popup_open {
                render_popup(
                    frame,
                    changed_indices.iter().filter_map(|&index| repos.get(index)),
                );
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc if popup_open => popup_open = false,
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('p') => popup_open = !popup_open,
                KeyCode::Char('y') if popup_open => {
                    apply_changes(changed_indices.iter().filter_map(|&index| repos.get(index)))
                        .map_err(std::io::Error::other)?;
                    for &index in &changed_indices {
                        original_visibility[index] = repos[index].visibility;
                    }
                    changed_indices.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => table_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => table_state.select_previous(),
                KeyCode::Char('g') | KeyCode::Home => table_state.select_first(),
                KeyCode::Char('G') | KeyCode::End => table_state.select_last(),
                KeyCode::Tab => {
                    filter = filter.next();
                    table_state.select(Some(0));
                }
                KeyCode::Char(' ') => {
                    if let Some(index) = table_state
                        .selected()
                        .and_then(|selected| visible_indices.get(selected).copied())
                    {
                        repos[index].visibility = !repos[index].visibility;
                        if repos[index].visibility == original_visibility[index] {
                            changed_indices.remove(&index);
                        } else {
                            changed_indices.insert(index);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn render_popup<'a>(frame: &mut Frame, diff_data: impl Iterator<Item = &'a RepoData>) {
    let popup_area = frame
        .area()
        .centered(Constraint::Percentage(80), Constraint::Percentage(70));
    frame.render_widget(Clear, popup_area);

    let [table_area, help_area] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(popup_area);

    let changed_repos: Vec<_> = diff_data.collect();
    let rows = if changed_repos.is_empty() {
        vec![Row::new(["No pending changes", "", ""])]
    } else {
        changed_repos
            .iter()
            .map(|repo| {
                Row::new([
                    Cell::from(repo.name.as_str()),
                    Cell::from(Span::styled(
                        repo.visibility.as_str(),
                        Style::default().fg(match repo.visibility {
                            RepoType::Private => Color::Red,
                            RepoType::Public => Color::Green,
                        }),
                    )),
                    Cell::from(repo.description.as_deref().unwrap_or("—")),
                ])
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(55),
        ],
    )
    .header(
        Row::new(["Repository", "Visibility", "Description"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Pending changes"),
    );

    frame.render_widget(table, table_area);
    frame.render_widget(
        Paragraph::new("y: apply visibility changes to GitHub · p / Esc: close"),
        help_area,
    );
}

fn render(
    frame: &mut Frame,
    repos: &[RepoData],
    visible_indices: &[usize],
    table_state: &mut TableState,
    filter: RepoFilter,
) {
    let [table_area, help_area] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(frame.area());

    let rows = visible_indices.iter().map(|&index| {
        let repo = &repos[index];
        Row::new([
            Cell::from(repo.name.as_str()),
            Cell::from(Span::styled(
                repo.visibility.as_str(),
                Style::default().fg(match repo.visibility {
                    RepoType::Private => Color::Red,
                    RepoType::Public => Color::Green,
                }),
            )),
            Cell::from(repo.description.as_deref().unwrap_or("—")),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(55),
        ],
    )
    .header(
        Row::new(["Repository", "Visibility", "Description"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ")
    .block(Block::default().borders(Borders::ALL).title("Repo-Vision"));

    frame.render_stateful_widget(table, table_area, table_state);

    let help = Paragraph::new(format!(
        "Filter: {} · Tab: change · Space: toggle visibility locally · ↑/↓ or j/k: navigate · q/Esc: quit · p: pending changes  {}/{}",
        filter.label(),
        table_state.selected().map_or(0, |index| index + 1),
        visible_indices.len(),
    ));
    frame.render_widget(help, help_area);
}
