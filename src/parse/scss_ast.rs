//! —— SCSS AST ——
//!
//! 完整 Sass 特性：变量、控制流、mixin、函数、模块系统。
//! .scss / .sass 文件解析产出此类型。

// 暂通过 type alias 指向原类型，Phase B 将独立为完整 enum。
// 此时已建立独立模块边界，后续可无缝替换为独立定义。

pub use super::ast::{BinOp, BinOpKind, ConfigVar, FunctionRefData, InterpSegment};
pub use super::ast::{MixinRefData, Param, Separator, UnaryOp, VarFlags};

use super::ast;

/// SCSS 语法树——.scss / .sass 文件解析产出。
pub type ScssAst = ast::Ast;

/// SCSS 语法树节点——完整 Sass 特性。
pub type ScssNode = ast::Node;

/// SCSS 值表达式——完整 Sass 表达式。
pub type ScssValue = ast::Value;

/// SCSS 函数调用参数。
pub type ScssArg = ast::Arg;

/// SCSS mixin 引用数据。
pub type ScssMixinRefData = ast::MixinRefData;

/// SCSS 函数引用数据。
pub type ScssFunctionRefData = ast::FunctionRefData;
