// x64 Windows calling convention code generator
use crate::ir::{FunctionIR, Instr, Operand};
use std::collections::HashMap;


// Emit x64 assembly for a single function using Windows x64 calling convention.
pub fn emit_function(f: &FunctionIR) -> String {

    // first pass: collect string literals
    let mut str_pool: HashMap<String, String> = HashMap::new();
    let _str_count = 0;

    // find string literals in instructions
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

            // also check BinOp operands
            Instr::BinOp { left, right, .. } => {
                if let Operand::ConstString(s) = left { let hash = crc32fast::hash(s.as_bytes()); let lbl = format!("LSTR_{}", hash); str_pool.entry(s.clone()).or_insert(lbl); }
                if let Operand::ConstString(s) = right { let hash = crc32fast::hash(s.as_bytes()); let lbl = format!("LSTR_{}", hash); str_pool.entry(s.clone()).or_insert(lbl); }
            }
            _ => {}
        }
    }


    // second pass: assign stack slots to locals and temps
    let mut slots: HashMap<String, i32> = HashMap::new();
    let mut offset = 0i32; 


    // assign slots for params first
    for p in &f.params {
        if !slots.contains_key(p) {
            offset += 8;
            slots.insert(p.clone(), offset);
        }
    }


    // assign slots for locals and temps
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



    // align frame size to 16 bytes
    let mut frame_size = ((offset + 15) / 16) * 16;
    if frame_size < 32 { frame_size = 32; }


    // prologue -> intro segment of the function
    let mut out = String::new();
    out.push_str(&format!("; function {}\n", f.name));
    out.push_str("push rbp\n");
    out.push_str("mov rbp, rsp\n");
    out.push_str(&format!("sub rsp, {}\n", frame_size));



    // emit instructions
    for instr in &f.instrs {
        match instr {

            // store local: load src into rax, store rax into local slot
            Instr::StoreLocal { name, src } => {
                emit_load_operand(&mut out, src, &slots);
                let off = slots.get(name).unwrap();
                out.push_str(&format!("mov [rbp-{}], rax\n", off));
            }

            // binary op: load left and right, apply op, store result
                Instr::BinOp { dest, op, left, right } => {
                emit_load_operand(&mut out, left, &slots);
                emit_load_operand_to_reg(&mut out, right, &slots, "rdx");
                let asmop = match op.as_str() {
                    "+" => "add rax, rdx",
                    "-" => "sub rax, rdx",
                    "*" => "imul rax, rdx",
                    "/" => "cqo\n    idiv rdx",
                    other => other,
                };

                // emit operation
                out.push_str(&format!("    {}\n", asmop));
                let off = slots.get(dest).unwrap();
                out.push_str(&format!("mov [rbp-{}], rax\n", off));
            }

            
            Instr::Call { dest, name, args } => {
                let regs = ["rcx","rdx","r8","r9"]; 
                for (i, a) in args.iter().enumerate() {
                    if i < 4 {
                        emit_load_operand_to_reg(&mut out, a, &slots, regs[i]);
                    } else {
                        emit_load_operand(&mut out, a, &slots);
                        out.push_str("push rax\n");
                    }
                }
                out.push_str(&format!("call {}\n", name));
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

    out.push_str("mov rsp, rbp\n");
    out.push_str("pop rbp\n");
    out.push_str("ret\n");

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

fn find_label_for_string(s: &str, _slots: &HashMap<String, i32>) -> String {
    let h = crc32fast::hash(s.as_bytes());
    format!("LSTR_{}", h)
}
