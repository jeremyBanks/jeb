use std::collections::HashMap;

use crate::ast::Node;
use crate::token::TokenKind;
use crate::wasm::{self, op, FuncBody, FuncType, LocalDecl, Module};

#[derive(Debug)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "compile error: {}", self.0)
    }
}

impl std::error::Error for CompileError {}

/// Value type in our compiler's type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValType {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Void,
}

impl ValType {
    fn to_wasm(self) -> Option<u8> {
        match self {
            ValType::I32 | ValType::Bool => Some(wasm::TYPE_I32),
            ValType::I64 => Some(wasm::TYPE_I64),
            ValType::F32 => Some(wasm::TYPE_F32),
            ValType::F64 => Some(wasm::TYPE_F64),
            ValType::Void => None,
        }
    }

    fn from_name(name: &str) -> Result<ValType, CompileError> {
        match name {
            "i32" => Ok(ValType::I32),
            "i64" => Ok(ValType::I64),
            "f32" => Ok(ValType::F32),
            "f64" => Ok(ValType::F64),
            "bool" => Ok(ValType::Bool),
            "()" => Ok(ValType::Void),
            _ => Err(CompileError(format!("unknown type: {}", name))),
        }
    }
}

/// Information about a local variable in a function.
#[derive(Debug, Clone)]
struct LocalVar {
    index: u32,
    val_type: ValType,
}

/// Information about a compiled function.
#[derive(Debug, Clone)]
struct FuncInfo {
    /// Index in the module's function index space (imports + local).
    index: u32,
    /// Parameter types.
    params: Vec<ValType>,
    /// Return type.
    ret: ValType,
}

struct FuncCompiler<'a> {
    /// Wasm bytecode being built.
    code: Vec<u8>,
    /// Local variables: name -> (index, type).
    locals: HashMap<String, LocalVar>,
    /// Total number of locals allocated (params + declared locals).
    local_count: u32,
    /// Local declarations to emit (type groups for non-param locals).
    local_decls: Vec<LocalDecl>,
    /// The function's return type.
    #[allow(dead_code)]
    ret_type: ValType,
    /// Access to module-level info.
    module_ctx: &'a ModuleCtx,
    /// Current loop nesting depth (for break/continue label resolution).
    loop_depth: u32,
}

struct ModuleCtx {
    functions: HashMap<String, FuncInfo>,
    globals: HashMap<String, (u32, ValType)>,
}

/// Compile a file AST node into WASM bytes.
pub fn compile(file: &Node) -> Result<Vec<u8>, CompileError> {
    assert_eq!(file.kind, "file");

    let mut module = Module::new();
    let mut module_ctx = ModuleCtx {
        functions: HashMap::new(),
        globals: HashMap::new(),
    };

    let items = file.child_nodes("items");

    // First pass: register all function signatures so we can handle forward references.
    let mut func_index: u32 = 0;
    for item in items {
        match item.kind {
            "fn" => {
                let name = item.child_token("name").unwrap().text.clone();
                let params: Vec<ValType> = item
                    .child_nodes("params")
                    .iter()
                    .map(|p| resolve_type(p.child_node("type").unwrap()))
                    .collect::<Result<_, _>>()?;
                let ret = if let Some(rt) = item.child_node("return_type") {
                    resolve_type(rt)?
                } else {
                    ValType::Void
                };

                let wasm_params: Vec<u8> = params.iter().filter_map(|t| t.to_wasm()).collect();
                let wasm_results: Vec<u8> = ret.to_wasm().into_iter().collect();

                let type_idx = module.add_type(FuncType {
                    params: wasm_params,
                    results: wasm_results,
                });
                module.functions.push(type_idx);

                module_ctx.functions.insert(
                    name.clone(),
                    FuncInfo {
                        index: func_index,
                        params: params.clone(),
                        ret,
                    },
                );

                // Export all public functions (for now, export everything).
                module.exports.push(wasm::Export {
                    name: name.clone(),
                    kind: wasm::EXPORT_FUNC,
                    index: func_index,
                });

                func_index += 1;
            }
            "const" | "static" => {
                // Handle globals
                let name = item.child_token("name").unwrap().text.clone();
                let ty = resolve_type(item.child_node("type").unwrap())?;
                let wasm_type = ty
                    .to_wasm()
                    .ok_or_else(|| CompileError("void global".into()))?;
                let is_mut = item.kind == "static";
                let init = compile_const_init(item.child_node("value").unwrap())?;
                let global_idx = module.globals.len() as u32;
                module.globals.push(wasm::Global {
                    val_type: wasm_type,
                    mutable: is_mut,
                    init,
                });
                module_ctx.globals.insert(name, (global_idx, ty));
            }
            _ => {}
        }
    }

    // Second pass: compile function bodies.
    for item in items {
        if item.kind == "fn" {
            let body = compile_function(item, &module_ctx)?;
            module.bodies.push(body);
        }
    }

    // Add memory (1 page = 64KB) if any data or for general use.
    module.memory = Some((1, None));
    module.exports.push(wasm::Export {
        name: "memory".into(),
        kind: wasm::EXPORT_MEMORY,
        index: 0,
    });

    Ok(module.encode())
}

