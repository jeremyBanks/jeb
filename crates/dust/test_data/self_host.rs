// Self-hosting dust compiler
// Compiles dust source (i32-only subset) to WASM bytecode.
//
// All values are i32. All mutable state lives in linear memory.
// Entry point: compile(src_len: i32) -> i32 (returns output length)
//
// Memory layout:
//   0x00000: Source input (32KB)
//   0x08000: WASM output (32KB)
//   0x10000: Tokens (32KB, up to 2048 tokens, 16 bytes each)
//   0x20000: AST nodes (64KB, 28 bytes each)
//   0x30000: Children array (32KB, 4 bytes each)
//   0x38000: Function table (16KB, 24 bytes each)
//   0x3C000: Local variable table (8KB, 16 bytes each)
//   0x3E000: Global state (256 bytes)
//   0x40000: Section temp buffer (64KB)
//   0x50000: Code temp buffer (32KB)

// ================================================================
// MEMORY LAYOUT CONSTANTS
// ================================================================

fn SRC_BASE() -> i32 { 0 }
fn OUT_BASE() -> i32 { 65536 }
fn TOK_BASE() -> i32 { 131072 }
fn NODE_BASE() -> i32 { 393216 }
fn CHILD_BASE() -> i32 { 589824 }
fn FTAB_BASE() -> i32 { 655360 }
fn LTAB_BASE() -> i32 { 671744 }
fn GLOB() -> i32 { 688128 }
fn SEC_BUF() -> i32 { 696320 }
fn CODE_BUF() -> i32 { 827392 }
fn PSTACK_BASE() -> i32 { 892928 }

fn TOK_SIZE() -> i32 { 16 }
fn NODE_SIZE() -> i32 { 28 }
fn FTAB_ENTRY() -> i32 { 24 }
fn LTAB_ENTRY() -> i32 { 16 }

// ================================================================
// GLOBAL STATE OFFSETS
// ================================================================

fn G_SRC_LEN() -> i32 { 0 }
fn G_TOK_COUNT() -> i32 { 4 }
fn G_NODE_COUNT() -> i32 { 8 }
fn G_CHILD_COUNT() -> i32 { 12 }
fn G_FUNC_COUNT() -> i32 { 16 }
fn G_LOCAL_COUNT() -> i32 { 20 }
fn G_OUT_POS() -> i32 { 24 }
fn G_TOK_POS() -> i32 { 28 }
fn G_CODE_POS() -> i32 { 32 }
fn G_LOOP_DEPTH() -> i32 { 36 }
fn G_SEC_POS() -> i32 { 40 }
fn G_CUR_PARAMS() -> i32 { 44 }
fn G_PSTACK_PTR() -> i32 { 48 }
fn G_LIST_FIRST() -> i32 { 52 }

// ================================================================
// TOKEN KINDS
// ================================================================

fn TK_EOF() -> i32 { 0 }
fn TK_INT() -> i32 { 1 }
fn TK_IDENT() -> i32 { 2 }
fn TK_FN() -> i32 { 3 }
fn TK_LET() -> i32 { 4 }
fn TK_MUT() -> i32 { 5 }
fn TK_IF() -> i32 { 6 }
fn TK_ELSE() -> i32 { 7 }
fn TK_WHILE() -> i32 { 8 }
fn TK_LOOP() -> i32 { 9 }
fn TK_BREAK() -> i32 { 10 }
fn TK_CONTINUE() -> i32 { 11 }
fn TK_RETURN() -> i32 { 12 }
fn TK_I32_KW() -> i32 { 13 }
fn TK_LPAREN() -> i32 { 20 }
fn TK_RPAREN() -> i32 { 21 }
fn TK_LBRACE() -> i32 { 22 }
fn TK_RBRACE() -> i32 { 23 }
fn TK_PLUS() -> i32 { 30 }
fn TK_MINUS() -> i32 { 31 }
fn TK_STAR() -> i32 { 32 }
fn TK_SLASH() -> i32 { 33 }
fn TK_PERCENT() -> i32 { 34 }
fn TK_AMP() -> i32 { 35 }
fn TK_PIPE() -> i32 { 36 }
fn TK_CARET() -> i32 { 37 }
fn TK_SHL() -> i32 { 38 }
fn TK_SHR() -> i32 { 39 }
fn TK_BANG() -> i32 { 40 }
fn TK_AMPAMP() -> i32 { 41 }
fn TK_PIPEPIPE() -> i32 { 42 }
fn TK_EQEQ() -> i32 { 50 }
fn TK_BANGEQ() -> i32 { 51 }
fn TK_LT() -> i32 { 52 }
fn TK_GT() -> i32 { 53 }
fn TK_LTEQ() -> i32 { 54 }
fn TK_GTEQ() -> i32 { 55 }
fn TK_EQ() -> i32 { 60 }
fn TK_PLUSEQ() -> i32 { 61 }
fn TK_MINUSEQ() -> i32 { 62 }
fn TK_STAREQ() -> i32 { 63 }
fn TK_SLASHEQ() -> i32 { 64 }
fn TK_ARROW() -> i32 { 70 }
fn TK_COMMA() -> i32 { 74 }
fn TK_SEMI() -> i32 { 75 }
fn TK_COLON() -> i32 { 76 }

// ================================================================
// NODE KINDS
// ================================================================

fn NK_FILE() -> i32 { 1 }
fn NK_FN() -> i32 { 2 }
fn NK_PARAM() -> i32 { 3 }
fn NK_LET() -> i32 { 4 }
fn NK_RETURN() -> i32 { 5 }
fn NK_IF() -> i32 { 6 }
fn NK_WHILE() -> i32 { 7 }
fn NK_LOOP() -> i32 { 8 }
fn NK_BREAK() -> i32 { 9 }
fn NK_CONTINUE() -> i32 { 10 }
fn NK_BLOCK() -> i32 { 11 }
fn NK_EXPR_STMT() -> i32 { 12 }
fn NK_ASSIGN() -> i32 { 13 }
fn NK_LITERAL() -> i32 { 14 }
fn NK_IDENT() -> i32 { 15 }
fn NK_BINARY() -> i32 { 16 }
fn NK_UNARY() -> i32 { 17 }
fn NK_CALL() -> i32 { 18 }
fn NK_TAIL_EXPR() -> i32 { 19 }

// ================================================================
// GLOBAL STATE ACCESSORS
// ================================================================

fn get_glob(offset: i32) -> i32 { load_i32(GLOB() + offset) }
fn set_glob(offset: i32, val: i32) { store_i32(GLOB() + offset, val); }

