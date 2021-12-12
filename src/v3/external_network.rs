use super::common::*;

/// Information about an external network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ExternalNetwork {
    /// The external name of this network, if it's different from the
    /// internal name we refer to as.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<RawOr<String>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(ExternalNetwork, {
    name, _hidden
});
