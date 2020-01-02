use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt, fs, io, panic,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::Duration,
};

use {
    env_logger, log, reqwest,
    serde::Serialize,
    tempfile::{tempdir, TempDir},
    url::Url,
};

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
    let patch_attr_0 = PatchDouble {
        variant: "double",
        value: 42.0,
    };
    let patch_attr_3 = PatchString {
        variant: "string",
        value: "foobarbaz",
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
            Http::PatchDouble(patch_attr_0),
        ),
        (
            "/api/v0/peripherals/0/attributes/3",
            Http::PatchString(patch_attr_3),
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
        Http::PatchDouble(data) => client
            .patch(base.as_str())
            .json(&data)
            .send()
            .expect("Request failed"),
        Http::PatchString(data) => client
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
    PatchDouble(PatchDouble),
    PatchString(PatchString),
}

/// Post data to create a new peripheral.
#[derive(Debug, Serialize)]
struct PostData {
    name: &'static str,
    library_id: usize,
}

/// Patch data to update an attribute value.
#[derive(Debug, Serialize)]
struct PatchDouble {
    variant: &'static str,
    value: f64,
}

/// Patch data to update an attribute value.
#[derive(Debug, Serialize)]
struct PatchString {
    variant: &'static str,
    value: &'static str,
}

/// Data that specifies the context within which the test is run.
#[derive(Debug)]
struct Context {
    bin_exe: PathBuf,
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
    library_file_src.push(artifacts_dir());
    library_file_src.push(format!("examples/{}", LIBRARY_FILENAME));

    let mut library_file_dest = PathBuf::from(library_dir.path());
    library_file_dest.push(LIBRARY_FILENAME);

    fs::copy(library_file_src.as_path(), library_file_dest.as_path())?;

    // Find the kpald binary
    let mut bin_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bin_exe.push(artifacts_dir());
    bin_exe.push("kpald");

    // Grab the server IP address and port from the environment in the form $ADDRESS:$PORT
    let server_addr: String = env::var_os("SERVER_ADDRESS")
        .unwrap_or_else(|| OsString::from("0.0.0.0:8000"))
        .into_string()
        .expect("Could not get SERVER_ADDRESS environment variable");
    let server_url =
        Url::parse(&format!("http://{}", &server_addr)).expect("Could not get base URL");

    // Start the server
    let daemon = start_daemon(
        bin_exe.as_path(),
        library_dir.path(),
        &server_addr,
        &server_url,
    )
    .unwrap();

    Ok(Context {
        bin_exe,
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
/// * `bin_exe` - The location of the daemon's binary file
/// * `library_dir` - The location of the peripheral library files
/// * `server_addr` - The address of the server in the form $ADDRESS:$PORT
/// * `server_url` - The URL of the server in the form $SCHEME://$ADDRESS:$PORT
fn start_daemon(
    bin_exe: &Path,
    library_dir: &Path,
    server_addr: &str,
    server_url: &Url,
) -> Result<Child, StartDaemonError> {
    let mut daemon = Command::new(bin_exe)
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

/// Determines the location of the build artifacts.
///
/// This function determines which artifacts to use for the integration tests by finding the
/// location of the current test binary. Everything else if located relative to this location.
fn artifacts_dir() -> PathBuf {
    let mut dir = env::current_exe().expect("Could not determine current executable");
    dir.pop(); // Drop executable name
    dir.pop(); // Move up one directory from deps
    dir
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