fn resolve_type(type_node: &Node) -> Result<ValType, CompileError> {
    match type_node.kind {
        "named_type" => {
            let name = type_node.child_token("name").unwrap().text.as_str();
            ValType::from_name(name)
        }
        _ => Err(CompileError(format!(
            "unsupported type node: {}",
            type_node.kind
        ))),
    }
}

fn compile_const_init(value: &Node) -> Result<Vec<u8>, CompileError> {
    match value.kind {
        "literal" => {
            let tok = value.child_token("value").unwrap();
            match &tok.kind {
                TokenKind::IntLiteral(v) => Ok(wasm::init_i32(*v as i32)),
                TokenKind::FloatLiteral(v) => Ok(wasm::init_f64(*v)),
                TokenKind::BoolLiteral(b) => Ok(wasm::init_i32(if *b { 1 } else { 0 })),
                TokenKind::True => Ok(wasm::init_i32(1)),
                TokenKind::False => Ok(wasm::init_i32(0)),
                _ => Err(CompileError("unsupported const initializer".into())),
            }
        }
        _ => Err(CompileError("const init must be a literal".into())),
    }
}

fn compile_function(func: &Node, module_ctx: &ModuleCtx) -> Result<FuncBody, CompileError> {
    let name = func.child_token("name").unwrap().text.clone();
    let info = module_ctx.functions.get(&name).unwrap();

    let mut fc = FuncCompiler {
        code: Vec::new(),
        locals: HashMap::new(),
        local_count: 0,
        local_decls: Vec::new(),
        ret_type: info.ret,
        module_ctx,
        loop_depth: 0,
    };

    // Register parameters as locals.
    let params = func.child_nodes("params");
    for (i, param) in params.iter().enumerate() {
        let param_name = param.child_token("name").unwrap().text.clone();
        let param_type = info.params[i];
        fc.locals.insert(
            param_name,
            LocalVar {
                index: fc.local_count,
                val_type: param_type,
            },
        );
        fc.local_count += 1;
    }

    // Compile function body.
    let body = func.child_node("body").unwrap();
    fc.compile_block(body)?;

    // Ensure function ends with `end`.
    fc.code.push(op::END);

    Ok(FuncBody {
        locals: fc.local_decls.clone(),
        code: fc.code,
    })
}

impl<'a> FuncCompiler<'a> {
    fn alloc_local(&mut self, ty: ValType) -> u32 {
        let wasm_type = ty.to_wasm().unwrap_or(wasm::TYPE_I32);
        let idx = self.local_count;
        self.local_count += 1;
        self.local_decls.push(LocalDecl {
            count: 1,
            val_type: wasm_type,
        });
        idx
    }

