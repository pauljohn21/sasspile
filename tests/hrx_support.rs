//! 内联 HRX 解析支持模块——替代 hrx-auditor crate 依赖。
//!
//! 提供三个核心功能：
//! - `parse_hrx()` — 解析 HRX 文本为 `HrxArchive`
//! - `Vfs::from_archive()` — 从归档构建虚拟文件系统

#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
//! - `ParsedHrx` — 解析后的测试用例集合
//!
//! 每个测试文件是独立 crate，`dead_code` lint 会误报——全局抑制。

#![allow(dead_code)]

use std::collections::BTreeMap;
use tracing::{info, info_span};

// ─── HRX 解析（从 hrx-auditor 抄过来，移除 anyhow 依赖） ──────────────────

/// HRX 文件中的一条条目
#[derive(Debug, Clone)]
pub struct HrxEntry {
    /// 完整虚拟路径，如 "multi/leading/input.scss"
    pub path: String,
    /// 文件内容（目录分隔段为空 String）
    pub body: String,
}

/// HRX 文件解析后的归档
#[derive(Debug, Clone)]
pub struct HrxArchive {
    pub entries: Vec<HrxEntry>,
}

impl HrxArchive {
    /// 获取所有文件路径（不含目录分隔段）
    #[allow(dead_code)]
    #[must_use]
    pub fn file_paths(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| !e.path.is_empty())
            .map(|e| e.path.as_str())
            .collect()
    }

    /// 获取指定路径的文件内容
    #[allow(dead_code)]
    #[must_use]
    pub fn get_file(&self, path: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.path == path)
            .map(|e| e.body.as_str())
    }
}

/// HRX 分隔符前缀
const FILE_HEADER: &str = "<===> ";
const DIR_SEPARATOR: &str = "<===>";

/// 解析 HRX 文本内容。
///
/// HRX 格式:
///   <===> path/to/file
///   file body...
///   <===> another/file
///   ...
///   <===>
///   ================================================================================
///   (目录分隔段，无路径)
///
/// 返回 `Result<HrxArchive, String>` 以兼容旧 `hrx_auditor` 调用模式。
pub fn parse_hrx(content: &str) -> Result<HrxArchive, String> {
    let _span = info_span!("parse_hrx", stage = "parse", module = "vfs").entered();

    let mut entries = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line == DIR_SEPARATOR {
            // 目录分隔段：<===> 后跟一行 80 个 =
            if lines.peek().is_some_and(|next| next.starts_with("==")) {
                lines.next();
            }
            entries.push(HrxEntry {
                path: String::new(),
                body: String::new(),
            });
            continue;
        }

        if let Some(rest) = line.strip_prefix(FILE_HEADER) {
            let path = rest.trim().to_string();

            if path.is_empty() {
                continue;
            }

            // 收集 body 直到下一个 <===> 或文件结束
            let mut body_lines = Vec::new();
            while let Some(body_line) = lines.peek() {
                if body_line.starts_with(FILE_HEADER) || *body_line == DIR_SEPARATOR {
                    break;
                }
                body_lines.push(*body_line);
                lines.next();
            }

            // HRX 约定：body 末尾有一个换行，但最后一个条目可能没有
            let body = body_lines.join("\n");

            entries.push(HrxEntry { path, body });
        }
    }

    let entry_count = entries.len();
    info!(entry_count, "hrx parsed");

    if entry_count == 0 {
        return Err("no entries found in HRX content".to_string());
    }

    Ok(HrxArchive { entries })
}

// ─── VFS（从 hrx-auditor 抄过来） ─────────────────────────────────────────

/// VFS 节点
#[derive(Debug, Clone)]
pub struct VfsNode {
    pub name: String,
    pub children: BTreeMap<String, VfsNode>,
    pub files: Vec<(String, String)>, // (filename, body)
}

impl VfsNode {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: BTreeMap::new(),
            files: Vec::new(),
        }
    }
}

/// VFS 根节点
pub struct Vfs {
    pub root: VfsNode,
}

impl Vfs {
    /// 从 `HrxArchive` 构建 VFS 树
    #[must_use]
    pub fn from_archive(archive: &HrxArchive) -> Self {
        let mut root = VfsNode::new(".");

        for entry in &archive.entries {
            if entry.path.is_empty() {
                continue; // 跳过目录分隔段
            }

            let parts: Vec<&str> = entry.path.split('/').collect();
            if parts.len() == 1 {
                // 根级文件
                root.files.push((entry.path.clone(), entry.body.clone()));
            } else {
                // 嵌套文件：遍历目录部分
                let (dirs, file_name) = parts.split_at(parts.len() - 1);
                let mut current = &mut root;

                for dir in dirs {
                    current = current
                        .children
                        .entry(dir.to_string())
                        .or_insert_with(|| VfsNode::new(dir));
                }

                current
                    .files
                    .push((file_name[0].to_string(), entry.body.clone()));
            }
        }

        Self { root }
    }

    /// 计算最大嵌套深度
    #[allow(dead_code)]
    #[must_use]
    pub fn max_depth(&self) -> usize {
        Self::node_depth(&self.root)
    }

    fn node_depth(node: &VfsNode) -> usize {
        if node.children.is_empty() {
            0
        } else {
            1 + node
                .children
                .values()
                .map(Self::node_depth)
                .max()
                .unwrap_or(0)
        }
    }

    /// 递归遍历，返回 (`dir_path`, files) 列表
    #[must_use]
    pub fn walk(&self) -> Vec<(String, Vec<(String, String)>)> {
        let mut result = Vec::new();
        Self::walk_node(&self.root, ".", &mut result);
        result
    }

