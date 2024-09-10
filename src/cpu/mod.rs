pub mod gpu;
pub mod instructions;
pub mod registers;

use self::gpu::*;
use self::instructions::*;
use self::registers::Registers;

struct MemoryBus {
    memory: [u8; 0xFFFF],
    gpu: Gpu,
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            gpu::VRAM_BEGIN..=gpu::VRAM_END => self.gpu.read_vram(address - gpu::VRAM_BEGIN),
            _ => self.memory[address],
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            gpu::VRAM_BEGIN..=gpu::VRAM_END => {
                self.gpu.write_vram(address - gpu::VRAM_BEGIN, value)
            }
            _ => self.memory[address] = value,
        }
    }
}

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
    is_halted: bool,
}

impl Cpu {
    pub fn new(boot_rom: Option<Vec<u8>>, game_rom: Vec<u8>) -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            bus: MemoryBus {
                memory: [0; 0xFFFF],
                gpu: Gpu::new(),
            },
            is_halted: false,
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        if self.is_halted {
            return self.pc;
        }
        match instruction {
            Instruction::ADD(target) => match target {
                ArithmeticTarget::A => {
                    let value = self.registers.a;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::B => {
                    let value = self.registers.b;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::D => {
                    let value = self.registers.d;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::E => {
                    let value = self.registers.e;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::H => {
                    let value = self.registers.h;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::L => {
                    let value = self.registers.l;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
            },
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::B => self.registers.b,
                        LoadByteSource::C => self.registers.c,
                        LoadByteSource::D => self.registers.d,
                        LoadByteSource::E => self.registers.e,
                        LoadByteSource::H => self.registers.h,
                        LoadByteSource::L => self.registers.l,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::B => self.registers.b = source_value,
                        LoadByteTarget::C => self.registers.c = source_value,
                        LoadByteTarget::D => self.registers.d = source_value,
                        LoadByteTarget::E => self.registers.e = source_value,
                        LoadByteTarget::H => self.registers.h = source_value,
                        LoadByteTarget::L => self.registers.l = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                    };
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                _ => panic!("TODO: implement more load types"),
            },
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::BC => self.registers.set_bc(result),
                    StackTarget::DE => self.registers.set_de(result),
                    StackTarget::HL => self.registers.set_hl(result),
                    StackTarget::AF => self.registers.set_af(result),
                };
                self.pc.wrapping_add(1)
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::HL => self.registers.get_hl(),
                };
                self.push(value);
                self.pc.wrapping_add(1)
            }
            Instruction::CALL(test) => {
                let jump_condition = match test {
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.call(jump_condition)
            }
            Instruction::RET(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.return_(jump_condition)
            }
            Instruction::NOP => self.pc.wrapping_add(1),
            Instruction::ADDHL(target) => match target {
                ADDHLTarget::BC => {
                    let value = self.registers.get_bc();
                    self.add_hl(value);
                    self.pc.wrapping_add(2)
                }
                ADDHLTarget::DE => {
                    let value = self.registers.get_de();
                    self.add_hl(value);
                    self.pc.wrapping_add(2)
                }
                ADDHLTarget::HL => {
                    let value = self.registers.get_hl();
                    self.add_hl(value);
                    self.pc.wrapping_add(2)
                }
                ADDHLTarget::SP => {
                    let value = self.sp;
                    self.add_hl(value);
                    self.pc.wrapping_add(2)
                }
            },
            Instruction::INC(target) => match target {
                IncDecTarget::A => {
                    let value = self.registers.a;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::B => {
                    let value = self.registers.b;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.b = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::C => {
                    let value = self.registers.c;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.c = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::D => {
                    let value = self.registers.d;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.d = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::E => {
                    let value = self.registers.e;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.e = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::H => {
                    let value = self.registers.h;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.h = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::L => {
                    let value = self.registers.l;
                    let (new_value, did_overflow) = value.overflowing_add(1);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = false;
                    self.registers.f.carry = did_overflow;
                    self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;
                    self.registers.l = new_value;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::BC => {
                    let value = self.registers.get_bc();
                    self.registers.set_bc(value.wrapping_add(1));
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::DE => {
                    let value = self.registers.get_de();
                    self.registers.set_de(value.wrapping_add(1));
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::SP => {
                    let value = self.sp;
                    self.sp = value.wrapping_add(1);
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::HL => {
                    let value = self.bus.read_byte(self.registers.get_hl());
                    let new_value = self.add(value);
                    self.bus.write_byte(self.registers.get_hl(), new_value);
                    self.pc.wrapping_add(3)
                }
            },
            Instruction::DEC(target) => match target {
                IncDecTarget::A => {
                    let value = self.registers.a;
                    let new_value = value.wrapping_sub(1);
                    self.registers.a = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::B => {
                    let value = self.registers.b;
                    let new_value = value.wrapping_sub(1);
                    self.registers.b = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::C => {
                    let value = self.registers.c;
                    let new_value = value.wrapping_sub(1);
                    self.registers.c = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::D => {
                    let value = self.registers.d;
                    let new_value = value.wrapping_sub(1);
                    self.registers.d = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::E => {
                    let value = self.registers.e;
                    let new_value = value.wrapping_sub(1);
                    self.registers.e = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::H => {
                    let value = self.registers.h;
                    let new_value = value.wrapping_sub(1);
                    self.registers.h = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::L => {
                    let value = self.registers.l;
                    let new_value = value.wrapping_sub(1);
                    self.registers.l = new_value;
                    self.registers.f.half_carry = (value & 0xF) == 0;
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.pc.wrapping_add(1)
                }
                IncDecTarget::BC => {
                    self.registers
                        .set_bc(self.registers.get_bc().wrapping_sub(1));
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::DE => {
                    self.registers
                        .set_de(self.registers.get_de().wrapping_sub(1));
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::SP => {
                    self.sp = self.sp.wrapping_sub(1);
                    self.pc.wrapping_add(2)
                }
                IncDecTarget::HL => {
                    let target = self.registers.get_hl();
                    let new_value = self.bus.read_byte(target).wrapping_sub(1);
                    self.bus.write_byte(target, new_value);
                    self.registers.f.zero = new_value == 0;
                    self.registers.f.subtract = true;
                    self.registers.f.half_carry = (new_value & 0xF) == 0;
                    self.pc.wrapping_add(3)
                }
            },
            Instruction::RLA => {
                panic!("TODO: implement RLA");
            }
            Instruction::RLCA => {
                panic!("TODO: implement RLCA");
            }
            Instruction::RRA => {
                panic!("TODO: implement RRA");
            }
            Instruction::RRCA => {
                panic!("TODO: implement RRA");
            }
        }
    }

    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
            (most_significant_byte << 8) | least_significant_byte
        } else {
            self.pc.wrapping_add(3)
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    fn add_hl(&mut self, value: u16) {
        let (new_value, did_overflow) = self.registers.get_hl().overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.get_hl() & 0xFF) + (value & 0xFF) > 0xFF;
        self.registers.set_hl(new_value);
    }

    fn pop(&mut self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let most_significant_byte = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (most_significant_byte << 8) | least_significant_byte
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    pub fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            let description = format!(
                "0x{}{:x}",
                if prefixed { "cb" } else { "" },
                instruction_byte
            );
            panic!("Unknown instruction found for: {}", description);
        };

        self.pc = next_pc;
    }

    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    fn read_next_word(&self) -> u16 {
        ((self.bus.read_byte(self.pc + 2) as u16) << 8) | (self.bus.read_byte(self.pc + 1) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_instruction {
        ( $instruction:expr, $( $($register:ident).* => $value:expr ),* ) => {
            {
                let mut cpu = Cpu::new(None, vec![0; 0xFFFF]);
                $(
                    cpu.registers$(.$register)* = $value;
                )*
                cpu.execute($instruction);
                cpu
            }
        };
    }

    macro_rules! check_flags {
        ( $cpu:ident, zero => $zero:ident, subtract => $subtract:ident, half_carry => $half_carry:ident, carry => $carry:ident ) => {{
            let flags = $cpu.registers.f;
            println!("Flags: {:?}", flags);
            assert_eq!(flags.zero, $zero, "Zero flag");
            assert_eq!(flags.subtract, $subtract, "Subtract flag");
            assert_eq!(flags.half_carry, $half_carry, "Half Carry flag");
            assert_eq!(flags.carry, $carry, "Carry flag");
        }};
    }

    #[test]
    fn test_overflow_add_sanity() {
        let a: u16 = 0x0002;
        let b: u16 = 0xFFFF;
        let (result, did_overflow) = a.overflowing_add(b);
        assert_eq!(result, 1);
        assert!(did_overflow);

        let a: u8 = 0x01;
        let b: u8 = 0xFF;
        let (result, did_overflow) = a.overflowing_add(b);
        assert_eq!(result, 0);
        assert!(did_overflow);
    }

    #[test]
    fn test_overflow_sub_sanity() {
        let a: u16 = 0x0000;
        let b: u16 = 0x0001;
        let (result, did_underflow) = a.overflowing_sub(b);
        assert_eq!(result, 0xFFFF);
        assert!(did_underflow);
    }

    #[test]
    fn test_basic_add_b() {
        let cpu = test_instruction!(Instruction::ADD(ArithmeticTarget::B), a => 0x1, b => 0x3);

        assert_eq!(cpu.registers.a, 0x4);
        check_flags!(cpu, zero => false, subtract => false, half_carry => false, carry => false);
    }

    #[test]
    fn test_basic_add_c() {
        let cpu = test_instruction!(Instruction::ADD(ArithmeticTarget::C), a => 0x00, c => 0x1);

        assert_eq!(cpu.registers.a, 0x1);
        check_flags!(cpu, zero => false, subtract => false, half_carry => false, carry => false);
    }

    #[test]
    fn test_basic_add_c_w_carry() {
        let cpu = test_instruction!(Instruction::ADD(ArithmeticTarget::C), a => 0x02, c => 0xFF);

        assert_eq!(cpu.registers.a, 0x01);
        check_flags!(cpu, zero => false, subtract => false, half_carry => true, carry => true);
    }

    #[test]
    fn test_hl_add_c() {
        let cpu = test_instruction!(Instruction::ADDHL(ADDHLTarget::BC), h => 0x00, l => 0x00, b => 0x0, c => 0x1);

        assert_eq!(cpu.registers.get_hl(), 0x1);
        check_flags!(cpu, zero => false, subtract => false, half_carry => false, carry => false);
    }

    #[test]
    fn test_hl_add_w_half_carry() {
        let cpu = test_instruction!(Instruction::ADDHL(ADDHLTarget::BC), h => 0b0000_0000, l => 0b0000_0001, b => 0b0011_1111, c => 0b1111_1111);

        assert_eq!(cpu.registers.get_hl(), 0b0100_0000_0000_0000);
        check_flags!(cpu, zero => false, subtract => false, half_carry => true, carry => false);
    }

    #[test]
    fn test_hl_add_w_full_carry() {
        let cpu = test_instruction!(Instruction::ADDHL(ADDHLTarget::BC), h => 0x00, l => 0x01, b => 0xFF, c => 0xFF);

        assert_eq!(cpu.registers.get_hl(), 0x00);
        check_flags!(cpu, zero => true, subtract => false, half_carry => true, carry => true);
    }

    #[test]
    fn test_dec_a() {
        let cpu = test_instruction!(Instruction::DEC(IncDecTarget::A), a => 0x01);
        assert_eq!(cpu.registers.a, 0x00);
        check_flags!(cpu, zero => true, subtract => true, half_carry => false, carry => false);
    }

    #[test]
    fn test_dec_a_w_hc() {
        let cpu = test_instruction!(Instruction::DEC(IncDecTarget::A), a => 0x10);
        assert_eq!(cpu.registers.a, 0x0F);
        check_flags!(cpu, zero => false, subtract => true, half_carry => true, carry => false);
    }
}
