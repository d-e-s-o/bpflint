//! Tests for lints.

// Basic lint validation.
mod validate;

// Tests for individual lints go below here.

#[path = "get-current-task.rs"]
mod get_current_task;
#[path = "probe-read.rs"]
mod probe_read;
#[path = "unstable-attach-point.rs"]
mod unstable_attach_point;
#[path = "untyped-map-member.rs"]
mod untyped_map_member;
