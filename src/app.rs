use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::color::Color;
use crate::evaluator::{EvalError, ResolvedTheme, resolve_theme};
use crate::export::Exporter;
use crate::export::alacritty::AlacrittyExporter;
use crate::palette::{Palette, generate_palette};
use crate::params::{ParamKey, ThemeParams};
use crate::rules::{AdjustOp, Rule, RuleKind, RuleSet, SourceRef, available_sources, starter_rule};
use crate::tokens::{PaletteSlot, TokenRole};
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Tokens,
    Params,
    Inspector,
}

impl FocusPane {
    pub const fn next(self) -> Self {
        match self {
            Self::Tokens => Self::Params,
            Self::Params => Self::Inspector,
            Self::Inspector => Self::Tokens,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Tokens => Self::Inspector,
            Self::Params => Self::Tokens,
            Self::Inspector => Self::Params,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Tokens => "Tokens",
            Self::Params => "Params",
            Self::Inspector => "Inspector",
        }
    }
}

pub struct App {
    pub params: ThemeParams,
    pub rules: RuleSet,
    pub palette: Palette,
    pub resolved: ResolvedTheme,
    pub selected_token: usize,
    pub selected_param: usize,
    pub inspector_field: usize,
    pub focus: FocusPane,
    pub status: String,
    pub should_quit: bool,
    pub export_path: PathBuf,
}

impl App {
    pub fn new() -> Result<Self, EvalError> {
        let params = ThemeParams::default();
        let rules = RuleSet::default();
        let palette = generate_palette(&params);
        let resolved = resolve_theme(palette.clone(), &rules)?;

        Ok(Self {
            params,
            rules,
            palette,
            resolved,
            selected_token: 0,
            selected_param: 0,
            inspector_field: 0,
            focus: FocusPane::Tokens,
            status: "Theme generator ready.".to_string(),
            should_quit: false,
            export_path: PathBuf::from("exports/alacritty-theme.toml"),
        })
    }

    pub fn selected_role(&self) -> TokenRole {
        TokenRole::ALL[self.selected_token]
    }

    pub fn selected_param_key(&self) -> ParamKey {
        ParamKey::ALL[self.selected_param]
    }

    pub fn selected_rule(&self) -> &Rule {
        self.rules
            .get(self.selected_role())
            .expect("every token should have a rule")
    }

    pub fn current_token_color(&self) -> Color {
        self.resolved
            .token(self.selected_role())
            .expect("resolved theme should contain every token")
    }

    pub fn theme_color(&self, role: TokenRole) -> Color {
        self.resolved
            .token(role)
            .expect("resolved theme should contain every token")
    }

    pub fn palette_color(&self, slot: PaletteSlot) -> Color {
        self.palette
            .get(slot)
            .expect("palette should contain every slot")
    }

    pub fn inspector_field_count(&self) -> usize {
        1 + match self.selected_rule() {
            Rule::Alias { .. } => 1,
            Rule::Mix { .. } => 3,
            Rule::Adjust { .. } => 3,
            Rule::Fixed { .. } => 1,
        }
    }