fn get_src_len() -> i32 { get_glob(G_SRC_LEN()) }
fn set_src_len(v: i32) { set_glob(G_SRC_LEN(), v); }
fn get_tok_count() -> i32 { get_glob(G_TOK_COUNT()) }
fn set_tok_count(v: i32) { set_glob(G_TOK_COUNT(), v); }
fn get_node_count() -> i32 { get_glob(G_NODE_COUNT()) }
fn set_node_count(v: i32) { set_glob(G_NODE_COUNT(), v); }
fn get_child_count() -> i32 { get_glob(G_CHILD_COUNT()) }
fn set_child_count(v: i32) { set_glob(G_CHILD_COUNT(), v); }
fn get_func_count() -> i32 { get_glob(G_FUNC_COUNT()) }
fn set_func_count(v: i32) { set_glob(G_FUNC_COUNT(), v); }
fn get_local_count() -> i32 { get_glob(G_LOCAL_COUNT()) }
fn set_local_count(v: i32) { set_glob(G_LOCAL_COUNT(), v); }
fn get_out_pos() -> i32 { get_glob(G_OUT_POS()) }
fn set_out_pos(v: i32) { set_glob(G_OUT_POS(), v); }
fn get_tok_pos() -> i32 { get_glob(G_TOK_POS()) }
fn set_tok_pos(v: i32) { set_glob(G_TOK_POS(), v); }
fn get_code_pos() -> i32 { get_glob(G_CODE_POS()) }
fn set_code_pos(v: i32) { set_glob(G_CODE_POS(), v); }
fn get_loop_depth() -> i32 { get_glob(G_LOOP_DEPTH()) }
fn set_loop_depth(v: i32) { set_glob(G_LOOP_DEPTH(), v); }
fn get_sec_pos() -> i32 { get_glob(G_SEC_POS()) }
fn set_sec_pos(v: i32) { set_glob(G_SEC_POS(), v); }
fn get_cur_params() -> i32 { get_glob(G_CUR_PARAMS()) }
fn set_cur_params(v: i32) { set_glob(G_CUR_PARAMS(), v); }
fn get_pstack_ptr() -> i32 { get_glob(G_PSTACK_PTR()) }
fn set_pstack_ptr(v: i32) { set_glob(G_PSTACK_PTR(), v); }
fn get_list_first() -> i32 { get_glob(G_LIST_FIRST()) }

// Begin collecting a child list. Returns the stack position to pass to end_list.
fn begin_list() -> i32 { get_pstack_ptr() }

// Push a node index onto the parse stack (NOT the children array).
fn list_push(node_idx: i32) {
    let mut p: i32 = get_pstack_ptr();
    store_i32(PSTACK_BASE() + p * 4, node_idx);
    set_pstack_ptr(p + 1);
}

// Flush all items since begin_list to the children array contiguously.
// Returns the count. The first_child index is accessible via get_list_first().
fn end_list(start: i32) -> i32 {
    let mut end: i32 = get_pstack_ptr();
    let mut count: i32 = end - start;
    let mut first_child: i32 = get_child_count();
    set_glob(G_LIST_FIRST(), first_child);
    let mut i: i32 = 0;
    while i < count {
        push_child(load_i32(PSTACK_BASE() + (start + i) * 4));
        i += 1;
    }
    set_pstack_ptr(start);
    count
}

// ================================================================
// UTILITY FUNCTIONS
// ================================================================

fn is_alpha(c: i32) -> i32 {
    if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || c == 95 { return 1; }
    0
}

fn is_digit(c: i32) -> i32 {
    if c >= 48 && c <= 57 { return 1; }
    0
}

fn is_alnum(c: i32) -> i32 {
    if is_alpha(c) != 0 || is_digit(c) != 0 { return 1; }
    0
}

fn is_hex_digit(c: i32) -> i32 {
    if is_digit(c) != 0 { return 1; }
    if c >= 65 && c <= 70 { return 1; }
    if c >= 97 && c <= 102 { return 1; }
    0
}

fn hex_val(c: i32) -> i32 {
    if c >= 48 && c <= 57 { return c - 48; }
    if c >= 65 && c <= 70 { return c - 55; }
    if c >= 97 && c <= 102 { return c - 87; }
    0
}

fn is_whitespace(c: i32) -> i32 {
    if c == 32 || c == 9 || c == 10 || c == 13 { return 1; }
    0
}

fn str_eq(a: i32, alen: i32, b: i32, blen: i32) -> i32 {
    if alen != blen { return 0; }
    let mut i: i32 = 0;
    while i < alen {
        if load_byte(a + i) != load_byte(b + i) { return 0; }
        i += 1;
    }
    1
}

fn leb_u_size(v: i32) -> i32 {
    if v < 128 { return 1; }
    if v < 16384 { return 2; }
    if v < 2097152 { return 3; }
    if v < 268435456 { return 4; }
    5
}

// ================================================================
// TOKEN ACCESS
// ================================================================

fn tok_addr(idx: i32) -> i32 { TOK_BASE() + idx * TOK_SIZE() }
fn tok_kind(idx: i32) -> i32 { load_i32(tok_addr(idx)) }
fn tok_start(idx: i32) -> i32 { load_i32(tok_addr(idx) + 4) }
fn tok_end(idx: i32) -> i32 { load_i32(tok_addr(idx) + 8) }
fn tok_value(idx: i32) -> i32 { load_i32(tok_addr(idx) + 12) }
fn tok_len(idx: i32) -> i32 { tok_end(idx) - tok_start(idx) }

fn set_token(idx: i32, kind: i32, start: i32, end: i32, value: i32) {
    let mut a: i32 = tok_addr(idx);
    store_i32(a, kind);
    store_i32(a + 4, start);
    store_i32(a + 8, end);
    store_i32(a + 12, value);
}

// ================================================================
// NODE ACCESS
// ================================================================

fn node_addr(idx: i32) -> i32 { NODE_BASE() + idx * NODE_SIZE() }
fn node_kind(idx: i32) -> i32 { load_i32(node_addr(idx)) }
fn node_d1(idx: i32) -> i32 { load_i32(node_addr(idx) + 4) }
fn node_d2(idx: i32) -> i32 { load_i32(node_addr(idx) + 8) }
fn node_d3(idx: i32) -> i32 { load_i32(node_addr(idx) + 12) }
fn node_d4(idx: i32) -> i32 { load_i32(node_addr(idx) + 16) }
fn node_d5(idx: i32) -> i32 { load_i32(node_addr(idx) + 20) }

fn new_node(kind: i32, d1: i32, d2: i32, d3: i32, d4: i32, d5: i32) -> i32 {
    let mut idx: i32 = get_node_count();
    let mut a: i32 = node_addr(idx);
    store_i32(a, kind);
    store_i32(a + 4, d1);
    store_i32(a + 8, d2);
    store_i32(a + 12, d3);
    store_i32(a + 16, d4);
    store_i32(a + 20, d5);
    set_node_count(idx + 1);
    idx
}

// ================================================================
// CHILDREN ARRAY
// ================================================================

fn child_at(idx: i32) -> i32 { load_i32(CHILD_BASE() + idx * 4) }

