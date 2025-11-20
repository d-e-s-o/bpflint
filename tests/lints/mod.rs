//! Tests for lints.

// Basic lint validation.
mod validate;

// Tests for individual lints go below here.

#[path = "bpf-loop.rs"]
mod bpf_loop;
#[path = "core-read.rs"]
mod core_read;
#[path = "get-current-task.rs"]
mod get_current_task;
#[path = "perfbuf-usage.rs"]
mod perfbuf_usage;
#[path = "probe-read.rs"]
mod probe_read;
#[path = "unrolled-for-loop.rs"]
mod unrolled_for_loop;
#[path = "unstable-attach-point.rs"]
mod unstable_attach_point;
#[path = "untyped-map-member.rs"]
mod untyped_map_member;
