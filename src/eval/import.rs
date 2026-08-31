//! @import 指令处理。

use super::*;

impl Evaluator {
    /// @import 指令处理。
    pub(crate) fn eval_import(
        url: &str,
        modifier: &str,
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        if url.starts_with("sass:") {
            return Ok((vec![], env.add_module(url.to_string())));
        }
        let is_css = url.ends_with(".css")
            || url.starts_with("https://")
            || url.starts_with("url(")
            || !modifier.is_empty()
            || url.split("\", \"").any(|u| u.trim_matches('"').ends_with(".css"));
        if is_css {
            let urls: Vec<&str> = url.split("\", \"").collect();
            let mut nodes = Vec::new();
            for u in &urls {
                let u = u.trim_matches('"');
                let params = if modifier.is_empty() {
                    format!("\"{u}\"")
                } else {
                    format!("\"{u}\" {modifier}")
                };
                nodes.push(CssNode::AtRule {
                    name: "import".to_string(),
                    params: Some(params),
                    children: vec![],
                    has_body: false,
                });
            }
            return Ok((nodes, env));
        }
        let base = env.get_base_path();
        let load_paths = env.get_load_paths().to_vec();
        // @import 文件歧义检测
        Self::check_resolve_ambiguity(base, url, &load_paths)?;
        if let Some(path) = Self::resolve_file(base, url, &load_paths) {
            return Self::load_import(&path, env);
        }
        if !url.ends_with(".css") && !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("url(") && modifier.is_empty() {
            return Err(SassError::Module(format!("Can't find stylesheet to import: {url}")));
        }
        let params = if modifier.is_empty() {
            format!("\"{url}\"")
        } else {
            format!("\"{url}\" {modifier}")
        };
        Ok((
            vec![CssNode::AtRule {
                name: "import".to_string(),
                params: Some(params),
                children: vec![],
                has_body: false,
            }],
            env,
        ))
    }
}
