pub trait StripCodeBlock {
    fn strip_code_block(&self) -> &str;
}

impl StripCodeBlock for str {
    fn strip_code_block(&self) -> &str {
        let trimmed = self.trim();
        if trimmed.starts_with("```") {
            if let Some(pos) = trimmed.find('\n') {
                let inner = &trimmed[pos + 1 ..];
                if inner.ends_with("```") {
                    let inner = &inner[.. inner.len() - 3];
                    return inner.trim();
                }
            }
        }
        trimmed
    }
}
