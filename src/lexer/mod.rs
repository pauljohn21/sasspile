mod state;
pub use state::LexerState;
use rxrust::prelude::*;
use crate::types::*;

pub fn scan(source: &str) -> TokenStream {
    let src = source.to_string();
    Shared::create(move |subscriber| {
        let mut st = LexerState::new();
        for ch in src.chars() {
            let toks = st.feed(ch);
            for t in toks {
                if !matches!(t, Token::Whitespace) {
                    subscriber.next(t);
                }
            }
        }
        // flush remaining buf, then resolve any trailing pending_slash as Slash
        let mut final_toks = Vec::new();
        st.flush_buf_checked(&mut final_toks);
        if st.take_pending_slash() {
            final_toks.push(Token::Slash);
        }
        for t in final_toks {
            if !matches!(t, Token::Whitespace) {
                subscriber.next(t);
            }
        }
        subscriber.next(Token::Eof);
        subscriber.complete();
    }).box_it()
}
