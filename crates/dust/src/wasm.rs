/// Low-level WASM binary encoder.
/// Emits valid WASM modules following the spec at
/// https://webassembly.github.io/spec/core/binary/

// Section IDs
const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_MEMORY: u8 = 5;
const SECTION_GLOBAL: u8 = 6;
const SECTION_EXPORT: u8 = 7;
const SECTION_START: u8 = 8;
const SECTION_CODE: u8 = 10;
const SECTION_DATA: u8 = 11;

// Value types
pub const TYPE_I32: u8 = 0x7F;
pub const TYPE_I64: u8 = 0x7E;
pub const TYPE_F32: u8 = 0x7D;
pub const TYPE_F64: u8 = 0x7C;
const TYPE_FUNC: u8 = 0x60;

// Export kinds
pub const EXPORT_FUNC: u8 = 0x00;
pub const EXPORT_MEMORY: u8 = 0x02;
pub const EXPORT_GLOBAL: u8 = 0x03;

// Import kinds
pub const IMPORT_FUNC: u8 = 0x00;

// Global mutability
pub const GLOBAL_CONST: u8 = 0x00;
pub const GLOBAL_MUT: u8 = 0x01;

// Opcodes
pub mod op {
    // Control
    pub const UNREACHABLE: u8 = 0x00;
    pub const NOP: u8 = 0x01;
    pub const BLOCK: u8 = 0x02;
    pub const LOOP: u8 = 0x03;
    pub const IF: u8 = 0x04;
    pub const ELSE: u8 = 0x05;
    pub const END: u8 = 0x0B;
    pub const BR: u8 = 0x0C;
    pub const BR_IF: u8 = 0x0D;
    pub const RETURN: u8 = 0x0F;
    pub const CALL: u8 = 0x10;
    pub const DROP: u8 = 0x1A;
    pub const SELECT: u8 = 0x1B;

    // Variable access
    pub const LOCAL_GET: u8 = 0x20;
    pub const LOCAL_SET: u8 = 0x21;
    pub const LOCAL_TEE: u8 = 0x22;
    pub const GLOBAL_GET: u8 = 0x23;
    pub const GLOBAL_SET: u8 = 0x24;

    // Memory
    pub const I32_LOAD: u8 = 0x28;
    pub const I64_LOAD: u8 = 0x29;
    pub const F32_LOAD: u8 = 0x2A;
    pub const F64_LOAD: u8 = 0x2B;
    pub const I32_STORE: u8 = 0x36;
    pub const I64_STORE: u8 = 0x37;
    pub const F32_STORE: u8 = 0x38;
    pub const F64_STORE: u8 = 0x39;

    // Constants
    pub const I32_CONST: u8 = 0x41;
    pub const I64_CONST: u8 = 0x42;
    pub const F32_CONST: u8 = 0x43;
    pub const F64_CONST: u8 = 0x44;

    // i32 comparison
    pub const I32_EQZ: u8 = 0x45;
    pub const I32_EQ: u8 = 0x46;
    pub const I32_NE: u8 = 0x47;
    pub const I32_LT_S: u8 = 0x48;
    pub const I32_LT_U: u8 = 0x49;
    pub const I32_GT_S: u8 = 0x4A;
    pub const I32_GT_U: u8 = 0x4B;
    pub const I32_LE_S: u8 = 0x4C;
    pub const I32_LE_U: u8 = 0x4D;
    pub const I32_GE_S: u8 = 0x4E;
    pub const I32_GE_U: u8 = 0x4F;

    // i64 comparison
    pub const I64_EQZ: u8 = 0x50;
    pub const I64_EQ: u8 = 0x51;
    pub const I64_NE: u8 = 0x52;
    pub const I64_LT_S: u8 = 0x53;
    pub const I64_GT_S: u8 = 0x55;
    pub const I64_LE_S: u8 = 0x56;
    pub const I64_GE_S: u8 = 0x58;

