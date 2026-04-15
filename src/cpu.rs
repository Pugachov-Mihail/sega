use crate::bus::Bus;
use crate::cpu::Instruction::*;

#[repr(C)]
pub struct CPU {
    // 8 регистров данных (D0 - D7). Здесь происходят вычисления.
    pub d: [u32; 8],

    // 8 адресных регистров (A0 - A7). A7 обычно используется как Stack Pointer (SP).
    pub a: [u32; 8],

    // Program Counter (Счетчик команд). Указывает на адрес в памяти,
    // где лежит следующая инструкция для выполнения.
    pub pc: u32,

    // Status Register (Регистр статуса). Хранит флаги (например:
    // "был ли результат прошлого сложения равен нулю?" или "произошло ли переполнение?").
    pub sr: u16,

    pub bus: Bus,
}

impl CPU {
    pub fn new() -> Self {
        CPU{
            d: [0; 8],
            a: [0; 8],
            pc: 0x200,
            sr: 0,
            bus: Bus::new(),
        }
    }

    pub fn fetch(&mut self) -> u16 {
        let first = self.bus.read_u8(self.pc);
        let second = self.bus.read_u8(self.pc + 1);
        let opcode = (first as u16) << 8 | (second as u16);
        self.pc += 2;
        opcode
    }

    pub fn decode(&mut self, opcode: u16) -> Instruction {
        match opcode {
            0x4CDD => {
                let mask = self.fetch();
                return MovemLPostIncA5 {mask}
            }
            0x4C9D => {
                let mask = self.fetch();
                return MovemWPostIncA5 {mask}
            }
            0x4EB9 => {
                let addr = self.read_memory_u32(self.pc);
                self.pc += 4;
                return Jsr { addr }
            },
            0x4E75 => {
                return Rts
            },
            0x4AB9 => {
                let addr = self.read_memory_u32(self.pc);
                self.pc += 4;
                return TstL {addr}
            },
            0x4FF8 => {
                let word = self.fetch();
                let addr = (word as i16) as i32 as u32;
                return LeaA7 { addr }
            }
            0x4A79 => {
                let addr = self.read_memory_u32(self.pc);
                self.pc += 4;
                return TstW { addr }
            }
            0x4BFA => {
                let extension_word_pc = self.pc;
                let offset = self.fetch();
                let sing_offset = (offset as i16) as i32;
                let addr = (extension_word_pc as i32 + sing_offset) as u32;
                return LeaPcA5 {addr}
            }
            _ => {}
        }

        let group = (opcode & 0xF000) >> 12;
        let register = (opcode & 0x0E00) >> 9;
        let mode = (opcode & 0x01F8) >> 3;
        let src_reg = opcode & 0x0007;
        let data = opcode & 0x00FF;

        match group {
            0xD if mode == 0x10 => {
                Add { src: src_reg as u8, dest: register as u8 }
            },
            0x7 => {
                Moveq { register: register as u8, data: data as u8 }
            },
            0x6 => {
                let condition = (opcode & 0x0F00) >> 8;
                let offset = data as i8;
                if condition == 0x6 {
                    Bne { offset }
                } else {
                    Unknown(opcode)
                }
            },
           _ => Unknown(opcode)
        }
    }

