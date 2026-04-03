//! Intermediate Representation (IR) Module
//!
//! This module defines the core data structures that represent
//! a skill in a structured, typed form.

mod builder;
mod constraint;
mod example;
mod permission;
mod procedure;
mod skill_ir;

pub use builder::build_ir;
pub use constraint::{Constraint, ConstraintLevel, ConstraintScope};
pub use example::{Example, ExampleDifficulty};
pub use permission::{Permission, PermissionKind};
pub use procedure::{ErrorHandlingStrategy, ProcedureStep};
pub use skill_ir::{SecurityLevel, SkillIR};
