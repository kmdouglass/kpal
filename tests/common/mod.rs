//! Common code used by the integration tests.

mod errors;
mod requests;

use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::Duration,
};

use {
    env_logger, log,
    tempfile::{tempdir, TempDir},
    url::Url,
};

pub use errors::{CommonError, StartDaemonError};
pub use requests::*;

const LIBRARY_FILENAME: &str = "libbasic-plugin.so";

/// Data that specifies the context within which the test is run.
#[derive(Debug)]
pub struct Context {
    pub bin_exe: PathBuf,
    pub daemon: Child,
    pub library_dir: TempDir,
    pub server_addr: String,
    pub server_url: Url,
}

/// Sets up a clean working directory and daemon before an integration test is run.
pub fn set_up() -> Result<Context, CommonError> {
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
pub fn tear_down(mut context: Context) {
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