    pub fn execute(&mut self, inst: Instruction) {
        match inst {
            MovemLPostIncA5 {mask} => {
                
            }
            MovemWPostIncA5 {mask} => {
                println!("MOVEM.W: Чтение множества регистров по маске {:#018b} из (A5)+", mask);

                for i in 0..16 {
                    if (mask & (1 << i)) != 0 {
                        let val =self.read_memory_u16(self.a[6]);

                        let sing_extended = (val as i16) as i32 as u32;

                        if i < 8 {
                            self.d[i as usize] = sing_extended;
                            println!(" -> Загружен D{}: {:#010X}", i, sing_extended);
                        } else {
                            self.a[(i - 8)as usize] = sing_extended;
                            println!(" -> Загружен A{}: {:#010X}", i - 8, sing_extended);
                        }
                        self.a[6] += 2;
                    }
                }
            }
            LeaPcA5 {addr} => {
                println!("LEA: Вычислен PC-относительный адрес {:#010X}. Сохраняем указатель в A5", addr);
                self.a[1] = addr;
            }
            TstW { addr } => {
                let val = self.read_memory_u16(addr);
                println!("TST.W: Чтение адреса {:#010X}, получено значение {:#06X}", addr, val);
                self.sr &= !0x0C;
                if val == 0 {
                    self.sr |= 0x04;
                }

                if (val & 0x8000) != 0 {
                    self.sr |= 0x08;
                }
            },
            LeaA7 { addr } => {
                println!("LEA: Инициализация Стека. A7 установлен в {:#010X}", addr);
                self.a[2] = addr;
            }
            TstL { addr } => {
                let val = self.read_memory_u32(addr);
                println!("TST.L: Проверяем значение {:#010X} по адресу {:#010X}", val, addr);
                self.sr &= !0x0C;
                if val == 0 {
                    self.sr |= 0x04;
                }
                if (val & 0x80000000) != 0 {
                    self.sr |= 0x08;
                }
            },
           Moveq { register, data } => {
                let extended_data  = data as i8 as i32 as u32;
                self.d[register as usize] = extended_data;
                println!("MOVEQ: кладем {:#04X} в D{}", extended_data, register);
            }
            Add { src, dest } => {
                let val1 = self.d[dest as usize];
                let val2 = self.d[src as usize];

                let (result, carry) = val1.overflowing_add(val2);
                self.d[dest as usize] = result;

                // ИДЕАЛЬНО ЧИСТЫЙ БЛОК ФЛАГОВ:
                // 1. Z (Zero)
                if result == 0 { self.sr |= 0x04; } else { self.sr &= !0x04; }

                // 2. N (Negative)
                if (result & 0x8000_0000) != 0 { self.sr |= 0x08; } else { self.sr &= !0x08; }

                // 3. C (Carry)
                if carry { self.sr |= 0x01; } else { self.sr &= !0x01; }

                println!("ADD: сложили {:#010X} и {:#010X}, результат: {:#010X}", val1, val2, result);
            }
            Bne { offset } => {
                if (self.sr & 0x04) == 0 {
                    self.pc = (self.pc as i32 + offset as i32) as u32;
                    println!("BNE: Результат не ноль! Прыгаем на адрес {:#06X}", self.pc);
                } else {
                    println!("BNE: Результат ноль! Цикл завершен.");
                }
            },
            Jsr { addr } => {
                self.push32(self.pc);
                self.pc = addr;
                println!("JSR: Прыгаем в подпрограмму по адресу {:#06X}", addr);
            },
            Rts {} => {
                self.pc = self.pop32();
                println!("RTS: Возврат из подпрограммы на адрес {:#06X}", self.pc);
            }
            Unknown(raw_opcode) => {
                panic!("КРИТИЧЕСКАЯ ОШИБКА: Неизвестная инструкция {:#06X}", raw_opcode);
            }
        }
    }

    pub fn write_memory_u16(&mut self, addr: u32, val: u16) {
        self.bus.write_u8(addr, (val >> 8) as u8);
        self.bus.write_u8(addr + 1, val as u8) ;
    }

    pub fn read_memory_u16(&self, addr: u32) -> u16 {
     let b0 = self.bus.read_u8(addr) as u16;
        let b1 = self.bus.read_u8(addr + 1) as u16;
        (b0 << 8) | b1
    }

    pub fn write_memory_u32(&mut self, addr: u32, val: u32) {
        self.bus.write_u8(addr , ((val >> 24) & 0xFF) as u8);
        self.bus.write_u8(addr + 1 , ((val >> 16) & 0xFF) as u8);
        self.bus.write_u8(addr + 2 , ((val >> 8) & 0xFF) as u8);
        self.bus.write_u8(addr + 3, (val & 0xFF) as u8);
    }

    pub fn read_memory_u32(&self, addr: u32) -> u32 {
        let b0 = self.bus.read_u8(addr ) as u32;
        let b1 = self.bus.read_u8(addr + 1) as u32;
        let b2 = self.bus.read_u8(addr + 2) as u32;
        let b3 = self.bus.read_u8(addr + 3) as u32;

        (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    }

    // Операция PUSH: Сначала УМЕНЬШАЕМ указатель стека (A7), затем пишем
    pub fn push32(&mut self, val: u32) {
        self.a[7] -= 4;
        let sp = self.a[7];
        self.write_memory_u32(sp, val);
    }

    // Операция POP: Сначала читаем, затем УВЕЛИЧИВАЕМ указатель стека
    pub fn pop32(&mut self) -> u32 {
        let sp = self.a[7];
        let val = self.read_memory_u32(sp);
        self.a[7] += 4;
        val
    }


}

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
}