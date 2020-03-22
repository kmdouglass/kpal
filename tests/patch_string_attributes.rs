//! Integration test that verifies that string attributes are correctly updated.
mod common;

use {
    log, reqwest,
    serde::{Deserialize, Serialize},
};

use common::{set_up, tear_down, CommonError, Get, Patch, Post, Request};

#[test]
fn test_patch_string_attributes() {
    let context = set_up().expect("Setup failed");
    log::debug!("{:?}", context);

    // This matches the ID of the string attribute in the BasicPlugin example.
    let attribute_id = 3;

    let post_data = PostData {
        name: "foo",
        library_id: 0,
    };
    let patch_data = PatchData {
        r#type: "string",
        value: "helloworld",
    };
    let expected_before_patch = Attribute {
        id: attribute_id,
        name: "msg".to_string(),
        value: Value {
            value: "foobar".to_string(),
            r#type: "string".to_string(),
        },
    };
    let expected_after_patch = Attribute {
        id: attribute_id,
        name: "msg".to_string(),
        value: Value {
            value: "helloworld".to_string(),
            r#type: "string".to_string(),
        },
    };

    #[rustfmt::skip]
    let cases: Vec<Case> = vec![
        (Box::new(Get::new(&context.server_url, "/api/v0/libraries")), None),
        (Box::new(Get::new(&context.server_url, "/api/v0/libraries/0")), None),
        (Box::new(Post::new(&context.server_url, "/api/v0/peripherals", post_data)), None),
        (
            Box::new(Get::new(
                &context.server_url,
                &format!("/api/v0/peripherals/0/attributes/{}", attribute_id))),
            Some(expected_before_patch)
        ),
        (
            Box::new(Patch::new(
                &context.server_url,
                &format!("/api/v0/peripherals/0/attributes/{}", attribute_id),
                patch_data
            )),
            None
        ),
        (
            Box::new(Get::new(
                &context.server_url,
                &format!("/api/v0/peripherals/0/attributes/{}", attribute_id))),
            Some(expected_after_patch)
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
struct PostData {
    name: &'static str,
    library_id: usize,
}

/// Patch data to update an attribute value.
#[derive(Debug, Serialize)]
struct PatchData<T> {
    r#type: &'static str,
    value: T,
}

/// Represents an attribute returned by the daemon.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Attribute {
    id: usize,
    name: String,
    value: Value,
}

/// Represents a value returned by the daemon.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Value {
    r#type: String,
    value: String,
}