    fn compile_block(&mut self, block: &Node) -> Result<(), CompileError> {
        let stmts = block.child_nodes("stmts");
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Node) -> Result<(), CompileError> {
        match stmt.kind {
            "let" => self.compile_let(stmt),
            "return" => self.compile_return(stmt),
            "expr_stmt" => {
                let expr = stmt.child_node("expr").unwrap();
                let ty = self.compile_expr(expr)?;
                // Drop the value if the expression produces one.
                if ty != ValType::Void {
                    self.code.push(op::DROP);
                }
                Ok(())
            }
            "tail_expr" => {
                let expr = stmt.child_node("expr").unwrap();
                self.compile_expr(expr)?;
                // Leave value on stack as implicit return.
                Ok(())
            }
            "assign" => self.compile_assign(stmt),
            "if" => {
                self.compile_if(stmt)?;
                Ok(())
            }
            "while" => self.compile_while(stmt),
            "loop" => self.compile_loop(stmt),
            "break" => {
                // Break out of the current loop's block.
                // In our encoding: loop { block { ... br 1 to exit ... } }
                // br 1 breaks out of the outer block wrapping the loop.
                self.code.push(op::BR);
                wasm::encode_u32(&mut self.code, 1);
                Ok(())
            }
            "continue" => {
                // Continue to the top of the loop.
                // br 0 branches to the loop header.
                self.code.push(op::BR);
                wasm::encode_u32(&mut self.code, 0);
                Ok(())
            }
            _ => Err(CompileError(format!("unsupported stmt: {}", stmt.kind))),
        }
    }

    fn compile_let(&mut self, stmt: &Node) -> Result<(), CompileError> {
        let name = stmt.child_token("name").unwrap().text.clone();
        let ty = if let Some(type_node) = stmt.child_node("type") {
            resolve_type(type_node)?
        } else {
            ValType::I32 // default
        };

        let idx = self.alloc_local(ty);
        self.locals.insert(name, LocalVar { index: idx, val_type: ty });

        if let Some(init) = stmt.child_node("init") {
            self.compile_expr(init)?;
            self.code.push(op::LOCAL_SET);
            wasm::encode_u32(&mut self.code, idx);
        }

        Ok(())
    }

    fn compile_return(&mut self, stmt: &Node) -> Result<(), CompileError> {
        if let Some(value) = stmt.child_node("value") {
            self.compile_expr(value)?;
        }
        self.code.push(op::RETURN);
        Ok(())
    }

    fn compile_assign(&mut self, stmt: &Node) -> Result<(), CompileError> {
        let target = stmt.child_node("target").unwrap();
        let op_tok = stmt.child_token("op").unwrap();
        let value = stmt.child_node("value").unwrap();

        match target.kind {
            "ident" => {
                let name = target.child_token("name").unwrap().text.as_str();

                // Check for compound assignment.
                match &op_tok.kind {
                    TokenKind::Eq => {
                        self.compile_expr(value)?;
                    }
                    TokenKind::PlusEq
                    | TokenKind::MinusEq
                    | TokenKind::StarEq
                    | TokenKind::SlashEq => {
                        // Load current value, compute, store.
                        self.compile_expr(target)?;
                        self.compile_expr(value)?;
                        let var_ty = self.get_var_type(name)?;
                        self.emit_arith_op(&op_tok.kind, var_ty)?;
                    }
                    _ => {
                        return Err(CompileError(format!(
                            "unsupported assignment operator: {:?}",
                            op_tok.kind
                        )));
                    }
                }

                // Store to local or global.
                if let Some(local) = self.locals.get(name) {
                    let idx = local.index;
                    self.code.push(op::LOCAL_SET);
                    wasm::encode_u32(&mut self.code, idx);
                } else if let Some((gidx, _)) = self.module_ctx.globals.get(name) {
                    self.code.push(op::GLOBAL_SET);
                    wasm::encode_u32(&mut self.code, *gidx);
                } else {
                    return Err(CompileError(format!("undefined variable: {}", name)));
                }
            }
            _ => {
                return Err(CompileError(format!(
                    "unsupported assignment target: {}",
                    target.kind
                )));
            }
        }

        Ok(())
    }

