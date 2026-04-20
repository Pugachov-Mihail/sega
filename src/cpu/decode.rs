use super::CPU;
use super::instructions::{Instruction, Size, EA};
use super::instructions::Instruction::*;

impl CPU {
    pub fn decode(&mut self, opcode: u16) -> Instruction {
        // --- Семейство команд пересылки данных MOVE ---
        // Старшие 2 бита у MOVE всегда равны 00.
        if (opcode & 0xC000) == 0x0000 {
            // Извлекаем размер (биты 13-12)
            let size_bits = (opcode >> 12) & 0x03;

            // У MOVE специфичная кодировка: 1=Byte, 3=Word, 2=Long
            if size_bits == 1 || size_bits == 2 || size_bits == 3 {
                let size = match size_bits {
                    1 => Size::Byte,
                    2 => Size::Long,
                    3 => Size::Word,
                    _ => unreachable!(),
                };

                // Источник (Source) - младшие 6 бит: [Mode 5-3] [Reg 2-0]
                let src_mode = (opcode >> 3) & 0x07;
                let src_reg  = opcode & 0x07;
                let src = self.parse_ea(src_mode, src_reg);

                // Приемник (Destination) - биты 11-6.
                // ВНИМАНИЕ: У приемника биты перевернуты! [Reg 11-9] [Mode 8-6]
                let dst_reg  = (opcode >> 9) & 0x07;
                let dst_mode = (opcode >> 6) & 0x07;
                let dst = self.parse_ea(dst_mode, dst_reg);

                return Instruction::Move { size, src, dst };
            }


            // --- Семейство MOVE ---
            if (opcode & 0xC000) == 0x0000 {
                let size_bits = (opcode >> 12) & 0x03;
                if size_bits == 1 || size_bits == 2 || size_bits == 3 {
                    let size = match size_bits {
                        1 => Size::Byte,
                        2 => Size::Long,
                        3 => Size::Word,
                        _ => unreachable!(),
                    };
                    let src_mode = (opcode >> 3) & 0x07;
                    let src_reg  = opcode & 0x07;
                    let src = self.parse_ea(src_mode, src_reg); // <--- ИСПОЛЬЗУЕМ self!

                    let dst_reg  = (opcode >> 9) & 0x07;
                    let dst_mode = (opcode >> 6) & 0x07;
                    let dst = self.parse_ea(dst_mode, dst_reg); // <--- ИСПОЛЬЗУЕМ self!

                    return Instruction::Move { size, src, dst };
                }
            }
        }

        if (opcode & 0xF1C0) == 0x41C0 {
            let dst_reg = ((opcode >> 9) & 0x07) as u8;
            let src_mode = (opcode >> 3) & 0x07;
            let src_reg = opcode & 0x07;

            let src = self.parse_ea(src_mode, src_reg);
            return Instruction::Lea { src, dst_reg };
        }
        return Instruction::Unknown(opcode);
    }
    pub fn parse_ea(&mut self, mode: u16, reg: u16) -> EA {
        match mode {
            0b000 => EA::DataRegDirect(reg as u8),
            0b001 => EA::AddrRegDirect(reg as u8),
            0b010 => EA::AddrRegIndirect(reg as u8),
            0b011 => EA::AddrRegPostInc(reg as u8),
            0b100 => EA::AddrRegPreDec(reg as u8),
            0b111 => match reg {
                0b000 => {
                    // Absolute Short: читаем следующее слово как знаковое i16
                    let w = self.fetch() as i16;
                    EA::AbsoluteWord(w)
                },
                _ => unimplemented!("Режим 7, рег {} еще не реализован!", reg),
            },
            _ => unimplemented!("Режим адресации Mode: {:#03b} не реализован!", mode),
        }
    }
}

