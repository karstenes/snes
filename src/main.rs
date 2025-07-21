#![allow(unused_variables, dead_code, unused_mut)]

mod cartridge;
mod cpu;
mod debugger;
mod memory;
mod registers;

use cartridge::*;
use clap::Parser;
use color_eyre::Result;
use cpu::*;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use debugger::{debug_simulation, render_wrapped_instructions, DisassemblerContext, Flag};
use log::{error, trace};
use pretty_env_logger::{env_logger::fmt::Target, formatted_builder};
use ratatui::{
    layout::Constraint,
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use registers::*;
use std::path::PathBuf;
use std::{time::Duration, time::Instant};
use symbols::scrollbar;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::debugger::DisassemblerError;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// sfc format ROM file
    #[arg(index = 1)]
    rom: PathBuf,

    /// Bypass checksum verification
    #[arg(short, long, default_value_t = false)]
    checksum: bool,

    /// Disable TUI mode
    #[arg(short, long, default_value_t = true)]
    tui: bool,
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Edit,
    Error,
}

#[derive(Debug, Default)]
pub struct App {
    scroll_state: ScrollbarState,
    code_scroll_state: ScrollbarState,
    stack_scroll: u16,
    code_scroll: u16,
    disassembled: DisassemblerContext,
    current_pc: u32,
    branch_taken: bool,
    disassembler_ptr: usize,
    disassembly_start: u32,
    disassembly_end: u32,
    input: Input,
    input_mode: InputMode,
    subroutine_stack: Vec<u32>,
    breakpoint: u32,
    breakpoint_set: bool,
    run: bool,
}

#[derive(Debug, Clone)]
pub struct Console {
    cpu: CPU,
    cartridge: Cartridge,
    ram: Vec<u8>,
    mmio: MMIORegisters,
    dma: DMARegisters,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn ui(f: &mut Frame, app: &mut App, snes: &Console) {
    let size = f.area();

    let chunks = Layout::horizontal([
        Constraint::Percentage(100),
        Constraint::Min(15),
        Constraint::Min(20),
    ])
    .split(size);

    let left = if app.subroutine_stack.len() > 0 {
        Layout::vertical([Constraint::Min(3), Constraint::Percentage(100)])
            .spacing(0)
            .split(chunks[0])
    } else {
        Layout::vertical([Constraint::Percentage(100)]).split(chunks[0])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let mut rendertext: Vec<Line> = vec![Line::default(); app.disassembled.lines.len()];

    for (i, line) in rendertext.iter_mut().enumerate() {
        let mut line_string = String::default();
        for _ in 0..app.disassembled.branchdepth - app.disassembled.lines[i].flags.len() {
            line_string.push(' ');
        }
        for flag in app.disassembled.lines[i].flags.iter() {
            match flag {
                Flag::BranchStart(_) => line_string.push('┏'),
                Flag::BranchCont(_) => line_string.push('┃'),
                Flag::BranchEnd(_) => line_string.push('┗'),
            }
        }
        line_string.push_str(format!("{:}", app.disassembled.lines[i].disassembled).as_str());
        if app.disassembled.lines[i].location == snes.cpu.get_pc() {
            *line = Line::from(line_string).on_green().black();
            app.disassembler_ptr = i;
        } else {
            *line = Line::from(line_string);
        }
    }

    let height = if app.subroutine_stack.len() > 0 {
        left[1].height as usize
    } else {
        left[0].height as usize
    };
    let scroll = if app.disassembler_ptr > height / 2 {
        app.disassembler_ptr - height / 2
    } else {
        0
    };

    let instr_text = Text::from(rendertext);

    let reg_text = Text::from(format!(
        "{}\n{}\n{}",
        snes.cpu, app.disassembler_ptr, height
    ));

    let stack: Vec<Line> = snes.ram[0x000000..=0x00FFFF]
        .iter()
        .enumerate()
        .map(|x| {
            if ((x.0 == snes.cpu.S as usize) && !snes.cpu.P.e)
                || ((x.0 == ((snes.cpu.S & 0x00FF) | 0x0100) as usize) && snes.cpu.P.e)
            {
                Line::from(format!("{:04X}: {:02X}", x.0, x.1))
                    .on_green()
                    .black()
            } else {
                Line::from(format!("{:04X}: {:02X}", x.0, x.1))
            }
        })
        .collect();

    let stack_text = Text::from(stack);

    app.scroll_state = app
        .scroll_state
        .content_length(65536)
        .position((app.stack_scroll % 0xFFFF) as usize);

    app.code_scroll_state = app
        .code_scroll_state
        .content_length(app.disassembled.lines.len())
        .position(scroll);

    let instr_par = Paragraph::new(instr_text)
        .left_aligned()
        .block(block)
        .scroll((scroll as u16, 0));

    let stackblock = Block::default()
        .title_top(Line::from("Stack".bold()).centered())
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let stack_par = Paragraph::new(stack_text)
        .left_aligned()
        .block(stackblock)
        .scroll((app.stack_scroll as u16, 0));

    let regblock = Block::default()
        .title_top(Line::from("Registers".bold()).centered())
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let reg_par = Paragraph::new(reg_text).left_aligned().block(regblock);

    if app.subroutine_stack.len() > 0 {
        let subroutineblock = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);
        let subroutine_text = Text::from(format!(
            "Subroutine ${:06X}",
            app.subroutine_stack.last().unwrap()
        ));
        let subroutine_par = Paragraph::new(subroutine_text)
            .left_aligned()
            .block(subroutineblock);
        f.render_widget(subroutine_par, left[0]);
        f.render_widget(instr_par, left[1]);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .track_symbol(None)
                .begin_symbol(None)
                .end_symbol(None),
            left[1].inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut app.code_scroll_state,
        );
    } else {
        f.render_widget(instr_par, left[0]);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .track_symbol(None)
                .begin_symbol(None)
                .end_symbol(None),
            left[0].inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut app.code_scroll_state,
        );
    }

    f.render_widget(stack_par, chunks[1]);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None),
        chunks[1].inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
    f.render_widget(reg_par, chunks[2]);

    if let InputMode::Edit = &app.input_mode {
        let popup = Block::default()
            .title("Edit")
            .borders(Borders::ALL)
            .border_set(border::DOUBLE);

        let area = centered_rect(70, 10, chunks[0]);
        let text = Text::from(app.input.value());
        let par = Paragraph::new(text).block(popup);
        f.render_widget(Clear, area);
        f.render_widget(par, area);
    }
}