    // f64 comparison
    pub const F64_EQ: u8 = 0x61;
    pub const F64_NE: u8 = 0x62;
    pub const F64_LT: u8 = 0x63;
    pub const F64_GT: u8 = 0x64;
    pub const F64_LE: u8 = 0x65;
    pub const F64_GE: u8 = 0x66;

    // i32 arithmetic
    pub const I32_ADD: u8 = 0x6A;
    pub const I32_SUB: u8 = 0x6B;
    pub const I32_MUL: u8 = 0x6C;
    pub const I32_DIV_S: u8 = 0x6D;
    pub const I32_DIV_U: u8 = 0x6E;
    pub const I32_REM_S: u8 = 0x6F;
    pub const I32_REM_U: u8 = 0x70;
    pub const I32_AND: u8 = 0x71;
    pub const I32_OR: u8 = 0x72;
    pub const I32_XOR: u8 = 0x73;
    pub const I32_SHL: u8 = 0x74;
    pub const I32_SHR_S: u8 = 0x75;
    pub const I32_SHR_U: u8 = 0x76;

    // i64 arithmetic
    pub const I64_ADD: u8 = 0x7C;
    pub const I64_SUB: u8 = 0x7D;
    pub const I64_MUL: u8 = 0x7E;
    pub const I64_DIV_S: u8 = 0x7F;
    pub const I64_REM_S: u8 = 0x81;
    pub const I64_AND: u8 = 0x83;
    pub const I64_OR: u8 = 0x84;
    pub const I64_XOR: u8 = 0x85;
    pub const I64_SHL: u8 = 0x86;
    pub const I64_SHR_S: u8 = 0x87;

    // f64 arithmetic
    pub const F64_ADD: u8 = 0xA0;
    pub const F64_SUB: u8 = 0xA1;
    pub const F64_MUL: u8 = 0xA2;
    pub const F64_DIV: u8 = 0xA3;
    pub const F64_NEG: u8 = 0x9A;

    // Conversions
    pub const I32_WRAP_I64: u8 = 0xA7;
    pub const I64_EXTEND_I32_S: u8 = 0xAC;
    pub const F64_CONVERT_I32_S: u8 = 0xB7;
    pub const F64_CONVERT_I64_S: u8 = 0xB9;
    pub const I32_TRUNC_F64_S: u8 = 0xAA;
    pub const I64_TRUNC_F64_S: u8 = 0xB0;
}

/// A type signature for a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<u8>,
    pub results: Vec<u8>,
}

/// An import entry.
#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Func(u32), // type index
}

/// An export entry.
#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub kind: u8,
    pub index: u32,
}

/// A global variable.
#[derive(Debug, Clone)]
pub struct Global {
    pub val_type: u8,
    pub mutable: bool,
    pub init: Vec<u8>, // init expression bytes (including END)
}

/// A data segment for linear memory initialization.
#[derive(Debug, Clone)]
pub struct DataSegment {
    pub offset: u32,
    pub data: Vec<u8>,
}

/// Local variable declaration in a function body.
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub count: u32,
    pub val_type: u8,
}

/// A complete function body.
#[derive(Debug, Clone)]
pub struct FuncBody {
    pub locals: Vec<LocalDecl>,
    pub code: Vec<u8>, // bytecode including trailing END
}

/// Builder for a WASM module.
pub struct Module {
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub functions: Vec<u32>, // type indices for locally defined functions
    pub exports: Vec<Export>,
    pub globals: Vec<Global>,
    pub memory: Option<(u32, Option<u32>)>, // (min_pages, max_pages)
    pub start: Option<u32>,
    pub bodies: Vec<FuncBody>,
    pub data: Vec<DataSegment>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
            globals: Vec::new(),
            memory: None,
            start: None,
            bodies: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Add a function type signature, returning its index.
    /// Deduplicates identical signatures.
    pub fn add_type(&mut self, ft: FuncType) -> u32 {
        if let Some(idx) = self.types.iter().position(|t| t == &ft) {
            return idx as u32;
        }
        let idx = self.types.len() as u32;
        self.types.push(ft);
        idx
    }

