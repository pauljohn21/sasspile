use sasspile::parse;

#[test]
fn linear_gradient_to() {
    let cases = vec![
        ("to_dir", "background: linear-gradient(to right, red, blue);\n"),
        ("to_dir_ml", "background: linear-gradient(\n  to right,\n  red,\n  blue\n);\n"),
        ("to_var", "background: linear-gradient(to $side, red, blue);\n"),
        ("to_only", "background: linear-gradient(\n  to $side,\n  red 0%,\n  blue 100%\n);\n"),
    ];
    
    let mut results = Vec::new();
    for (name, content) in cases {
        let (_, d) = parse(content);
        let (e, _, _) = d.counts();
        results.push(format!("[{}] {name}: {} errs: {:?}", 
            if e == 0 {"OK"} else {"FAIL"},
            e,
            d.errors().iter().map(|x| &x.message).collect::<Vec<_>>()));
    }
    let _ = std::fs::write("/tmp/cp_to.log", results.join("\n"));
}