fn push_child(node_idx: i32) -> i32 {
    let mut c: i32 = get_child_count();
    store_i32(CHILD_BASE() + c * 4, node_idx);
    set_child_count(c + 1);
    c
}

// ================================================================
// FUNCTION TABLE
// ================================================================

fn ftab_addr(idx: i32) -> i32 { FTAB_BASE() + idx * FTAB_ENTRY() }
fn ftab_name_start(idx: i32) -> i32 { load_i32(ftab_addr(idx)) }
fn ftab_name_len(idx: i32) -> i32 { load_i32(ftab_addr(idx) + 4) }
fn ftab_param_count(idx: i32) -> i32 { load_i32(ftab_addr(idx) + 8) }
fn ftab_has_return(idx: i32) -> i32 { load_i32(ftab_addr(idx) + 12) }
fn ftab_func_index(idx: i32) -> i32 { load_i32(ftab_addr(idx) + 16) }

fn add_func(ns: i32, nl: i32, pc: i32, hr: i32, fi: i32) {
    let mut idx: i32 = get_func_count();
    let mut a: i32 = ftab_addr(idx);
    store_i32(a, ns);
    store_i32(a + 4, nl);
    store_i32(a + 8, pc);
    store_i32(a + 12, hr);
    store_i32(a + 16, fi);
    set_func_count(idx + 1);
}

fn find_func(ns: i32, nl: i32) -> i32 {
    let mut i: i32 = 0;
    let mut count: i32 = get_func_count();
    while i < count {
        if str_eq(ns, nl, ftab_name_start(i), ftab_name_len(i)) != 0 {
            return i;
        }
        i += 1;
    }
    return -1;
}

// ================================================================
// LOCAL VARIABLE TABLE
// ================================================================

fn ltab_addr(idx: i32) -> i32 { LTAB_BASE() + idx * LTAB_ENTRY() }
fn ltab_name_start(idx: i32) -> i32 { load_i32(ltab_addr(idx)) }
fn ltab_name_len(idx: i32) -> i32 { load_i32(ltab_addr(idx) + 4) }
fn ltab_local_index(idx: i32) -> i32 { load_i32(ltab_addr(idx) + 8) }

fn add_local(ns: i32, nl: i32) -> i32 {
    let mut idx: i32 = get_local_count();
    let mut a: i32 = ltab_addr(idx);
    store_i32(a, ns);
    store_i32(a + 4, nl);
    store_i32(a + 8, idx);
    set_local_count(idx + 1);
    idx
}

fn find_local(ns: i32, nl: i32) -> i32 {
    let mut i: i32 = get_local_count() - 1;
    while i >= 0 {
        if str_eq(ns, nl, ltab_name_start(i), ltab_name_len(i)) != 0 {
            return ltab_local_index(i);
        }
        i -= 1;
    }
    return -1;
}

// ================================================================
// LEXER
// ================================================================

fn src_byte(pos: i32) -> i32 { load_byte(SRC_BASE() + pos) }

fn classify_ident(start: i32, len: i32) -> i32 {
    let mut s: i32 = SRC_BASE() + start;
    if len == 2 {
        if load_byte(s) == 102 && load_byte(s + 1) == 110 { return TK_FN(); }
        if load_byte(s) == 105 && load_byte(s + 1) == 102 { return TK_IF(); }
    }
    if len == 3 {
        if load_byte(s) == 108 && load_byte(s + 1) == 101 && load_byte(s + 2) == 116 { return TK_LET(); }
        if load_byte(s) == 109 && load_byte(s + 1) == 117 && load_byte(s + 2) == 116 { return TK_MUT(); }
        if load_byte(s) == 105 && load_byte(s + 1) == 51 && load_byte(s + 2) == 50 { return TK_I32_KW(); }
    }
    if len == 4 {
        if load_byte(s) == 101 && load_byte(s + 1) == 108 && load_byte(s + 2) == 115 && load_byte(s + 3) == 101 { return TK_ELSE(); }
        if load_byte(s) == 108 && load_byte(s + 1) == 111 && load_byte(s + 2) == 111 && load_byte(s + 3) == 112 { return TK_LOOP(); }
    }
    if len == 5 {
        if load_byte(s) == 119 && load_byte(s + 1) == 104 && load_byte(s + 2) == 105 && load_byte(s + 3) == 108 && load_byte(s + 4) == 101 { return TK_WHILE(); }
        if load_byte(s) == 98 && load_byte(s + 1) == 114 && load_byte(s + 2) == 101 && load_byte(s + 3) == 97 && load_byte(s + 4) == 107 { return TK_BREAK(); }
    }
    if len == 6 {
        if load_byte(s) == 114 && load_byte(s + 1) == 101 && load_byte(s + 2) == 116 && load_byte(s + 3) == 117 && load_byte(s + 4) == 114 && load_byte(s + 5) == 110 { return TK_RETURN(); }
    }
    if len == 8 {
        if load_byte(s) == 99 && load_byte(s + 1) == 111 && load_byte(s + 2) == 110 && load_byte(s + 3) == 116 && load_byte(s + 4) == 105 && load_byte(s + 5) == 110 && load_byte(s + 6) == 117 && load_byte(s + 7) == 101 { return TK_CONTINUE(); }
    }
    TK_IDENT()
}

