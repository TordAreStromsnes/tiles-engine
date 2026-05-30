use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tiles_renderer::{PreviewSnapshot, SpriteBatch, SpriteInstance};

use crate::{
    RuntimeSafetyBudgetCheck, RuntimeSafetyBudgetProfile, RuntimeSafetyBudgetUsage,
    RuntimeSafetyBudgetViolation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaytestDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaytestSnapshotDiagnostic {
    pub severity: PlaytestDiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaytestSnapshotValidationReport {
    pub launch_allowed: bool,
    pub safety_budget_profile_id: String,
    pub error_count: usize,
    pub warning_count: usize,
    pub usage: RuntimeSafetyBudgetUsage,
    pub budget_check: RuntimeSafetyBudgetCheck,
    pub diagnostics: Vec<PlaytestSnapshotDiagnostic>,
}

impl PlaytestSnapshotDiagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: PlaytestDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            field: None,
            actual: None,
            limit: None,
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: PlaytestDiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            field: None,
            actual: None,
            limit: None,
        }
    }

    fn budget_error(violation: &RuntimeSafetyBudgetViolation) -> Self {
        Self {
            severity: PlaytestDiagnosticSeverity::Error,
            code: "runtimeSafetyBudgetExceeded".to_string(),
            message: format!(
                "Runtime safety budget `{}` is {} but the playtest snapshot estimates {}.",
                violation.field, violation.limit, violation.actual
            ),
            field: Some(violation.field.clone()),
            actual: Some(violation.actual),
            limit: Some(violation.limit),
        }
    }
}

pub fn validate_playtest_snapshot(
    snapshot: &PreviewSnapshot,
    profile: &RuntimeSafetyBudgetProfile,
) -> PlaytestSnapshotValidationReport {
    let usage = estimate_playtest_snapshot_usage(snapshot);
    let budget_check = profile.check_usage(&usage);
    let mut diagnostics = Vec::new();

    if let Err(error) = profile.validate() {
        diagnostics.push(PlaytestSnapshotDiagnostic::error(
            "safetyBudgetProfileInvalid",
            format!("Runtime safety budget profile is invalid: {error}"),
        ));
    }

    if let Err(error) = snapshot.validate() {
        diagnostics.push(PlaytestSnapshotDiagnostic::error(
            "snapshotInvalid",
            format!("Playtest snapshot is invalid: {error}"),
        ));
    }

    if snapshot.source.trim().is_empty() {
        diagnostics.push(PlaytestSnapshotDiagnostic::warning(
            "snapshotSourceMissing",
            "Playtest snapshot is missing source metadata; launch can continue, but editor diagnostics will have less context.",
        ));
    }

    let missing_source_rect_count = snapshot_instance_iter(snapshot)
        .filter(|instance| instance.source.source_rect.is_none())
        .count();
    if missing_source_rect_count > 0 {
        diagnostics.push(PlaytestSnapshotDiagnostic::warning(
            "snapshotSourceRectsMissing",
            format!(
                "{missing_source_rect_count} sprite instance(s) do not include source rectangles; launch can continue, but atlas budget estimates may be conservative."
            ),
        ));
    }

    for violation in &budget_check.violations {
        diagnostics.push(PlaytestSnapshotDiagnostic::budget_error(violation));
    }

    for warning in &budget_check.warnings {
        diagnostics.push(PlaytestSnapshotDiagnostic::warning(
            warning.code.clone(),
            warning.message.clone(),
        ));
    }

    let error_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == PlaytestDiagnosticSeverity::Error)
        .count();
    let warning_count = diagnostics.len().saturating_sub(error_count);

    PlaytestSnapshotValidationReport {
        launch_allowed: error_count == 0,
        safety_budget_profile_id: profile.id.clone(),
        error_count,
        warning_count,
        usage,
        budget_check,
        diagnostics,
    }
}

pub fn estimate_playtest_snapshot_usage(snapshot: &PreviewSnapshot) -> RuntimeSafetyBudgetUsage {
    let texture_dimension_px = snapshot_instance_iter(snapshot)
        .filter_map(|instance| instance.source.source_rect)
        .map(|rect| rect.width.max(rect.height))
        .max()
        .unwrap_or(1);

    let atlas_dimension_px = estimated_atlas_dimension_px(snapshot).max(texture_dimension_px);
    let map_cells = snapshot
        .scene
        .grid
        .columns
        .saturating_mul(snapshot.scene.grid.rows);
    let scene_instances = saturating_usize_to_u32(snapshot.scene_batch.instances.len());
    let overlay_instances = saturating_usize_to_u32(snapshot.editor_overlay_batch.instances.len());
    let total_instances = scene_instances.saturating_add(overlay_instances);

    RuntimeSafetyBudgetUsage {
        texture_dimension_px,
        atlas_dimension_px,
        map_cells,
        entities_per_map: scene_instances,
        active_particles: 0,
        memory_estimate_mb: estimate_memory_mb(atlas_dimension_px, total_instances),
        light_emitters: 0,
        rule_evaluations_per_tick: 0,
    }
}

