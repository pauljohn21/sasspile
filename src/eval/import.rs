//! @import 指令处理。

use super::*;
use std::path::Path;

impl Evaluator {
    /// @import 指令处理。
    pub(crate) fn eval_import(url: &str, modifier: &str, env: Env) -> Result<(Vec<CssNode>, Env)> {
        match url.starts_with("sass:") {
            true => return Ok((vec![], env.add_module(url.to_string()))),
            false => {}
        }
        let is_css = Path::new(url)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("css"))
            || url.starts_with("https://")
            || url.starts_with("http://")
            || url.starts_with("//")
            || url.starts_with("url(")
            || !modifier.is_empty()
            || url.split(", ").any(|u| {
                Path::new(u.trim_matches('"'))
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("css"))
            });
        match is_css {
            true => {
            let urls: Vec<&str> = url.split(", ").collect();
            let nodes: Vec<CssNode> = urls
                .iter()
                .map(|u| {
                    let u = u.trim_matches('"');
                    // http/https URL 用 url() 形式
                    let is_url = u.starts_with("http://") || u.starts_with("https://") || u.starts_with("//");
                    let params = match (is_url, modifier.is_empty()) {
                        (true, _) => format!("url({u})"),
                        (false, true) => format!("\"{u}\""),
                        (false, false) => format!("\"{u}\" {modifier}"),
                    };
                    CssNode::AtRule {
                        name: "import".to_string(),
                        params: Some(params),
                        children: vec![],
                        has_body: false,
                    }
                })
                .collect();
            return Ok((nodes, env));
            }
            false => {}
        }
        let base = env.get_base_path();
        let load_paths = env.get_load_paths().to_vec();
        // @import 文件歧义检测
        Self::check_resolve_ambiguity(base, url, &load_paths)?;
        if let Some(path) = Self::resolve_file_import(base, url, &load_paths) {
            return Self::load_import(&path, env);
        }
        if !(Path::new(url)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("css")))
            && !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("url(")
            && modifier.is_empty()
        {
            return Err(SassError::Module(format!(
                "Can't find stylesheet to import: {url}"
            )));
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
