mod cpu;
mod bus;
mod enums;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();

    // 1. Вставляем картридж в виртуальную консоль
    cpu.bus.load_rom("./game/Sonic & Knuckles.bin");

    // 2. Аппаратный сброс (Reset Sequence)
    // Процессор M68k аппаратно читает свои стартовые параметры из начала памяти

    // Читаем начальный Указатель стека (SP / A7) из адреса 0x000000
    let initial_sp = cpu.read_memory_u32(0x000000);
    cpu.a[1] = initial_sp;

    // Читаем начальный Счетчик команд (PC / Точка входа) из адреса 0x000004
    let initial_pc = cpu.read_memory_u32(0x000004);
    cpu.pc = initial_pc;

    cpu.sr = 0x2700;

    println!("--- СИСТЕМА ЗАПУЩЕНА ---");
    println!("Начальный SP:  {:#010X}", cpu.a[1]);
    println!("Начальный PC:  {:#010X}", cpu.pc);
    println!("------------------------");

    // 3. Запускаем конвейер Fetch, чтобы просто ПОСМОТРЕТЬ на первые инструкции Соника!
    // Мы пока не будем вызывать decode и execute, так как игра использует
    // кучу новых команд, которые наш декодер пока не понимает.
    let mut i = 0;
    loop {
        let opcode = cpu.fetch();
        let inst = cpu.decode(opcode);
        cpu.execute(inst);
        i += 1;
        println!("циклов {}",i)
    }
}