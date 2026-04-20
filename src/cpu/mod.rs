pub mod instructions;
pub mod decode;
pub mod execute;

use bus::Bus;
use crate::bus;

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

    pub usp: u32,

    pub bus: Bus,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            d: [0; 8],
            a: [0; 8],
            pc: 0x200,
            sr: 0,
            usp: 0,
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

    fn set_flags_u16(&mut self, val: u16) {
        self.sr &= !0x0F;

        if val == 0 {
            self.sr |= 0x04;
        }
        if (val & 0x08) != 0 {
            self.sr |= 0x08;
        }
    }
}