enum DebuggerCommand {
    Breakpoint(u32),
    NMI,
    Default,
}

fn execute_command(command: &str, snes: &mut Console) -> Result<DebuggerCommand> {
    if command.is_empty() {
        return Ok(DebuggerCommand::Default);
    };

    let commandparts: Vec<&str> = command.split_whitespace().collect();

    Ok(match commandparts[0].to_lowercase().as_str() {
        "x" => {
            snes.cpu.X = u16::from_str_radix(commandparts[1], 16)?;
            DebuggerCommand::Default
        }
        "b" => DebuggerCommand::Breakpoint(u32::from_str_radix(commandparts[1], 16)?),
        "nmi" => DebuggerCommand::NMI,
        "p" => {
            snes.cpu.set_p(u8::from_str_radix(commandparts[1], 2)?);
            DebuggerCommand::Default
        }
        _ => DebuggerCommand::Default,
    })
}

fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::formatted_builder()
        .target(Target::Stderr)
        .init();

    let args = Args::parse();

    let (mut terminal, tui) = if args.tui {
        (Some(ratatui::init()), true)
    } else {
        (None, false)
    };

    let cartridge = load_rom(&args.rom, args.checksum)?;

    let mut ram = vec![0; 0x200000];

    let mut snes = Console {
        cpu: CPU::new(),
        cartridge,
        ram,
        mmio: MMIORegisters::default(),
        dma: DMARegisters::default(),
    };
    snes.cpu.PC = snes.cartridge.header.interrupt_vectors.reset;
    // let op = memory::read_byte(&snes, snes.cpu.get_pc())?;
    // let instr = cpu::decode_instruction(&snes, op)?;
    // cpu::execute_instruction(&mut snes, &instr)?;

    // snes.cpu.PC = snes.cartridge.header.interrupt_vectors.nmi_emu;

    let tick_rate = Duration::from_millis(100);
    let mut app = App::default();
    app.disassembler_ptr = 0;
    app.current_pc = snes.cpu.get_pc();
    let mut last_tick = Instant::now();
    'mainloop: loop {
        if !tui {
            let op = memory::read_byte(&snes, snes.cpu.get_pc())?;
            let instr = cpu::decode_instruction(&snes, op, snes.cpu.get_pc())?;
            cpu::execute_instruction(&mut snes, &instr)?;
            // let mut trash: String = String::default();
            // io::stdin().read_line(&mut trash)?;
            trace!("Next");
        } else {
            if app.run {
                let op = memory::read_byte(&snes, snes.cpu.get_pc())?;
                let instr = cpu::decode_instruction(&snes, op, snes.cpu.get_pc())?;
                let res = cpu::execute_instruction(&mut snes, &instr)?;
                app.current_pc = snes.cpu.get_pc();
                if app.current_pc == app.breakpoint {
                    app.run = false;
                }
                continue 'mainloop;
            }
            if app.disassembler_ptr >= app.disassembled.lines.len()
                || app.disassembled.lines.is_empty()
            {
                let temp = match debug_simulation(&snes, 100) {
                    Ok(r) => r,
                    Err(e) => {
                        ratatui::restore();
                        match e {
                            DisassemblerError::DisassemblyError(de) => {
                                error!("Error simulating instruction\n{:}", de);
                                error!("Status:\n{:}", de.status.cpu);
                                return Err(de.source);
                            }
                            DisassemblerError::Other(e) => return Err(e),
                        }
                    }
                };
                app.disassembly_start = temp.startloc;
                app.disassembly_end = temp.endloc;
                app.disassembled = render_wrapped_instructions(temp);
                app.disassembler_ptr = 0;
            }
            terminal
                .as_mut()
                .unwrap()
                .draw(|f| ui(f, &mut app, &snes))?;
            let timeout: Duration = tick_rate.saturating_sub(last_tick.elapsed());
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match app.input_mode {
                        InputMode::Normal if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Char('q') => break 'mainloop,
                            KeyCode::Up => {
                                app.stack_scroll = app.stack_scroll.wrapping_sub(1);
                            }
                            KeyCode::Down => {
                                app.stack_scroll = app.stack_scroll.wrapping_add(1);
                            }
                            KeyCode::PageUp => {
                                app.stack_scroll = app.stack_scroll.wrapping_sub(0x10);
                            }
                            KeyCode::PageDown => {
                                app.stack_scroll = app.stack_scroll.wrapping_add(0x10);
                            }
                            KeyCode::Enter => {
                                if snes.cpu.P.e {
                                    app.stack_scroll = snes.cpu.S.to_le_bytes()[0] as u16 | 0x0100;
                                } else {
                                    app.stack_scroll = snes.cpu.S;
                                }
                            }
                            KeyCode::Char('n') => {
                                trace!("Next");
                                let op = memory::read_byte(&snes, snes.cpu.get_pc())?;
                                let instr = cpu::decode_instruction(&snes, op, snes.cpu.get_pc())?;
                                let res = cpu::execute_instruction(&mut snes, &instr)?;
                                app.branch_taken =
                                    matches!(res, cpu::CPUExecutionResult::BranchTaken);
                                if app.branch_taken
                                    && (snes.cpu.get_pc() >= app.disassembly_end
                                        || snes.cpu.get_pc() < app.disassembly_start)
                                {
                                    app.disassembler_ptr = 0;
                                    app.disassembled = DisassemblerContext::default();
                                    app.branch_taken = false;
                                }
                                if matches!(res, cpu::CPUExecutionResult::Jump) {
                                    app.disassembler_ptr = 0;
                                    app.disassembled = DisassemblerContext::default();
                                }
                                if let CPUExecutionResult::Subroutine(addr) = res {
                                    app.disassembled = DisassemblerContext::default();
                                    app.disassembler_ptr = 0;
                                    app.subroutine_stack.push(addr);
                                }
                                if let CPUExecutionResult::Return = res {
                                    app.disassembled = DisassemblerContext::default();
                                    app.disassembler_ptr = 0;
                                    app.subroutine_stack.pop();
                                }
                                app.current_pc = snes.cpu.get_pc();
                            }
                            KeyCode::Char('r') => {
                                let temp = match debug_simulation(&snes, 100) {
                                    Ok(r) => r,
                                    Err(e) => {
                                        ratatui::restore();
                                        match e {
                                            DisassemblerError::DisassemblyError(de) => {
                                                println!("Error simulating instruction\n{:}", de);
                                                println!("Status:\n{:}", de.status.cpu);
                                                return Err(de.source);
                                            }
                                            DisassemblerError::Other(e) => return Err(e),
                                        }
                                    }
                                };
                                app.disassembled = render_wrapped_instructions(temp);
                                app.disassembler_ptr = 0;
                            }
                            KeyCode::Char('c') => app.run = true,
                            KeyCode::Char('/') => app.input_mode = InputMode::Edit,
                            _ => {}
                        },
                        InputMode::Edit => match key.code {
                            KeyCode::Esc => app.input_mode = InputMode::Normal,
                            KeyCode::Enter => {
                                let cmd = execute_command(app.input.value(), &mut snes)?;
                                match cmd {
                                    DebuggerCommand::Breakpoint(addr) => {
                                        app.breakpoint = addr;
                                        app.breakpoint_set = true;
                                    }
                                    DebuggerCommand::NMI => {
                                        cpu::execute_nmi(&mut snes)?;
                                        app.disassembler_ptr = app.disassembled.lines.len()
                                    }
                                    DebuggerCommand::Default => {}
                                }
                                app.input.reset();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
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
    }

    ratatui::restore();

    Ok(())
}