    fn compile_while(&mut self, stmt: &Node) -> Result<(), CompileError> {
        let cond = stmt.child_node("cond").unwrap();
        let body = stmt.child_node("body").unwrap();

        // block {           ; label 1 (break target)
        //   loop {          ; label 0 (continue target)
        //     <cond>
        //     i32.eqz
        //     br_if 1       ; break out of block if cond is false
        //     <body>
        //     br 0          ; continue to loop header
        //   }
        // }
        self.code.push(op::BLOCK);
        self.code.push(wasm::BLOCK_VOID);
        self.code.push(op::LOOP);
        self.code.push(wasm::BLOCK_VOID);

        self.loop_depth += 1;

        self.compile_expr(cond)?;
        self.code.push(op::I32_EQZ);
        self.code.push(op::BR_IF);
        wasm::encode_u32(&mut self.code, 1);

        self.compile_block(body)?;

        self.code.push(op::BR);
        wasm::encode_u32(&mut self.code, 0);

        self.loop_depth -= 1;

        self.code.push(op::END); // end loop
        self.code.push(op::END); // end block

        Ok(())
    }

    fn compile_loop(&mut self, stmt: &Node) -> Result<(), CompileError> {
        let body = stmt.child_node("body").unwrap();

        // block {           ; label 1 (break target)
        //   loop {          ; label 0 (continue target)
        //     <body>
        //     br 0          ; continue to loop header
        //   }
        // }
        self.code.push(op::BLOCK);
        self.code.push(wasm::BLOCK_VOID);
        self.code.push(op::LOOP);
        self.code.push(wasm::BLOCK_VOID);

        self.loop_depth += 1;
        self.compile_block(body)?;
        self.loop_depth -= 1;

        self.code.push(op::BR);
        wasm::encode_u32(&mut self.code, 0);

        self.code.push(op::END); // end loop
        self.code.push(op::END); // end block

        Ok(())
    }

    fn compile_if(&mut self, node: &Node) -> Result<ValType, CompileError> {
        let cond = node.child_node("cond").unwrap();
        let then_block = node.child_node("then").unwrap();

        self.compile_expr(cond)?;

        let has_else = node.has_child("else");

        if has_else {
            // if-else can produce a value
            // For now, use void block type
            self.code.push(op::IF);
            self.code.push(wasm::BLOCK_VOID);
            self.compile_block(then_block)?;
            self.code.push(op::ELSE);
            let else_node = node.child_node("else").unwrap();
            if else_node.kind == "if" {
                self.compile_if(else_node)?;
            } else {
                self.compile_block(else_node)?;
            }
            self.code.push(op::END);
        } else {
            self.code.push(op::IF);
            self.code.push(wasm::BLOCK_VOID);
            self.compile_block(then_block)?;
            self.code.push(op::END);
        }

        Ok(ValType::Void)
    }

    fn compile_expr(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        match expr.kind {
            "literal" => self.compile_literal(expr),
            "ident" => self.compile_ident(expr),
            "binary" => self.compile_binary(expr),
            "unary" => self.compile_unary(expr),
            "call" => self.compile_call(expr),
            "if" => self.compile_if(expr),
            "block" => {
                self.compile_block(expr)?;
                Ok(ValType::Void)
            }
            _ => Err(CompileError(format!("unsupported expr: {}", expr.kind))),
        }
    }

    fn compile_literal(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        let tok = expr.child_token("value").unwrap();
        match &tok.kind {
            TokenKind::IntLiteral(v) => {
                // Determine if i32 or i64 from suffix or default to i32.
                if tok.text.ends_with("i64") || *v > u32::MAX as u64 {
                    self.code.push(op::I64_CONST);
                    wasm::encode_i64(&mut self.code, *v as i64);
                    Ok(ValType::I64)
                } else {
                    self.code.push(op::I32_CONST);
                    wasm::encode_i32(&mut self.code, *v as i32);
                    Ok(ValType::I32)
                }
            }
            TokenKind::FloatLiteral(v) => {
                self.code.push(op::F64_CONST);
                self.code.extend_from_slice(&v.to_le_bytes());
                Ok(ValType::F64)
            }
            TokenKind::BoolLiteral(b) => {
                self.code.push(op::I32_CONST);
                wasm::encode_i32(&mut self.code, if *b { 1 } else { 0 });
                Ok(ValType::Bool)
            }
            TokenKind::True => {
                self.code.push(op::I32_CONST);
                wasm::encode_i32(&mut self.code, 1);
                Ok(ValType::Bool)
            }
            TokenKind::False => {
                self.code.push(op::I32_CONST);
                wasm::encode_i32(&mut self.code, 0);
                Ok(ValType::Bool)
            }
            _ => Err(CompileError(format!(
                "unsupported literal: {:?}",
                tok.kind
            ))),
        }
    }

