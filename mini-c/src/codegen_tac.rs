use crate::ir::{FunctionIR, Instr, Operand};

// Emit a simple pseudo-assembly ('cutall' style) from TAC.
pub fn emit_function(f: &FunctionIR) -> String {
    let mut out = String::new();
    out.push_str(&format!(".func {}({})\n", f.name, f.params.join(", ")));
    out.push_str("{
");
    for instr in &f.instrs {
        let line = match instr {
            Instr::StoreLocal { name, src } => format!("  MOV %{}, {}", name, fmt_operand(src)),
            Instr::Call { dest, name, args } => {
                let a = args.iter().map(|o| fmt_operand(o)).collect::<Vec<_>>().join(", ");
                if let Some(d) = dest {
                    format!("  {} = CALL {}({})", d, name, a)
                } else {
                    format!("  CALL {}({})", name, a)
                }
            }
            Instr::Return { src } => {
                if let Some(s) = src { format!("  RET {}", fmt_operand(s)) } else { "  RET".to_string() }
            }
            Instr::BinOp { dest, op, left, right } => format!("  {} = {} {} {}", dest, fmt_operand(left), op, fmt_operand(right)),
        };
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn fmt_operand(o: &Operand) -> String {
    match o {
        Operand::Temp(t) => t.clone(),
        Operand::Local(n) => format!("%{}", n),
        Operand::ConstInt(i) => format!("{}", i),
        Operand::ConstFloat(f) => format!("{}", f),
        Operand::ConstString(s) => format!("\"{}\"", s),
    }
}
