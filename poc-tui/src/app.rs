use crate::{
    debugger_ctx::DebuggerCtx,
    event::{AppEvent, Event, EventHandler},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nix::unistd::Pid;
use ratatui::{DefaultTerminal, widgets::ListState};

#[derive(Clone, Copy, Debug)]
pub enum Command {
    StartProcess,
    ParsePerfMap,
}

impl Command {
    pub fn title(self) -> &'static str {
        match self {
            Command::StartProcess => "Start process",
            Command::ParsePerfMap => "Parse perfmap output",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    StartProcessPopup,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,

    pub list_state: ListState,
    pub mapping_list_state: ListState,
    pub commands: Vec<Command>,

    pub mode: Mode,
    // popup input
    pub attach_input: String,

    pub debugger_ctx: DebuggerCtx,

    pub disas_str: String,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); // default selection

        let mut mapping_list_state = ListState::default();
        mapping_list_state.select(Some(0)); // default selection

        Self {
            running: true,
            events: EventHandler::new(),
            list_state,
            mapping_list_state,
            commands: vec![Command::StartProcess, Command::ParsePerfMap],
            mode: Mode::Normal,
            attach_input: "".into(),
            debugger_ctx: DebuggerCtx {
                pid: Pid::from_raw(0),
                function_mapping: None,
            },
            disas_str: String::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // Global quit (optional)
        if key_event.code == KeyCode::Char('q')
            && key_event.modifiers.is_empty()
            && self.mode == Mode::Normal
        {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        match self.mode {
            Mode::Normal => match key_event.code {
                KeyCode::Enter => {
                    self.activate_selected();
                }
                KeyCode::Down => {
                    if self.debugger_ctx.function_mapping.is_some() {
                        self.select_next_function();
                        self.disassemble();
                    } else {
                        self.select_next_command();
                    }
                }
                KeyCode::Up => {
                    if self.debugger_ctx.function_mapping.is_some() {
                        self.select_prev_function();
                        self.disassemble();
                    } else {
                        self.select_prev_command();
                    }
                }
                _ => {}
            },

            Mode::StartProcessPopup => match key_event.code {
                KeyCode::Esc => self.close_attach_popup(),
                KeyCode::Enter => self.confirm_attach(),
                KeyCode::Backspace => self.input_backspace(),
                KeyCode::Char(c) => {
                    // ignore Ctrl/Alt combos
                    if !key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && !key_event.modifiers.contains(KeyModifiers::ALT)
                    {
                        self.input_push(c);
                    }
                }
                _ => {}
            },
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn select_next_command(&mut self) {
        let len = self.commands.len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i + 1 >= len { 0 } else { i + 1 };
        self.list_state.select(Some(next));
    }

    pub fn select_prev_command(&mut self) {
        let len = self.commands.len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let prev = if i == 0 { len - 1 } else { i - 1 };
        self.list_state.select(Some(prev));
    }

    pub fn select_next_function(&mut self) {
        match &self.debugger_ctx.function_mapping {
            Some(fm) => {
                let len = fm.name_to_meta.len();
                if len == 0 {
                    self.mapping_list_state.select(None);
                    return;
                }

                let i = self.mapping_list_state.selected().unwrap_or(0);
                let next = if i + 1 >= len { 0 } else { i + 1 };
                self.mapping_list_state.select(Some(next));
            }
            None => {
                self.mapping_list_state.select(None);
                return;
            }
        }
    }

    pub fn select_prev_function(&mut self) {
        match &self.debugger_ctx.function_mapping {
            Some(fm) => {
                let len = fm.name_to_meta.len();
                if len == 0 {
                    self.mapping_list_state.select(None);
                    return;
                }

                let i = self.mapping_list_state.selected().unwrap_or(0);
                let prev = if i == 0 { len - 1 } else { i - 1 };
                self.mapping_list_state.select(Some(prev));
            }
            None => {
                self.mapping_list_state.select(None);
                return;
            }
        }
    }

    pub fn activate_selected(&mut self) {
        let Some(cmd) = self.selected_command() else {
            return;
        };

        match cmd {
            Command::StartProcess => {
                self.open_attach_popup();
            }
            Command::ParsePerfMap => {
                self.parse_perfmap_output();
                self.disassemble();
            }
        }
    }

    pub fn selected_command(&self) -> Option<Command> {
        let i = self.list_state.selected()?;
        self.commands.get(i).copied()
    }

    pub fn parse_perfmap_output(&mut self) {
        self.debugger_ctx.parse_perfmap("wasm_binary").unwrap();
    }

    pub fn open_attach_popup(&mut self) {
        self.mode = Mode::StartProcessPopup;
        self.attach_input.clear();
    }

    pub fn confirm_attach(&mut self) {
        let s = self.attach_input.trim().to_string();
        if s.is_empty() {
            self.close_attach_popup();
            return;
        }

        self.debugger_ctx.run_command(&s).unwrap();

        self.close_attach_popup();
    }

    pub fn close_attach_popup(&mut self) {
        self.mode = Mode::Normal;
    }

    // Input editing helpers
    pub fn input_push(&mut self, c: char) {
        self.attach_input.push(c);
    }

    pub fn input_backspace(&mut self) {
        self.attach_input.pop();
    }

    pub fn disassemble(&mut self) {
        self.disas_str = self
            .debugger_ctx
            .disassemble(self.mapping_list_state.selected().unwrap())
            .unwrap();
    }
}
