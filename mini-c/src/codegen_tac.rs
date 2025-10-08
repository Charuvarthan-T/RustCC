// Code generation for Three Address Code (TAC) from intermediate representation (IR)
use crate::ir::{FunctionIR, Instr, Operand};


// Emit a function in a simple TAC-like format for inspection.
pub fn emit_function(f: &FunctionIR) -> String {
    let mut out = String::new();
    out.push_str(&format!(".func {}({})\n", f.name, f.params.join(", ")));
    out.push_str("{
");

// Generate TAC for each instruction
    for instr in &f.instrs {

        // format one line of TAC
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

            // return TAC instruction
            Instr::Return { src } => {
                if let Some(s) = src { format!("  RET {}", fmt_operand(s)) } else { "  RET".to_string() }
            }
            
            // binary operation TAC instruction
            Instr::BinOp { dest, op, left, right } => format!("  {} = {} {} {}", dest, fmt_operand(left), op, fmt_operand(right)),
        };

        // append line to output
        out.push_str(&line);
        out.push('\n');
    }

    // function end
    out.push_str("}\n");
    out
}


// Helper to format an operand for TAC output
fn fmt_operand(o: &Operand) -> String {
    match o {
        Operand::Temp(t) => t.clone(),
        Operand::Local(n) => format!("%{}", n),
        Operand::ConstInt(i) => format!("{}", i),
        Operand::ConstFloat(f) => format!("{}", f),
        Operand::ConstString(s) => format!("\"{}\"", s),
    }
}