    /// Total number of functions (imports + local).
    pub fn func_count(&self) -> u32 {
        let import_funcs = self
            .imports
            .iter()
            .filter(|i| matches!(i.kind, ImportKind::Func(_)))
            .count() as u32;
        import_funcs + self.functions.len() as u32
    }

    /// Number of imported functions.
    pub fn import_func_count(&self) -> u32 {
        self.imports
            .iter()
            .filter(|i| matches!(i.kind, ImportKind::Func(_)))
            .count() as u32
    }

    /// Encode this module to a WASM binary.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // Magic number and version
        out.extend_from_slice(b"\0asm");
        out.extend_from_slice(&[1, 0, 0, 0]);

        // Type section
        if !self.types.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.types.len() as u32);
            for ft in &self.types {
                section.push(TYPE_FUNC);
                encode_u32(&mut section, ft.params.len() as u32);
                section.extend_from_slice(&ft.params);
                encode_u32(&mut section, ft.results.len() as u32);
                section.extend_from_slice(&ft.results);
            }
            emit_section(&mut out, SECTION_TYPE, &section);
        }

        // Import section
        if !self.imports.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.imports.len() as u32);
            for imp in &self.imports {
                encode_name(&mut section, &imp.module);
                encode_name(&mut section, &imp.name);
                match &imp.kind {
                    ImportKind::Func(type_idx) => {
                        section.push(IMPORT_FUNC);
                        encode_u32(&mut section, *type_idx);
                    }
                }
            }
            emit_section(&mut out, SECTION_IMPORT, &section);
        }

        // Function section
        if !self.functions.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.functions.len() as u32);
            for &type_idx in &self.functions {
                encode_u32(&mut section, type_idx);
            }
            emit_section(&mut out, SECTION_FUNCTION, &section);
        }

        // Memory section
        if let Some((min, max)) = self.memory {
            let mut section = Vec::new();
            encode_u32(&mut section, 1); // one memory
            if let Some(max) = max {
                section.push(0x01); // has max
                encode_u32(&mut section, min);
                encode_u32(&mut section, max);
            } else {
                section.push(0x00); // no max
                encode_u32(&mut section, min);
            }
            emit_section(&mut out, SECTION_MEMORY, &section);
        }

        // Global section
        if !self.globals.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.globals.len() as u32);
            for g in &self.globals {
                section.push(g.val_type);
                section.push(if g.mutable { GLOBAL_MUT } else { GLOBAL_CONST });
                section.extend_from_slice(&g.init);
            }
            emit_section(&mut out, SECTION_GLOBAL, &section);
        }

        // Export section
        if !self.exports.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.exports.len() as u32);
            for exp in &self.exports {
                encode_name(&mut section, &exp.name);
                section.push(exp.kind);
                encode_u32(&mut section, exp.index);
            }
            emit_section(&mut out, SECTION_EXPORT, &section);
        }

        // Start section
        if let Some(start_idx) = self.start {
            let mut section = Vec::new();
            encode_u32(&mut section, start_idx);
            emit_section(&mut out, SECTION_START, &section);
        }

        // Code section
        if !self.bodies.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.bodies.len() as u32);
            for body in &self.bodies {
                let mut func_bytes = Vec::new();
                encode_u32(&mut func_bytes, body.locals.len() as u32);
                for local in &body.locals {
                    encode_u32(&mut func_bytes, local.count);
                    func_bytes.push(local.val_type);
                }
                func_bytes.extend_from_slice(&body.code);
                // Size-prefix the function body
                encode_u32(&mut section, func_bytes.len() as u32);
                section.extend_from_slice(&func_bytes);
            }
            emit_section(&mut out, SECTION_CODE, &section);
        }

        // Data section
        if !self.data.is_empty() {
            let mut section = Vec::new();
            encode_u32(&mut section, self.data.len() as u32);
            for seg in &self.data {
                section.push(0x00); // active segment, memory 0
                // offset expression: i32.const <offset> end
                section.push(op::I32_CONST);
                encode_i32(&mut section, seg.offset as i32);
                section.push(op::END);
                encode_u32(&mut section, seg.data.len() as u32);
                section.extend_from_slice(&seg.data);
            }
            emit_section(&mut out, SECTION_DATA, &section);
        }

        out
    }
}