    fn walk_node(
        node: &VfsNode,
        current_path: &str,
        result: &mut Vec<(String, Vec<(String, String)>)>,
    ) {
        result.push((current_path.to_string(), node.files.clone()));

        for (name, child) in &node.children {
            let new_path = if current_path == "." {
                name.clone()
            } else {
                format!("{current_path}/{name}")
            };
            Self::walk_node(child, &new_path, result);
        }
    }
}

// ─── 测试用例解析 ─────────────────────────────────────────────────────────

/// 测试用例——包含所有文件和期望输出。
pub struct HrxCase {
    pub files: Vec<(String, String)>,
    pub input_path: String,
    pub expected_output: String,
    pub expect_error: bool,
}

/// 解析 HRX 内容为测试用例列表。
///
/// 所有文件共享同一个 VFS——不按 `===` 分组隔离。
/// 文件路径加上 HRX 文件所在目录作为前缀，
/// 使 `@use 'callable/arguments/mixin/utils'` 等绝对路径能正确解析。
pub fn parse_hrx_to_cases(content: &str, hrx_rel_path: &str) -> Vec<HrxCase> {
    let span = info_span!("parse_hrx", hrx = %hrx_rel_path);
    let _enter = span.enter();

    // 从 HRX 相对路径提取目录前缀：`callable/arguments.hrx` → `callable/arguments`
    let prefix = hrx_rel_path.strip_suffix(".hrx").unwrap_or(hrx_rel_path);

    let Ok(archive) = parse_hrx(content) else {
        return Vec::new();
    };
    let vfs = Vfs::from_archive(&archive);
    let dirs = vfs.walk();

    // 展平所有 .scss/.css/.sass 文件——加上 HRX 目录前缀
    let all_files: Vec<(String, String)> = dirs
        .iter()
        .flat_map(|(dir_path, files)| {
            let dp = dir_path.clone();
            files.iter().map(move |(f, c)| {
                let base = if dp == "." {
                    f.clone()
                } else {
                    format!("{dp}/{f}")
                };
                let prefixed = if prefix.is_empty() {
                    base
                } else {
                    format!("{prefix}/{base}")
                };
                (prefixed, c.clone())
            })
        })
        .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass"))
        .collect();

    let mut cases = Vec::new();
    for (dir_path, files) in &dirs {
        // 找 input.scss 或 input.sass
        let input_file = files
            .iter()
            .find(|(f, _)| f == "input.scss" || f == "input.sass");

        if input_file.is_none() {
            continue;
        }

        let (input_name, _) = input_file.unwrap();

        let input_base = if dir_path == "." {
            input_name.clone()
        } else {
            format!("{dir_path}/{input_name}")
        };
        let input_path = if prefix.is_empty() {
            input_base
        } else {
            format!("{prefix}/{input_base}")
        };

        // 查找同目录下的 output.css 和 error
        let expected_output = files
            .iter()
            .find(|(f, _)| f == "output.css")
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        let expect_error = files.iter().any(|(f, _)| f == "error");

        cases.push(HrxCase {
            files: all_files.clone(),
            input_path,
            expected_output,
            expect_error,
        });
    }
    cases
}

/// 运行单个测试用例——写入临时目录并编译。
pub fn run_case(case: &HrxCase) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 {
        return false;
    }

    let input = &case.input_path;
    let n_files = case.files.len();
    let expect_error = case.expect_error;
    let span = info_span!("sass_spec_case", input = %input, n_files, expect_error);
    let _enter = span.enter();

    let tmp_dir = std::env::temp_dir().join(format!(
        "sass-spec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        vec![tmp_dir.clone()],
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if case.expect_error {
        result.is_err()
    } else {
        match result {
            Ok(actual) => actual.trim() == case.expected_output.trim(),
            Err(_) => false,
        }
    }
}

/// 兼容旧接口：`ParsedHrx` 返回 `(files, input_path, expected_output, expect_error)` 列表。
///
/// 用于 `diag_detail.rs`、`cf_diag.rs` 等需要按 `===` 分组访问的测试。
pub struct ParsedCase {
    pub files: Vec<(String, String)>,
    pub input_path: String,
    pub expected_output: String,
    pub expect_error: bool,
}

/// 解析 HRX 为 `ParsedCase` 列表（兼容旧 `ParsedHrx` 接口）。
#[must_use]
pub fn parse_hrx_legacy(content: &str) -> Vec<ParsedCase> {
    let Ok(archive) = parse_hrx(content) else {
        return Vec::new();
    };
    let vfs = Vfs::from_archive(&archive);
    let dirs = vfs.walk();

    // 展平所有文件
    let all_files: Vec<(String, String)> = dirs
        .iter()
        .flat_map(|(dir_path, files)| {
            let dp = dir_path.clone();
            files.iter().map(move |(f, c)| {
                let path = if dp == "." {
                    f.clone()
                } else {
                    format!("{dp}/{f}")
                };
                (path, c.clone())
            })
        })
        .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass"))
        .collect();

    let mut cases = Vec::new();
    for (dir_path, files) in &dirs {
        let input_file = files
            .iter()
            .find(|(f, _)| f == "input.scss" || f == "input.sass");

        if input_file.is_none() {
            continue;
        }

        let (input_name, _) = input_file.unwrap();

        let input_path = if dir_path == "." {
            input_name.clone()
        } else {
            format!("{dir_path}/{input_name}")
        };

        let expected_output = files
            .iter()
            .find(|(f, _)| f == "output.css")
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        let expect_error = files.iter().any(|(f, _)| f == "error");

        cases.push(ParsedCase {
            files: all_files.clone(),
            input_path,
            expected_output,
            expect_error,
        });
    }
    cases
}
