use std::fmt;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ProductRole {
    Foundation,
    Creator,
    Viewer,
    Developer,
    Research,
    Learning,
}

impl fmt::Display for ProductRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ProductRole::Foundation => "foundation",
            ProductRole::Creator => "creator",
            ProductRole::Viewer => "viewer",
            ProductRole::Developer => "developer",
            ProductRole::Research => "research",
            ProductRole::Learning => "learning",
        };

        formatter.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct TenchProduct {
    pub slug: &'static str,
    pub name: &'static str,
    pub role: ProductRole,
    pub output_kind: &'static str,
    pub summary: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RuntimePolicy {
    pub local_first: bool,
    pub allows_cloud_fallback: bool,
    pub prompt_logging: bool,
    pub metrics_scope: &'static str,
}

pub const RUNTIME_POLICY: RuntimePolicy = RuntimePolicy {
    local_first: true,
    allows_cloud_fallback: true,
    prompt_logging: false,
    metrics_scope: "hardware, runtime, model, speed, resource, and error metrics only",
};

pub const PRODUCTS: [TenchProduct; 13] = [
    TenchProduct {
        slug: "tench-engine",
        name: "tench engine",
        role: ProductRole::Foundation,
        output_kind: "shared backend",
        summary: "local model runtime and cloud provider bridge",
    },
    TenchProduct {
        slug: "tench-view",
        name: "View",
        role: ProductRole::Viewer,
        output_kind: "desktop/mobile app",
        summary: "fast image viewing, folders, tags, and AI classification",
    },
    TenchProduct {
        slug: "tench-pixel-design",
        name: "Pixel Design",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "image editing with AI assisted creation and correction",
    },
    TenchProduct {
        slug: "tench-player",
        name: "Player",
        role: ProductRole::Viewer,
        output_kind: "desktop/mobile app",
        summary: "fast video playback, playlists, subtitles, and scene summaries",
    },
    TenchProduct {
        slug: "tench-composer",
        name: "Composer",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "video cutting, subtitles, scene analysis, and export pipeline",
    },
    TenchProduct {
        slug: "tench-code",
        name: "tench code",
        role: ProductRole::Developer,
        output_kind: "desktop/mobile app",
        summary: "project indexing, code editing, review, and local context QA",
    },
    TenchProduct {
        slug: "tench-story",
        name: "tench story",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "long-form writing, story planning, and AI assisted drafting",
    },
    TenchProduct {
        slug: "tench-universe",
        name: "tench universe",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "character chat, scenario worlds, memory, and interactive story modes",
    },
    TenchProduct {
        slug: "tench-research",
        name: "tench research",
        role: ProductRole::Research,
        output_kind: "desktop/mobile app",
        summary: "web research, source summaries, reports, and approved actions",
    },
    TenchProduct {
        slug: "tench-study",
        name: "tench study",
        role: ProductRole::Learning,
        output_kind: "desktop/mobile app",
        summary: "concept explanations, review queues, notes, and learning plans",
    },
    TenchProduct {
        slug: "tench-docs",
        name: "tench docs",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "local-first AI assisted word processor and document editor",
    },
    TenchProduct {
        slug: "tench-sheets",
        name: "tench sheets",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "local-first AI assisted spreadsheets, formulas, and data analysis",
    },
    TenchProduct {
        slug: "tench-slides",
        name: "tench slides",
        role: ProductRole::Creator,
        output_kind: "desktop/mobile app",
        summary: "local-first AI assisted presentations, slides, and speaker tools",
    },
];