    fn compile_ident(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        let name = expr.child_token("name").unwrap().text.as_str();

        if let Some(local) = self.locals.get(name) {
            let idx = local.index;
            let ty = local.val_type;
            self.code.push(op::LOCAL_GET);
            wasm::encode_u32(&mut self.code, idx);
            Ok(ty)
        } else if let Some((gidx, ty)) = self.module_ctx.globals.get(name) {
            self.code.push(op::GLOBAL_GET);
            wasm::encode_u32(&mut self.code, *gidx);
            Ok(*ty)
        } else {
            Err(CompileError(format!("undefined variable: {}", name)))
        }
    }

    fn compile_binary(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        let lhs = expr.child_node("lhs").unwrap();
        let rhs = expr.child_node("rhs").unwrap();
        let op_tok = expr.child_token("op").unwrap();

        let lhs_ty = self.compile_expr(lhs)?;
        let rhs_ty = self.compile_expr(rhs)?;

        // Type coercion: if one side is f64 and the other i32, convert i32 to f64.
        let ty = match (lhs_ty, rhs_ty) {
            (ValType::F64, ValType::F64) => ValType::F64,
            (ValType::I64, ValType::I64) => ValType::I64,
            (ValType::I32, ValType::I32) | (ValType::Bool, ValType::Bool) => ValType::I32,
            (ValType::I32, ValType::I64) | (ValType::I64, ValType::I32) => ValType::I64,
            (ValType::Bool, ValType::I32) | (ValType::I32, ValType::Bool) => ValType::I32,
            _ => lhs_ty,
        };

        // Emit the operator.
        let is_comparison = matches!(
            op_tok.kind,
            TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
        );

        match ty {
            ValType::I32 | ValType::Bool => match &op_tok.kind {
                TokenKind::Plus => self.code.push(op::I32_ADD),
                TokenKind::Minus => self.code.push(op::I32_SUB),
                TokenKind::Star => self.code.push(op::I32_MUL),
                TokenKind::Slash => self.code.push(op::I32_DIV_S),
                TokenKind::Percent => self.code.push(op::I32_REM_S),
                TokenKind::Amp => self.code.push(op::I32_AND),
                TokenKind::Pipe => self.code.push(op::I32_OR),
                TokenKind::Caret => self.code.push(op::I32_XOR),
                TokenKind::Shl => self.code.push(op::I32_SHL),
                TokenKind::Shr => self.code.push(op::I32_SHR_S),
                TokenKind::EqEq => self.code.push(op::I32_EQ),
                TokenKind::NotEq => self.code.push(op::I32_NE),
                TokenKind::Lt => self.code.push(op::I32_LT_S),
                TokenKind::Gt => self.code.push(op::I32_GT_S),
                TokenKind::LtEq => self.code.push(op::I32_LE_S),
                TokenKind::GtEq => self.code.push(op::I32_GE_S),
                TokenKind::AmpAmp => self.code.push(op::I32_AND),
                TokenKind::PipePipe => self.code.push(op::I32_OR),
                _ => {
                    return Err(CompileError(format!(
                        "unsupported i32 operator: {:?}",
                        op_tok.kind
                    )));
                }
            },
            ValType::I64 => match &op_tok.kind {
                TokenKind::Plus => self.code.push(op::I64_ADD),
                TokenKind::Minus => self.code.push(op::I64_SUB),
                TokenKind::Star => self.code.push(op::I64_MUL),
                TokenKind::Slash => self.code.push(op::I64_DIV_S),
                TokenKind::Percent => self.code.push(op::I64_REM_S),
                TokenKind::Amp => self.code.push(op::I64_AND),
                TokenKind::Pipe => self.code.push(op::I64_OR),
                TokenKind::Caret => self.code.push(op::I64_XOR),
                TokenKind::Shl => self.code.push(op::I64_SHL),
                TokenKind::Shr => self.code.push(op::I64_SHR_S),
                TokenKind::EqEq => self.code.push(op::I64_EQ),
                TokenKind::NotEq => self.code.push(op::I64_NE),
                TokenKind::Lt => self.code.push(op::I64_LT_S),
                TokenKind::Gt => self.code.push(op::I64_GT_S),
                TokenKind::LtEq => self.code.push(op::I64_LE_S),
                TokenKind::GtEq => self.code.push(op::I64_GE_S),
                _ => {
                    return Err(CompileError(format!(
                        "unsupported i64 operator: {:?}",
                        op_tok.kind
                    )));
                }
            },
            ValType::F64 => match &op_tok.kind {
                TokenKind::Plus => self.code.push(op::F64_ADD),
                TokenKind::Minus => self.code.push(op::F64_SUB),
                TokenKind::Star => self.code.push(op::F64_MUL),
                TokenKind::Slash => self.code.push(op::F64_DIV),
                TokenKind::EqEq => self.code.push(op::F64_EQ),
                TokenKind::NotEq => self.code.push(op::F64_NE),
                TokenKind::Lt => self.code.push(op::F64_LT),
                TokenKind::Gt => self.code.push(op::F64_GT),
                TokenKind::LtEq => self.code.push(op::F64_LE),
                TokenKind::GtEq => self.code.push(op::F64_GE),
                _ => {
                    return Err(CompileError(format!(
                        "unsupported f64 operator: {:?}",
                        op_tok.kind
                    )));
                }
            },
            _ => {
                return Err(CompileError(format!(
                    "unsupported type for binary op: {:?}",
                    ty
                )));
            }
        }

        if is_comparison {
            Ok(ValType::Bool)
        } else {
            Ok(ty)
        }
    }

