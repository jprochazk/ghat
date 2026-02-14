pub mod cron;

use indexmap::IndexMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// === Workflow =======================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    #[serde(default)]
    pub name: String,

    #[serde(rename = "run-name", default, skip_serializing_if = "Option::is_none")]
    pub run_name: Option<String>,

    pub on: Triggers,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Defaults>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<Concurrency>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub jobs: IndexMap<String, Job>,
}

// === Triggers =======================================================

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Triggers {
    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub push: Option<Push>,

    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub pull_request: Option<PullRequest>,

    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub pull_request_target: Option<PullRequest>,

    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub issue_comment: Option<IssueComment>,

    #[serde(
        default,
        deserialize_with = "nullable_vec",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub schedule: Vec<Schedule>,

    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub workflow_dispatch: Option<WorkflowDispatch>,

    #[serde(
        default,
        deserialize_with = "nullable_trigger",
        skip_serializing_if = "Option::is_none"
    )]
    pub repository_dispatch: Option<RepositoryDispatch>,
}

// === Trigger types ==================================================

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Push {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,

    #[serde(
        rename = "paths-ignore",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub paths_ignore: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,

    #[serde(
        rename = "paths-ignore",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub paths_ignore: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueComment {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub cron: cron::Cron,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDispatch {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub inputs: IndexMap<String, DispatchInput>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryDispatch {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
}

// === Permissions ====================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permissions {
    WriteAll,
    ReadAll,
    Scoped(ScopedPermissions),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedPermissions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents: Option<PermissionLevel>,

    #[serde(
        rename = "pull-requests",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pull_requests: Option<PermissionLevel>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packages: Option<PermissionLevel>,

    #[serde(rename = "id-token", default, skip_serializing_if = "Option::is_none")]
    pub id_token: Option<PermissionLevel>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployments: Option<PermissionLevel>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<PermissionLevel>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestations: Option<PermissionLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    Read,
    Write,
    None,
}

impl Serialize for Permissions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Permissions::WriteAll => serializer.serialize_str("write-all"),
            Permissions::ReadAll => serializer.serialize_str("read-all"),
            Permissions::Scoped(scoped) => scoped.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Permissions;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("'write-all', 'read-all', or a permissions mapping")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "write-all" => Ok(Permissions::WriteAll),
                    "read-all" => Ok(Permissions::ReadAll),
                    other => Err(E::custom(format!("unknown permission level: {other}"))),
                }
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let scoped = ScopedPermissions::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(Permissions::Scoped(scoped))
            }
        }

        deserializer.deserialize_any(V)
    }
}

// === Concurrency ====================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concurrency {
    pub group: String,
    pub cancel_in_progress: bool,
}

impl Serialize for Concurrency {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.cancel_in_progress {
            serializer.serialize_str(&self.group)
        } else {
            use serde::ser::SerializeMap;
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("group", &self.group)?;
            map.serialize_entry("cancel-in-progress", &self.cancel_in_progress)?;
            map.end()
        }
    }
}

impl<'de> Deserialize<'de> for Concurrency {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Concurrency;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or concurrency mapping")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Concurrency {
                    group: value.to_owned(),
                    cancel_in_progress: false,
                })
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                struct ConcurrencyMap {
                    group: String,
                    #[serde(rename = "cancel-in-progress", default)]
                    cancel_in_progress: bool,
                }
                let c =
                    ConcurrencyMap::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(Concurrency {
                    group: c.group,
                    cancel_in_progress: c.cancel_in_progress,
                })
            }
        }

        deserializer.deserialize_any(V)
    }
}

// === Defaults =======================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<RunDefaults>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    #[serde(
        rename = "working-directory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub working_directory: Option<String>,
}

// === Job ============================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "runs-on", default, skip_serializing_if = "Option::is_none")]
    pub runs_on: Option<RunsOn>,

    #[serde(
        default,
        deserialize_with = "string_or_vec",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub needs: Vec<String>,

    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub if_condition: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Defaults>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<Concurrency>,

    #[serde(
        rename = "timeout-minutes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub timeout_minutes: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Strategy>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub outputs: IndexMap<String, String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,

    // Reusable workflow fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub with: IndexMap<String, serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secrets: Option<serde_json::Value>,
}

// === RunsOn =========================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunsOn {
    Label(String),
    Labels(Vec<String>),
}

impl Serialize for RunsOn {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RunsOn::Label(s) => serializer.serialize_str(s),
            RunsOn::Labels(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RunsOn {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> serde::de::Visitor<'de> for V {
            type Value = RunsOn;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or array of strings")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(RunsOn::Label(value.to_owned()))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                seq: A,
            ) -> Result<Self::Value, A::Error> {
                let labels =
                    Vec::<String>::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
                Ok(RunsOn::Labels(labels))
            }
        }

        deserializer.deserialize_any(V)
    }
}

// === Strategy =======================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    #[serde(default)]
    pub matrix: serde_json::Value,

    #[serde(rename = "fail-fast", default, skip_serializing_if = "Option::is_none")]
    pub fail_fast: Option<bool>,

    #[serde(
        rename = "max-parallel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_parallel: Option<u32>,
}

// === Step ===========================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    #[serde(
        rename = "working-directory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub working_directory: Option<String>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub with: IndexMap<String, serde_json::Value>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,

    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub if_condition: Option<String>,

    #[serde(
        rename = "timeout-minutes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub timeout_minutes: Option<u32>,

    #[serde(
        rename = "continue-on-error",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub continue_on_error: Option<bool>,
}

// === Serde helpers ==================================================

/// Deserializes an `Option<T>` where a YAML null value (key present but no
/// value) is treated as `Some(T::default())` rather than `None`.
/// Combined with `#[serde(default)]`, absent keys still produce `None`.
fn nullable_trigger<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Some(
        Option::<T>::deserialize(deserializer)?.unwrap_or_default(),
    ))
}

/// Deserializes a `Vec<T>` where a YAML null is treated as an empty vec.
fn nullable_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Deserializes a single string or an array of strings into `Vec<String>`.
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string or array of strings")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
            Vec::<String>::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(V)
}

// === Tests ==========================================================

#[cfg(test)]
mod tests;
