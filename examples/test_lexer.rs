use chamber_lexer::{Lexer, token_text};

fn main() {
    let source = "!trill!C";
    let tokens = Lexer::new(source).tokenize();
    
    for token in &tokens {
        let text = token_text(source, token);
        println!("{:?}: {:?} (len={})", token.kind, text, text.len());
    }
}
