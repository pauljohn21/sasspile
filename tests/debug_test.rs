use rxrust::prelude::*;
use rxrust::observable::Observable;
use rx_scss::eval::eval_stream;
use rx_scss::lexer::scan;
use rx_scss::parser::parse_stream;
use rx_scss::runtime::create_runtime;

#[test]
fn debug_full_pipeline() {
    let input = "body { color: red; }";

    println!("=== Step 1: Lexer ===");
    let tokens = scan(input);
    let token_list = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let tl = token_list.clone();
    tokens.subscribe(move |tok| {
        tl.lock().unwrap().push(tok);
    });
    let toks = token_list.lock().unwrap();
    println!("Tokens count: {}", toks.len());
    for (i, t) in toks.iter().enumerate() {
        println!("  [{}]: {:?}", i, t);
    }
    drop(toks);

    println!("=== Step 2: Parser ===");
    let (ctx, _bus) = create_runtime();
    let tokens2 = scan(input);
    let ast = parse_stream(tokens2, ctx.scope_id());
    let ast_list = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let al = ast_list.clone();
    ast.subscribe(move |node| {
        al.lock().unwrap().push(node);
    });
    let nodes = ast_list.lock().unwrap();
    println!("AST nodes count: {}", nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        println!("  [{}]: {:?}", i, n);
    }
    drop(nodes);

    println!("=== Step 3: Eval ===");
    let (ctx2, _bus2) = create_runtime();
    let tokens3 = scan(input);
    let ast3 = parse_stream(tokens3, ctx2.scope_id());
    let css = eval_stream(ast3, ctx2);
    let css_list = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let cl = css_list.clone();
    css.subscribe(move |stmt| {
        cl.lock().unwrap().push(stmt);
    });
    let stmts = css_list.lock().unwrap();
    println!("CSS statements count: {}", stmts.len());
    for (i, s) in stmts.iter().enumerate() {
        println!("  [{}]: {:?}", i, s);
    }
}
