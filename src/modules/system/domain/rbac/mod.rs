pub mod entity;
pub mod repository;

pub use entity::{Permission, Role};
pub use repository::{
    CreatePermissionCmd, CreateRoleCmd, RbacRepository, UpdatePermissionCmd, UpdateRoleCmd,
};
