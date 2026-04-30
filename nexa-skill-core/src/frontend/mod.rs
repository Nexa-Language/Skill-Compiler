//! Frontend Module
//!
//! This module handles parsing SKILL.md files into RawAST.
//!
//! # Components
//!
//! - `frontmatter`: Parse YAML frontmatter
//! - `markdown`: Parse Markdown body using pulldown-cmark
//! - `ast`: Build RawAST from parsed components

mod ast;
mod frontmatter;
mod markdown;

pub use ast::{ASTBuilder, RawAST};
pub use frontmatter::{FrontmatterMeta, extract_frontmatter};
pub use markdown::{MarkdownBody, RawApproach, RawProcedureStep, SectionKind, classify_section, parse_markdown_body};
