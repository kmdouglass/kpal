//! Tests that all of the routes in the user API are reachable and return HTTP success codes.
mod common;

use {log, reqwest, serde::Serialize};

use common::{set_up, tear_down, CommonError, Get, Patch, Post, Request};

#[test]
fn test_user_api() {
    let context = set_up().expect("Setup failed");
    log::debug!("{:?}", context);

    let post_data = PostData {
        name: "foo",
        library_id: 0,
    };
    let patch_attr_0 = PatchData {
        variant: "double",
        value: 42.0,
    };
    let patch_attr_3 = PatchData {
        variant: "string",
        value: "foobarbaz",
    };
    #[rustfmt::skip]
    let cases: Vec<Case> = vec![
        Box::new(Get::new(&context.server_url, "/api/v0/libraries")),
        Box::new(Get::new(&context.server_url, "/api/v0/libraries/0")),
        Box::new(Post::new(&context.server_url, "/api/v0/peripherals", post_data)),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes/0")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes/1")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes/2")),
        Box::new(Get::new(&context.server_url, "/api/v0/peripherals/0/attributes/3")),
        Box::new(Patch::new(&context.server_url,
            "/api/v0/peripherals/0/attributes/0",
            patch_attr_0,
        )),
        Box::new(Patch::new(
            &context.server_url,
            "/api/v0/peripherals/0/attributes/3",
            patch_attr_3,
        )),
    ];

    let result = {
        let mut success = true;
        for case in &cases {
            let result = subtest_user_api(case);
            match result {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        log::error!(
                            "Received error response from server. Aborting tests. {{ {:?} }}",
                            resp
                        );
                        success = false;
                        break;
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
    };
    tear_down(context);

    assert!(result)
}

/// Performs a single check for the integration test of the user API.
///
/// # Arguments
///
/// * `(route, http)` - The API route and HTTP request to test
/// * `base` - The base URL to the server
fn subtest_user_api(case: &Case) -> Result<reqwest::Response, CommonError> {
    log::info!("Testing route: {}", case.url());
    let client = reqwest::Client::new();

    log::debug!("Making HTTP {:?} request to {}", case.verb(), case.url());
    case.exec(&client).map_err(|e| e.into())
}

/// Data that represents a single test case.
type Case = Box<dyn Request>;

/// Post data to create a new peripheral.
#[derive(Debug, Serialize)]
struct PostData {
    name: &'static str,
    library_id: usize,
}

/// Patch data to update an attribute value.
#[derive(Debug, Serialize)]
struct PatchData<T> {
    variant: &'static str,
    value: T,
}
