//! ## Logging macros
//!
//! Encapsulate the `log` macros if the `log` feature is active to reduce the number of `#[cfg(feature = "log")]` in code.
//!
//! If the feature is **not** active, calling these macro is a no-op.

/// Macro encapsulating the `log::debug!` macro if the `log` feature is active.
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => ({
        #[cfg(feature = "log")]
        log::debug!(
            $($args)*
        )
    });
}

/// Macro encapsulating the `log::warn!` macro if the `log` feature is active.
#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => ({
        #[cfg(feature = "log")]
        log::warn!(
            $($args)*
        )
    });
}

/// Macro encapsulating the `log::info!` macro if the `log` feature is active.
#[macro_export]
macro_rules! info {
    ($($args:tt)+) => ({
        #[cfg(feature = "log")]
        log::info!(
            $($args)*
        )
    });
}
