use std::fs::File;
use std::io::Read;

pub struct Bus {
    pub ram: [u8; 65536],
    pub rom: Vec<u8>,
}

impl Bus {
    pub fn new() -> Self {
        Bus{
            ram: [0; 65536],
            rom: vec![0; 4*1024*1024],
        }
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        match addr {
            0x000000..=0x3FFFFF => self.rom[addr as usize],
            0xFF0000..=0xFFFFFF => self.ram[(addr & 0xFFFF) as usize],
            0xA10008..=0xA1000B => 0x00,
            0xA10000..=0xA10007 => 0x00,
            0x00A10008..=0x00A1000B => 0x00,
            0x00A10000..=0x00A1001F => 0x00,
            _ => 0,
        }
    }
    pub fn write_u8(&mut self, addr: u32, data: u8) {
        match addr {
            0x000000..=0x3FFFFF => {},
            0xFF0000..=0xFFFFFF => self.ram[(addr & 0xFFFF) as usize] = data,
            _ => {}
        }
    }
    pub fn load_rom(&mut self,path: &str) {
        let mut file = File::open(path).expect("Файл не удалось открыть");

        let bytes_read = file.read(&mut self.rom).expect("Ошибка про чтении");

        println!("картридж успешно загружен! прочитано байт: {}", bytes_read);
    }
}