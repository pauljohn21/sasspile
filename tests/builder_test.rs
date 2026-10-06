use rx_scss::builder::{CompileBuilder, SchedulerConfig};
use rx_scss::types::{InputSyntax, OutputStyle};

#[test]
fn builder_default_values() {
    let _b = CompileBuilder::new();
    // Default is Scss + Expanded (can't assert pub(crate) fields directly from integration test)
}

#[test]
fn builder_chain_syntax() {
    let _b = CompileBuilder::new()
        .syntax(InputSyntax::Css);
}

#[test]
fn builder_chain_scss_shorthand() {
    let _b = CompileBuilder::new().scss();
}

#[test]
fn builder_chain_compressed() {
    let _b = CompileBuilder::new().compressed();
}

#[test]
fn builder_chain_expanded() {
    let _b = CompileBuilder::new().expanded();
}

#[test]
fn builder_chain_nested() {
    let _b = CompileBuilder::new().nested();
}

#[test]
fn builder_include_path() {
    let _b = CompileBuilder::new()
        .include_path("/tmp/scss")
        .include_path("/var/lib/scss");
}

#[test]
fn builder_scheduler() {
    let _b = CompileBuilder::new()
        .scheduler(SchedulerConfig::ThreadPool(4));
}

#[test]
fn builder_single_thread_scheduler() {
    let _b = CompileBuilder::new()
        .scheduler(SchedulerConfig::SingleThread);
}

#[test]
fn builder_compile_string_works() {
    let result = CompileBuilder::new()
        .compile_string("body { color: red; }");
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("body"));
    assert!(css.contains("color: red;"));
}

#[test]
fn builder_compile_compressed() {
    let result = CompileBuilder::new()
        .compressed()
        .compile_string(".a { color: blue; }");
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(!css.contains('\n'));
}

#[test]
fn builder_build_session() {
    let _session = CompileBuilder::new()
        .expanded()
        .build();
}

#[test]
fn builder_full_chain() {
    let _ = CompileBuilder::new()
        .syntax(InputSyntax::Scss)
        .compressed()
        .include_path("./scss")
        .scheduler(SchedulerConfig::ThreadPool(2));
}

#[test]
fn test_send_sync_static() {
    fn assert_send<T: Send + Sync + 'static>() {}
    assert_send::<CompileBuilder>();
}

#[test]
fn builder_compile_from_path() {
    use std::io::Write;
    let mut tmp = std::env::temp_dir();
    tmp.push("rx_scss_test_tmp.scss");

    {
        let mut f = std::fs::File::create(&tmp).unwrap();
        write!(f, "p {{ font-size: 14px; }}").unwrap();
    }

    let result = CompileBuilder::new().compile_file(&tmp);
    let _ = std::fs::remove_file(&tmp);

    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("font-size: 14px;"));
}
