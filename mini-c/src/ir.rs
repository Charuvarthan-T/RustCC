use std::fmt;

#[derive(Clone, Debug)]
pub enum Operand {
    Temp(String),
    Local(String),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Temp(t) => write!(f, "{}", t),
            Operand::Local(n) => write!(f, "%{}", n),
            Operand::ConstInt(i) => write!(f, "{}", i),
            Operand::ConstFloat(fl) => write!(f, "{}", fl),
            Operand::ConstString(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Instr {
    StoreLocal { name: String, src: Operand },
    Call { dest: Option<String>, name: String, args: Vec<Operand> },
    Return { src: Option<Operand> },
    BinOp { dest: String, op: String, left: Operand, right: Operand },
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::StoreLocal { name, src } => write!(f, "store %{} <- {}", name, src),
            Instr::Call { dest, name, args } => {
                if let Some(d) = dest {
                    write!(f, "{} = call {}({})", d, name, args.iter().map(|o| format!("{}", o)).collect::<Vec<_>>().join(", "))
                } else {
                    write!(f, "call {}({})", name, args.iter().map(|o| format!("{}", o)).collect::<Vec<_>>().join(", "))
                }
            }
            Instr::Return { src } => {
                if let Some(s) = src { write!(f, "return {}", s) } else { write!(f, "return") }
            }
            Instr::BinOp { dest, op, left, right } => write!(f, "{} = {} {} {}", dest, left, op, right),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionIR {
    pub name: String,
    pub params: Vec<String>,
    pub instrs: Vec<Instr>,
}

impl fmt::Display for FunctionIR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "func {}({}) {{", self.name, self.params.join(", "))?;
        for instr in &self.instrs {
            writeln!(f, "  {}", instr)?;
        }
        write!(f, "}}")
    }
}
