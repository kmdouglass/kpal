//! Integration test that verifies that values may be set on pre-init attributes.
pub mod common;

use std::collections::HashMap;

use {
    log, reqwest,
    serde::{Deserialize, Serialize},
};

use common::{set_up, tear_down, CommonError, Get, Post, Request};

#[test]
fn test_pre_init_attributes() {
    let context = set_up().expect("Setup failed");
    log::debug!("{:?}", context);

    let post_data = PostData::<f64> {
        name: "foo",
        library_id: 0,
        attributes: None,
    };
    let expected = Attribute {
        id: 0,
        name: "x".to_string(),
        pre_init: true,
        value: 0.0,
        variant: "double".to_string(),
    };

    let mut attributes = HashMap::new();
    attributes.insert(
        0,
        Value {
            id: 0,
            variant: "double",
            value: 999.99,
        },
    );
    let post_data_pre_init = PostData {
        name: "bar",
        library_id: 0,
        attributes: Some(attributes),
    };
    let expected_pre_init = Attribute {
        id: 0,
        name: "x".to_string(),
        pre_init: true,
        value: 999.99,
        variant: "double".to_string(),
    };

    #[rustfmt::skip]
    let cases: Vec<Case> = vec![
        (Box::new(Get::new(&context.server_url, "/api/v0/libraries")), None),
        (Box::new(Get::new(&context.server_url, "/api/v0/libraries/0")), None),
        (Box::new(Post::new(&context.server_url, "/api/v0/peripherals", post_data)), None),
        (Box::new(Post::new(&context.server_url, "/api/v0/peripherals", post_data_pre_init)), None),
        (
            Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes/0")),
            Some(expected)
        ),
        (
            Box::new(Get::new(&context.server_url, "/api/v0/peripherals/1/attributes/0")),
            Some(expected_pre_init)
        ),
    ];

    let result = run_tests(cases);
    tear_down(context);

    assert!(result)
}

/// Loop over each test case and assert that it worked as expected.
fn run_tests(cases: Vec<Case>) -> bool {
    let mut success = true;
    for (case, expected) in &cases {
        let result = make_request(case.as_ref());
        match result {
            Ok(mut resp) => {
                if !resp.status().is_success() {
                    log::error!(
                        "Received error response from server. Aborting tests. {{ {:?} }}",
                        resp
                    );
                    success = false;
                    break;
                } else if resp.status().is_success() && expected.is_some() {
                    let expected = expected.as_ref().unwrap();
                    let resp_json: Attribute = match resp.json() {
                        Ok(json) => json,
                        Err(_) => {
                            log::error!("Could not unmarshal json");
                            success = false;
                            break;
                        }
                    };

                    if resp_json != *expected {
                        log::error!("Expected: {:?}, Actual: {:?}", expected, resp_json);
                        success = false;
                        break;
                    }
                }
            }
            Err(err) => {
                log::error!(
                    "Error when querying server. Aborting tests. {{ {:?} }}",
                    err
                );
                success = false;
                break;
            }
        };
    }
    success
}

/// Performs a single request to the test daemon.
///
/// # Arguments
///
/// * `(route, http)` - The API route and HTTP request to test
/// * `base` - The base URL to the server
fn make_request(req: &(dyn Request)) -> Result<reqwest::Response, CommonError> {
    log::info!("Testing route: {}", req.url());
    let client = reqwest::Client::new();

    log::debug!("Making HTTP {:?} request to {}", req.verb(), req.url());
    req.exec(&client).map_err(|e| e.into())
}

/// Data that represents a single test case.
type Case = (Box<dyn Request>, Option<Attribute>);

/// Post data to create a new peripheral.
#[derive(Debug, Serialize)]
struct PostData<T> {
    name: &'static str,
    library_id: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<usize, Value<T>>>,
}

/// Represents an attribute returned by the daemon.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Attribute {
    id: usize,
    name: String,
    pre_init: bool,
    variant: String,
    value: f64,
}

/// Represents an attribute value.
#[derive(Debug, Serialize)]
struct Value<T> {
    id: usize,
    variant: &'static str,
    value: T,
}
