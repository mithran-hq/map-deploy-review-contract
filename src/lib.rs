use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("YAML parse failure: {0}")]
    Parse(String),
    #[error("validation failure")]
    Validation(Vec<ValidationFinding>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationFinding {
    pub level: FindingLevel,
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FindingLevel {
    Error,
    Warning,
}

/// The manifest tier signal, normalized + validated. `free` (default) | `reserved`.
pub fn normalize_workload_tier(raw: Option<&str>) -> String {
    match raw.map(str::trim) {
        Some("reserved") => "reserved".to_string(),
        _ => "free".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub manifest: NormalizedManifest,
    pub findings: Vec<ValidationFinding>,
}

pub const MAP_DEPLOY_REVIEW_CONTRACT_SCHEMA: &str = "map-deploy-review-contract/v1";
pub const MAP_MANIFEST_API_VERSION: &str = "map.mithran/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MapDeployReviewStatus {
    Passed,
    Blocked,
}

impl MapDeployReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Blocked => "blocked",
        }
    }
}

/// Stable deterministic MAP deploy-review contract for local CLI pre-flight.
///
/// This is the control-plane-owned manifest review surface that `map deploy-review`
/// should consume. It deliberately calls the same `parse_and_validate` function used
/// by deploy intake, so the CLI gets real `ERR_*` codes without copying validator
/// logic or inventing a warning tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapDeployReviewContract {
    pub schema_version: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub manifest_path: String,
    pub status: MapDeployReviewStatus,
    pub findings: Vec<ValidationFinding>,
    pub finding_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_summary: Option<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: NormalizedMetadata,
    pub identity: NormalizedIdentity,
    pub capabilities: Vec<NormalizedCapability>,
    pub resources: NormalizedResources,
    pub non_secret_env: Vec<NormalizedEnvVar>,
    pub secret_bindings: Vec<NormalizedSecretBinding>,
    pub oauth_api: Option<NormalizedOauthApi>,
    pub public_edge: Option<NormalizedPublicEdge>,
    pub app_env: BTreeMap<String, NormalizedAppEnv>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedMetadata {
    pub app_id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub domain: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub source_ref: String,
    pub normalized_at: String,
    pub tool_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedIdentity {
    pub project_ref: String,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedCapability {
    pub kind: String,
    pub route: Option<String>,
    pub runtime: String,
    pub startup: NormalizedStartup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedStartup {
    pub command: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NormalizedResources {
    pub requests: BTreeMap<String, String>,
    pub limits: BTreeMap<String, String>,
    pub concurrency: NormalizedConcurrency,
    pub replicas: NormalizedReplicas,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NormalizedConcurrency {
    pub max_in_flight: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NormalizedReplicas {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedEnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedSecretBinding {
    pub name: String,
    pub mount: NormalizedSecretMount,
    pub purpose: String,
    pub required: bool,
    pub rotation: NormalizedSecretRotation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedSecretMount {
    pub kind: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedSecretRotation {
    pub required: String,
    pub max_age_days: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedOauthApi {
    pub enabled: bool,
    pub path: String,
    pub audience: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub scope: NormalizedOauthScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedOauthScope {
    pub default: String,
    pub optional: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedPublicEdge {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub exposure: String,
    // ADR-0028 (M2.1): normalized workload tier (`free` | `reserved`); drives WorkloadClass.
    pub tier: String,
    pub allow: Vec<NormalizedPublicEdgeAllow>,
    pub review_exceptions: Vec<NormalizedReviewException>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedPublicEdgeAllow {
    #[serde(rename = "type")]
    pub allow_type: String,
    pub path: Option<String>,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedReviewException {
    pub policy_ref: String,
    pub reason: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedAppEnv {
    pub branch_pattern: String,
    // ADR-0016 / D16 (#50): the git refs that auto-deploy to this env (glob/prefix patterns like
    // `refs/heads/main`, `refs/heads/release/*`, `refs/tags/release/*`). Empty when the env is
    // explicit-deploy-only. Skipped from serialized output when empty to keep manifests that don't
    // declare triggers byte-stable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auto_deploy_refs: Vec<String>,
    pub image_tag: String,
    pub replicas: NormalizedReplicas,
    pub env_overrides: Vec<NormalizedEnvVar>,
    pub policy: BTreeMap<String, bool>,
}

#[derive(Debug, Deserialize)]
struct MithranManifest {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    metadata: Metadata,
    identity: Identity,
    #[serde(default)]
    capabilities: Vec<Capability>,
    #[serde(default)]
    resources: Resources,
    #[serde(default)]
    non_secret_env: Vec<NonSecretEnv>,
    #[serde(default)]
    secret_bindings: Vec<SecretBinding>,
    #[serde(default)]
    oauth_api: Option<OauthApi>,
    #[serde(default)]
    public_edge: Option<PublicEdge>,
    #[serde(default)]
    app_env: BTreeMap<String, AppEnv>,
    #[serde(default)]
    public_api: Option<YamlValue>,
    #[serde(default)]
    public: Option<YamlValue>,
    #[serde(default)]
    body_size_limit_mb: Option<YamlValue>,
    #[serde(default)]
    live_grants: Option<YamlValue>,
    #[serde(default)]
    roles: Option<YamlValue>,
    #[serde(default)]
    grants: Option<YamlValue>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    app_id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    domain: BTreeMap<String, String>,
    #[serde(default)]
    labels: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    project_ref: String,
    #[serde(default)]
    environment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capability {
    kind: String,
    #[serde(default)]
    route: Option<String>,
    runtime: String,
    startup: Startup,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Startup {
    command: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Resources {
    #[serde(default)]
    requests: BTreeMap<String, String>,
    #[serde(default)]
    limits: BTreeMap<String, String>,
    #[serde(default)]
    concurrency: Concurrency,
    #[serde(default)]
    replicas: Replicas,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Concurrency {
    #[serde(default)]
    max_in_flight: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Replicas {
    #[serde(default)]
    min: Option<u64>,
    #[serde(default)]
    max: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct NonSecretEnv {
    name: String,
    value: String,
    #[serde(default)]
    secret_ref: Option<YamlValue>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
struct SecretBinding {
    name: String,
    mount: SecretMount,
    purpose: String,
    required: bool,
    rotation: SecretRotation,
    #[serde(default)]
    value: Option<YamlValue>,
    #[serde(default)]
    secret_ref: Option<YamlValue>,
    #[serde(default)]
    key: Option<YamlValue>,
    #[serde(default)]
    provider_ref: Option<YamlValue>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecretMount {
    kind: String,
    target: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecretRotation {
    required: String,
    #[serde(default)]
    max_age_days: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OauthApi {
    enabled: bool,
    path: String,
    #[serde(default)]
    audience: Vec<String>,
    #[serde(default)]
    allowed_methods: Vec<String>,
    scope: OauthScope,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OauthScope {
    default: String,
    #[serde(default)]
    optional: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PublicEdge {
    #[serde(rename = "type")]
    edge_type: String,
    exposure: String,
    // ADR-0028 (M2.1): explicit paid signal. `reserved` => WorkloadClass::Reserved; absent/other
    // => free-app. Optional + defaulted so existing manifests are unchanged.
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    allow: Vec<PublicEdgeAllow>,
    #[serde(default)]
    review_exceptions: Vec<ReviewException>,
    #[serde(default)]
    waf_expression: Option<YamlValue>,
    #[serde(default)]
    cloud_armor_expression: Option<YamlValue>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
struct PublicEdgeAllow {
    #[serde(rename = "type")]
    allow_type: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    audience: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewException {
    policy_ref: String,
    reason: String,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppEnv {
    branch_pattern: String,
    // ADR-0016 / D16 (#50): explicit per-env deploy-trigger refs. Optional; absent/empty means the
    // env is explicit-deploy-only and the built-in default policy decides for pushed refs.
    #[serde(default)]
    auto_deploy_refs: Option<Vec<String>>,
    image_tag: String,
    #[serde(default)]
    replicas: Replicas,
    #[serde(default)]
    env_overrides: Vec<NonSecretEnv>,
    #[serde(default)]
    policy: AppPolicy,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Default, Deserialize)]
struct AppPolicy {
    #[serde(default)]
    protect_in_review: Option<bool>,
    #[serde(default)]
    require_map_review: Option<bool>,
    #[serde(default)]
    allow_traffic_without_review: Option<bool>,
    #[serde(flatten)]
    extra: BTreeMap<String, YamlValue>,
}

pub fn parse_and_validate(raw_yaml: &str) -> Result<ParseResult, ManifestError> {
    let raw_value = parse_yaml(raw_yaml)?;
    let parsed: MithranManifest =
        serde_yaml::from_value(raw_value).map_err(|err| ManifestError::Parse(err.to_string()))?;
    validate_and_normalize(parsed)
}

pub fn review_manifest(
    manifest_yaml: Option<&str>,
    manifest_path: impl Into<String>,
) -> MapDeployReviewContract {
    let manifest_path = manifest_path.into();
    let Some(raw_yaml) = manifest_yaml else {
        return map_deploy_review_blocked(
            manifest_path.clone(),
            vec![ValidationFinding {
                level: FindingLevel::Error,
                code: "ERR_MANIFEST_MISSING".into(),
                message: format!("{manifest_path} manifest is missing"),
                path: manifest_path,
            }],
        );
    };

    match parse_and_validate(raw_yaml) {
        Ok(result) if result.findings.is_empty() => {
            let finding_codes = result
                .findings
                .iter()
                .map(|finding| finding.code.clone())
                .collect();
            MapDeployReviewContract {
                schema_version: MAP_DEPLOY_REVIEW_CONTRACT_SCHEMA.into(),
                api_version: MAP_MANIFEST_API_VERSION.into(),
                manifest_path,
                status: MapDeployReviewStatus::Passed,
                findings: result.findings,
                finding_codes,
                normalized_summary: Some(result.manifest.redacted_digest_json()),
            }
        }
        Ok(result) => map_deploy_review_blocked(manifest_path, result.findings),
        Err(ManifestError::Parse(message)) => map_deploy_review_blocked(
            manifest_path.clone(),
            vec![ValidationFinding {
                level: FindingLevel::Error,
                code: "ERR_MANIFEST_PARSE".into(),
                message,
                path: manifest_path,
            }],
        ),
        Err(ManifestError::Validation(findings)) => {
            map_deploy_review_blocked(manifest_path, findings)
        }
    }
}

fn map_deploy_review_blocked(
    manifest_path: String,
    findings: Vec<ValidationFinding>,
) -> MapDeployReviewContract {
    let finding_codes = findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect();
    MapDeployReviewContract {
        schema_version: MAP_DEPLOY_REVIEW_CONTRACT_SCHEMA.into(),
        api_version: MAP_MANIFEST_API_VERSION.into(),
        manifest_path,
        status: MapDeployReviewStatus::Blocked,
        findings,
        finding_codes,
        normalized_summary: None,
    }
}

fn parse_yaml(raw_yaml: &str) -> Result<YamlValue, ManifestError> {
    serde_yaml::from_str(raw_yaml).map_err(|err| ManifestError::Parse(err.to_string()))
}

fn validate_and_normalize(manifest: MithranManifest) -> Result<ParseResult, ManifestError> {
    let mut findings = Vec::new();

    push_if(
        &mut findings,
        manifest.api_version != "map.mithran/v1",
        "ERR_API_VERSION",
        "apiVersion must be map.mithran/v1",
        "apiVersion",
    );
    push_if(
        &mut findings,
        manifest.kind != "MithranApp",
        "ERR_KIND",
        "kind must be MithranApp",
        "kind",
    );
    push_if(
        &mut findings,
        manifest.metadata.app_id.trim().is_empty(),
        "ERR_METADATA_APP_ID",
        "metadata.app_id is required",
        "metadata.app_id",
    );
    push_if(
        &mut findings,
        manifest.metadata.name.trim().is_empty(),
        "ERR_METADATA_NAME",
        "metadata.name is required",
        "metadata.name",
    );
    push_if(
        &mut findings,
        manifest.identity.project_ref.trim().is_empty(),
        "ERR_IDENTITY_PROJECT_REF",
        "identity.project_ref is required",
        "identity.project_ref",
    );

    collect_forbidden_top_level(&manifest, &mut findings);

    for extra_key in manifest.extra.keys() {
        finding(
            &mut findings,
            "ERR_UNKNOWN_FIELD",
            "unknown top-level manifest field is not part of mithran.yaml v1",
            &format!("{extra_key}"),
        );
    }

    validate_capabilities(&manifest.capabilities, &mut findings);
    validate_env_list("non_secret_env", &manifest.non_secret_env, &mut findings);
    validate_secret_bindings(&manifest.secret_bindings, &mut findings);
    validate_oauth_api(&manifest.oauth_api, &mut findings);
    validate_public_edge(&manifest.public_edge, &mut findings);
    validate_app_env(&manifest.app_env, &mut findings);

    findings.sort_by(|a, b| a.code.cmp(&b.code).then(a.path.cmp(&b.path)));
    if findings.iter().any(|f| f.level == FindingLevel::Error) {
        return Err(ManifestError::Validation(findings));
    }

    Ok(ParseResult {
        manifest: normalize_manifest(manifest),
        findings,
    })
}

fn collect_forbidden_top_level(manifest: &MithranManifest, findings: &mut Vec<ValidationFinding>) {
    if manifest.public_api.is_some() {
        finding(
            findings,
            "ERR_PUBLIC_API",
            "public_api is removed; use oauth_api only",
            "public_api",
        );
    }
    if manifest.public.is_some() {
        finding(
            findings,
            "ERR_PUBLIC",
            "boolean public mode is removed; use oauth_api and typed public_edge",
            "public",
        );
    }
    if manifest.body_size_limit_mb.is_some() {
        finding(
            findings,
            "ERR_BODY_SIZE_LIMIT",
            "body_size_limit_mb has no direct MAP v1 manifest equivalent",
            "body_size_limit_mb",
        );
    }
    if manifest.live_grants.is_some() || manifest.roles.is_some() || manifest.grants.is_some() {
        finding(
            findings,
            "ERR_LIVE_GRANTS",
            "live roles and grants belong in access.yaml/control-plane state",
            "grants",
        );
    }
}

fn validate_capabilities(capabilities: &[Capability], findings: &mut Vec<ValidationFinding>) {
    let allowed = ["http", "worker", "scheduler"];
    for (idx, cap) in capabilities.iter().enumerate() {
        let path = format!("capabilities[{idx}]");
        if !allowed.contains(&cap.kind.as_str()) {
            finding(
                findings,
                "ERR_CAPABILITY_KIND",
                "capability kind must be http, worker, or scheduler",
                &format!("{path}.kind"),
            );
        }
        if matches!(cap.kind.as_str(), "http" | "scheduler")
            && cap.route.as_deref().unwrap_or_default().trim().is_empty()
        {
            finding(
                findings,
                "ERR_CAPABILITY_ROUTE",
                "http and scheduler capabilities require route",
                &format!("{path}.route"),
            );
        }
        push_if(
            findings,
            cap.runtime.trim().is_empty(),
            "ERR_CAPABILITY_RUNTIME",
            "capability runtime is required",
            &format!("{path}.runtime"),
        );
        push_if(
            findings,
            cap.startup.command.trim().is_empty(),
            "ERR_CAPABILITY_STARTUP",
            "capability startup.command is required",
            &format!("{path}.startup.command"),
        );
    }
}

fn validate_env_list(prefix: &str, env: &[NonSecretEnv], findings: &mut Vec<ValidationFinding>) {
    for (idx, item) in env.iter().enumerate() {
        let path = format!("{prefix}[{idx}]");
        push_if(
            findings,
            item.name.trim().is_empty(),
            "ERR_ENV_NAME",
            "non-secret env name is required",
            &format!("{path}.name"),
        );
        if item.secret_ref.is_some() || item.extra.contains_key("secret") {
            finding(
                findings,
                "ERR_SECRET_ENV_REF",
                "secret references are forbidden in non_secret_env",
                &path,
            );
        }
        if sensitive_env_name(&item.name) {
            finding(
                findings,
                "ERR_SECRET_VALUE",
                "secret-like env names are forbidden in non_secret_env",
                &format!("{path}.name"),
            );
        }
        if !item.extra.is_empty() {
            finding(
                findings,
                "ERR_UNKNOWN_FIELD",
                "unknown non_secret_env fields are not allowed",
                &path,
            );
        }
        if looks_like_secret_value(&item.value) {
            finding(
                findings,
                "ERR_SECRET_VALUE",
                "secret-looking values are forbidden in non_secret_env",
                &format!("{path}.value"),
            );
        }
    }
}

fn validate_secret_bindings(bindings: &[SecretBinding], findings: &mut Vec<ValidationFinding>) {
    for (idx, binding) in bindings.iter().enumerate() {
        let path = format!("secret_bindings[{idx}]");
        push_if(
            findings,
            binding.name.trim().is_empty(),
            "ERR_SECRET_BINDING",
            "secret binding name is required",
            &format!("{path}.name"),
        );
        push_if(
            findings,
            !matches!(binding.mount.kind.as_str(), "env" | "file"),
            "ERR_SECRET_BINDING_MOUNT",
            "secret binding mount.kind must be env or file",
            &format!("{path}.mount.kind"),
        );
        push_if(
            findings,
            binding.mount.target.trim().is_empty(),
            "ERR_SECRET_BINDING_MOUNT",
            "secret binding mount.target is required",
            &format!("{path}.mount.target"),
        );
        push_if(
            findings,
            binding.purpose.trim().is_empty(),
            "ERR_SECRET_BINDING_PURPOSE",
            "secret binding purpose is required",
            &format!("{path}.purpose"),
        );
        push_if(
            findings,
            !matches!(binding.rotation.required.as_str(), "always" | "never"),
            "ERR_SECRET_BINDING_ROTATION",
            "rotation.required must be always or never",
            &format!("{path}.rotation.required"),
        );
        if binding.value.is_some()
            || binding.secret_ref.is_some()
            || binding.key.is_some()
            || binding.provider_ref.is_some()
        {
            finding(
                findings,
                "ERR_SECRET_REF",
                "secret_bindings may include binding metadata only, not values or raw refs",
                &path,
            );
        }
        if !binding.extra.is_empty() {
            finding(
                findings,
                "ERR_UNKNOWN_FIELD",
                "unknown secret_bindings fields are not allowed",
                &path,
            );
        }
    }
}

fn validate_oauth_api(oauth_api: &Option<OauthApi>, findings: &mut Vec<ValidationFinding>) {
    if let Some(oauth) = oauth_api {
        if oauth.enabled {
            push_if(
                findings,
                oauth.path.trim().is_empty(),
                "ERR_OAUTH_API",
                "enabled oauth_api requires path",
                "oauth_api.path",
            );
            push_if(
                findings,
                oauth.audience.is_empty(),
                "ERR_OAUTH_API",
                "enabled oauth_api requires audience",
                "oauth_api.audience",
            );
            push_if(
                findings,
                oauth.allowed_methods.is_empty(),
                "ERR_OAUTH_API",
                "enabled oauth_api requires allowed_methods",
                "oauth_api.allowed_methods",
            );
            push_if(
                findings,
                oauth.scope.default.trim().is_empty(),
                "ERR_OAUTH_API",
                "enabled oauth_api requires scope.default",
                "oauth_api.scope.default",
            );
        }
        if !oauth.extra.is_empty() {
            let code = if oauth.extra.contains_key("api_key") {
                "ERR_PUBLIC_API"
            } else {
                "ERR_UNKNOWN_FIELD"
            };
            finding(
                findings,
                code,
                "unknown oauth_api fields are not allowed",
                "oauth_api",
            );
        }
    }
}

fn validate_public_edge(edge: &Option<PublicEdge>, findings: &mut Vec<ValidationFinding>) {
    if let Some(edge) = edge {
        push_if(
            findings,
            edge.edge_type != "platform",
            "ERR_PUBLIC_EDGE_TYPE",
            "public_edge.type must be platform",
            "public_edge.type",
        );
        push_if(
            findings,
            !matches!(
                edge.exposure.as_str(),
                "none" | "protected" | "public-redirect" | "public"
            ),
            "ERR_PUBLIC_EDGE_EXPOSURE",
            "public_edge.exposure must be none, protected, public-redirect, or public",
            "public_edge.exposure",
        );
        if edge.waf_expression.is_some()
            || edge.cloud_armor_expression.is_some()
            || !edge.extra.is_empty()
        {
            finding(
                findings,
                "ERR_PUBLIC_EDGE_EXPR",
                "public_edge accepts typed policy only, not WAF/Cloud Armor expressions",
                "public_edge",
            );
        }
        for (idx, allow) in edge.allow.iter().enumerate() {
            if !matches!(allow.allow_type.as_str(), "oauth_api" | "admin_redirect") {
                finding(
                    findings,
                    "ERR_PUBLIC_EDGE_ALLOW",
                    "public_edge.allow type must be oauth_api or admin_redirect",
                    &format!("public_edge.allow[{idx}].type"),
                );
            }
            if !allow.extra.is_empty() {
                finding(
                    findings,
                    "ERR_UNKNOWN_FIELD",
                    "unknown public_edge.allow fields are not allowed",
                    &format!("public_edge.allow[{idx}]"),
                );
            }
        }
        for (idx, exception) in edge.review_exceptions.iter().enumerate() {
            push_if(
                findings,
                exception.policy_ref.trim().is_empty() || exception.reason.trim().is_empty(),
                "ERR_PUBLIC_EDGE_EXCEPTION",
                "review_exceptions require policy_ref and reason",
                &format!("public_edge.review_exceptions[{idx}]"),
            );
        }
    }
}

fn validate_app_env(app_env: &BTreeMap<String, AppEnv>, findings: &mut Vec<ValidationFinding>) {
    for (name, env) in app_env {
        let path = format!("app_env.{name}");
        push_if(
            findings,
            env.branch_pattern.trim().is_empty(),
            "ERR_APP_ENV",
            "app_env branch_pattern is required",
            &format!("{path}.branch_pattern"),
        );
        push_if(
            findings,
            env.image_tag.trim().is_empty(),
            "ERR_APP_ENV",
            "app_env image_tag is required",
            &format!("{path}.image_tag"),
        );
        // ADR-0016 / D16 (#50): each declared deploy-trigger pattern must be a non-empty git ref
        // pattern (`refs/...`); mirrors the branch_pattern requirement above.
        if let Some(refs) = &env.auto_deploy_refs {
            for (idx, pattern) in refs.iter().enumerate() {
                push_if(
                    findings,
                    pattern.trim().is_empty() || !pattern.starts_with("refs/"),
                    "ERR_APP_ENV",
                    "app_env auto_deploy_refs entries must be non-empty and start with refs/",
                    &format!("{path}.auto_deploy_refs[{idx}]"),
                );
            }
        }
        validate_env_list(
            &format!("{path}.env_overrides"),
            &env.env_overrides,
            findings,
        );
        if !env.extra.is_empty() {
            finding(
                findings,
                "ERR_UNKNOWN_FIELD",
                "unknown app_env fields are not allowed",
                &path,
            );
        }
        if !env.policy.extra.is_empty() {
            finding(
                findings,
                "ERR_UNKNOWN_FIELD",
                "unknown app_env policy fields are not allowed",
                &format!("{path}.policy"),
            );
        }
    }
}

fn normalize_manifest(manifest: MithranManifest) -> NormalizedManifest {
    let mut capabilities = manifest
        .capabilities
        .into_iter()
        .map(|cap| NormalizedCapability {
            kind: cap.kind,
            route: cap.route,
            runtime: cap.runtime,
            startup: NormalizedStartup {
                command: cap.startup.command,
            },
        })
        .collect::<Vec<_>>();
    capabilities.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then(a.route.cmp(&b.route))
            .then(a.runtime.cmp(&b.runtime))
    });

    let mut non_secret_env = manifest
        .non_secret_env
        .into_iter()
        .map(|env| NormalizedEnvVar {
            name: env.name,
            value: env.value,
        })
        .collect::<Vec<_>>();
    non_secret_env.sort_by(|a, b| a.name.cmp(&b.name));

    let mut secret_bindings = manifest
        .secret_bindings
        .into_iter()
        .map(|binding| NormalizedSecretBinding {
            name: binding.name,
            mount: NormalizedSecretMount {
                kind: binding.mount.kind,
                target: binding.mount.target,
            },
            purpose: binding.purpose,
            required: binding.required,
            rotation: NormalizedSecretRotation {
                required: binding.rotation.required,
                max_age_days: binding.rotation.max_age_days,
            },
        })
        .collect::<Vec<_>>();
    secret_bindings.sort_by(|a, b| a.name.cmp(&b.name));

    let oauth_api = manifest.oauth_api.map(|oauth| NormalizedOauthApi {
        enabled: oauth.enabled,
        path: oauth.path,
        audience: sorted_strings(oauth.audience),
        allowed_methods: sorted_strings(oauth.allowed_methods),
        scope: NormalizedOauthScope {
            default: oauth.scope.default,
            optional: sorted_strings(oauth.scope.optional),
        },
    });

    let public_edge = manifest.public_edge.map(|edge| {
        let mut allow = edge
            .allow
            .into_iter()
            .map(|item| NormalizedPublicEdgeAllow {
                allow_type: item.allow_type,
                path: item.path,
                audience: sorted_strings(item.audience),
            })
            .collect::<Vec<_>>();
        allow.sort_by(|a, b| a.allow_type.cmp(&b.allow_type).then(a.path.cmp(&b.path)));

        let mut review_exceptions = edge
            .review_exceptions
            .into_iter()
            .map(|item| NormalizedReviewException {
                policy_ref: item.policy_ref,
                reason: item.reason,
                scope: item.scope,
            })
            .collect::<Vec<_>>();
        review_exceptions.sort_by(|a, b| {
            a.policy_ref
                .cmp(&b.policy_ref)
                .then(a.reason.cmp(&b.reason))
        });

        NormalizedPublicEdge {
            edge_type: edge.edge_type,
            exposure: edge.exposure,
            tier: normalize_workload_tier(edge.tier.as_deref()),
            allow,
            review_exceptions,
        }
    });

    let app_env = manifest
        .app_env
        .into_iter()
        .map(|(name, env)| {
            let mut env_overrides = env
                .env_overrides
                .into_iter()
                .map(|var| NormalizedEnvVar {
                    name: var.name,
                    value: var.value,
                })
                .collect::<Vec<_>>();
            env_overrides.sort_by(|a, b| a.name.cmp(&b.name));
            (
                name,
                NormalizedAppEnv {
                    branch_pattern: env.branch_pattern,
                    auto_deploy_refs: env.auto_deploy_refs.unwrap_or_default(),
                    image_tag: env.image_tag,
                    replicas: NormalizedReplicas {
                        min: env.replicas.min,
                        max: env.replicas.max,
                    },
                    env_overrides,
                    policy: normalize_policy(env.policy),
                },
            )
        })
        .collect();

    NormalizedManifest {
        api_version: manifest.api_version,
        kind: manifest.kind,
        metadata: NormalizedMetadata {
            app_id: manifest.metadata.app_id,
            name: manifest.metadata.name,
            description: manifest.metadata.description,
            tags: sorted_strings(manifest.metadata.tags),
            domain: manifest.metadata.domain,
            labels: manifest.metadata.labels,
            source_ref: "source://inline/mithran.yaml".into(),
            normalized_at: "1970-01-01T00:00:00Z".into(),
            tool_revision: env!("CARGO_PKG_VERSION").into(),
        },
        identity: NormalizedIdentity {
            project_ref: manifest.identity.project_ref,
            environment: manifest.identity.environment,
        },
        capabilities,
        resources: NormalizedResources {
            requests: manifest.resources.requests,
            limits: manifest.resources.limits,
            concurrency: NormalizedConcurrency {
                max_in_flight: manifest.resources.concurrency.max_in_flight,
            },
            replicas: NormalizedReplicas {
                min: manifest.resources.replicas.min,
                max: manifest.resources.replicas.max,
            },
        },
        non_secret_env,
        secret_bindings,
        oauth_api,
        public_edge,
        app_env,
    }
}

fn looks_like_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk_live_")
        || value.starts_with("ghp_")
        || value.starts_with("github_pat_")
        || value.contains("-----BEGIN PRIVATE KEY-----")
        || lower.contains("password=")
        || lower.contains("client_secret")
        || lower.starts_with("${{")
}

fn sensitive_env_name(name: &str) -> bool {
    let marker = name.to_ascii_lowercase();
    marker.contains("secret")
        || marker.contains("token")
        || marker.contains("password")
        || marker.contains("private_key")
        || marker == "api_key"
}

fn push_if(
    findings: &mut Vec<ValidationFinding>,
    condition: bool,
    code: &str,
    message: &str,
    path: &str,
) {
    if condition {
        finding(findings, code, message, path);
    }
}

fn finding(findings: &mut Vec<ValidationFinding>, code: &str, message: &str, path: &str) {
    findings.push(ValidationFinding {
        level: FindingLevel::Error,
        code: code.into(),
        message: message.into(),
        path: path.into(),
    });
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort_unstable();
    values.dedup();
    values
}

fn normalize_policy(policy: AppPolicy) -> BTreeMap<String, bool> {
    let mut result = BTreeMap::new();
    result.insert(
        "protect_in_review".into(),
        policy.protect_in_review.unwrap_or(false),
    );
    result.insert(
        "require_map_review".into(),
        policy.require_map_review.unwrap_or(false),
    );
    result.insert(
        "allow_traffic_without_review".into(),
        policy.allow_traffic_without_review.unwrap_or(false),
    );
    result
}

impl NormalizedManifest {
    pub fn redacted_digest_json(&self) -> JsonValue {
        json!({
            "apiVersion": self.api_version,
            "kind": self.kind,
            "metadata": self.metadata,
            "identity": self.identity,
            "capabilities": self.capabilities,
            "resources": self.resources,
            "non_secret_env": self.non_secret_env.iter().map(|item| {
                json!({"name": item.name, "value": "***"})
            }).collect::<Vec<_>>(),
            "secret_bindings": self.secret_bindings.iter().map(|item| {
                json!({
                    "name": item.name,
                    "mount": item.mount,
                    "purpose": item.purpose,
                    "required": item.required,
                    "rotation": item.rotation,
                })
            }).collect::<Vec<_>>(),
            "oauth_api": self.oauth_api,
            "public_edge": self.public_edge,
            "app_env": self.app_env,
        })
    }
}

pub fn finding_codes(error: ManifestError) -> Vec<String> {
    match error {
        ManifestError::Validation(findings) => findings.into_iter().map(|f| f.code).collect(),
        ManifestError::Parse(message) => vec![format!("PARSE:{message}")],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    struct MapDeployReviewContractFixture {
        name: String,
        #[serde(default)]
        missing_manifest: bool,
        manifest: Option<String>,
        expected_status: String,
        expected_codes: Vec<String>,
        expected_paths: Vec<String>,
    }

    #[test]
    fn map_deploy_review_contract_matches_checked_in_fixtures() {
        let cases: Vec<MapDeployReviewContractFixture> = serde_yaml::from_str(include_str!(
            "../tests/fixtures/map-deploy-review-contract/cases.yml"
        ))
        .expect("fixture matrix parses");

        for case in cases {
            let manifest = if case.missing_manifest {
                None
            } else {
                Some(
                    case.manifest
                        .as_deref()
                        .expect("non-missing fixture has manifest"),
                )
            };
            let review = map_deploy_review_contract(manifest, "mithran.yaml");

            assert_eq!(
                review.status.as_str(),
                case.expected_status,
                "case {} status",
                case.name
            );
            assert_eq!(
                review.finding_codes, case.expected_codes,
                "case {} finding codes",
                case.name
            );
            let finding_paths = review
                .findings
                .iter()
                .map(|finding| finding.path.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                finding_paths, case.expected_paths,
                "case {} paths",
                case.name
            );
            assert!(
                review
                    .findings
                    .iter()
                    .all(|finding| !finding.message.trim().is_empty()),
                "case {} messages are present",
                case.name
            );
            assert_eq!(
                review.schema_version, MAP_DEPLOY_REVIEW_CONTRACT_SCHEMA,
                "case {} schema",
                case.name
            );
            assert_eq!(
                review.api_version, MAP_MANIFEST_API_VERSION,
                "case {} manifest API version",
                case.name
            );
            assert_eq!(
                review.manifest_path, "mithran.yaml",
                "case {} path",
                case.name
            );
            if review.status == MapDeployReviewStatus::Passed {
                assert!(
                    review.normalized_summary.is_some(),
                    "case {} passed reviews include redacted normalized summary",
                    case.name
                );
            } else {
                assert!(
                    review.normalized_summary.is_none(),
                    "case {} blocked reviews do not emit normalized summary",
                    case.name
                );
                assert!(
                    review
                        .findings
                        .iter()
                        .all(|finding| finding.level == FindingLevel::Error),
                    "case {} blocked findings are hard errors",
                    case.name
                );
            }
        }
    }
}

/// Backward-compatible alias for callers using the control-plane function name.
pub fn map_deploy_review_contract(
    manifest_yaml: Option<&str>,
    manifest_path: impl Into<String>,
) -> MapDeployReviewContract {
    review_manifest(manifest_yaml, manifest_path)
}
