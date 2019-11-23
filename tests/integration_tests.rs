use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use env_logger;
use log;
use reqwest;
use serde::Serialize;
use tempfile::{tempdir, TempDir};
use url::Url;

const LIBRARY_FILENAME: &str = "libbasic-plugin.so";

/// Tests that all of the routes in the user API are reachable and return HTTP success codes.
#[test]
fn test_user_api() {
    let context = set_up().expect("Setup failed");
    log::debug!("{:?}", context);

    let post_data = PostData {
        name: "foo",
        library_id: 0,
    };
    let patch_data = PatchData {
        variant: "float",
        value: 42.0,
    };
    let cases: Vec<Case> = vec![
        ("/api/v0/libraries", Http::Get),
        ("/api/v0/libraries/0", Http::Get),
        ("/api/v0/peripherals", Http::Post(post_data)),
        ("/api/v0/peripherals", Http::Get),
        ("/api/v0/peripherals/0", Http::Get),
        ("/api/v0/peripherals/0/attributes", Http::Get),
        ("/api/v0/peripherals/0/attributes/0", Http::Get),
        (
            "/api/v0/peripherals/0/attributes/0",
            Http::Patch(patch_data),
        ),
    ];
    let result = {
        let result = panic::catch_unwind(|| {
            for case in &cases {
                subtest_user_api(case, &context.server_url);
            }
        });
        tear_down(context);
        result
    };

    assert!(result.is_ok());
}

/// Performs a single check for the integration test of the user API.
///
/// # Arguments
///
/// * `(route, http)` - The API route and HTTP request to test
/// * `base` - The base URL to the server
fn subtest_user_api((route, http): &Case, base: &Url) {
    log::info!("Testing route: {}", route);
    let client = reqwest::Client::new();
    let base = base
        .join(route)
        .expect("Could not produce full URL for the test");

    log::debug!("Making HTTP request {:?} to {}", http, base);
    let req = match http {
        Http::Get => client.get(base.as_str()).send().expect("Request failed"),
        Http::Post(data) => client
            .post(base.as_str())
            .json(&data)
            .send()
            .expect("Request failed"),
        Http::Patch(data) => client
            .patch(base.as_str())
            .json(&data)
            .send()
            .expect("Request failed"),
    };

    log::debug!("Made request {:?}", req);
    assert!(req.status().is_success());
}

/// Data that represents a single test case.
type Case = (&'static str, Http);

/// Helper enum for a HTTP request.
#[derive(Debug)]
enum Http {
    Get,
    Post(PostData),
    Patch(PatchData),
}

/// Post data to create a new peripheral.
#[derive(Debug, Serialize)]
struct PostData {
    name: &'static str,
    library_id: usize,
}

/// Patch data to update an attribute value.
#[derive(Debug, Serialize)]
struct PatchData {
    variant: &'static str,
    value: f64,
}

/// Data that specifies the context within which the test is run.
#[derive(Debug)]
struct Context {
    bin_dir: PathBuf,
    daemon: Child,
    library_dir: TempDir,
    server_addr: String,
    server_url: Url,
}

/// Sets up a clean working directory and daemon before an integration test is run.
fn set_up() -> Result<Context, io::Error> {
    let _ = env_logger::builder().is_test(true).try_init();

    // Set up the temporary directory to hold library files
    let library_dir = tempdir()?;
    let mut library_file_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    library_file_src.push(format!("target/debug/examples/{}", LIBRARY_FILENAME));

    let mut library_file_dest = PathBuf::from(library_dir.path());
    library_file_dest.push(LIBRARY_FILENAME);

    fs::copy(library_file_src.as_path(), library_file_dest.as_path())?;

    // Find the kpald binary
    let mut bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bin_dir.push("target/debug/kpald");

    // Grab the server IP address and port from the environment in the form $ADDRESS:$PORT
    let server_addr: String = env::var_os("SERVER_ADDRESS")
        .unwrap_or(OsString::from("0.0.0.0:8000"))
        .into_string()
        .expect("Could not get SERVER_ADDRESS environment variable");
    let server_url =
        Url::parse(&format!("http://{}", &server_addr)).expect("Could not get base URL");

    // Start the server
    let daemon = start_daemon(
        bin_dir.as_path(),
        library_dir.path(),
        &server_addr,
        &server_url,
    )
    .unwrap();

    Ok(Context {
        bin_dir,
        daemon,
        library_dir,
        server_addr,
        server_url,
    })
}

/// Cleans up any resoruces that were created for an integration test.
///
/// # Arguments
///
/// * `context` - Values that define the context within which the tests are run.
fn tear_down(mut context: Context) {
    let _ = context.daemon.kill();
}

/// Starts the daemon for a test.
///
/// This method must ensure that the daemon process is killed if any error occurs during the setup.
///
/// # Arguments
///
/// * `bin_dir` - The location of the daemon's binary file
/// * `library_dir` - The location of the peripheral library files
/// * `server_addr` - The address of the server in the form $ADDRESS:$PORT
/// * `server_url` - The URL of the server in the form $SCHEME://$ADDRESS:$PORT
fn start_daemon(
    bin_dir: &Path,
    library_dir: &Path,
    server_addr: &str,
    server_url: &Url,
) -> Result<Child, StartDaemonError> {
    let mut daemon = Command::new(bin_dir)
        .arg("--library-dir")
        .arg(library_dir)
        .arg("--server-address")
        .arg(server_addr)
        .spawn()
        .expect("daemon failed to start");

    let mut attempt = 0;
    let num_attempts = 3;
    let mut sleep_time = 250;
    while let Err(e) = reqwest::get(server_url.as_str()) {
        log::debug!(
            "Server is not ready: {}\nRetrying in {} ms...",
            e,
            sleep_time
        );
        attempt += 1;
        if attempt == num_attempts {
            log::error!("Maximum number of attempts reached. Killing the daemon...");
            let _ = daemon.kill();
            return Err(StartDaemonError {});
        }

        thread::sleep(Duration::from_millis(sleep_time));
        sleep_time *= 2;
    }

    Ok(daemon)
}

/// Indicates that an error occured when starting the daemon.
#[derive(Debug)]
struct StartDaemonError {}

impl fmt::Display for StartDaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StartDaemonError")
    }
}

impl Error for StartDaemonError {
    fn description(&self) -> &str {
        "An error occurred when starting the daemon"
    }
}
