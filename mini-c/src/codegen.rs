// codegen.rs
// Simple interpreter for mini-c (Phase 4)
// This implements a minimal runtime that can execute the AST directly.
// It supports integers, floats, chars, local variables, params, function calls
// and a builtin `printf` that understands a basic "%d" and "%f" format.

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
	Int(i64),
	Float(f64),
	Char(char),
	Void,
}

impl Value {
	fn as_int(&self) -> Option<i64> {
		match self {
			Value::Int(i) => Some(*i),
			_ => None,
		}
	}

	fn as_float(&self) -> Option<f64> {
		match self {
			Value::Float(f) => Some(*f),
			_ => None,
		}
	}
}

type Locals = HashMap<String, Value>;

// Execute the whole program. Returns the exit code of `main` (0..255) on success
// or an Err string on runtime error.
pub fn run(program: &Program) -> Result<i32, String> {
	// find main
	let main_func = program.functions.iter().find(|f| f.name == "main");
	if main_func.is_none() {
		return Err("No `main` function found".to_string());
	}
	let main = main_func.unwrap();
	// execute main with no args
	match execute_function(main, program, vec![]) {
		Ok(v) => match v {
			Value::Int(i) => Ok((i & 0xff) as i32),
			Value::Void => Ok(0),
			_ => Ok(0),
		},
		Err(e) => Err(e),
	}
}

fn execute_function(func: &Function, program: &Program, args: Vec<Value>) -> Result<Value, String> {
	// create locals and populate params
	let mut locals: Locals = HashMap::new();
	for (i, (ty, name)) in func.params.iter().enumerate() {
		if i < args.len() {
			locals.insert(name.clone(), args[i].clone());
		} else {
			// missing param -> default to zero-like
			let v = match ty {
				Type::Int => Value::Int(0),
				Type::Float => Value::Float(0.0),
				Type::Char => Value::Char('\0'),
				Type::Void => Value::Void,
			};
			locals.insert(name.clone(), v);
		}
	}

	// execute statements sequentially
	for stmt in &func.body.stmts {
		if let Some(ret) = execute_stmt(stmt, &mut locals, program)? {
			return Ok(ret);
		}
	}
	// no explicit return -> default
	Ok(Value::Void)
}

// Execute a statement. Returns Ok(Some(value)) if a return occurred with that value.
fn execute_stmt(stmt: &Stmt, locals: &mut Locals, program: &Program) -> Result<Option<Value>, String> {
	match stmt {
		Stmt::VarDecl { ty: _, name, value } => {
			let v = eval_expr(value, locals, program)?;
			locals.insert(name.clone(), v);
			Ok(None)
		}
		Stmt::ExprStmt(e) => {
			let _ = eval_expr(e, locals, program)?;
			Ok(None)
		}
		Stmt::Return(expr) => {
			let v = eval_expr(expr, locals, program)?;
			Ok(Some(v))
		}
	}
}