    pub fn tick(&mut self) {}

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.focus = self.focus.next(),
            KeyCode::BackTab => self.focus = self.focus.previous(),
            KeyCode::Char('e') => self.export_alacritty(),
            KeyCode::Char('r') => self.reset(),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Left | KeyCode::Char('h') => self.adjust_current(-1),
            KeyCode::Right | KeyCode::Char('l') => self.adjust_current(1),
            _ => {}
        }
    }

    fn move_selection(&mut self, direction: i32) {
        match self.focus {
            FocusPane::Tokens => {
                self.selected_token =
                    cycle_index(self.selected_token, TokenRole::ALL.len(), direction);
                self.inspector_field = self
                    .inspector_field
                    .min(self.inspector_field_count().saturating_sub(1));
                self.status = format!("Selected token {}", self.selected_role().label());
            }
            FocusPane::Params => {
                self.selected_param =
                    cycle_index(self.selected_param, ParamKey::ALL.len(), direction);
                self.status = format!("Selected param {}", self.selected_param_key().label());
            }
            FocusPane::Inspector => {
                self.inspector_field = cycle_index(
                    self.inspector_field,
                    self.inspector_field_count(),
                    direction,
                );
            }
        }
    }

    fn adjust_current(&mut self, direction: i32) {
        match self.focus {
            FocusPane::Tokens => {}
            FocusPane::Params => self.adjust_param(direction),
            FocusPane::Inspector => self.adjust_rule(direction),
        }
    }

    fn adjust_param(&mut self, direction: i32) {
        let key = self.selected_param_key();
        let mut params = self.params.clone();
        key.adjust(&mut params, direction);
        self.params = params;
        if let Err(err) = self.recompute() {
            self.status = format!("Failed to update {}: {err}", key.label());
        } else {
            self.status = format!("{} -> {}", key.label(), key.format_value(&self.params));
        }
    }

    fn adjust_rule(&mut self, direction: i32) {
        let role = self.selected_role();
        let current_color = self.current_token_color();
        let selection_mix = self.params.selection_mix;
        let field = self.inspector_field;
        let fixed_color_options = self.fixed_color_options();

        self.try_mutate_rules(|rules| {
            let rule = rules
                .get_mut(role)
                .expect("selected token should always be editable");
            match field {
                0 => {
                    let current = rule.kind();
                    let next = cycle_rule_kind(current, direction);
                    *rule = starter_rule(next, role, current_color, selection_mix);
                }
                _ => match rule {
                    Rule::Alias { source } => {
                        if field == 1 {
                            *source = cycle_source(source, role, direction);
                        }
                    }
                    Rule::Mix { a, b, ratio } => match field {
                        1 => *a = cycle_source(a, role, direction),
                        2 => *b = cycle_source(b, role, direction),
                        3 => *ratio = (*ratio + direction as f32 * 0.05).clamp(0.0, 1.0),
                        _ => {}
                    },
                    Rule::Adjust { source, op, amount } => match field {
                        1 => *source = cycle_source(source, role, direction),
                        2 => *op = cycle_adjust_op(*op, direction),
                        3 => *amount = (*amount + direction as f32 * 0.02).clamp(0.0, 1.0),
                        _ => {}
                    },
                    Rule::Fixed { color } => {
                        if field == 1 {
                            *color = cycle_fixed_color(*color, &fixed_color_options, direction);
                        }
                    }
                },
            }
        });
    }

    fn try_mutate_rules(&mut self, update: impl FnOnce(&mut RuleSet)) {
        let previous_rules = self.rules.clone();
        update(&mut self.rules);

        if let Err(err) = self.recompute() {
            self.rules = previous_rules;
            let _ = self.recompute();
            self.status = format!("Rule change rejected: {err}");
        } else {
            self.inspector_field = self
                .inspector_field
                .min(self.inspector_field_count().saturating_sub(1));
            self.status = format!("Updated {}", self.selected_role().label());
        }
    }

    fn recompute(&mut self) -> Result<(), EvalError> {
        self.palette = generate_palette(&self.params);
        self.resolved = resolve_theme(self.palette.clone(), &self.rules)?;
        Ok(())
    }

    fn export_alacritty(&mut self) {
        let exporter = AlacrittyExporter;
        let path = self.export_path.clone();

        match exporter.export(&self.resolved) {
            Ok(output) => match write_export(&path, &output) {
                Ok(()) => {
                    self.status =
                        format!("Exported {} theme to {}", exporter.name(), path.display());
                }
                Err(err) => {
                    self.status = format!("Failed to write export: {err}");
                }
            },
            Err(err) => {
                self.status = format!("Export failed: {err}");
            }
        }
    }

    fn reset(&mut self) {
        self.params = ThemeParams::default();
        self.rules = RuleSet::default();
        self.selected_token = 0;
        self.selected_param = 0;
        self.inspector_field = 0;
        match self.recompute() {
            Ok(()) => self.status = "Reset to defaults.".to_string(),
            Err(err) => self.status = format!("Reset failed: {err}"),
        }
    }

    fn fixed_color_options(&self) -> Vec<Color> {
        let mut options = Vec::new();
        for slot in PaletteSlot::ALL {
            options.push(self.palette_color(slot));
        }
        for role in TokenRole::ALL {
            options.push(self.theme_color(role));
        }
        options.extend([
            Color::from_hex("#FF6B6B").expect("valid color"),
            Color::from_hex("#FFD166").expect("valid color"),
            Color::from_hex("#06D6A0").expect("valid color"),
            Color::from_hex("#4DABF7").expect("valid color"),
            Color::from_hex("#C77DFF").expect("valid color"),
        ]);
        options
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if app.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        app.tick();
    }
}

fn cycle_index(current: usize, len: usize, direction: i32) -> usize {
    let len = len as i32;
    ((current as i32 + direction).rem_euclid(len)) as usize
}

fn cycle_rule_kind(current: RuleKind, direction: i32) -> RuleKind {
    let index = RuleKind::ALL
        .iter()
        .position(|kind| *kind == current)
        .unwrap_or_default();
    RuleKind::ALL[cycle_index(index, RuleKind::ALL.len(), direction)]
}

fn cycle_source(current: &SourceRef, role: TokenRole, direction: i32) -> SourceRef {
    let options = available_sources(role);
    let index = options
        .iter()
        .position(|option| option == current)
        .unwrap_or_default();
    options[cycle_index(index, options.len(), direction)].clone()
}

fn cycle_adjust_op(current: AdjustOp, direction: i32) -> AdjustOp {
    let index = AdjustOp::ALL
        .iter()
        .position(|op| *op == current)
        .unwrap_or_default();
    AdjustOp::ALL[cycle_index(index, AdjustOp::ALL.len(), direction)]
}

fn cycle_fixed_color(current: Color, options: &[Color], direction: i32) -> Color {
    let index = options
        .iter()
        .position(|candidate| candidate.approx_eq(current))
        .unwrap_or_default();
    options[cycle_index(index, options.len(), direction)]
}

fn write_export(path: &PathBuf, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