    fn compile_unary(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        let op_tok = expr.child_token("op").unwrap();
        let operand = expr.child_node("operand").unwrap();
        let ty = self.compile_expr(operand)?;

        match &op_tok.kind {
            TokenKind::Minus => match ty {
                ValType::I32 => {
                    // 0 - x
                    // We need to put 0 before the operand value.
                    // Simpler: use (0 - x) pattern. But operand is already on stack.
                    // Let's use a local to save and reload.
                    let tmp = self.alloc_local(ty);
                    self.code.push(op::LOCAL_SET);
                    wasm::encode_u32(&mut self.code, tmp);
                    self.code.push(op::I32_CONST);
                    wasm::encode_i32(&mut self.code, 0);
                    self.code.push(op::LOCAL_GET);
                    wasm::encode_u32(&mut self.code, tmp);
                    self.code.push(op::I32_SUB);
                    Ok(ValType::I32)
                }
                ValType::I64 => {
                    let tmp = self.alloc_local(ty);
                    self.code.push(op::LOCAL_SET);
                    wasm::encode_u32(&mut self.code, tmp);
                    self.code.push(op::I64_CONST);
                    wasm::encode_i64(&mut self.code, 0);
                    self.code.push(op::LOCAL_GET);
                    wasm::encode_u32(&mut self.code, tmp);
                    self.code.push(op::I64_SUB);
                    Ok(ValType::I64)
                }
                ValType::F64 => {
                    self.code.push(op::F64_NEG);
                    Ok(ValType::F64)
                }
                _ => Err(CompileError("cannot negate this type".into())),
            },
            TokenKind::Bang => {
                self.code.push(op::I32_EQZ);
                Ok(ValType::Bool)
            }
            _ => Err(CompileError(format!(
                "unsupported unary op: {:?}",
                op_tok.kind
            ))),
        }
    }