fn eval_expr(expr: &Expr, locals: &mut Locals, program: &Program) -> Result<Value, String> {
	match expr {
		Expr::Number(n) => Ok(Value::Int(*n)),
		Expr::FloatNumber(f) => Ok(Value::Float(*f)),
		Expr::CharLiteral(c) => Ok(Value::Char(*c)),
	Expr::StringLiteral(_s) => Ok(Value::Void), // strings not stored as runtime Value for now
		Expr::Ident(name) => {
			if let Some(v) = locals.get(name) {
				Ok(v.clone())
			} else {
				Err(format!("Undefined variable at runtime: {}", name))
			}
		}
		Expr::Unary { op, expr } => {
			let v = eval_expr(expr, locals, program)?;
			match (op, v) {
				(UnaryOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
				(UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
				(UnaryOp::Not, Value::Int(i)) => Ok(Value::Int((i == 0) as i64)),
				_ => Err("Unsupported unary operation or type".to_string()),
			}
		}
		Expr::Binary { op, left, right } => {
			let l = eval_expr(left, locals, program)?;
			let r = eval_expr(right, locals, program)?;
			match (l, r) {
				(Value::Int(a), Value::Int(b)) => match op {
					BinaryOp::Add => Ok(Value::Int(a + b)),
					BinaryOp::Sub => Ok(Value::Int(a - b)),
					BinaryOp::Mul => Ok(Value::Int(a * b)),
					BinaryOp::Div => Ok(Value::Int(a / b)),
				},
				(Value::Float(a), Value::Float(b)) => match op {
					BinaryOp::Add => Ok(Value::Float(a + b)),
					BinaryOp::Sub => Ok(Value::Float(a - b)),
					BinaryOp::Mul => Ok(Value::Float(a * b)),
					BinaryOp::Div => Ok(Value::Float(a / b)),
				},
				// simple mixed int/float coercion
				(Value::Int(a), Value::Float(b)) => {
					let af = a as f64;
					match op {
						BinaryOp::Add => Ok(Value::Float(af + b)),
						BinaryOp::Sub => Ok(Value::Float(af - b)),
						BinaryOp::Mul => Ok(Value::Float(af * b)),
						BinaryOp::Div => Ok(Value::Float(af / b)),
					}
				}
				(Value::Float(a), Value::Int(b)) => {
					let bf = b as f64;
					match op {
						BinaryOp::Add => Ok(Value::Float(a + bf)),
						BinaryOp::Sub => Ok(Value::Float(a - bf)),
						BinaryOp::Mul => Ok(Value::Float(a * bf)),
						BinaryOp::Div => Ok(Value::Float(a / bf)),
					}
				}
				_ => Err("Unsupported binary operand types".to_string()),
			}
		}
		Expr::Assign { name, value } => {
			let v = eval_expr(value, locals, program)?;
			locals.insert(name.clone(), v.clone());
			Ok(v)
		}
		Expr::Call { name, args } => {
			// builtin printf handling
			if name == "printf" {
				// very small subset: first arg must be string literal
				if args.is_empty() {
					return Err("printf requires at least a format string".to_string());
				}
				// evaluate first arg specially if it's a string literal
				let fmt = match &args[0] {
					Expr::StringLiteral(s) => s.clone(),
					_other => {
						// allow evaluated string-like via expression (not implemented)
						return Err("printf: first argument must be a string literal in this runtime".to_string());
					}
				};
				// evaluate remaining args
				let mut vals: Vec<Value> = Vec::new();
				for a in &args[1..] {
					vals.push(eval_expr(a, locals, program)?);
				}
				// support %d and %f only
				let mut out = String::new();
				let mut arg_i = 0;
				let mut chars = fmt.chars().peekable();
				while let Some(ch) = chars.next() {
					if ch == '%' {
						if let Some(&next) = chars.peek() {
							if next == 'd' {
								chars.next();
								if arg_i < vals.len() {
									if let Some(iv) = vals[arg_i].as_int() {
										out.push_str(&format!("{}", iv));
									} else {
										return Err("printf: %d with non-int argument".to_string());
									}
								}
								arg_i += 1;
								continue;
							} else if next == 'f' {
								chars.next();
								if arg_i < vals.len() {
									if let Some(fv) = vals[arg_i].as_float() {
										out.push_str(&format!("{}", fv));
									} else {
										return Err("printf: %f with non-float argument".to_string());
									}
								}
								arg_i += 1;
								continue;
							}
						}
						// unsupported format, print % literally
						out.push('%');
					} else {
						out.push(ch);
					}
				}
				print!("{}", out);
				return Ok(Value::Int(out.len() as i64));
			}

			// user-defined functions
			if let Some(f) = program.functions.iter().find(|ff| ff.name == *name) {
				// evaluate args
				let mut evaled: Vec<Value> = Vec::new();
				for a in args {
					evaled.push(eval_expr(a, locals, program)?);
				}
				return execute_function(f, program, evaled);
			}

			Err(format!("Unknown function called at runtime: {}", name))
		}
	}
}


