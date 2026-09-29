use sasspile::compile_with_files;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let cases: Vec<(&str, HashMap<&str, &str>, &str)> = vec![
        // 基本 @use 变量引用 (单引号)
        (
            "@use 'other';\na {b: other.$member}",
            HashMap::from([("other.scss", "$member: value;")]),
            "sq_use_var_ref",
        ),
        // 双引号 @use
        (
            "@use \"other\";\na {b: other.$member}",
            HashMap::from([("other.scss", "$member: value;")]),
            "dq_use_var_ref",
        ),
        // @use + function 调用
        (
            "@use 'other';\na {b: other.member()}",
            HashMap::from([("other.scss", "@function member() { @return value }")]),
            "sq_use_fn_call",
        ),
        // @use + mixin include
        (
            "@use 'other';\n@include other.member;",
            HashMap::from([("other.scss", "@mixin member() {a {b: c}}")]),
            "sq_use_mixin_include",
        ),
        // @use ... with() 配置
        (
            "@use 'other' with ($member: configured);\na {b: other.$member}",
            HashMap::from([("other.scss", "$member: value;")]),
            "sq_use_with_config",
        ),
        // basename: URL 带路径
        (
            "@use 'foo/bar/../baz/qux/other';\na {b: other.$variable}",
            HashMap::from([("foo/baz/qux/other.scss", "$variable: value;")]),
            "path_basename",
        ),
        // without_extensions: URL 带多个扩展名
        (
            "@use 'other.foo.bar.baz.scss';\na {b: other.$variable}",
            HashMap::from([("other.foo.bar.baz.scss", "$variable: value;")]),
            "without_extensions",
        ),
        // without_underscore: URL 带下划线
        (
            "@use '_other';\na {b: other.$variable}",
            HashMap::from([("_other.scss", "$variable: value;")]),
            "without_underscore",
        ),
    ];

    for (input, files, desc) in cases {
        let owned: HashMap<String, String> = files
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let output = compile_with_files(input, &owned);
        tracing::info!(case = desc, output = %output, "diag_result");
    }
}
