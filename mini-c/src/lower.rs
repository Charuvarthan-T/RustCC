use crate::ast::*;
use crate::ir::{FunctionIR, Instr, Operand};

struct LowerState {
    tmp: usize,
}

impl LowerState {
    fn new() -> Self { LowerState { tmp: 0 } }
    fn gen_tmp(&mut self) -> String { let id = self.tmp; self.tmp += 1; format!("t{}", id) }
}

fn lower_expr(expr: &Expr, state: &mut LowerState, instrs: &mut Vec<Instr>) -> Operand {
    match expr {
        Expr::Number(n) => Operand::ConstInt(*n),
        Expr::FloatNumber(f) => Operand::ConstFloat(*f),
        Expr::CharLiteral(c) => Operand::ConstInt(*c as i64),
        Expr::Ident(name) => Operand::Local(name.clone()),
        Expr::Unary { op, expr } => {
            let o = lower_expr(expr, state, instrs);
            let dest = state.gen_tmp();
            match op {
                crate::ast::UnaryOp::Neg => {
                    instrs.push(Instr::BinOp { dest: dest.clone(), op: "neg".to_string(), left: o.clone(), right: Operand::ConstInt(0) });
                }
                crate::ast::UnaryOp::Not => {
                    instrs.push(Instr::BinOp { dest: dest.clone(), op: "not".to_string(), left: o.clone(), right: Operand::ConstInt(0) });
                }
            }
            Operand::Temp(dest)
        }
        Expr::Binary { op, left, right } => {
            let l = lower_expr(left, state, instrs);
            let r = lower_expr(right, state, instrs);
            let dest = state.gen_tmp();
            let opname = match op {
                crate::ast::BinaryOp::Add => "+",
                crate::ast::BinaryOp::Sub => "-",
                crate::ast::BinaryOp::Mul => "*",
                crate::ast::BinaryOp::Div => "/",
            };
            instrs.push(Instr::BinOp { dest: dest.clone(), op: opname.to_string(), left: l, right: r });
            Operand::Temp(dest)
        }
        Expr::Assign { name, value } => {
            let v = lower_expr(value, state, instrs);
            instrs.push(Instr::StoreLocal { name: name.clone(), src: v.clone() });
            Operand::Local(name.clone())
        }
        Expr::Call { name, args } => {
            let mut op_args = Vec::new();
            for a in args {
                op_args.push(lower_expr(a, state, instrs));
            }
            let dest = state.gen_tmp();
            instrs.push(Instr::Call { dest: Some(dest.clone()), name: name.clone(), args: op_args });
            Operand::Temp(dest)
        }
        Expr::StringLiteral(s) => Operand::ConstString(s.clone()),
    }
}

pub fn lower_program(prog: &crate::ast::Program) -> Vec<FunctionIR> {
    let mut res = Vec::new();
    for func in &prog.functions {
        let mut state = LowerState::new();
        let mut instrs: Vec<Instr> = Vec::new();
        // params are locals; no explicit instructions needed
        for stmt in &func.body.stmts {
            match stmt {
                crate::ast::Stmt::VarDecl { name, value, .. } => {
                    let v = lower_expr(value, &mut state, &mut instrs);
                    instrs.push(Instr::StoreLocal { name: name.clone(), src: v });
                }
                crate::ast::Stmt::ExprStmt(e) => {
                    lower_expr(e, &mut state, &mut instrs);
                }
                crate::ast::Stmt::Return(e) => {
                    let v = lower_expr(e, &mut state, &mut instrs);
                    instrs.push(Instr::Return { src: Some(v) });
                }
            }
        }
        let fir = FunctionIR { name: func.name.clone(), params: func.params.iter().map(|(_, n)| n.clone()).collect(), instrs };
        res.push(fir);
    }
    res
}
