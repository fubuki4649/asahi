/// Attempts to unwrap a `Result`, returning from the caller function if it is an `Err`.
///
/// This macro is useful for early returns in functions that do not return a `Result`
/// themselves but need to short-circuit on error.
///
/// # Parameters
/// - `$expr`: A `Result<T, E>` expression to unwrap.
/// - `$err_return` (optional): The value to return from the function if the result is `Err`.
///   Defaults to `Err(Status::InternalServerError)`
///
/// # Example
/// ```
/// let val = unwrap_or_return!(some_result, Status::BadRequest);
/// ```
#[macro_export]
macro_rules! unwrap_or_return {
    ($expr:expr, $err_return:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return $err_return,
        }
    };
}
