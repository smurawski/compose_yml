//! Support for validating a `docker-compose.yml` file against the official
//! schema.

use lazy_static::lazy_static;
use std::ops::Deref;
use url::Url;

use super::File;
use crate::errors::*;

/// Schema for `docker-compose.yml` version 3.9.
const COMPOSE_3_9_SCHEMA_STR: &str = include_str!("config_schema_v3.9.json");

/// Load and parse a built-in JSON file, panicking if it contains invalid
/// JSON.
fn load_schema_json(json: &'static str) -> serde_json::Value {
    match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => panic!("cannot parse built-in schema: {}", err),
    }
}

lazy_static! {
    /// Parsed schema for `docker-compose.yml` version 2.0.
    static ref COMPOSE_3_9_SCHEMA: serde_json::Value =
    load_schema_json(COMPOSE_3_9_SCHEMA_STR);
}

/// Validate a `File` against the official JSON schema provided by
/// the Compose Spec `https://github.com/compose-spec/compose-spec`.
pub fn validate_file(file: &File) -> Result<()> {
    let schema_value = COMPOSE_3_9_SCHEMA.deref();

    let mut scope = valico::json_schema::Scope::new();
    let id = Url::parse("http://example.com/config_schema.json")
        .expect("internal schema URL should be valid");
    let schema_result =
        scope.compile_and_return_with_id(&id, schema_value.clone(), false);
    let schema = match schema_result {
        Ok(schema) => schema,
        Err(err) => panic!("cannot parse built-in schema: {:?}", err),
    };

    let value = serde_json::to_value(&file).map_err(Error::validation_failed)?;
    let validation_state = schema.validate(&value);
    if validation_state.is_strictly_valid() {
        Ok(())
    } else {
        Err(Error::validation_failed(Error::does_not_conform_to_schema(
            validation_state,
        )))
    }
}
