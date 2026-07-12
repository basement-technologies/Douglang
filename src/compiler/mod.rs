mod runtime;

use std::collections::HashMap;

use crate::ast::{BoolOper, Condition, DougChain, Expr, SetOper, Stmt};
use crate::token::ValueLiteral;
use runtime::RUNTIME;

fn bool_oper_to_c(oper: &BoolOper) -> &'static str {
    match oper {
        BoolOper::Equal => "==",
        BoolOper::NotEqual => "!=",
        BoolOper::Greater => ">",
        BoolOper::Less => "<",
        BoolOper::GreaterEqual => ">=",
        BoolOper::LessEqual => "<=",
    }
}

pub struct Compiler {
    lines: Vec<String>,
    indent: usize,
    tmp: usize,
    funcs: HashMap<String, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            lines: Vec::new(),
            indent: 1,
            tmp: 0,
            funcs: HashMap::new(),
        }
    }

    fn emit(&mut self, line: &str) {
        let prefix = "    ".repeat(self.indent);
        self.lines.push(format!("{prefix}{line}"));
    }

    fn new_tmp_var(&mut self) -> String {
        self.tmp += 1;
        format!("_t{}", self.tmp)
    }

    fn cstring_literal(s: &str) -> String {
        let mut out = String::from('"');
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    }

    fn doug_index(chains: &[DougChain], start: &str) -> String {
        let mut expr = start.to_string();
        for (i, chain) in chains.iter().enumerate() {
            let value: i64 = 1 << (chain.count - 1);
            let oper = if i % 2 == 0 { '+' } else { '-' };
            expr = format!("({expr} {oper} {value})");
        }
        expr
    }

    fn ffi_arg(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                ValueLiteral::Str(s) => Self::cstring_literal(s),
                ValueLiteral::Float(v) => format!("(double){v:?}"),
                ValueLiteral::Int(v) => format!("(long long){v}LL"),
            },
            Expr::DougSequence { .. } => {
                let val = self.eval_expr(expr);
                let t = self.new_tmp_var();
                self.emit(&format!("DougValue {t} = {val};"));
                format!(
                    "(({t}).kind == DV_STRING ? (long long)(size_t)dv_as_cstr({t}) : dv_as_int({t}))"
                )
            }
        }
    }

    fn eval_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                ValueLiteral::Str(s) => {
                    format!("dv_make_string({})", Self::cstring_literal(s))
                }
                ValueLiteral::Float(v) => format!("dv_make_double({v:?})"),
                ValueLiteral::Int(v) => format!("dv_make_int({v}LL)"),
            },
            Expr::DougSequence { chains } => {
                let idx = Self::doug_index(chains, "0LL");
                format!("dv_get({idx})")
            }
        }
    }

    fn eval_cond(&mut self, cond: &Condition) -> String {
        let left = self.eval_expr(&cond.left);
        let right = self.eval_expr(&cond.right);
        let oper = bool_oper_to_c(&cond.oper);
        let lt = self.new_tmp_var();
        let rt = self.new_tmp_var();
        self.emit(&format!("DougValue {lt} = {left};"));
        self.emit(&format!("DougValue {rt} = {right};"));
        format!(
            "((({lt}).kind == DV_STRING || ({rt}).kind == DV_STRING) \
             ? (strcmp(dv_to_string({lt}), dv_to_string({rt})) {oper} 0) \
             : (dv_as_double({lt}) {oper} dv_as_double({rt})))"
        )
    }

    fn comp_block(&mut self, nodes: &[Stmt]) {
        for node in nodes {
            match node {
                Stmt::Set { value, oper } => {
                    let val = self.eval_expr(value);
                    match oper {
                        SetOper::Set => self.emit(&format!("dv_set(dv_index, {val});")),
                        SetOper::Add => {
                            self.emit(&format!(
                                "dv_set(dv_index, dv_add(dv_get(dv_index), {val}));"
                            ));
                        }
                        SetOper::Sub => {
                            self.emit(&format!(
                                "dv_set(dv_index, dv_arith(dv_get(dv_index), {val}, '-'));"
                            ));
                        }
                        SetOper::Mul => {
                            self.emit(&format!(
                                "dv_set(dv_index, dv_arith(dv_get(dv_index), {val}, '*'));"
                            ));
                        }
                        SetOper::Div => {
                            self.emit(&format!(
                                "dv_set(dv_index, dv_arith(dv_get(dv_index), {val}, '/'));"
                            ));
                        }
                        SetOper::Mod => {
                            self.emit(&format!(
                                "dv_set(dv_index, dv_arith(dv_get(dv_index), {val}, '%'));"
                            ));
                        }
                    }
                }

                Stmt::Tts {
                    msg,
                    use_index,
                    overlap: _,
                } => {
                    if *use_index {
                        self.emit("dv_tts(dv_get(dv_index));");
                    } else if let Some(expr) = msg {
                        let val = self.eval_expr(expr);
                        self.emit(&format!("dv_tts({val});"));
                    }
                }

                Stmt::Doug { chains, reset } => {
                    let start = if *reset { "0LL" } else { "dv_index" };
                    let idx = Self::doug_index(chains, start);
                    self.emit(&format!("dv_index = {idx};"));
                }

                Stmt::Loop { body } => {
                    self.emit("while (1) {");
                    self.indent += 1;
                    self.comp_block(body);
                    self.indent -= 1;
                    self.emit("}");
                }

                Stmt::Goud => {
                    self.emit("break;");
                }

                Stmt::Rigged { func: func_name, args } => {
                    self.funcs.insert(func_name.clone(), args.len());
                    let call_args: Vec<String> =
                        args.iter().map(|a| self.ffi_arg(a)).collect();
                    let joined = call_args.join(", ");
                    self.emit(&format!(
                        "dv_set(dv_index, dv_make_int((long long){func_name}({joined})));"
                    ));
                }

                Stmt::Prediction {
                    believe_body,
                    doubt_body,
                    condition,
                } => {
                    let cond = self.eval_cond(condition);
                    self.emit(&format!("if ({cond}) {{"));
                    self.indent += 1;
                    self.comp_block(believe_body);
                    self.indent -= 1;
                    self.emit("} else {");
                    self.indent += 1;
                    self.comp_block(doubt_body);
                    self.indent -= 1;
                    self.emit("}");
                }
            }
        }
    }

    pub fn compile(&mut self, nodes: &[Stmt]) -> String {
        self.comp_block(nodes);

        let body = self.lines.join("\n");

        let mut decls = String::new();
        for name in self.funcs.keys() {
            decls.push_str(&format!("extern int {name}();\n"));
        }

        format!(
            "{RUNTIME}\n{decls}\nint main(void) {{\n    dv_ensure(&dv_right, &dv_right_len, 0);\n{body}\n    return 0;\n}}\n"
        )
    }
}
