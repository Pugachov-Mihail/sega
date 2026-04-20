#[derive(Debug, Clone, Copy)]
pub enum Size {
    Byte,
    Word,
    Long,
}

impl Size {
    // Вспомогательный метод: сколько байт памяти занимает этот размер
    pub fn bytes(&self) -> u32 {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
            Size::Long => 4,
        }
    }
}

// Эффективный адрес (Effective Address)
#[derive(Debug, Clone, Copy)]
pub enum EA {
    DataRegDirect(u8),     // Dn (от 0 до 7)
    AddrRegDirect(u8),     // An (от 0 до 7)
    AddrRegIndirect(u8),   // (An)
    AddrRegPostInc(u8),    // (An)+
    AddrRegPreDec(u8),     // -(An)
    Immediate32(u32),      // Непосредственное 32-битное значение
    AbsoluteWord(i16),     // Абсолютный короткий адрес
}

// Наша новая, компактная архитектура инструкций
#[derive(Debug)] // Инструкции обычно не нужно копировать (Copy), только печатать
pub enum Instruction {
    Unknown(u16),

    // Универсальная пересылка данных!
    Move { size: Size, src: EA, dst: EA },

    // Универсальная математика
    Add { size: Size, src: EA, dst: EA },

    // Универсальный цикл
    Dbra { reg: u8, offset: i16 },

    // Системные команды
    MoveToUsp { reg: u8 },

    Lea { src: EA, dst_reg: u8 },
}