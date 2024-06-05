#![allow(unused_variables, dead_code)]

mod cpu;
mod cartridge;
mod memory;

use anyhow::Context;
use anyhow::Ok;
use cartridge::*;
use cpu::*;
use crossterm::ExecutableCommand;
use crossterm::QueueableCommand;
use std::env;
use std::path;
use anyhow::Result;
use pretty_env_logger;
use std::{io, time::Duration, time::Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    queue
};
use ratatui::{
    prelude::*,
    layout::Constraint,
    symbols::border,
    widgets::{block::*, *},
    
};
use symbols::scrollbar;

#[derive(Debug, Default)]
pub struct App {
    scroll_state: ScrollbarState,
    stack_scroll: usize
}

#[derive(Debug)]
pub struct Console {
    cpu: CPU,
    cartridge: Cartridge,
    ram: Vec<u8>
}

fn ui(f: &mut Frame, app: &mut App, snes: &Console) {
    let size = f.size();

    let chunks = Layout::horizontal([
        Constraint::Percentage(100),
        Constraint::Min(15),
        Constraint::Min(20)
    ])
    .split(size);

    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let counter_text = 
    Text::from(vec![
        Line::raw("LDA #%00000001"),
        Line::raw("STA $4015"),
        Line::raw("LDA #%01010100"),
        Line::styled("STA $4000", Style::new().on_green().black()),
        Line::raw("LDA #$C9"),
        Line::raw("STA $4002"),
        Line::raw("LDA #%00010001"),
        Line::raw("STA $4003"),
        Line::raw("RTS")]);

    let reg_text = Text::from(format!("{}", snes.cpu));

    let stack: Vec<Line> = snes.ram[0x010000..=0x01FFFF].iter()
        .enumerate()
        .map(|x| if x.0 == snes.cpu.S as usize {
            Line::from(format!("{:04X}: {:02X}", x.0, x.1)).on_green().black()
        } else {
            Line::from(format!("{:04X}: {:02X}", x.0, x.1))
        })
        .collect();

    let stack_text = Text::from(stack);

    app.scroll_state = app.scroll_state.content_length(65536);

    let instr_par = Paragraph::new(counter_text)
        .left_aligned()
        .block(block);

    let stackblock = Block::default()
        .title(Title::from("Stack".bold()).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let stack_par = Paragraph::new(stack_text)
        .left_aligned()
        .block(stackblock)
        .scroll((app.stack_scroll as u16, 0));
    

    let regblock = Block::default()
        .title(Title::from("Registers".bold()).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let reg_par= Paragraph::new(reg_text)
        .left_aligned()
        .block(regblock);

    f.render_widget(instr_par, chunks[0]);

    f.render_widget(stack_par, chunks[1]);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None),
            chunks[1].inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
        &mut app.scroll_state
    );
    f.render_widget(reg_par, chunks[2]);

}

fn main() -> Result<()> {
    pretty_env_logger::init();
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args: Vec<String> = env::args().collect();

    let file_path = path::Path::new(&args[1]);

    let cartridge = load_rom(file_path)?;

    let mut ram = vec![0; 0x200000];

    let mut snes = Console {
        cpu: CPU::new(),
        cartridge,
        ram
    };
    let reset = memory::read_word(&snes, snes.cartridge.header.interrupt_vectors.reset as u32)? as u32;
    snes.cpu.PC = reset;
    let instr = memory::read_byte(&snes, reset)?;
    
    let tick_rate = Duration::from_millis(100);
    let mut app = App::default();
    let mut last_tick = Instant::now();

    'mainloop: loop {
        terminal.draw(|f| ui(f, &mut app, &snes))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break 'mainloop,
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
