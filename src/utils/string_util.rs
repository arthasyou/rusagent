pub trait StripCodeBlock {
    fn strip_code_block(&self) -> &str;
}

impl StripCodeBlock for str {
    fn strip_code_block(&self) -> &str {
        let trimmed = self.trim();
        if trimmed.starts_with("```")
            && let Some(pos) = trimmed.find('\n') {
                let inner = &trimmed[pos + 1 ..];
                if let Some(inner) = inner.strip_suffix("```") {
                    return inner.trim();
                }
            }
        trimmed
    }
}
