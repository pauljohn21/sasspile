//! LexerState — 词法分析内部状态

/// 词法分析状态
#[derive(Debug, Clone)]
pub struct LexerState {
    /// 当前位置
    pub pos: u32,
    /// 行号（从 1 开始）
    pub line: u32,
    /// 列号（从 1 开始）
    pub col: u32,
}

impl LexerState {
    /// 创建初始状态
    pub fn new() -> Self {
        Self {
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// 前进一个字符，更新位置
    pub fn advance(&mut self, c: char) {
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    /// 查看当前位置（不消费）
    pub fn peek(&self) -> u32 {
        self.pos
    }
}

impl Default for LexerState {
    fn default() -> Self {
        Self::new()
    }
}
