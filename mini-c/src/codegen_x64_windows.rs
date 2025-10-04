use crate::ir::{FunctionIR, Instr, Operand};
use std::collections::HashMap;

// Simple Windows x64 emitter. Produces readable Intel-style assembly for inspection.
pub fn emit_function(f: &FunctionIR) -> String {
    // collect string constants
    let mut str_pool: HashMap<String, String> = HashMap::new();
    let mut str_count = 0;
    for instr in &f.instrs {
        match instr {
            Instr::Call { args, .. } => {
                for a in args {
                    if let Operand::ConstString(s) = a {
                        let hash = crc32fast::hash(s.as_bytes());
                        let lbl = format!("LSTR_{}", hash);
                        str_pool.entry(s.clone()).or_insert(lbl);
                    }
                }
            }
            Instr::BinOp { left, right, .. } => {
                if let Operand::ConstString(s) = left { let hash = crc32fast::hash(s.as_bytes()); let lbl = format!("LSTR_{}", hash); str_pool.entry(s.clone()).or_insert(lbl); }
                if let Operand::ConstString(s) = right { let hash = crc32fast::hash(s.as_bytes()); let lbl = format!("LSTR_{}", hash); str_pool.entry(s.clone()).or_insert(lbl); }
            }
            _ => {}
        }
    }

    // collect temps and locals to assign stack offsets
    let mut slots: HashMap<String, i32> = HashMap::new();
    let mut offset = 0i32; // positive grows
    // params are locals too
    for p in &f.params {
        if !slots.contains_key(p) {
            offset += 8;
            slots.insert(p.clone(), offset);
        }
    }
    for instr in &f.instrs {
        match instr {
            Instr::StoreLocal { name, .. } => {
                if !slots.contains_key(name) {
                    offset += 8;
                    slots.insert(name.clone(), offset);
                }
            }
            Instr::BinOp { dest, .. } => {
                if !slots.contains_key(dest) {
                    offset += 8;
                    slots.insert(dest.clone(), offset);
                }
            }
            Instr::Call { dest: Some(d), .. } => {
                if !slots.contains_key(d) {
                    offset += 8;
                    slots.insert(d.clone(), offset);
                }
            }
            _ => {}
        }
    }

    // frame size: round up to 16 and include 32-byte shadow space
    let mut frame_size = ((offset + 15) / 16) * 16;
    if frame_size < 32 { frame_size = 32; }

    let mut out = String::new();
    out.push_str(&format!("; function {}\n", f.name));
    out.push_str("push rbp\n");
    out.push_str("mov rbp, rsp\n");
    out.push_str(&format!("sub rsp, {}\n", frame_size));

    // emit instructions
    for instr in &f.instrs {
        match instr {
            Instr::StoreLocal { name, src } => {
                // load src into rax
                emit_load_operand(&mut out, src, &slots);
                let off = slots.get(name).unwrap();
                out.push_str(&format!("mov [rbp-{}], rax\n", off));
            }
            Instr::BinOp { dest, op, left, right } => {
                emit_load_operand(&mut out, left, &slots);
                // move left in rax, load right into rdx
                emit_load_operand_to_reg(&mut out, right, &slots, "rdx");
                let asmop = match op.as_str() {
                    "+" => "add rax, rdx",
                    "-" => "sub rax, rdx",
                    "*" => "imul rax, rdx",
                    "/" => "cqo\n    idiv rdx",
                    other => other,
                };
                out.push_str(&format!("    {}\n", asmop));
                let off = slots.get(dest).unwrap();
                out.push_str(&format!("mov [rbp-{}], rax\n", off));
            }
            Instr::Call { dest, name, args } => {
                // move up to 4 args into RCX,RDX,R8,R9
                let regs = ["rcx","rdx","r8","r9"]; 
                for (i, a) in args.iter().enumerate() {
                    if i < 4 {
                        emit_load_operand_to_reg(&mut out, a, &slots, regs[i]);
                    } else {
                        // push on stack in reverse order later — for simplicity push
                        emit_load_operand(&mut out, a, &slots);
                        out.push_str("push rax\n");
                    }
                }
                out.push_str(&format!("call {}\n", name));
                // store return if needed
                if let Some(d) = dest {
                    let off = slots.get(d).unwrap();
                    out.push_str(&format!("mov [rbp-{}], rax\n", off));
                }
            }
            Instr::Return { src } => {
                if let Some(s) = src {
                    emit_load_operand(&mut out, s, &slots);
                    out.push_str("mov rsp, rbp\n");
                    out.push_str("pop rbp\n");
                    out.push_str("ret\n");
                } else {
                    out.push_str("mov rsp, rbp\n");
                    out.push_str("pop rbp\n");
                    out.push_str("ret\n");
                }
            }
        }
    }

    // epilogue if not already returned
    out.push_str("mov rsp, rbp\n");
    out.push_str("pop rbp\n");
    out.push_str("ret\n");

    // emit data section for strings
    if !str_pool.is_empty() {
        out.push_str("\n; data section\n");
        for (s, lbl) in &str_pool {
            out.push_str(&format!("{}: db \"{}\",0\n", lbl, s.replace("\"","\\\"")));
        }
    }

    out
}

fn emit_load_operand(out: &mut String, op: &Operand, slots: &HashMap<String, i32>) {
    match op {
        Operand::Temp(t) => {
            let off = slots.get(t).unwrap();
            out.push_str(&format!("mov rax, [rbp-{}]\n", off));
        }
        Operand::Local(n) => {
            let off = slots.get(n).unwrap();
            out.push_str(&format!("mov rax, [rbp-{}]\n", off));
        }
        Operand::ConstInt(i) => {
            out.push_str(&format!("mov rax, {}\n", i));
        }
        Operand::ConstFloat(f) => {
            out.push_str(&format!("; load float {} into rax (not implemented)\n", f));
            out.push_str("mov rax, 0\n");
        }
        Operand::ConstString(s) => {
            // placeholder: load address of string label into rax
            out.push_str(&format!("lea rax, [rel {}] ; string {}\n", find_label_for_string(s, slots), s));
        }
    }
}

fn emit_load_operand_to_reg(out: &mut String, op: &Operand, slots: &HashMap<String, i32>, reg: &str) {
    match op {
        Operand::Temp(t) => { let off = slots.get(t).unwrap(); out.push_str(&format!("mov {}, [rbp-{}]\n", reg, off)); }
        Operand::Local(n) => { let off = slots.get(n).unwrap(); out.push_str(&format!("mov {}, [rbp-{}]\n", reg, off)); }
        Operand::ConstInt(i) => { out.push_str(&format!("mov {}, {}\n", reg, i)); }
        Operand::ConstFloat(f) => { out.push_str(&format!("; mov {} <- float {} (not implemented)\n", reg, f)); out.push_str(&format!("mov {}, 0\n", reg)); }
        Operand::ConstString(s) => { out.push_str(&format!("lea {}, [rel {}] ; string {}\n", reg, find_label_for_string(s, slots), s)); }
    }
}

// helper to synthesize a label name for a given string; here we hash the string to a small label
fn find_label_for_string(s: &str, _slots: &HashMap<String, i32>) -> String {
    let h = crc32fast::hash(s.as_bytes());
    format!("LSTR_{}", h)
}
