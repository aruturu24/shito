use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Tabs};
use ratatui::Terminal;
use std::io::Stdout;
use std::time::{Duration, Instant};

use crate::db::Db;
use crate::dice;
use crate::models::{all_skills, Character};

pub enum Mode {
    List,
    Details,
    Edit,
    EditAddItem,
    CreateName,
    CreateClass,
    CreateRace,
    CreateAbilities,
    CreateHpMax,
    CreateAc,
    CreateSpeed,
    CreateSkills,
    Roll,
}

pub struct App {
    pub db: Db,
    pub items: Vec<Character>,
    pub selected: usize,
    pub mode: Mode,
    pub input: String,
    pub status: String,
    pub last_tick: Instant,
    wizard: Option<NewCharDraft>,
    selected_spell_level: usize,
    detail_tab: usize,
}

impl App {
    pub fn new(db: Db) -> Result<Self> {
        let items = db.list_characters()?;
        Ok(Self {
            db,
            items,
            selected: 0,
            mode: Mode::List,
            input: String::new(),
            status: String::from("q: quit • n: new • e: edit • d: delete • r: roll • +/- hp • [/] slot • 1-9 select slot"),
            last_tick: Instant::now(),
            wizard: None,
            selected_spell_level: 1,
            detail_tab: 0,
        })
    }

    fn current_mut(&mut self) -> Option<&mut Character> {
        self.items.get_mut(self.selected)
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let tick_rate = Duration::from_millis(200);

        let res = self.loop_draw(&mut terminal, tick_rate);

        // Restore terminal
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::event::DisableMouseCapture,
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;
        res
    }

