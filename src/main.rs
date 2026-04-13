mod cpu;
use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();

    // 1. Инициализируем дно стека в конце памяти (4 МБ)
    cpu.a[4] = 4 * 1024 * 1024;

    // ОСНОВНАЯ ПРОГРАММА (начинается с 0x200):
    // Инструкция 1: Вызов функции JSR 0x00000300
    cpu.write_memory_u16(0x200, 0x4EB9);
    cpu.write_memory_u32(0x202, 0x00000300);
    // Инструкция 2: Сюда мы должны вернуться после RTS!
    cpu.write_memory_u16(0x206, 0x7299); // MOVEQ #0x99, D1

    // ПОДПРОГРАММА (начинается с 0x300):
    cpu.write_memory_u16(0x300, 0x7477); // MOVEQ #0x77, D2
    cpu.write_memory_u16(0x302, 0x4E75); // RTS - вернуться назад!

    // Крутим конвейер ровно 4 раза
    for _ in 0..4 {
        let opcode = cpu.fetch();
        let instr = cpu.decode(opcode);
        cpu.execute(instr);
    }

    println!("Финальный PC: {:#06X}", cpu.pc);
    println!("Регистр D1: {:#X}", cpu.d[5]);
    println!("Регистр D2: {:#X}", cpu.d[6]);
}