// SQL formatting utilities for future enhancements

pub struct SqlFormatter;

impl SqlFormatter {
    pub fn format_identifier(name: &str) -> String {
        format!("`{name}`")
    }

    pub fn escape_string(s: &str) -> String {
        s.replace("'", "''")
    }

    pub fn format_string_literal(s: &str) -> String {
        format!("'{}'", Self::escape_string(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_identifier() {
        assert_eq!(SqlFormatter::format_identifier("test"), "`test`");
        assert_eq!(
            SqlFormatter::format_identifier("table name"),
            "`table name`"
        );
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(SqlFormatter::escape_string("test"), "test");
        assert_eq!(SqlFormatter::escape_string("test's"), "test''s");
        assert_eq!(SqlFormatter::escape_string("'quoted'"), "''quoted''");
    }

    #[test]
    fn test_format_string_literal() {
        assert_eq!(SqlFormatter::format_string_literal("test"), "'test'");
        assert_eq!(SqlFormatter::format_string_literal("test's"), "'test''s'");
    }
}
