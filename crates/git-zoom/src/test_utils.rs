/// Process-wide mutex to serialize unit tests that call `std::env::set_current_dir`.
/// All test modules that modify the working directory must hold this lock for their
/// entire duration to prevent races in parallel test execution.
pub static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