fn lex_all() {
    let mut pos: i32 = 0;
    let mut sl: i32 = get_src_len();
    let mut ti: i32 = 0;

    while pos < sl {
        // Skip whitespace
        while pos < sl && is_whitespace(src_byte(pos)) != 0 {
            pos += 1;
        }
        if pos >= sl { break; }

        let mut c: i32 = src_byte(pos);

        // Line comments
        if c == 47 && pos + 1 < sl && src_byte(pos + 1) == 47 {
            pos += 2;
            while pos < sl && src_byte(pos) != 10 {
                pos += 1;
            }
            continue;
        }

        // Numbers
        if is_digit(c) != 0 {
            let mut start: i32 = pos;
            let mut val: i32 = 0;
            if c == 48 && pos + 1 < sl && (src_byte(pos + 1) == 120 || src_byte(pos + 1) == 88) {
                pos += 2;
                while pos < sl && is_hex_digit(src_byte(pos)) != 0 {
                    val = val * 16 + hex_val(src_byte(pos));
                    pos += 1;
                }
            } else {
                while pos < sl && is_digit(src_byte(pos)) != 0 {
                    val = val * 10 + (src_byte(pos) - 48);
                    pos += 1;
                }
            }
            set_token(ti, TK_INT(), start, pos, val);
            ti += 1;
            continue;
        }

        // Identifiers and keywords
        if is_alpha(c) != 0 {
            let mut start: i32 = pos;
            while pos < sl && is_alnum(src_byte(pos)) != 0 {
                pos += 1;
            }
            let mut kind: i32 = classify_ident(start, pos - start);
            set_token(ti, kind, start, pos, 0);
            ti += 1;
            continue;
        }

        // Single-char punctuation
        let mut start: i32 = pos;
        if c == 40 { set_token(ti, TK_LPAREN(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 41 { set_token(ti, TK_RPAREN(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 123 { set_token(ti, TK_LBRACE(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 125 { set_token(ti, TK_RBRACE(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 44 { set_token(ti, TK_COMMA(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 59 { set_token(ti, TK_SEMI(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 58 { set_token(ti, TK_COLON(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 94 { set_token(ti, TK_CARET(), start, pos + 1, 0); ti += 1; pos += 1; continue; }
        if c == 37 { set_token(ti, TK_PERCENT(), start, pos + 1, 0); ti += 1; pos += 1; continue; }

        // Two-char operators
        let mut c2: i32 = 0;
        if pos + 1 < sl { c2 = src_byte(pos + 1); }

        if c == 43 {
            if c2 == 61 { set_token(ti, TK_PLUSEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_PLUS(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 45 {
            if c2 == 61 { set_token(ti, TK_MINUSEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            if c2 == 62 { set_token(ti, TK_ARROW(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_MINUS(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 42 {
            if c2 == 61 { set_token(ti, TK_STAREQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_STAR(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 47 {
            if c2 == 61 { set_token(ti, TK_SLASHEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_SLASH(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 61 {
            if c2 == 61 { set_token(ti, TK_EQEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_EQ(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 33 {
            if c2 == 61 { set_token(ti, TK_BANGEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_BANG(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 60 {
            if c2 == 61 { set_token(ti, TK_LTEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            if c2 == 60 { set_token(ti, TK_SHL(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_LT(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 62 {
            if c2 == 61 { set_token(ti, TK_GTEQ(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            if c2 == 62 { set_token(ti, TK_SHR(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_GT(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 38 {
            if c2 == 38 { set_token(ti, TK_AMPAMP(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_AMP(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }
        if c == 124 {
            if c2 == 124 { set_token(ti, TK_PIPEPIPE(), start, pos + 2, 0); ti += 1; pos += 2; continue; }
            set_token(ti, TK_PIPE(), start, pos + 1, 0); ti += 1; pos += 1; continue;
        }

        // Unknown char, skip
        pos += 1;
    }

    set_token(ti, TK_EOF(), pos, pos, 0);
    set_tok_count(ti + 1);
}

// ================================================================
// PARSER
// ================================================================

fn peek_kind() -> i32 { tok_kind(get_tok_pos()) }

fn advance() -> i32 {
    let mut p: i32 = get_tok_pos();
    set_tok_pos(p + 1);
    p
}

fn expect(kind: i32) -> i32 {
    if peek_kind() != kind { unreachable(); }
    advance()
}

fn eat(kind: i32) -> i32 {
    if peek_kind() == kind { advance(); return 1; }
    0
}

fn parse_file() -> i32 {
    let mut mark: i32 = begin_list();
    while peek_kind() != TK_EOF() {
        let mut item: i32 = parse_fn_def();
        list_push(item);
    }
    let mut count: i32 = end_list(mark);
    let mut first_child: i32 = get_list_first();
    new_node(NK_FILE(), first_child, count, 0, 0, 0)
}

fn parse_fn_def() -> i32 {
    expect(TK_FN());
    let mut name_tok: i32 = expect(TK_IDENT());
    expect(TK_LPAREN());

    let mut param_mark: i32 = begin_list();
    let mut param_count: i32 = 0;
    while peek_kind() != TK_RPAREN() {
        if param_count > 0 { expect(TK_COMMA()); }
        let mut pname: i32 = expect(TK_IDENT());
        expect(TK_COLON());
        expect(TK_I32_KW());
        let mut pnode: i32 = new_node(NK_PARAM(), pname, 0, 0, 0, 0);
        list_push(pnode);
        param_count += 1;
    }
    expect(TK_RPAREN());
    param_count = end_list(param_mark);
    let mut first_param: i32 = get_list_first();

    let mut has_return: i32 = 0;
    if eat(TK_ARROW()) != 0 {
        expect(TK_I32_KW());
        has_return = 1;
    }

    let mut body: i32 = parse_block();
    new_node(NK_FN(), name_tok, first_param, param_count, has_return, body)
}

fn parse_block() -> i32 {
    expect(TK_LBRACE());
    let mut mark: i32 = begin_list();
    while peek_kind() != TK_RBRACE() {
        let mut stmt: i32 = parse_stmt();
        list_push(stmt);
    }
    expect(TK_RBRACE());
    let mut count: i32 = end_list(mark);
    let mut first_stmt: i32 = get_list_first();
    new_node(NK_BLOCK(), first_stmt, count, 0, 0, 0)
}

fn parse_stmt() -> i32 {
    let mut k: i32 = peek_kind();
    if k == TK_LET() { return parse_let(); }
    if k == TK_RETURN() { return parse_return(); }
    if k == TK_IF() { return parse_if(); }
    if k == TK_WHILE() { return parse_while_stmt(); }
    if k == TK_LOOP() { return parse_loop_stmt(); }
    if k == TK_BREAK() {
        advance();
        expect(TK_SEMI());
        return new_node(NK_BREAK(), 0, 0, 0, 0, 0);
    }
    if k == TK_CONTINUE() {
        advance();
        expect(TK_SEMI());
        return new_node(NK_CONTINUE(), 0, 0, 0, 0, 0);
    }
    parse_expr_or_assign()
}

fn parse_let() -> i32 {
    expect(TK_LET());
    eat(TK_MUT());
    let mut name_tok: i32 = expect(TK_IDENT());
    if eat(TK_COLON()) != 0 {
        expect(TK_I32_KW());
    }
    let mut init: i32 = -1;
    if eat(TK_EQ()) != 0 {
        init = parse_expr(0);
    }
    expect(TK_SEMI());
    new_node(NK_LET(), name_tok, init, 0, 0, 0)
}

fn parse_return() -> i32 {
    expect(TK_RETURN());
    let mut val: i32 = -1;
    if peek_kind() != TK_SEMI() {
        val = parse_expr(0);
    }
    expect(TK_SEMI());
    new_node(NK_RETURN(), val, 0, 0, 0, 0)
}

fn parse_if() -> i32 {
    expect(TK_IF());
    let mut cond: i32 = parse_expr(0);
    let mut then_blk: i32 = parse_block();
    let mut else_node: i32 = -1;
    if eat(TK_ELSE()) != 0 {
        if peek_kind() == TK_IF() {
            else_node = parse_if();
        } else {
            else_node = parse_block();
        }
    }
    new_node(NK_IF(), cond, then_blk, else_node, 0, 0)
}

fn parse_while_stmt() -> i32 {
    expect(TK_WHILE());
    let mut cond: i32 = parse_expr(0);
    let mut body: i32 = parse_block();
    new_node(NK_WHILE(), cond, body, 0, 0, 0)
}

fn parse_loop_stmt() -> i32 {
    expect(TK_LOOP());
    let mut body: i32 = parse_block();
    new_node(NK_LOOP(), body, 0, 0, 0, 0)
}

fn parse_expr_or_assign() -> i32 {
    let mut expr: i32 = parse_expr(0);
    let mut k: i32 = peek_kind();
    if k == TK_EQ() || k == TK_PLUSEQ() || k == TK_MINUSEQ() || k == TK_STAREQ() || k == TK_SLASHEQ() {
        let mut op_kind: i32 = k;
        advance();
        let mut rhs: i32 = parse_expr(0);
        expect(TK_SEMI());
        return new_node(NK_ASSIGN(), expr, op_kind, rhs, 0, 0);
    }
    if eat(TK_SEMI()) != 0 {
        return new_node(NK_EXPR_STMT(), expr, 0, 0, 0, 0);
    }
    new_node(NK_TAIL_EXPR(), expr, 0, 0, 0, 0)
}

fn get_prec(tk: i32) -> i32 {
    if tk == TK_PIPEPIPE() { return 1; }
    if tk == TK_AMPAMP() { return 2; }
    if tk == TK_EQEQ() || tk == TK_BANGEQ() || tk == TK_LT() || tk == TK_GT() || tk == TK_LTEQ() || tk == TK_GTEQ() { return 3; }
    if tk == TK_PIPE() { return 4; }
    if tk == TK_CARET() { return 5; }
    if tk == TK_AMP() { return 6; }
    if tk == TK_SHL() || tk == TK_SHR() { return 7; }
    if tk == TK_PLUS() || tk == TK_MINUS() { return 8; }
    if tk == TK_STAR() || tk == TK_SLASH() || tk == TK_PERCENT() { return 9; }
    0
}

fn parse_expr(min_prec: i32) -> i32 {
    let mut lhs: i32 = parse_unary();
    loop {
        let mut k: i32 = peek_kind();
        let mut prec: i32 = get_prec(k);
        if prec == 0 || prec < min_prec { break; }
        let mut op_tok: i32 = advance();
        let mut rhs: i32 = parse_expr(prec + 1);
        lhs = new_node(NK_BINARY(), lhs, tok_kind(op_tok), rhs, 0, 0);
    }
    lhs
}

fn parse_unary() -> i32 {
    let mut k: i32 = peek_kind();
    if k == TK_MINUS() || k == TK_BANG() {
        let mut op_tok: i32 = advance();
        let mut operand: i32 = parse_unary();
        return new_node(NK_UNARY(), tok_kind(op_tok), operand, 0, 0, 0);
    }
    parse_primary()
}

fn parse_primary() -> i32 {
    let mut k: i32 = peek_kind();
    if k == TK_INT() {
        let mut tok: i32 = advance();
        return new_node(NK_LITERAL(), tok_value(tok), 0, 0, 0, 0);
    }
    if k == TK_IDENT() {
        let mut tok: i32 = advance();
        if peek_kind() == TK_LPAREN() {
            return parse_call(tok);
        }
        return new_node(NK_IDENT(), tok, 0, 0, 0, 0);
    }
    if k == TK_LPAREN() {
        advance();
        let mut expr: i32 = parse_expr(0);
        expect(TK_RPAREN());
        return expr;
    }
    if k == TK_IF() {
        return parse_if();
    }
    if k == TK_LBRACE() {
        return parse_block();
    }
    unreachable();
    0
}

fn parse_call(callee_tok: i32) -> i32 {
    expect(TK_LPAREN());
    let mut mark: i32 = begin_list();
    let mut arg_count: i32 = 0;
    while peek_kind() != TK_RPAREN() {
        if arg_count > 0 { expect(TK_COMMA()); }
        let mut arg: i32 = parse_expr(0);
        list_push(arg);
        arg_count += 1;
    }
    expect(TK_RPAREN());
    arg_count = end_list(mark);
    let mut first_arg: i32 = get_list_first();
    new_node(NK_CALL(), callee_tok, first_arg, arg_count, 0, 0)
}

// ================================================================
// OUTPUT HELPERS (write to WASM output buffer)
// ================================================================

fn out_byte(b: i32) {
    let mut p: i32 = get_out_pos();
    store_byte(OUT_BASE() + p, b);
    set_out_pos(p + 1);
}

fn out_leb_u(v: i32) {
    let mut val: i32 = v;
    loop {
        let mut b: i32 = val & 127;
        val = val >> 7;
        if val != 0 { b = b | 128; }
        out_byte(b);
        if val == 0 { break; }
    }
}

// ================================================================
// SECTION BUFFER HELPERS (write to temp section buffer)
// ================================================================

fn sec_byte(b: i32) {
    let mut p: i32 = get_sec_pos();
    store_byte(SEC_BUF() + p, b);
    set_sec_pos(p + 1);
}

fn sec_leb_u(v: i32) {
    let mut val: i32 = v;
    loop {
        let mut b: i32 = val & 127;
        val = val >> 7;
        if val != 0 { b = b | 128; }
        sec_byte(b);
        if val == 0 { break; }
    }
}

fn sec_leb_s(v: i32) {
    let mut val: i32 = v;
    let mut more: i32 = 1;
    while more != 0 {
        let mut b: i32 = val & 127;
        val = val >> 7;
        if (val == 0 && (b & 64) == 0) || (val == -1 && (b & 64) != 0) {
            more = 0;
        } else {
            b = b | 128;
        }
        sec_byte(b);
    }
}

fn flush_section(section_id: i32) {
    let mut size: i32 = get_sec_pos();
    out_byte(section_id);
    out_leb_u(size);
    let mut i: i32 = 0;
    while i < size {
        out_byte(load_byte(SEC_BUF() + i));
        i += 1;
    }
    set_sec_pos(0);
}

// ================================================================
// CODE BUFFER HELPERS (write to temp code buffer for current func)
// ================================================================

fn code_byte(b: i32) {
    let mut p: i32 = get_code_pos();
    store_byte(CODE_BUF() + p, b);
    set_code_pos(p + 1);
}

fn code_leb_u(v: i32) {
    let mut val: i32 = v;
    loop {
        let mut b: i32 = val & 127;
        val = val >> 7;
        if val != 0 { b = b | 128; }
        code_byte(b);
        if val == 0 { break; }
    }
}

fn code_leb_s(v: i32) {
    let mut val: i32 = v;
    let mut more: i32 = 1;
    while more != 0 {
        let mut b: i32 = val & 127;
        val = val >> 7;
        if (val == 0 && (b & 64) == 0) || (val == -1 && (b & 64) != 0) {
            more = 0;
        } else {
            b = b | 128;
        }
        code_byte(b);
    }
}

// ================================================================
// WASM MODULE EMISSION
// ================================================================

fn emit_header() {
    out_byte(0);    // \0
    out_byte(97);   // a
    out_byte(115);  // s
    out_byte(109);  // m
    out_byte(1);    // version
    out_byte(0);
    out_byte(0);
    out_byte(0);
}

fn emit_type_section() {
    set_sec_pos(0);
    let mut max_params: i32 = 0;
    let mut i: i32 = 0;
    while i < get_func_count() {
        let mut pc: i32 = ftab_param_count(i);
        if pc > max_params { max_params = pc; }
        i += 1;
    }
    let mut type_count: i32 = (max_params + 1) * 2;
    sec_leb_u(type_count);
    let mut p: i32 = 0;
    while p <= max_params {
        // void return variant: type index = p * 2
        sec_byte(96);
        sec_leb_u(p);
        let mut j: i32 = 0;
        while j < p { sec_byte(127); j += 1; }
        sec_leb_u(0);
        // i32 return variant: type index = p * 2 + 1
        sec_byte(96);
        sec_leb_u(p);
        j = 0;
        while j < p { sec_byte(127); j += 1; }
        sec_leb_u(1);
        sec_byte(127);
        p += 1;
    }
    flush_section(1);
}

fn emit_function_section() {
    set_sec_pos(0);
    let mut count: i32 = get_func_count();
    sec_leb_u(count);
    let mut i: i32 = 0;
    while i < count {
        let mut type_idx: i32 = ftab_param_count(i) * 2 + ftab_has_return(i);
        sec_leb_u(type_idx);
        i += 1;
    }
    flush_section(3);
}

fn emit_memory_section() {
    set_sec_pos(0);
    sec_leb_u(1);
    sec_byte(0);
    sec_leb_u(1);
    flush_section(5);
}

fn emit_export_section() {
    set_sec_pos(0);
    let mut count: i32 = get_func_count() + 1;
    sec_leb_u(count);
    let mut i: i32 = 0;
    while i < get_func_count() {
        let mut ns: i32 = ftab_name_start(i);
        let mut nl: i32 = ftab_name_len(i);
        sec_leb_u(nl);
        let mut j: i32 = 0;
        while j < nl {
            sec_byte(load_byte(SRC_BASE() + ns + j));
            j += 1;
        }
        sec_byte(0);
        sec_leb_u(ftab_func_index(i));
        i += 1;
    }
    // Export memory as "memory"
    sec_leb_u(6);
    sec_byte(109); sec_byte(101); sec_byte(109); sec_byte(111); sec_byte(114); sec_byte(121);
    sec_byte(2);
    sec_leb_u(0);
    flush_section(7);
}

// ================================================================
// INTRINSIC DETECTION
// ================================================================

fn match_name(ns: i32, nl: i32, expected_len: i32, b0: i32, b1: i32, b2: i32, b3: i32, b4: i32) -> i32 {
    if nl != expected_len { return 0; }
    let mut s: i32 = SRC_BASE() + ns;
    if expected_len >= 1 && load_byte(s) != b0 { return 0; }
    if expected_len >= 2 && load_byte(s + 1) != b1 { return 0; }
    if expected_len >= 3 && load_byte(s + 2) != b2 { return 0; }
    if expected_len >= 4 && load_byte(s + 3) != b3 { return 0; }
    if expected_len >= 5 && load_byte(s + 4) != b4 { return 0; }
    1
}

fn match_tail(ns: i32, nl: i32, offset: i32, b0: i32, b1: i32, b2: i32, b3: i32, b4: i32, b5: i32) -> i32 {
    let mut s: i32 = SRC_BASE() + ns + offset;
    let mut remain: i32 = nl - offset;
    if remain >= 1 && load_byte(s) != b0 { return 0; }
    if remain >= 2 && load_byte(s + 1) != b1 { return 0; }
    if remain >= 3 && load_byte(s + 2) != b2 { return 0; }
    if remain >= 4 && load_byte(s + 3) != b3 { return 0; }
    if remain >= 5 && load_byte(s + 4) != b4 { return 0; }
    if remain >= 6 && load_byte(s + 5) != b5 { return 0; }
    1
}

// "load_byte" = 108 111 97 100 95 98 121 116 101 (9 chars)
fn is_load_byte(ns: i32, nl: i32) -> i32 {
    if nl != 9 { return 0; }
    if match_name(ns, nl, 9, 108, 111, 97, 100, 95) == 0 { return 0; }
    match_tail(ns, nl, 5, 98, 121, 116, 101, 0, 0)
}

// "store_byte" = 115 116 111 114 101 95 98 121 116 101 (10 chars)
fn is_store_byte(ns: i32, nl: i32) -> i32 {
    if nl != 10 { return 0; }
    if match_name(ns, nl, 10, 115, 116, 111, 114, 101) == 0 { return 0; }
    match_tail(ns, nl, 5, 95, 98, 121, 116, 101, 0)
}

// "load_i32" = 108 111 97 100 95 105 51 50 (8 chars)
fn is_load_i32(ns: i32, nl: i32) -> i32 {
    if nl != 8 { return 0; }
    if match_name(ns, nl, 8, 108, 111, 97, 100, 95) == 0 { return 0; }
    match_tail(ns, nl, 5, 105, 51, 50, 0, 0, 0)
}

// "store_i32" = 115 116 111 114 101 95 105 51 50 (9 chars)
fn is_store_i32(ns: i32, nl: i32) -> i32 {
    if nl != 9 { return 0; }
    if match_name(ns, nl, 9, 115, 116, 111, 114, 101) == 0 { return 0; }
    match_tail(ns, nl, 5, 95, 105, 51, 50, 0, 0)
}

// "memory_grow" = 109 101 109 111 114 121 95 103 114 111 119 (11 chars)
fn is_memory_grow(ns: i32, nl: i32) -> i32 {
    if nl != 11 { return 0; }
    if match_name(ns, nl, 11, 109, 101, 109, 111, 114) == 0 { return 0; }
    match_tail(ns, nl, 5, 121, 95, 103, 114, 111, 119)
}

// "memory_size" = 109 101 109 111 114 121 95 115 105 122 101 (11 chars)
fn is_memory_size(ns: i32, nl: i32) -> i32 {
    if nl != 11 { return 0; }
    if match_name(ns, nl, 11, 109, 101, 109, 111, 114) == 0 { return 0; }
    match_tail(ns, nl, 5, 121, 95, 115, 105, 122, 101)
}

// "unreachable" = 117 110 114 101 97 99 104 97 98 108 101 (11 chars)
fn is_unreachable(ns: i32, nl: i32) -> i32 {
    if nl != 11 { return 0; }
    if match_name(ns, nl, 11, 117, 110, 114, 101, 97) == 0 { return 0; }
    match_tail(ns, nl, 5, 99, 104, 97, 98, 108, 101)
}

// ================================================================
// CODE GENERATION
// ================================================================

fn register_functions(file_node: i32) {
    let mut first_child: i32 = node_d1(file_node);
    let mut count: i32 = node_d2(file_node);
    let mut func_idx: i32 = 0;
    let mut i: i32 = 0;
    while i < count {
        let mut fn_node: i32 = child_at(first_child + i);
        if node_kind(fn_node) == NK_FN() {
            let mut name_tok: i32 = node_d1(fn_node);
            let mut ns: i32 = tok_start(name_tok);
            let mut nl: i32 = tok_len(name_tok);
            let mut param_count: i32 = node_d3(fn_node);
            let mut has_return: i32 = node_d4(fn_node);
            add_func(ns, nl, param_count, has_return, func_idx);
            func_idx += 1;
        }
        i += 1;
    }
}

fn compile_func_body(fn_node: i32) {
    set_local_count(0);
    set_code_pos(0);
    set_loop_depth(0);

    let mut param_count: i32 = node_d3(fn_node);
    set_cur_params(param_count);

    // Register params as locals
    let mut first_param_child: i32 = node_d2(fn_node);
    let mut pi: i32 = 0;
    while pi < param_count {
        let mut param_node: i32 = child_at(first_param_child + pi);
        let mut name_tok: i32 = node_d1(param_node);
        add_local(tok_start(name_tok), tok_len(name_tok));
        pi += 1;
    }

    // Compile body
    let mut body_node: i32 = node_d5(fn_node);
    gen_block(body_node);
    code_byte(11); // end

    // Write function body to section buffer
    let mut code_size: i32 = get_code_pos();
    let mut extra_locals: i32 = get_local_count() - param_count;

    let mut body_content_size: i32 = 0;
    if extra_locals > 0 {
        body_content_size = 1 + leb_u_size(extra_locals) + 1 + code_size;
    } else {
        body_content_size = 1 + code_size;
    }

    sec_leb_u(body_content_size);
    if extra_locals > 0 {
        sec_leb_u(1);
        sec_leb_u(extra_locals);
        sec_byte(127);
    } else {
        sec_leb_u(0);
    }

    let mut i: i32 = 0;
    while i < code_size {
        sec_byte(load_byte(CODE_BUF() + i));
        i += 1;
    }
}

fn emit_code_section(file_node: i32) {
    set_sec_pos(0);
    let mut count: i32 = get_func_count();
    sec_leb_u(count);

    let mut first_child: i32 = node_d1(file_node);
    let mut child_count: i32 = node_d2(file_node);
    let mut ci: i32 = 0;
    while ci < child_count {
        let mut fn_node: i32 = child_at(first_child + ci);
        if node_kind(fn_node) == NK_FN() {
            compile_func_body(fn_node);
        }
        ci += 1;
    }
    flush_section(10);
}

// ================================================================
// STATEMENT CODE GENERATION
// ================================================================

fn gen_block(block_node: i32) {
    let mut first_stmt: i32 = node_d1(block_node);
    let mut count: i32 = node_d2(block_node);
    let mut i: i32 = 0;
    while i < count {
        gen_stmt(child_at(first_stmt + i));
        i += 1;
    }
}

fn gen_stmt(node: i32) {
    let mut k: i32 = node_kind(node);
    if k == NK_LET() { gen_let(node); return; }
    if k == NK_RETURN() { gen_return(node); return; }
    if k == NK_IF() { gen_if(node); return; }
    if k == NK_WHILE() { gen_while(node); return; }
    if k == NK_LOOP() { gen_loop(node); return; }
    if k == NK_BREAK() {
        code_byte(12); // br
        code_leb_u(1);
        return;
    }
    if k == NK_CONTINUE() {
        code_byte(12); // br
        code_leb_u(0);
        return;
    }
    if k == NK_EXPR_STMT() {
        let mut expr: i32 = node_d1(node);
        gen_expr(expr);
        if expr_has_value(expr) != 0 {
            code_byte(26); // drop
        }
        return;
    }
    if k == NK_TAIL_EXPR() {
        gen_expr(node_d1(node));
        return;
    }
    if k == NK_ASSIGN() { gen_assign(node); return; }
    unreachable();
}

fn expr_has_value(node: i32) -> i32 {
    let mut k: i32 = node_kind(node);
    if k == NK_LITERAL() || k == NK_IDENT() || k == NK_BINARY() || k == NK_UNARY() { return 1; }
    if k == NK_CALL() {
        let mut callee_tok: i32 = node_d1(node);
        let mut ns: i32 = tok_start(callee_tok);
        let mut nl: i32 = tok_len(callee_tok);
        if is_store_byte(ns, nl) != 0 { return 0; }
        if is_store_i32(ns, nl) != 0 { return 0; }
        if is_unreachable(ns, nl) != 0 { return 0; }
        let mut fi: i32 = find_func(ns, nl);
        if fi >= 0 { return ftab_has_return(fi); }
        return 1;
    }
    0
}

fn gen_let(node: i32) {
    let mut name_tok: i32 = node_d1(node);
    let mut init_expr: i32 = node_d2(node);
    let mut local_idx: i32 = add_local(tok_start(name_tok), tok_len(name_tok));
    if init_expr != -1 {
        gen_expr(init_expr);
        code_byte(33); // local.set
        code_leb_u(local_idx);
    }
}

fn gen_return(node: i32) {
    let mut val: i32 = node_d1(node);
    if val != -1 {
        gen_expr(val);
    }
    code_byte(15); // return
}

fn gen_if(node: i32) {
    let mut cond: i32 = node_d1(node);
    let mut then_blk: i32 = node_d2(node);
    let mut else_node: i32 = node_d3(node);
    gen_expr(cond);
    code_byte(4);  // if
    code_byte(64); // void block type
    gen_block(then_blk);
    if else_node != -1 {
        code_byte(5); // else
        if node_kind(else_node) == NK_IF() {
            gen_if(else_node);
        } else {
            gen_block(else_node);
        }
    }
    code_byte(11); // end
}

fn gen_while(node: i32) {
    let mut cond: i32 = node_d1(node);
    let mut body: i32 = node_d2(node);
    code_byte(2);  // block
    code_byte(64); // void
    code_byte(3);  // loop
    code_byte(64); // void
    gen_expr(cond);
    code_byte(69); // i32.eqz
    code_byte(13); // br_if
    code_leb_u(1);
    gen_block(body);
    code_byte(12); // br
    code_leb_u(0);
    code_byte(11); // end loop
    code_byte(11); // end block
}

fn gen_loop(node: i32) {
    let mut body: i32 = node_d1(node);
    code_byte(2);  // block
    code_byte(64); // void
    code_byte(3);  // loop
    code_byte(64); // void
    gen_block(body);
    code_byte(12); // br
    code_leb_u(0);
    code_byte(11); // end loop
    code_byte(11); // end block
}

fn gen_assign(node: i32) {
    let mut target_expr: i32 = node_d1(node);
    let mut op_kind: i32 = node_d2(node);
    let mut rhs: i32 = node_d3(node);
    let mut name_tok: i32 = node_d1(target_expr);
    let mut ns: i32 = tok_start(name_tok);
    let mut nl: i32 = tok_len(name_tok);
    let mut local_idx: i32 = find_local(ns, nl);

    if op_kind == TK_EQ() {
        gen_expr(rhs);
    } else {
        code_byte(32); // local.get
        code_leb_u(local_idx);
        gen_expr(rhs);
        if op_kind == TK_PLUSEQ() { code_byte(106); }
        if op_kind == TK_MINUSEQ() { code_byte(107); }
        if op_kind == TK_STAREQ() { code_byte(108); }
        if op_kind == TK_SLASHEQ() { code_byte(109); }
    }
    code_byte(33); // local.set
    code_leb_u(local_idx);
}

// ================================================================
// EXPRESSION CODE GENERATION
// ================================================================

fn gen_expr(node: i32) {
    let mut k: i32 = node_kind(node);
    if k == NK_LITERAL() {
        code_byte(65); // i32.const
        code_leb_s(node_d1(node));
        return;
    }
    if k == NK_IDENT() {
        let mut name_tok: i32 = node_d1(node);
        let mut ns: i32 = tok_start(name_tok);
        let mut nl: i32 = tok_len(name_tok);
        let mut local_idx: i32 = find_local(ns, nl);
        code_byte(32); // local.get
        code_leb_u(local_idx);
        return;
    }
    if k == NK_BINARY() { gen_binary(node); return; }
    if k == NK_UNARY() { gen_unary(node); return; }
    if k == NK_CALL() { gen_call(node); return; }
    if k == NK_IF() { gen_if(node); return; }
    if k == NK_BLOCK() { gen_block(node); return; }
    unreachable();
}

fn gen_binary(node: i32) {
    let mut lhs: i32 = node_d1(node);
    let mut op: i32 = node_d2(node);
    let mut rhs: i32 = node_d3(node);
    gen_expr(lhs);
    gen_expr(rhs);
    if op == TK_PLUS() { code_byte(106); return; }     // i32.add
    if op == TK_MINUS() { code_byte(107); return; }    // i32.sub
    if op == TK_STAR() { code_byte(108); return; }     // i32.mul
    if op == TK_SLASH() { code_byte(109); return; }    // i32.div_s
    if op == TK_PERCENT() { code_byte(111); return; }  // i32.rem_s
    if op == TK_AMP() { code_byte(113); return; }      // i32.and
    if op == TK_PIPE() { code_byte(114); return; }     // i32.or
    if op == TK_CARET() { code_byte(115); return; }    // i32.xor
    if op == TK_SHL() { code_byte(116); return; }      // i32.shl
    if op == TK_SHR() { code_byte(117); return; }      // i32.shr_s
    if op == TK_EQEQ() { code_byte(70); return; }     // i32.eq
    if op == TK_BANGEQ() { code_byte(71); return; }   // i32.ne
    if op == TK_LT() { code_byte(72); return; }       // i32.lt_s
    if op == TK_GT() { code_byte(74); return; }       // i32.gt_s
    if op == TK_LTEQ() { code_byte(76); return; }     // i32.le_s
    if op == TK_GTEQ() { code_byte(78); return; }     // i32.ge_s
    if op == TK_AMPAMP() { code_byte(113); return; }   // i32.and
    if op == TK_PIPEPIPE() { code_byte(114); return; } // i32.or
    unreachable();
}

fn gen_unary(node: i32) {
    let mut op: i32 = node_d1(node);
    let mut operand: i32 = node_d2(node);
    if op == TK_BANG() {
        gen_expr(operand);
        code_byte(69); // i32.eqz
        return;
    }
    if op == TK_MINUS() {
        code_byte(65); // i32.const
        code_leb_s(0);
        gen_expr(operand);
        code_byte(107); // i32.sub
        return;
    }
    unreachable();
}

fn gen_call(node: i32) {
    let mut callee_tok: i32 = node_d1(node);
    let mut first_arg: i32 = node_d2(node);
    let mut arg_count: i32 = node_d3(node);
    let mut ns: i32 = tok_start(callee_tok);
    let mut nl: i32 = tok_len(callee_tok);

    // Check intrinsics
    if is_load_byte(ns, nl) != 0 {
        gen_expr(child_at(first_arg));
        code_byte(45); // i32.load8_u
        code_byte(0);  // align
        code_byte(0);  // offset
        return;
    }
    if is_store_byte(ns, nl) != 0 {
        gen_expr(child_at(first_arg));
        gen_expr(child_at(first_arg + 1));
        code_byte(58); // i32.store8
        code_byte(0);
        code_byte(0);
        return;
    }
    if is_load_i32(ns, nl) != 0 {
        gen_expr(child_at(first_arg));
        code_byte(40); // i32.load
        code_byte(2);  // align = 4
        code_byte(0);  // offset
        return;
    }
    if is_store_i32(ns, nl) != 0 {
        gen_expr(child_at(first_arg));
        gen_expr(child_at(first_arg + 1));
        code_byte(54); // i32.store
        code_byte(2);
        code_byte(0);
        return;
    }
    if is_memory_grow(ns, nl) != 0 {
        gen_expr(child_at(first_arg));
        code_byte(64); // memory.grow
        code_byte(0);
        return;
    }
    if is_memory_size(ns, nl) != 0 {
        code_byte(63); // memory.size
        code_byte(0);
        return;
    }
    if is_unreachable(ns, nl) != 0 {
        code_byte(0); // unreachable
        return;
    }

    // Regular function call
    let mut fi: i32 = find_func(ns, nl);
    let mut i: i32 = 0;
    while i < arg_count {
        gen_expr(child_at(first_arg + i));
        i += 1;
    }
    code_byte(16); // call
    code_leb_u(ftab_func_index(fi));
}

// ================================================================
// ENTRY POINT
// ================================================================

fn compile(src_len: i32) -> i32 {
    // Grow memory from 1 page to 16 pages
    memory_grow(15);

    // Initialize globals
    set_src_len(src_len);
    set_tok_count(0);
    set_node_count(0);
    set_child_count(0);
    set_func_count(0);
    set_local_count(0);
    set_out_pos(0);
    set_tok_pos(0);
    set_code_pos(0);
    set_loop_depth(0);
    set_sec_pos(0);
    set_pstack_ptr(0);

    // Phase 1: Lex
    lex_all();

    // Phase 2: Parse
    set_tok_pos(0);
    let mut file_node: i32 = parse_file();

    // Phase 3: Register functions
    register_functions(file_node);

    // Phase 4: Emit WASM
    set_out_pos(0);
    emit_header();
    emit_type_section();
    emit_function_section();
    emit_memory_section();
    emit_export_section();
    emit_code_section(file_node);

    get_out_pos()
}