    fn loop_draw(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>, tick_rate: Duration) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(self.last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.on_key(key.code)? { break; }
                    }
                }
            }
            if self.last_tick.elapsed() >= tick_rate {
                self.last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn on_key(&mut self, code: KeyCode) -> Result<bool> {
        match self.mode {
            Mode::List => match code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.items.is_empty() {
                        self.selected = (self.selected + 1).min(self.items.len().saturating_sub(1));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.items.is_empty() {
                        self.selected = self.selected.saturating_sub(1);
                    }
                }
                KeyCode::Enter => {
                    if !self.items.is_empty() { self.mode = Mode::Details; self.status = details_status(); }
                }
                KeyCode::Char('n') => {
                    self.mode = Mode::CreateName;
                    self.input.clear();
                    self.status = String::from("Create: Enter name. Enter to confirm. Esc to cancel.");
                    self.wizard = Some(NewCharDraft::default());
                }
                KeyCode::Char('e') => {
                    if self.current_mut().is_some() {
                        self.mode = Mode::Edit;
                        self.status = String::from("Editing: +/- hp • [/] adjust • 1-9 select slot • a/A add/remove item • l level up • s save • Esc cancel");
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(charac) = self.items.get(self.selected).cloned() {
                        if let Some(id) = charac.id { let _ = self.db.delete_character(id); }
                        let _ = self.reload();
                    }
                }
                KeyCode::Char('r') => {
                    self.mode = Mode::Roll;
                    self.input.clear();
                    self.status = String::from("Type dice (e.g., d20, 2d6) then Enter. Esc cancel");
                }
                _ => {}
            },
            Mode::CreateName => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Some(w) = &mut self.wizard { w.name = if self.input.trim().is_empty(){"Unnamed".into()} else { self.input.trim().into() }; }
                    self.input.clear();
                    self.mode = Mode::CreateClass; self.status = String::from("Enter class");
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateClass => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Some(w) = &mut self.wizard { w.class_name = if self.input.trim().is_empty(){"Fighter".into()} else { self.input.trim().into() }; }
                    self.input.clear();
                    self.mode = Mode::CreateRace; self.status = String::from("Enter race");
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateRace => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Some(w) = &mut self.wizard { w.race = if self.input.trim().is_empty(){"Human".into()} else { self.input.trim().into() }; }
                    self.input.clear();
                    self.mode = Mode::CreateAbilities; self.status = String::from("Enter abilities as STR DEX CON INT WIS CHA (e.g., 15 14 13 12 10 8)");
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateAbilities => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    let nums: Vec<i32> = self.input.split_whitespace().filter_map(|s| s.parse::<i32>().ok()).collect();
                    if nums.len() != 6 { self.status = String::from("Please enter exactly 6 numbers"); }
                    else {
                        if let Some(w) = &mut self.wizard { w.abilities = [nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]]; }
                        self.input.clear();
                        self.mode = Mode::CreateHpMax; self.status = String::from("Enter HP Max (number)");
                    }
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateHpMax => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Ok(v) = self.input.trim().parse::<i32>() {
                        if let Some(w) = &mut self.wizard { w.hp_max = v.max(1); w.hp_current = w.hp_max; }
                        self.input.clear();
                        self.mode = Mode::CreateAc; self.status = String::from("Enter Armor Class (AC) number");
                    } else { self.status = String::from("Please enter a valid number"); }
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateAc => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Ok(v) = self.input.trim().parse::<i32>() {
                        if let Some(w) = &mut self.wizard { w.armor_class = v; }
                        self.input.clear();
                        self.mode = Mode::CreateSpeed; self.status = String::from("Enter Speed (ft) number");
                    } else { self.status = String::from("Please enter a valid number"); }
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateSpeed => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    if let Ok(v) = self.input.trim().parse::<i32>() {
                        if let Some(w) = &mut self.wizard { w.speed = v; }
                        self.input.clear();
                        self.mode = Mode::CreateSkills; self.status = String::from("Enter skills separated by comma or semicolon (e.g., perception, stealth or stealth; perception)");
                    } else { self.status = String::from("Please enter a valid number"); }
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::CreateSkills => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Enter => {
                    let valids: Vec<String> = all_skills().into_iter().map(|(n, _)| n.to_string()).collect();
                    let normalized = self.input.replace(';', ",");
                    let picked: Vec<String> = normalized
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty() && valids.contains(s))
                        .collect();
                    if let Some(mut w) = self.wizard.take() {
                        w.skill_proficiencies = picked;
                        let mut c = Character::default();
                        c.name = w.name;
                        c.class_name = w.class_name;
                        c.race = w.race;
                        c.strength = w.abilities[0];
                        c.dexterity = w.abilities[1];
                        c.constitution = w.abilities[2];
                        c.intelligence = w.abilities[3];
                        c.wisdom = w.abilities[4];
                        c.charisma = w.abilities[5];
                        c.hp_max = w.hp_max.max(1);
                        c.hp_current = w.hp_current.clamp(0, c.hp_max);
                        c.armor_class = w.armor_class;
                        c.speed = w.speed;
                        c.skill_proficiencies = w.skill_proficiencies;
                        let _ = self.db.insert_character(&mut c);
                    }
                    let _ = self.reload();
                    self.mode = Mode::List; self.input.clear(); self.status = default_status();
                }
                KeyCode::Char(ch) => { self.input.push(ch); }
                _ => {}
            },
            Mode::Details => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Left | KeyCode::Char('h') => { self.detail_tab = self.detail_tab.saturating_sub(1); }
                KeyCode::Right | KeyCode::Char('l') => { self.detail_tab = (self.detail_tab + 1).min(2); }
                KeyCode::Char('e') => { if self.current_mut().is_some() { self.mode = Mode::Edit; self.status = edit_status(); } }
                KeyCode::Char('r') => { self.mode = Mode::Roll; self.input.clear(); self.status = String::from("Type: NdM, skill, or ability. Esc cancel"); }
                _ => {}
            },
            Mode::Edit => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.status = default_status(); }
                KeyCode::Char('+') => { if let Some(c) = self.current_mut(){ c.change_hp(1); let _ = self.save_current(); } }
                KeyCode::Char('-') => { if let Some(c) = self.current_mut(){ c.change_hp(-1); let _ = self.save_current(); } }
                KeyCode::Char('l') => { if let Some(c) = self.current_mut(){ c.level_up(); let _ = self.save_current(); } }
                KeyCode::Left | KeyCode::Char('h') => { self.detail_tab = self.detail_tab.saturating_sub(1); }
                KeyCode::Right | KeyCode::Char('l') => { self.detail_tab = (self.detail_tab + 1).min(2); }
                KeyCode::Char('1') => { self.selected_spell_level = 1; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('2') => { self.selected_spell_level = 2; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('3') => { self.selected_spell_level = 3; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('4') => { self.selected_spell_level = 4; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('5') => { self.selected_spell_level = 5; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('6') => { self.selected_spell_level = 6; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('7') => { self.selected_spell_level = 7; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('8') => { self.selected_spell_level = 8; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('9') => { self.selected_spell_level = 9; self.status = format!("Editing: Slot L{} selected", self.selected_spell_level); }
                KeyCode::Char('[') => {
                    let lvl = self.selected_spell_level;
                    if let Some(c) = self.current_mut(){ c.adjust_spell_slot(lvl, -1); }
                    let _ = self.save_current();
                }
                KeyCode::Char(']') => {
                    let lvl = self.selected_spell_level;
                    if let Some(c) = self.current_mut(){ c.adjust_spell_slot(lvl, 1); }
                    let _ = self.save_current();
                }
                KeyCode::Char('a') => { self.mode = Mode::EditAddItem; self.status = String::from("Type item then Enter to add. Esc cancel"); self.input.clear(); }
                KeyCode::Char('A') => { if let Some(c) = self.current_mut(){ if !c.inventory.is_empty(){ c.remove_item(c.inventory.len()-1); let _ = self.save_current(); } } }
                KeyCode::Char('s') => { let _ = self.save_current(); self.status = String::from("Saved."); }
                _ => {}
            },
            Mode::EditAddItem => match code {
                KeyCode::Esc => { self.mode = Mode::Edit; self.status = edit_status(); }
                KeyCode::Enter => {
                    let item = self.input.trim().to_string();
                    self.input.clear();
                    if let Some(c) = self.current_mut(){ c.add_item(item); let _ = self.save_current(); }
                    self.mode = Mode::Edit;
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
            Mode::Roll => match code {
                KeyCode::Esc => { self.mode = Mode::List; self.input.clear(); self.status = default_status(); }
                KeyCode::Enter => {
                    let inp = self.input.trim().to_lowercase();
                    let name = self.items.get(self.selected).map(|c| c.name.clone()).unwrap_or_default();
                    let mut desc = inp.clone();
                    let (total, rolls);
                    if let Some(c) = self.items.get(self.selected) {
                        if dice::parse_dice(&inp).is_some() {
                            let (t, r) = dice::roll(&inp, 0);
                            total = t; rolls = r;
                        } else if all_skills().iter().any(|(s, _)| *s == inp) {
                            let modi = c.skill_modifier(&inp);
                            let (t, r) = dice::roll("1d20", modi);
                            total = t; rolls = r; desc = format!("{} check", capitalize(&inp));
                        } else {
                            // ability?
                            let abi = ["str","dex","con","int","wis","cha"];
                            if abi.contains(&inp.as_str()) {
                                let modi = c.ability_modifier_by_name(&inp);
                                let (t, r) = dice::roll("1d20", modi);
                                total = t; rolls = r; desc = format!("{} ability", inp.to_uppercase());
                            } else {
                                let (t, r) = dice::roll("1d20", 0);
                                total = t; rolls = r; desc = "d20".into();
                            }
                        }
                    } else { let (t, r) = dice::roll("1d20", 0); total = t; rolls = r; }
                    self.status = format!("{} rolls {}: {:?} total {}", name, desc, rolls, total);
                    self.mode = Mode::List;
                    self.input.clear();
                }
                KeyCode::Char(ch) => self.input.push(ch),
                KeyCode::Backspace => { self.input.pop(); },
                _ => {}
            },
        }
        Ok(false)
    }

    fn save_current(&mut self) -> Result<()> {
        if let Some(c) = self.items.get(self.selected) {
            self.db.update_character(c)?;
        }
        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        self.items = self.db.list_characters()?;
        if self.selected >= self.items.len() { self.selected = self.items.len().saturating_sub(1); }
        Ok(())
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(size);

        self.draw_body(f, chunks[0]);
        self.draw_status(f, chunks[1]);
    }

    fn draw_body(&self, f: &mut ratatui::Frame, area: Rect) {
        match self.mode {
            Mode::List => {
                let items: Vec<ListItem> = self
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, it)| {
                        let label = format!("{} (Lv {}) - {}/{} HP - AC {}", it.name, it.level, it.hp_current, it.hp_max, it.armor_class);
                        let mut spans = vec![Span::raw(label)];
                        if i == self.selected { spans.push(Span::styled("  ▶", Style::default().fg(Color::Yellow))); }
                        ListItem::new(Line::from(spans))
                    })
                    .collect();
                let list = List::new(items).block(Block::default().title("Characters").borders(Borders::ALL));
                f.render_widget(list, area);
            }
            Mode::CreateName | Mode::CreateClass | Mode::CreateRace | Mode::CreateAbilities | Mode::CreateHpMax | Mode::CreateAc | Mode::CreateSpeed | Mode::CreateSkills | Mode::Roll => {
                let title = match self.mode {
                    Mode::CreateName => "Create: Name",
                    Mode::CreateClass => "Create: Class",
                    Mode::CreateRace => "Create: Race",
                    Mode::CreateAbilities => "Create: Abilities STR DEX CON INT WIS CHA",
                    Mode::CreateHpMax => "Create: HP Max",
                    Mode::CreateAc => "Create: Armor Class (AC)",
                    Mode::CreateSpeed => "Create: Speed (ft)",
                    Mode::CreateSkills => "Create: Skills (comma or semicolon-separated)",
                    Mode::Roll => "Roll: NdM or skill name",
                    _ => unreachable!(),
                };
                let p = Paragraph::new(self.input.clone())
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, area);
            }
            _ => {
                // Details view with tabs
                let tabs_titles = ["General", "Skills", "Inventory"].map(|t| Line::from(Span::styled(t, Style::default())));
                let tabs = Tabs::new(tabs_titles)
                    .select(self.detail_tab)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
                let vchunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(3)])
                    .split(area);
                f.render_widget(tabs, vchunks[0]);

                let detail = if let Some(c) = self.items.get(self.selected) {
                    match self.detail_tab {
                        0 => {
                            let mut text = Vec::new();
                            text.push(Line::from(Span::styled(format!("{} the {} {} (Lv {})", c.name, c.race, c.class_name, c.level), Style::default().add_modifier(Modifier::BOLD))));
                            text.push(Line::from(""));
                            text.push(Line::from(format!("HP {}/{}  AC {}  SPD {}", c.hp_current, c.hp_max, c.armor_class, c.speed)));
                            text.push(Line::from(format!("STR {} ({}), DEX {} ({}), CON {} ({}), INT {} ({}), WIS {} ({}), CHA {} ({})",
                                c.strength, Character::ability_mod(c.strength),
                                c.dexterity, Character::ability_mod(c.dexterity),
                                c.constitution, Character::ability_mod(c.constitution),
                                c.intelligence, Character::ability_mod(c.intelligence),
                                c.wisdom, Character::ability_mod(c.wisdom),
                                c.charisma, Character::ability_mod(c.charisma),
                            )));
                            text.push(Line::from(format!("Prof bonus: +{}", c.proficiency_bonus())));
                            text.push(Line::from(""));
                            text.push(Line::from("Spell slots (1-9):"));
                            text.push(Line::from(format!("{}", c.spell_slots.iter().enumerate().map(|(i, n)| format!("{}:{}", i+1, n)).collect::<Vec<_>>().join("  "))));
                            if let Some(n) = &c.notes { text.push(Line::from("")); text.push(Line::from("Notes:")); text.push(Line::from(n.clone())); }
                            Paragraph::new(text).block(Block::default().borders(Borders::ALL)).wrap(Wrap { trim: true })
                        }
                        1 => {
                            let mut text = Vec::new();
                            text.push(Line::from("Skills:"));
                            for (name, ability) in all_skills() {
                                let modif = c.skill_modifier(name);
                                let star = if c.skill_proficiencies.iter().any(|s| s.to_lowercase()==name) { "*" } else { "" };
                                text.push(Line::from(format!("  {}{} ({}): {}{}", capitalize(name), star, ability.to_uppercase(), if modif>=0 {"+"} else {""}, modif)));
                            }
                            Paragraph::new(text).block(Block::default().borders(Borders::ALL)).wrap(Wrap { trim: true })
                        }
                        _ => {
                            let mut text = Vec::new();
                            text.push(Line::from("Inventory:"));
                            if c.inventory.is_empty() { text.push(Line::from("  (empty)")); }
                            for it in &c.inventory { text.push(Line::from(format!("  - {}", it))); }
                            Paragraph::new(text).block(Block::default().borders(Borders::ALL)).wrap(Wrap { trim: true })
                        }
                    }
                } else {
                    Paragraph::new("No character selected").block(Block::default().borders(Borders::ALL))
                };
                f.render_widget(detail, vchunks[1]);
            }
        }
    }

    fn draw_status(&self, f: &mut ratatui::Frame, area: Rect) {
        let p = Paragraph::new(self.status.clone()).block(Block::default().title("Status").borders(Borders::ALL));
        f.render_widget(p, area);
    }
}

fn default_status() -> String {
    String::from("Enter: open details • q: quit • n: new • d: delete • r: roll")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
}

fn details_status() -> String {
    String::from("Arrows/Tabs: switch tabs • e: edit • r: roll • Esc: back")
}

fn edit_status() -> String {
    String::from("Editing: +/- hp • [/] adjust slot • 1-9 select • a/A add/remove item • l level up • s save • Esc: back")
}

#[derive(Default, Clone)]
struct NewCharDraft {
    name: String,
    class_name: String,
    race: String,
    abilities: [i32; 6], // STR DEX CON INT WIS CHA
    hp_max: i32,
    hp_current: i32,
    armor_class: i32,
    speed: i32,
    skill_proficiencies: Vec<String>,
}
