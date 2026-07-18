use crate::parser::ast::Stmt;

const LAUNCHER_TEMPLATE: &str = include_str!("launcher.c");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
}

impl CompileError {
    fn new(msg: impl Into<String>) -> Self {
        CompileError {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CompileError {}

pub struct Compiler {
    helper_path: Option<String>,
    source: String,
    linked_libs: Vec<String>,
}

impl Compiler {
    pub fn new(
        helper_path: Option<String>,
        source: impl Into<String>,
        linked_libs: Vec<String>,
    ) -> Self {
        Compiler {
            helper_path,
            source: source.into(),
            linked_libs,
        }
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
                '\0' => out.push_str("\\0"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    }

    pub fn compile(
        &mut self,
        _nodes: &[Stmt],
        _linked_libs: &[String],
    ) -> Result<String, CompileError> {
        let Some(helper_path) = &self.helper_path else {
            return Err(CompileError::new(
                "compiled launcher generation needs the douglang executable path",
            ));
        };

        let linked_libs = self
            .linked_libs
            .iter()
            .map(|lib| Self::cstring_literal(lib))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(LAUNCHER_TEMPLATE
            .replace(
                "__DOUGLANG_HELPER_PATH__",
                &Self::cstring_literal(helper_path),
            )
            .replace("__DOUGLANG_SOURCE__", &Self::cstring_literal(&self.source))
            .replace("__DOUGLANG_LINKS__", &linked_libs)
            .replace(
                "__DOUGLANG_LINK_COUNT__",
                &self.linked_libs.len().to_string(),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser};

    fn compile_src(source: &str) -> String {
        let ast = parser::parse(&lexer::lex(source).unwrap()).unwrap();
        let mut compiler = Compiler::new(
            Some("C:\\fake path\\douglang.exe".to_string()),
            source.to_string(),
            Vec::new(),
        );
        compiler.compile(&ast, &[]).unwrap()
    }

    #[test]
    fn generated_artifact_is_rust_runtime_launcher() {
        let c = compile_src("Bald set 1 tts");
        assert!(c.contains("--run-source-helper"));
        assert!(c.contains("DOUGLANG_SOURCE"));
        assert!(c.contains("DOUGLANG_HELPER_PATH"));
        assert!(!c.contains("static DougValue dv_add"));
        assert!(!c.contains("static long long dv_doug_index"));
    }
}
