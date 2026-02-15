//! Simple label-based Game Boy assembler
//! 
//! Converts pseudo-assembly with labels into zerodmg-codes Instructions with correct JR offsets.
//! 
//! Example:
//! ```
//! LOOP:
//!     LD A, E
//!     CP 85
//!     JR C, EXIT
//!     SUB 85
//!     INC BC
//!     JR LOOP
//! EXIT:
//!     LD A, E
//! ```

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum AsmLine {
    Label(String),
    Instruction(Instruction),
    Jump { cond: Option<FlagCondition>, target: String },
}

pub struct Assembler {
    lines: Vec<AsmLine>,
    labels: HashMap<String, usize>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            labels: HashMap::new(),
        }
    }
    
    pub fn label(&mut self, name: &str) -> &mut Self {
        self.lines.push(AsmLine::Label(name.to_string()));
        self
    }
    
    pub fn inst(&mut self, inst: Instruction) -> &mut Self {
        self.lines.push(AsmLine::Instruction(inst));
        self
    }
    
    pub fn jr(&mut self, target: &str) -> &mut Self {
        self.lines.push(AsmLine::Jump { cond: None, target: target.to_string() });
        self
    }
    
    pub fn jr_cond(&mut self, cond: FlagCondition, target: &str) -> &mut Self {
        self.lines.push(AsmLine::Jump { cond: Some(cond), target: target.to_string() });
        self
    }
    
    /// Assemble to final instruction list with resolved jumps
    pub fn assemble(mut self) -> Vec<Instruction> {
        // First pass: calculate label positions
        let mut pc = 0;
        let mut label_positions = HashMap::new();
        
        for line in &self.lines {
            match line {
                AsmLine::Label(name) => {
                    label_positions.insert(name.clone(), pc);
                }
                AsmLine::Instruction(inst) => {
                    pc += inst.clone().to_bytes().len();
                }
                AsmLine::Jump { .. } => {
                    pc += 2; // JR is always 2 bytes
                }
            }
        }
        
        // Second pass: generate instructions with correct offsets
        let mut result = Vec::new();
        let mut pc = 0;
        
        for line in &self.lines {
            match line {
                AsmLine::Label(_) => {
                    // Labels don't emit code
                }
                AsmLine::Instruction(inst) => {
                    let bytes_len = inst.clone().to_bytes().len();
                    result.push(inst.clone());
                    pc += bytes_len;
                }
                AsmLine::Jump { cond, target } => {
                    let target_pc = label_positions.get(target)
                        .expect(&format!("Undefined label: {}", target));
                    
                    let offset = (*target_pc as i32) - ((pc + 2) as i32);
                    
                    if offset < -128 || offset > 127 {
                        panic!("Jump offset {} out of range for JR to {}", offset, target);
                    }
                    
                    let inst = if let Some(cond) = cond {
                        Instruction::JR_IF(*cond, offset as i8)
                    } else {
                        Instruction::JR(offset as i8)
                    };
                    
                    result.push(inst);
                    pc += 2;
                }
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_loop() {
        let mut asm = Assembler::new();
        
        asm.label("LOOP")
            .inst(LD_8_IMMEDIATE(A, 5))
            .inst(DEC(A))
            .jr_cond(if_NZ, "LOOP")
            .label("DONE")
            .inst(HALT);
        
        let code = asm.assemble();
        
        // Should have: LD A,5 (2 bytes), DEC A (1 byte), JR NZ,-5 (2 bytes), HALT (1 byte)
        assert_eq!(code.len(), 4);
        
        // The JR should jump back 5 bytes (to LOOP)
        if let Instruction::JR_IF(_, offset) = code[2] {
            assert_eq!(offset, -5);
        } else {
            panic!("Expected JR_IF");
        }
    }
}

// Re-export for convenience
pub use Instruction::*;
pub use U8Register::*;
pub use U16Register::*;
pub use U8SecondaryRegister::*;
pub use FlagCondition::*;
