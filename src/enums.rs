#[derive(Debug)]
pub enum Instruction {
    Moveq{register: u8, data: u8},
    Unknown (u16),
    Add{src: u8, dest: u8},
    Bne{offset: i8},
    Jsr { addr: u32 }, // Jump to Subroutine
    Rts,               // Return from Subroutine
    TstL {addr: u32},
    LeaA7 {addr: u32},
    TstW { addr: u32},
    LeaPcA5 {addr: u32},
    MovemWPostIncA5 { mask: u16 },
    MovemLPostIncA5 { mask: u16 },
    MoveBDispA1D0 { offset: i16 },
    AndiB_Imm_D0 { imm: u8 },
    BeqS {offset: i8},
    MoveW_Ind_A4_D0,
    MoveL_PreDec_A0_D6,
    MoveA6ToUsp,
    MoveB_PostInc_A5_D5,
    MoveW_D5_Ind_A4,
    AddW_D7_D5,
}