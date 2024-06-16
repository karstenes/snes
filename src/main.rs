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
    stack_scroll: u16,
    current_instr_context: Vec<String>,
    current_instr_loc: usize,
    current_pc: u32
}

#[derive(Debug)]
pub struct Console {
    cpu: CPU,
    cartridge: Cartridge,
    ram: Vec<u8>
}


fn ui(f: &mut Frame, app: &mut App, snes: &Console) -> Result<()> {
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
    
    if app.current_instr_context.is_empty() {
        let opcode = memory::read_byte(&snes, snes.cpu.get_pc())?;
        let mut currinstr = cpu::decode_instruction(&snes, opcode)?;
        let mut PCtemp = snes.cpu.get_pc();
        app.current_instr_context.push(currinstr.to_string());
        while !(currinstr.opcode.is_jump() || currinstr.opcode.is_interrupt()) || PCtemp > (snes.cpu.get_pc() + 0x200) {
            PCtemp += currinstr.mode.length(false, false) as u32;
            let opcode = memory::read_byte(&snes, PCtemp)?;
            currinstr = cpu::decode_instruction(&snes, opcode)?;
            app.current_instr_context.push(currinstr.to_string());
        }
    }

    let rendertext: Vec<Line> = app.current_instr_context
        .iter()
        .enumerate()
        .map(|(i, f)| if i == app.current_instr_loc 
            {Line::from(f.clone()).on_green().black()} else {Line::from(f.clone())})
        .collect();

    if app.current_pc != snes.cpu.get_pc() {
        app.current_instr_loc += 1;
    }

    if app.current_instr_loc == app.current_instr_context.len()-1 {
        app.current_instr_loc = 0;
        app.current_instr_context.clear();
    }
        
    let instr_text = Text::from(rendertext);

    let reg_text = Text::from(format!("{}", snes.cpu));

    let stack: Vec<Line> = snes.ram[0x000000..=0x00FFFF].iter()
        .enumerate()
        .map(|x| if ((x.0 == snes.cpu.S as usize) && !snes.cpu.P.e) 
            || ((x.0 == ((snes.cpu.S & 0x00FF) | 0x0100) as usize) && snes.cpu.P.e) {
            Line::from(format!("{:04X}: {:02X}", x.0, x.1)).on_green().black()
        } else {
            Line::from(format!("{:04X}: {:02X}", x.0, x.1))
        })
        .collect();

    let stack_text = Text::from(stack);

    app.scroll_state = app.scroll_state
        .content_length(65536)
        .position((app.stack_scroll % 0xFFFF) as usize);

    let instr_par = Paragraph::new(instr_text)
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
    return Ok(())
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
    snes.cpu.PC = (reset & 0xFFFF) as u16;
    let op = memory::read_byte(&snes, snes.cpu.get_pc())?;
    let instr = cpu::decode_instruction(&snes, op)?;
    cpu::execute_instruction(&mut snes, instr)?;
    
    let tick_rate = Duration::from_millis(100);
    let mut app = App::default();
    app.current_instr_context = Vec::new();
    app.current_pc = snes.cpu.get_pc();
    let mut last_tick = Instant::now();
    'mainloop: loop {
        terminal.draw(|f| ui(f, &mut app, &snes).unwrap())?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break 'mainloop,
                    KeyCode::Up => {
                        app.stack_scroll = app.stack_scroll.wrapping_sub(1);
                    },
                    KeyCode::Down => {
                        app.stack_scroll = app.stack_scroll.wrapping_add(1);
                    },
                    KeyCode::PageUp => {
                        app.stack_scroll = app.stack_scroll.wrapping_sub(0x10);
                    },
                    KeyCode::PageDown => {
                        app.stack_scroll = app.stack_scroll.wrapping_add(0x10);
                    }
                    KeyCode::Enter => {
                        if snes.cpu.P.e {
                            app.stack_scroll = snes.cpu.S.to_le_bytes()[0] as u16 | 0x0100;
                        } else {
                            app.stack_scroll = snes.cpu.S;
                        }
                    },
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
