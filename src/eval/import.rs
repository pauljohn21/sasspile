//! —— @import 指令处理 ——
//!
//! 概要：处理 `@import` 指令，加载并内联引入其他 SCSS 文件。
//!
//! ## 核心概念
//! - 路径解析与文件查找
//! - 避免循环引用
//! - 将引入文件内容嵌入当前模块

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
            // 路径解析后再次检查后缀：resolve 返回的 .css 文件不应走 SCSS 管线。
            // 即使 URL 匹配了 is_css 检测，当 resolve 找到磁盘文件时仍需确认：
            // @import "existing.css" 应生成 @import url(...) 节点，不调用 load_import。
            let path_is_css = path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("css"));
            match path_is_css {
                true => {
                    let params = format!("\"{}\"", path.display());
                    return Ok((
                        vec![CssNode::AtRule {
                            name: "import".to_string(),
                            params: Some(params),
                            children: vec![],
                            has_body: false,
                        }],
                        env,
                    ));
                }
                false => {}
            }
            // 保存父选择器状态（env 在 load_import 中被 move）
            let (css, env) = Self::load_import(&path, env)?;
            return Ok((css, env));
        }
        let is_unresolved_scss = !(Path::new(url)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("css")))
            && !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("url(")
            && modifier.is_empty();
        match is_unresolved_scss {
            true => {
                return Err(SassError::Module(format!(
                    "Can't find stylesheet to import: {url}"
                )));
            }
            false => {}
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
