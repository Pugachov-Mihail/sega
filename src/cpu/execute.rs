use std::io::Read;
use super::CPU;
use super::instructions::*;
use super::instructions::Instruction::*;


impl CPU {
    pub fn get_ea_address(&self, ea: EA) -> u32 {
        match ea {
            EA::AddrRegIndirect(reg) => self.a[reg as usize],
            // Превращение i16 в i32, а затем в u32 аппаратно расширяет знак!
            EA::AbsoluteWord(w) => w as i32 as u32,
            _ => panic!("Критическая ошибка: Попытка вычислить адрес для неверного EA: {:?}", ea),
        }
    }
    pub fn execute(&mut self, instr: Instruction) {
        match instr {

            Instruction::Move { size, src, dst } => {
                // 1. Читаем данные через наш новый универсальный метод!
                // Благодаря #[derive(Copy)] мы можем передавать `size` и `src` просто по значению.
                let val = self.read_ea(size, src);

                // 2. Обновляем флаги регистра статуса SR
                self.sr &= !0x0F; // Очищаем N, Z, V, C

                if val == 0 {
                    self.sr |= 0x04; // Флаг Z
                }

                // Проверяем знак в зависимости от размера
                let is_negative = match size {
                    Size::Byte => (val & 0x80) != 0,
                    Size::Word => (val & 0x8000) != 0,
                    Size::Long => (val & 0x8000_0000) != 0,
                };
                if is_negative {
                    self.sr |= 0x08; // Флаг N
                }

                // 3. Записываем данные по месту назначения
                // (Функцию write_ea тебе предстоит написать по аналогии с read_ea!)
                // self.write_ea(size, dst, val);
            },

            // Наша старая проверенная системная команда
            Instruction::MoveToUsp { reg } => {
                let is_supervisor = (self.sr & 0x2000) != 0;
                if !is_supervisor {
                    panic!("permission denied");
                }
                self.usp = self.a[reg as usize];
            },

            Instruction::Add { size, src, dst } => {
                // 1. Универсальное чтение обоих операндов
                let src_val = self.read_ea(size, src);
                let dst_val = self.read_ea(size, dst);

                // 2. Сложение (используем wrapping_add для избежания паник переполнения в Rust)
                let res = dst_val.wrapping_add(src_val);

                // 3. Динамические битовые маски в зависимости от размера операнда
                let (mask, sign_bit) = match size {
                    Size::Byte => (0xFF, 0x80),
                    Size::Word => (0xFFFF, 0x8000),
                    Size::Long => (0xFFFFFFFF, 0x8000_0000),
                };

                let res_masked = res & mask;

                // 4. Универсальное вычисление флагов M68k
                self.sr &= !0x1F; // Очищаем X, N, Z, V, C

                if res_masked == 0 {
                    self.sr |= 0x04; // Флаг Z
                }
                if (res_masked & sign_bit) != 0 {
                    self.sr |= 0x08; // Флаг N
                }

                // Флаг переноса C и X.
                // Гениальный математический трюк: в беззнаковой арифметике,
                // если результат сложения оказался МЕНЬШЕ любого из слагаемых, значит был перенос!
                if res_masked < (src_val & mask) {
                    self.sr |= 0x01; // C
                    self.sr |= 0x10; // X
                }

                // Знаковое переполнение (V): знаки слагаемых равны, но знак результата другой
                let src_sign = (src_val & sign_bit) != 0;
                let dst_sign = (dst_val & sign_bit) != 0;
                let res_sign = (res_masked & sign_bit) != 0;

                if (src_sign == dst_sign) && (src_sign != res_sign) {
                    self.sr |= 0x02; // V
                }

                // 5. Универсальная запись результата!
                // (Внимание архитектора: для режимов вроде (An)+ здесь кроется подводный камень
                // двойного инкремента, но в ADD у Соника мы пишем в регистры, так что пока безопасно).
                self.write_ea(size, dst, res_masked);
            },

            Instruction::Dbra { reg, offset,  } => {
                // Всегда работает только с младшими 16 битами (Word) указанного регистра данных
                let counter = (self.d[reg as usize] & 0xFFFF) as u16;
                let result = counter.wrapping_sub(1);

                // Вклеиваем результат обратно
                self.d[reg as usize] = (self.d[reg as usize] & 0xFFFF0000) | (result as u32);

                // Если счетчик не достиг -1 (0xFFFF), совершаем прыжок
                if result != 0xFFFF {
                    let base_pc = self.pc - 2;
                    self.pc = (base_pc as i32 + offset as i32) as u32;
                }
            },
            Instruction::Lea { src, dst_reg } => {
                // 1. Вычисляем Эффективный Адрес (без чтения памяти по этому адресу!)
                let addr = self.get_ea_address(src);

                // 2. Записываем его в ТОЧНЫЙ индекс массива адресных регистров
                self.a[dst_reg as usize] = addr;

                // 3. Флаги SR остаются нетронутыми!
            },

            Instruction::Unknown(opcode) => {
                panic!("КРИТИЧЕСКАЯ ОШИБКА: Неизвестный опкод {:#06X} по адресу {:#010X}", opcode, self.pc - 2);
            },
        }
    }
    // --- Универсальный читатель адресов ---
    pub fn read_ea(&mut self, size: Size, ea: EA) -> u32 {
        match ea {
            // Не забудь добавить этот блок в твой существующий read_ea:
            EA::AbsoluteWord(w) => {
                let addr = w as i32 as u32;
                self.read_bus_by_size(addr, size)
            },
            // Читаем из регистра данных
            EA::DataRegDirect(reg) => {
                let val = self.d[reg as usize];
                match size {
                    Size::Byte => val & 0xFF,
                    Size::Word => val & 0xFFFF,
                    Size::Long => val,
                }
            },
            // Читаем сам адресный регистр
            EA::AddrRegDirect(reg) => {
                self.a[reg as usize]
            },
            // Читаем из памяти по адресу (An)
            EA::AddrRegIndirect(reg) => {
                let addr = self.a[reg as usize];
                self.read_bus_by_size(addr, size)
            },
            // Читаем из памяти по адресу (An)+ и сдвигаем указатель вперед
            EA::AddrRegPostInc(reg) => {
                let addr = self.a[reg as usize];
                let val = self.read_bus_by_size(addr, size);
                // Увеличиваем регистр на 1, 2 или 4 байта с помощью нашего метода .bytes()
                self.a[reg as usize] = self.a[reg as usize].wrapping_add(size.bytes());
                val
            },
            // Сначала вычитаем указатель -(An), затем читаем из памяти
            EA::AddrRegPreDec(reg) => {
                self.a[reg as usize] = self.a[reg as usize].wrapping_sub(size.bytes());
                let addr = self.a[reg as usize];
                self.read_bus_by_size(addr, size)
            },
            // Просто возвращаем зашитое число
            EA::Immediate32(val) => val,
        }
    }

