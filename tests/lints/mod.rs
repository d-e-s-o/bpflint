//! Tests for lints.

// Basic lint validation.
mod validate;

// Tests for individual lints go below here.

#[path = "core-read.rs"]
mod core_read;
#[path = "get-current-task.rs"]
mod get_current_task;
#[path = "perfbuf-usage.rs"]
mod perfbuf_usage;
#[path = "pragma-unroll-for-loop-bounded.rs"]
mod pragma_unroll_for_loop_bounded;
#[path = "probe-read.rs"]
mod probe_read;
#[path = "unstable-attach-point.rs"]
mod unstable_attach_point;
#[path = "untyped-map-member.rs"]
mod untyped_map_member;