fn estimated_atlas_dimension_px(snapshot: &PreviewSnapshot) -> u32 {
    let mut atlas_extents: HashMap<&str, (u32, u32)> = HashMap::new();

    for instance in snapshot_instance_iter(snapshot) {
        let Some(rect) = instance.source.source_rect else {
            continue;
        };
        let width = rect.x.saturating_add(rect.width);
        let height = rect.y.saturating_add(rect.height);
        let extent = atlas_extents
            .entry(instance.source.atlas_id.as_str())
            .or_insert((0, 0));
        extent.0 = extent.0.max(width);
        extent.1 = extent.1.max(height);
    }

    atlas_extents
        .values()
        .map(|(width, height)| (*width).max(*height))
        .max()
        .unwrap_or(1)
}

fn snapshot_instance_iter(snapshot: &PreviewSnapshot) -> impl Iterator<Item = &SpriteInstance> {
    batch_instance_iter(&snapshot.scene_batch)
        .chain(batch_instance_iter(&snapshot.editor_overlay_batch))
}

fn batch_instance_iter(batch: &SpriteBatch) -> impl Iterator<Item = &SpriteInstance> {
    batch.instances.iter()
}

fn estimate_memory_mb(atlas_dimension_px: u32, total_instances: u32) -> u32 {
    const BYTES_PER_PIXEL_RGBA8: u64 = 4;
    const INSTANCE_OVERHEAD_BYTES: u64 = 128;
    const BYTES_PER_MB: u64 = 1024 * 1024;

    let atlas_dimension = u64::from(atlas_dimension_px);
    let atlas_bytes = atlas_dimension
        .saturating_mul(atlas_dimension)
        .saturating_mul(BYTES_PER_PIXEL_RGBA8);
    let instance_bytes = u64::from(total_instances).saturating_mul(INSTANCE_OVERHEAD_BYTES);
    let bytes = atlas_bytes.saturating_add(instance_bytes);

    bytes
        .saturating_add(BYTES_PER_MB - 1)
        .checked_div(BYTES_PER_MB)
        .unwrap_or(u64::from(u32::MAX))
        .clamp(1, u64::from(u32::MAX)) as u32
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}

#[cfg(test)]
mod tests {
    use tiles_renderer::{default_preview_scene, preview_snapshot};

    use super::*;
    use crate::standard_runtime_safety_budget_profile;

    #[test]
    fn valid_preview_snapshot_allows_launch() {
        let snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        let profile = standard_runtime_safety_budget_profile();

        let report = validate_playtest_snapshot(&snapshot, &profile);

        assert!(report.launch_allowed);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.safety_budget_profile_id, profile.id);
        assert!(report.usage.map_cells > 0);
    }

    #[test]
    fn invalid_preview_snapshot_blocks_launch() {
        let mut snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        snapshot.scene.grid.columns = 0;

        let report =
            validate_playtest_snapshot(&snapshot, &standard_runtime_safety_budget_profile());

        assert!(!report.launch_allowed);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "snapshotInvalid"
                && diagnostic.severity == PlaytestDiagnosticSeverity::Error));
    }

    #[test]
    fn warning_diagnostics_do_not_block_launch() {
        let mut snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        snapshot.source.clear();

        let report =
            validate_playtest_snapshot(&snapshot, &standard_runtime_safety_budget_profile());

        assert!(report.launch_allowed);
        assert_eq!(report.error_count, 0);
        assert!(report.warning_count > 0);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "snapshotSourceMissing"));
    }

    #[test]
    fn safety_budget_violations_block_launch_with_field_diagnostics() {
        let snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        let mut profile = standard_runtime_safety_budget_profile();
        profile.limits.max_map_cells = 1;

        let report = validate_playtest_snapshot(&snapshot, &profile);

        assert!(!report.launch_allowed);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "runtimeSafetyBudgetExceeded"
            && diagnostic.field.as_deref() == Some("mapCells")
            && diagnostic.actual.is_some()
            && diagnostic.limit == Some(1)));
    }
}