    // Вспомогательный метод для удобного чтения с шины памяти
    pub fn read_bus_by_size(&mut self, addr: u32, size: Size) -> u32 {
        match size {
            Size::Byte => self.bus.read_u8(addr) as u32,
            Size::Word => self.read_memory_u16(addr) as u32,
            Size::Long => self.read_memory_u32(addr),
        }
    }

    pub fn write_ea(&mut self, size: Size, ea: EA, val: u32) {
        match ea {
              EA::AbsoluteWord(w) => {
                let addr = w as i32 as u32;
                self.write_bus_by_size(addr, size, val);
            }
            // Запись в регистр данных (сохраняем нетронутыми старшие биты!)
            EA::DataRegDirect(reg) => {
                let r = reg as usize;
                match size {
                    Size::Byte => self.d[r] = (self.d[r] & 0xFFFFFF00) | (val & 0xFF),
                    Size::Word => self.d[r] = (self.d[r] & 0xFFFF0000) | (val & 0xFFFF),
                    Size::Long => self.d[r] = val,
                }
            },
            // Запись в адресный регистр (особенность M68k: запись Byte запрещена, Word расширяется знаком)
            EA::AddrRegDirect(reg) => {
                let r = reg as usize;
                match size {
                    Size::Word => self.a[r] = (val as i16) as u32, // Расширение знака
                    Size::Long => self.a[r] = val,
                    _ => panic!("Аппаратная ошибка: попытка записать Byte в Адресный Регистр"),
                }
            },
            // Запись в память по адресу в регистре (An)
            EA::AddrRegIndirect(reg) => {
                let addr = self.a[reg as usize];
                self.write_bus_by_size(addr, size, val);
            },
            // Запись в память с постинкрементом (An)+
            EA::AddrRegPostInc(reg) => {
                let addr = self.a[reg as usize];
                self.write_bus_by_size(addr, size, val);
                // Увеличиваем указатель ПОСЛЕ записи
                self.a[reg as usize] = self.a[reg as usize].wrapping_add(size.bytes());
            },
            // Запись в память с предекрементом -(An)
            EA::AddrRegPreDec(reg) => {
                // Уменьшаем указатель ДО записи
                self.a[reg as usize] = self.a[reg as usize].wrapping_sub(size.bytes());
                let addr = self.a[reg as usize];
                self.write_bus_by_size(addr, size, val);
            },
            EA::Immediate32(_) => {
                panic!("Критическая ошибка эмулятора: Попытка записи в константу (Immediate)!");
            }
        }
    }

    // Вспомогательный метод (если у тебя его еще нет)
    pub fn write_bus_by_size(&mut self, addr: u32, size: Size, val: u32) {
        match size {
            Size::Byte => self.bus.write_u8(addr, val as u8),
            Size::Word => self.write_memory_u16(addr, val as u16),
            Size::Long => self.write_memory_u32(addr, val),
        }
    }
}