use super::Console;

enum AddrMode {
    Absolute,
    /// Absolute,X
    AbsoluteX,
    /// Absolute,Y
    AbsoluteY,
    /// (Absolute)
    AbsoluteIndirectWord,
    /// [Absolute]
    AbsoluteIndirectSWord,
    /// (Absolute,X)
    AbsoluteIndexedIndirect,
    Accumulator,
    Direct,
    /// Direct,X
    DirectX,
    /// Direct,Y
    DirectY,
    /// (Direct)
    DirectWord,
    /// [Direct]
    DirectSWord,
    /// (Direct,X)
    IndexedDirectWord,
    /// (Direct), Y
    DirectIndexedWord,
    /// [Direct], Y
    DirectIndexedSWord,
    Immediate,
    Implied,
    Long,
    /// Long,X
    LongX,
    RelativeByte,
    RelativeWord,
    SourceDestination,
    Stack,
    /// (Stack,S),Y
    StackIndexed
}

#[derive(Debug)]
pub struct Flags {
    /// Negative
    n: bool,
    /// Overflow
    v: bool,
    /// Memory width
    m: bool,
    /// Index register width
    x: bool,
    /// Decimal mode
    d: bool,
    /// Interrupt disable
    i: bool,
    /// Zero
    z: bool,
    /// Carry
    c: bool,
    /// Emulation mode
    e: bool,
    /// Break
    b: bool
}

#[derive(Debug)]
#[allow(non_snake_case)]
/// The 65C816 CPU
pub struct CPU {
    /// Accumulator (16 bit)
    pub A: u16,
    /// X Register (16 bit)
    pub X: u16,
    /// Y Register (16 bit)
    pub Y: u16,
    /// Stack Pointer (16 bit)
    pub S: u16,
    /// Databank Register (16 bit)
    pub DBR: u16,
    /// Direct Addressing Register (16 bit)
    pub D: u16,
    /// Program Bank Register (16 bit)
    pub K: u16,
    /// Flags Register
    pub P: Flags,
    /// Program Counter (16 bit)
    pub PC: u16    
}

impl Flags {
    fn new() -> Flags {
        Flags {
            n: false,
            v: false,
            m: false,
            x: false,
            d: false,
            i: false,
            z: false,
            c: false,
            e: false,
            b: false
        }
    }
}

impl CPU {
    /// Init CPU to 0
    pub fn new() -> CPU {
        CPU {
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            DBR: 0,
            D: 0,
            K: 0,
            P: Flags::new(),
            PC: 0
        }
    }
}

pub fn interpret_opcode(snes: &mut Console) {
    println!("{:?}", snes);
    println!("bruh");
}