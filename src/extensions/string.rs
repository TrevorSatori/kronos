pub trait StringExtensions {
    fn count_leading_whitespace(&self) -> usize;
    fn trim_leading_whitespace(&self) -> &str;
    fn strip_quotes(&self) -> &str;
}

impl StringExtensions for String {
    fn count_leading_whitespace(&self) -> usize {
        let mut indentation = 0;

        for char in self.chars() {
            if char.is_whitespace() {
                indentation += 1;
            } else {
                break;
            }
        }

        indentation
    }

    fn trim_leading_whitespace(&self) -> &str {
        self.strip_prefix(char::is_whitespace).unwrap_or(self).trim_start()
    }

    fn strip_quotes(&self) -> &str {
        self.trim_matches('"')
    }
}
