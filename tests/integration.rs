//! Integration tests exercising the public API of `camelcase-keys`.

use camelcase_keys::{camelcase_keys, camelcase_keys_with, Options};
use serde_json::json;

#[test]
fn api_response_shape() {
    let response = json!({
        "user_id": 1,
        "display_name": "Ada",
        "contact_info": { "email_address": "ada@example.com" }
    });
    let camelized = camelcase_keys_with(&response, &Options::new().deep(true));
    assert_eq!(
        camelized,
        json!({
            "userId": 1,
            "displayName": "Ada",
            "contactInfo": { "emailAddress": "ada@example.com" }
        })
    );
}

#[test]
fn exclude_and_stop_paths() {
    let value = json!({ "api_key": "secret", "nested_data": { "inner_key": 1 } });
    let opts = Options::new().deep(true).exclude(["api_key"]).stop_paths(["nested_data"]);
    assert_eq!(
        camelcase_keys_with(&value, &opts),
        json!({ "api_key": "secret", "nestedData": { "inner_key": 1 } })
    );
}

#[test]
fn array_of_records() {
    let rows = json!([{ "row_id": 1 }, { "row_id": 2 }]);
    assert_eq!(camelcase_keys(&rows), json!([{ "rowId": 1 }, { "rowId": 2 }]));
}

#[test]
fn pascal_and_numeric() {
    assert_eq!(
        camelcase_keys_with(&json!({ "user_name": 1 }), &Options::new().pascal_case(true)),
        json!({ "UserName": 1 })
    );
    assert_eq!(camelcase_keys(&json!({ "0": "a", "id_value": 2 })), json!({ "0": "a", "idValue": 2 }));
}
