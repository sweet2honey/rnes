use crate::{
    bit::{TestBit, WriteBit},
    cpu::AddressingMode::AbsoluteX,
};

const ZERO_FLAG_BIT: u8 = 1;
const NEGATIVE_FLAG_BIT: u8 = 7;

const RESET_OP_ADDR: u16 = 0xFFFC;

#[derive(Clone, Copy)]
enum AddressingMode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

#[derive(Clone, Copy)]
enum Instruction {
    Lda,
    Ldx,
    Ldy,
    Sta,
    Tax,
    Inx,
    Brk,
}

#[derive(Clone, Copy)]
struct Opcode {
    instruction: Instruction,
    mode: AddressingMode,
}

const OPCODES: [Opcode; 256] = {
    let mut table = [Opcode {
        instruction: Instruction::Brk,
        mode: AddressingMode::Implied,
    }; 256];

    // Implied
    table[0x00] = Opcode {
        instruction: Instruction::Brk,
        mode: AddressingMode::Implied,
    };
    table[0xAA] = Opcode {
        instruction: Instruction::Tax,
        mode: AddressingMode::Implied,
    };
    table[0xE8] = Opcode {
        instruction: Instruction::Inx,
        mode: AddressingMode::Implied,
    };

    // LDA
    table[0xA9] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::Immediate,
    };
    table[0xA5] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::ZeroPage,
    };
    table[0xB5] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::ZeroPageX,
    };
    table[0xAD] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::Absolute,
    };
    table[0xBD] = Opcode {
        instruction: Instruction::Lda,
        mode: AbsoluteX,
    };
    table[0xB9] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::AbsoluteY,
    };
    table[0xA1] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::IndirectX,
    };
    table[0xB1] = Opcode {
        instruction: Instruction::Lda,
        mode: AddressingMode::IndirectY,
    };

    // LDX
    table[0xA2] = Opcode {
        instruction: Instruction::Ldx,
        mode: AddressingMode::Immediate,
    };

    // LDY
    table[0xA0] = Opcode {
        instruction: Instruction::Ldy,
        mode: AddressingMode::Immediate,
    };

    // STA
    table[0x85] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::ZeroPage,
    };
    table[0x95] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::ZeroPageX,
    };
    table[0x8D] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::Absolute,
    };
    table[0x9D] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::AbsoluteX,
    };
    table[0x99] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::AbsoluteY,
    };
    table[0x81] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::IndirectX,
    };
    table[0x91] = Opcode {
        instruction: Instruction::Sta,
        mode: AddressingMode::IndirectY,
    };

    table
};

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub pc: u16,
    memory: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            pc: 0,
            memory: [0; 0x10000],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value
    }

    //* 小端寻址
    fn mem_read_u16(&self, addr: u16) -> u16 {
        let low = self.mem_read(addr);
        let high = self.mem_read(addr.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }

    fn mem_write_u16(&mut self, addr: u16, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.mem_write(addr, low);
        self.mem_write(addr.wrapping_add(1), high);
    }

    /// Reads the operand bytes, advances the PC, and returns the effective
    /// address of the operand. Returns `None` for implied addressing.
    fn get_operand_address(&mut self, mode: AddressingMode) -> Option<u16> {
        match mode {
            AddressingMode::Implied => None,

            // The operand is the next byte in the instruction stream.
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                Some(addr)
            }

            AddressingMode::ZeroPage => {
                let addr = self.mem_read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                Some(addr)
            }

            // u8 arithmetic wraps within page zero (0x00..=0xFF).
            AddressingMode::ZeroPageX => {
                let base = self.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Some(base.wrapping_add(self.register_x) as u16)
            }

            AddressingMode::ZeroPageY => {
                let base = self.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Some(base.wrapping_add(self.register_y) as u16)
            }

            AddressingMode::Absolute => {
                let addr = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                Some(addr)
            }

            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                Some(base.wrapping_add(self.register_x as u16))
            }

            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                Some(base.wrapping_add(self.register_y as u16))
            }

            // (Indirect, X): the zero-page byte + X points to a 16-bit pointer.
            AddressingMode::IndirectX => {
                let pointer = self.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);

                let index = pointer.wrapping_add(self.register_x);
                let lo = self.mem_read(index as u16);
                let hi = self.mem_read(index.wrapping_add(1) as u16);
                Some(u16::from_le_bytes([lo, hi]))
            }

            // (Indirect), Y: read a 16-bit pointer from zero page, then add Y.
            AddressingMode::IndirectY => {
                let pointer = self.mem_read(self.pc);
                self.pc = self.pc.wrapping_add(1);

                let lo = self.mem_read(pointer as u16);
                let hi = self.mem_read(pointer.wrapping_add(1) as u16);
                let base = u16::from_le_bytes([lo, hi]);
                Some(base.wrapping_add(self.register_y as u16))
            }
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        self.pc = self.mem_read_u16(RESET_OP_ADDR);
    }

    fn load(&mut self, program: Vec<u8>) {
        let len = program.len();
        self.memory[0x8000..0x8000 + len].copy_from_slice(&program);
        self.mem_write_u16(RESET_OP_ADDR, 0x8000);
    }

    fn run(&mut self) {
        loop {
            let opcode_raw_num = self.mem_read(self.pc);
            self.pc = self.pc.wrapping_add(1);

            let opcode = OPCODES[opcode_raw_num as usize];
            let operand = self.get_operand_address(opcode.mode);

            match opcode.instruction {
                Instruction::Lda => {
                    let value = self.mem_read(operand.expect("LDA has an operand"));
                    self.lda(value);
                }
                Instruction::Ldx => {
                    let value = self.mem_read(operand.expect("LDX has an operand"));
                    self.register_x = value;
                    self.update_zero_and_negative_flags(self.register_x);
                }
                Instruction::Ldy => {
                    let value = self.mem_read(operand.expect("LDY has an operand"));
                    self.register_y = value;
                    self.update_zero_and_negative_flags(self.register_y);
                }
                Instruction::Sta => {
                    let addr = operand.expect("STA has an operand");
                    self.mem_write(addr, self.register_a);
                }
                Instruction::Tax => self.tax(),
                Instruction::Inx => self.inx(),
                Instruction::Brk => return,
            }
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.write_bit(ZERO_FLAG_BIT, result == 0);
        self.status
            .write_bit(NEGATIVE_FLAG_BIT, result.test_bit(NEGATIVE_FLAG_BIT));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x0a, // LDA #$0A
            0xaa, // TAX
            0x00, // BRK
        ]);
        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa2, 0xff, // LDX #$FF
            0xe8, // INX -> $00
            0xe8, // INX -> $01
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_sta_zero_page() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0x85, 0x10, // STA $10
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x10), 0x55);
    }

    #[test]
    fn test_sta_indirect_x() {
        let mut cpu = CPU::new();

        // 指针位于零页 $0F..$10 = $8000，X = 1，因此从 ($0E+1) 处读指针
        cpu.mem_write(0x0f, 0x00);
        cpu.mem_write(0x10, 0x80);

        cpu.load_and_run(vec![
            0xa2, 0x01, // LDX #$01
            0xa9, 0x55, // LDA #$55
            0x81, 0x0e, // STA ($0E,X)
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x8000), 0x55);
    }

    // --- LDA 寻址模式 ---

    #[test]
    fn test_lda_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x12, 0x55);

        cpu.load_and_run(vec![
            0xa2, 0x02, // LDX #$02
            0xb5, 0x10, // LDA $10,X -> $12
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_zero_page_x_wraps() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x00, 0x44);

        cpu.load_and_run(vec![
            0xa2, 0x02, // LDX #$02
            0xb5, 0xfe, // LDA $FE,X -> 零页回绕到 $00
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x44);
    }

    #[test]
    fn test_lda_absolute() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x1234, 0x55);

        cpu.load_and_run(vec![
            0xad, 0x34, 0x12, // LDA $1234
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_absolute_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x1236, 0x55);

        cpu.load_and_run(vec![
            0xa2, 0x02, // LDX #$02
            0xbd, 0x34, 0x12, // LDA $1234,X -> $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_absolute_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x1236, 0x55);

        cpu.load_and_run(vec![
            0xa0, 0x02, // LDY #$02
            0xb9, 0x34, 0x12, // LDA $1234,Y -> $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_indirect_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x12, 0x34); // 指针低字节（$10 + X）
        cpu.mem_write(0x13, 0x12); // 指针高字节 -> $1234
        cpu.mem_write(0x1234, 0x55);

        cpu.load_and_run(vec![
            0xa2, 0x02, // LDX #$02
            0xa1, 0x10, // LDA ($10,X) -> 从 $12 读指针
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_indirect_x_wraps() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x01, 0x34); // 指针低字节（$FF + 2 回绕到 $01）
        cpu.mem_write(0x02, 0x12); // 指针高字节 -> $1234
        cpu.mem_write(0x1234, 0x55);

        cpu.load_and_run(vec![
            0xa2, 0x02, // LDX #$02
            0xa1, 0xff, // LDA ($FF,X) -> 指针位置回绕
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_indirect_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x34); // 指针低字节
        cpu.mem_write(0x11, 0x12); // 指针高字节 -> $1234
        cpu.mem_write(0x1236, 0x55);

        cpu.load_and_run(vec![
            0xa0, 0x02, // LDY #$02
            0xb1, 0x10, // LDA ($10),Y -> $1234 + Y = $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.register_a, 0x55);
    }

    // --- STA 寻址模式 ---

    #[test]
    fn test_sta_absolute() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0x8d, 0x34, 0x12, // STA $1234
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x1234), 0x55);
    }

    #[test]
    fn test_sta_zero_page_x() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0xa2, 0x02, // LDX #$02
            0x95, 0x10, // STA $10,X -> $12
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x12), 0x55);
    }

    #[test]
    fn test_sta_absolute_x() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0xa2, 0x02, // LDX #$02
            0x9d, 0x34, 0x12, // STA $1234,X -> $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x1236), 0x55);
    }

    #[test]
    fn test_sta_absolute_y() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0xa0, 0x02, // LDY #$02
            0x99, 0x34, 0x12, // STA $1234,Y -> $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x1236), 0x55);
    }

    #[test]
    fn test_sta_indirect_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x34); // 指针低字节
        cpu.mem_write(0x11, 0x12); // 指针高字节 -> $1234

        cpu.load_and_run(vec![
            0xa9, 0x55, // LDA #$55
            0xa0, 0x02, // LDY #$02
            0x91, 0x10, // STA ($10),Y -> $1236
            0x00, // BRK
        ]);

        assert_eq!(cpu.mem_read(0x1236), 0x55);
    }
}