fn emit_section(out: &mut Vec<u8>, id: u8, content: &[u8]) {
    out.push(id);
    encode_u32(out, content.len() as u32);
    out.extend_from_slice(content);
}

/// Encode a u32 as LEB128.
pub fn encode_u32(out: &mut Vec<u8>, mut val: u32) {
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if val == 0 {
            break;
        }
    }
}

/// Encode an i32 as signed LEB128.
pub fn encode_i32(out: &mut Vec<u8>, mut val: i32) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        let more = !((val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0));
        if more {
            out.push(byte | 0x80);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Encode an i64 as signed LEB128.
pub fn encode_i64(out: &mut Vec<u8>, mut val: i64) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        let more = !((val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0));
        if more {
            out.push(byte | 0x80);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Encode a UTF-8 name (length-prefixed).
pub fn encode_name(out: &mut Vec<u8>, name: &str) {
    encode_u32(out, name.len() as u32);
    out.extend_from_slice(name.as_bytes());
}

/// Block type for structured control flow.
pub const BLOCK_VOID: u8 = 0x40;

/// Helper: build an init expression for an i32 constant global.
pub fn init_i32(val: i32) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(op::I32_CONST);
    encode_i32(&mut v, val);
    v.push(op::END);
    v
}

/// Helper: build an init expression for an i64 constant global.
pub fn init_i64(val: i64) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(op::I64_CONST);
    encode_i64(&mut v, val);
    v.push(op::END);
    v
}

/// Helper: build an init expression for an f64 constant global.
pub fn init_f64(val: f64) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(op::F64_CONST);
    v.extend_from_slice(&val.to_le_bytes());
    v.push(op::END);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leb128_u32() {
        let mut buf = Vec::new();
        encode_u32(&mut buf, 0);
        assert_eq!(buf, vec![0]);

        buf.clear();
        encode_u32(&mut buf, 127);
        assert_eq!(buf, vec![127]);

        buf.clear();
        encode_u32(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);

        buf.clear();
        encode_u32(&mut buf, 624485);
        assert_eq!(buf, vec![0xE5, 0x8E, 0x26]);
    }

    #[test]
    fn test_leb128_i32() {
        let mut buf = Vec::new();
        encode_i32(&mut buf, 0);
        assert_eq!(buf, vec![0]);

        buf.clear();
        encode_i32(&mut buf, -1);
        assert_eq!(buf, vec![0x7F]);

        buf.clear();
        encode_i32(&mut buf, -123456);
        assert_eq!(buf, vec![0xC0, 0xBB, 0x78]);
    }

    #[test]
    fn test_empty_module() {
        let module = Module::new();
        let bytes = module.encode();
        assert_eq!(&bytes[0..4], b"\0asm");
        assert_eq!(&bytes[4..8], &[1, 0, 0, 0]);
        assert_eq!(bytes.len(), 8); // just header, no sections
    }

    #[test]
    fn test_module_with_function() {
        let mut module = Module::new();
        let type_idx = module.add_type(FuncType {
            params: vec![],
            results: vec![TYPE_I32],
        });
        module.functions.push(type_idx);
        module.bodies.push(FuncBody {
            locals: vec![],
            code: vec![op::I32_CONST, 42, op::END],
        });
        module.exports.push(Export {
            name: "answer".into(),
            kind: EXPORT_FUNC,
            index: 0,
        });
        let bytes = module.encode();
        assert_eq!(&bytes[0..4], b"\0asm");
        assert!(bytes.len() > 8);
    }
}
