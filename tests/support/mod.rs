#[allow(dead_code, unused_imports)]
pub mod mock_github;
#[allow(dead_code)]
pub mod project;

#[allow(unused_imports)]
pub use mock_github::MockGitHubServer;
#[allow(unused_imports)]
pub use project::TestProject;

/// Snapshot macro that auto-includes the test function name.
///
/// - `snapshot!(expr)` — unnamed, delegates to `insta::assert_snapshot!`
/// - `snapshot!("suffix", expr)` — named as `"{function_name}-{suffix}"`
#[macro_export]
macro_rules! snapshot {
    ($expr:expr) => {
        insta::assert_snapshot!($expr)
    };
    ($name:literal, $expr:expr) => {
        let value = $expr;
        insta::assert_snapshot!(format!("{}-{}", insta::_function_name!().replace("::", "__"), $name), value)
    };
    ($($tt:tt)*) => {
        insta::assert_snapshot!($($tt)*)
    };
}