    fn compile_call(&mut self, expr: &Node) -> Result<ValType, CompileError> {
        let callee = expr.child_node("callee").unwrap();
        let args = expr.child_nodes("args");

        let func_name = match callee.kind {
            "ident" => callee.child_token("name").unwrap().text.as_str(),
            _ => {
                return Err(CompileError(
                    "only direct function calls are supported".into(),
                ));
            }
        };

        let info = self
            .module_ctx
            .functions
            .get(func_name)
            .ok_or_else(|| CompileError(format!("undefined function: {}", func_name)))?
            .clone();

        if args.len() != info.params.len() {
            return Err(CompileError(format!(
                "function '{}' expects {} args, got {}",
                func_name,
                info.params.len(),
                args.len()
            )));
        }

        for arg in args {
            self.compile_expr(arg)?;
        }

        self.code.push(op::CALL);
        wasm::encode_u32(&mut self.code, info.index);

        Ok(info.ret)
    }

    fn get_var_type(&self, name: &str) -> Result<ValType, CompileError> {
        if let Some(local) = self.locals.get(name) {
            Ok(local.val_type)
        } else if let Some((_, ty)) = self.module_ctx.globals.get(name) {
            Ok(*ty)
        } else {
            Err(CompileError(format!("undefined variable: {}", name)))
        }
    }

    fn emit_arith_op(&mut self, kind: &TokenKind, ty: ValType) -> Result<(), CompileError> {
        match ty {
            ValType::I32 | ValType::Bool => match kind {
                TokenKind::PlusEq => self.code.push(op::I32_ADD),
                TokenKind::MinusEq => self.code.push(op::I32_SUB),
                TokenKind::StarEq => self.code.push(op::I32_MUL),
                TokenKind::SlashEq => self.code.push(op::I32_DIV_S),
                _ => return Err(CompileError(format!("unsupported compound op: {:?}", kind))),
            },
            ValType::I64 => match kind {
                TokenKind::PlusEq => self.code.push(op::I64_ADD),
                TokenKind::MinusEq => self.code.push(op::I64_SUB),
                TokenKind::StarEq => self.code.push(op::I64_MUL),
                TokenKind::SlashEq => self.code.push(op::I64_DIV_S),
                _ => return Err(CompileError(format!("unsupported compound op: {:?}", kind))),
            },
            ValType::F64 => match kind {
                TokenKind::PlusEq => self.code.push(op::F64_ADD),
                TokenKind::MinusEq => self.code.push(op::F64_SUB),
                TokenKind::StarEq => self.code.push(op::F64_MUL),
                TokenKind::SlashEq => self.code.push(op::F64_DIV),
                _ => return Err(CompileError(format!("unsupported compound op: {:?}", kind))),
            },
            _ => return Err(CompileError(format!("unsupported type for compound op: {:?}", ty))),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::token_tree::build_token_tree;

    fn compile_str(src: &str) -> Vec<u8> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let trees = build_token_tree(tokens).unwrap();
        let ast = Parser::new(&trees).parse_file().unwrap();
        compile(&ast).unwrap()
    }

    #[test]
    fn test_compile_empty_fn() {
        let bytes = compile_str("fn main() {}");
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_return_const() {
        let bytes = compile_str("fn answer() -> i32 { return 42; }");
        assert_eq!(&bytes[0..4], b"\0asm");
        assert!(bytes.len() > 8);
    }

    #[test]
    fn test_compile_add() {
        let bytes = compile_str("fn add(a: i32, b: i32) -> i32 { return a + b; }");
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_let_and_return() {
        let bytes = compile_str(
            "fn f() -> i32 { let x: i32 = 10; let y: i32 = 20; return x + y; }",
        );
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_if() {
        let bytes = compile_str(
            "fn f(x: i32) -> i32 { if x > 0 { return 1; } return 0; }",
        );
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_while() {
        let bytes = compile_str(
            "fn f() -> i32 { let mut i: i32 = 0; while i < 10 { i += 1; } return i; }",
        );
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_function_call() {
        let bytes = compile_str(
            "fn double(x: i32) -> i32 { return x + x; } fn main() -> i32 { return double(21); }",
        );
        assert_eq!(&bytes[0..4], b"\0asm");
    }
}
