#![doc = include_str!(concat!(env!("OUT_DIR"), "/README-rustdocified.md"))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use parking_lot::Mutex;

pub use crate::state::State;

use crate::internal::Item;

#[allow(missing_docs)]
pub mod internal;
mod macros;
mod state;

// ======================================================================
// CONST - PRIVATE

const DEFAULT_DRAW_RATE: usize = 20;
const DEFAULT_DRAW_INTERVAL: Duration =
    Duration::from_nanos(1_000_000_000 / DEFAULT_DRAW_RATE as u64);

const DEFAULT_DRAW_DELAY: Duration = Duration::from_millis(5);

const MIN_ETA_ELAPSED: Duration = Duration::from_millis(100);
const MIN_SPEED_ELAPSED: Duration = Duration::from_millis(100);

const BINARY_PREFIXES: &[&str] = &["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"];
const DECIMAL_PREFIXES: &[&str] = &["", "k", "M", "G", "T", "P", "E", "Z", "Y"];

// ======================================================================
// Error - PUBLIC

/// Represents all possible errors that can occur in this library.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Given items contain multiple `*_fill` items but at most one is allowed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::{progress, Error};
    ///
    /// assert_eq!(
    ///     progress!(10; bar_fill message_fill).err(),
    ///     Some(Error::MultipleFillItems)
    /// );
    /// ```
    MultipleFillItems,

    /// Given `total` is out-of-range of `u64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::{progress, Error};
    ///
    /// assert_eq!(progress!(-1).err(), Some(Error::TotalIsOutOfRange));
    /// ```
    TotalIsOutOfRange,
}

// ======================================================================
// Error - IMPL DISPLAY

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MultipleFillItems => {
                write!(f, "got multiple fill items, at most one is allowed")
            }

            Error::TotalIsOutOfRange => {
                write!(f, "total is out-of-range of `u64`")
            }
        }
    }
}

// ======================================================================
// Error - IMPL ERROR

impl StdError for Error {}

// ======================================================================
// Progress - PUBLIC

/// Progress indicator.
///
/// `Progress` is created either
/// - directly with [`progress!`] macro (if you don’t need custom configuration) or
/// - by first creating [`ProgressBuilder`] with [`progress_builder!`] macro,
///   setting custom options, and then creating `Progress` with [`build`].
///
/// `Progress` is drawn
/// - using background thread to guarantee timely updates
/// - only if terminal is detected
/// - to `STDERR` starting with `"\r"`
/// - from the moment `Progress` is created until `Progress` is finished or dropped
///
/// See crate index for [usage](crate#usage) and [examples](crate#examples).
///
/// [`build`]: crate::ProgressBuilder::build
#[derive(Clone)]
pub struct Progress {
    // This is `None` only in `Drop::drop`.
    drawer: Option<Arc<JoinHandle<()>>>,
    state: Arc<Mutex<State>>,
}

impl Progress {
    /// Finishes `Progress` with 100% completion.
    ///
    /// - Sets [`State`] of `Progress` to 100% completion.
    /// - Draws `Progress` once with additional `"\n"`
    ///   to move cursor to next line.
    /// - Finishes `Progress`, i.e. there will be no further draws.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// eprintln!("Begin");
    /// let progress = progress!(10)?;
    /// progress.finish();
    /// eprintln!("End");
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// Begin
    /// ################################################# 10/10 (0s)
    /// End
    /// ```
    pub fn finish(&self) {
        self.state.lock().finish(self.drawer.as_ref().unwrap());
    }

    /// Finishes and clears `Progress`.
    ///
    /// - Clears drawn `Progress` by overwriting with spaces + `"\r"`,
    ///   leaving cursor at start of the cleared line.
    /// - Finishes `Progress`, i.e. there will be no further draws.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// eprintln!("Begin");
    /// let progress = progress!(10)?;
    /// progress.finish_and_clear();
    /// eprintln!("End");
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// Begin
    /// End
    /// ```
    pub fn finish_and_clear(&self) {
        self.state
            .lock()
            .finish_and_clear(self.drawer.as_ref().unwrap());
    }

    /// Finishes `Progress`.
    ///
    /// - Draws `Progress` once with additional `"\n"`
    ///   to move cursor to next line.
    /// - Finishes `Progress`, i.e. there will be no further draws.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// eprintln!("Begin");
    /// let progress = progress!(10)?;
    /// progress.inc(6);
    /// progress.finish_at_current_pos();
    /// eprintln!("End");
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// Begin
    /// ##############################-------------------- 6/10 (0s)
    /// End
    /// ```
    pub fn finish_at_current_pos(&self) {
        self.state
            .lock()
            .finish_at_current_pos(self.drawer.as_ref().unwrap());
    }

    /// Increments position of `Progress`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// let progress = progress!(10)?;
    /// progress.inc(6);
    /// progress.finish_at_current_pos();
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// ##############################-------------------- 6/10 (0s)
    /// ```
    pub fn inc(&self, steps: u64) {
        self.state.lock().inc(steps, self.drawer.as_ref().unwrap());
    }

    /// Sets the message shown by item `message_fill`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// let progress = progress!(10; pos "/" total " " message_fill)?;
    /// progress.inc(6);
    /// progress.message("Hello, World!");
    /// progress.finish_at_current_pos();
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// 6/10 Hello, World!
    /// ```
    pub fn message(&self, message: impl Into<Cow<'static, str>>) {
        self.state
            .lock()
            .message(message, self.drawer.as_ref().unwrap());
    }

    /// Returns current state of `Progress`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// let progress = progress!(10)?;
    /// progress.inc(6);
    /// assert_eq!(progress.state().lock().pos(), 6);
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    pub fn state(&self) -> &Arc<Mutex<State>> {
        &self.state
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if let Ok(drawer) = Arc::try_unwrap(self.drawer.take().unwrap()) {
            let mut state = self.state.lock();
            if !state.is_finished() {
                state.finish_quietly(&drawer);
            }
            drop(state);
            let _ = drawer.join();
        }
    }
}

// ======================================================================
// Progress - CRATE

impl Progress {
    pub(crate) fn new(state: State) -> Self {
        let state = Arc::new(Mutex::new(state));

        let drawer = thread::spawn({
            let state = state.clone();
            move || loop {
                let mut state = state.lock();

                if state.is_finished() {
                    break;
                }

                let timeout = match state.try_draw() {
                    Ok(()) => None,
                    Err(timeout) => timeout,
                };

                drop(state);

                // NOTE: These may wake spuriously
                if let Some(timeout) = timeout {
                    thread::park_timeout(timeout);
                } else {
                    thread::park();
                }
            }
        });

        Self {
            drawer: Some(Arc::new(drawer)),
            state,
        }
    }
}

// ======================================================================
// ProgressBuilder - PUBLIC

/// A builder to create [`Progress`] with custom configuration.
///
/// See [custom configuration] for an example.
///
/// [custom configuration]: crate#custom-configuration
pub struct ProgressBuilder {
    total: Result<Option<u64>, Error>,
    pre_inc: bool,
    thousands_separator: String,
    items: Vec<Item>,
}

impl ProgressBuilder {
    /// Creates [`Progress`] using configuration of this `ProgressBuilder`.
    ///
    /// See [custom configuration] for an example.
    ///
    /// [custom configuration]: crate#custom-configuration
    pub fn build(self) -> Result<Progress, Error> {
        let state = State::new(
            self.total?,
            self.pre_inc,
            self.thousands_separator,
            self.items,
        )?;

        Ok(Progress::new(state))
    }

    /// Creates `ProgressBuilder` to configure [`Progress`].
    ///
    /// If `items` is empty then default items are used instead.
    ///
    /// [`progress_builder!`] macro should be used instead of this,
    /// which is same as `ProgressBuilder::new(items!(ITEMS))`.
    pub fn new(items: Vec<Item>) -> Self {
        let items = if items.is_empty() {
            // DEFAULT ITEMS
            items!(bar_fill " " pos "/" total " (" eta ")")
        } else {
            items
        };

        Self {
            total: Ok(None),
            pre_inc: false,
            thousands_separator: " ".to_owned(),
            items,
        }
    }

    /// Sets increment mode to `PreInc`.
    ///
    /// Increment mode can be `PostInc` (default) or `PreInc`.
    ///
    /// - `PostInc` means that progress position is
    ///   incremented after the associated work.
    ///     - For example incrementing position from 2 to 3 means that work
    ///       of step 3 has been completed and work of step 4 is about to begin.
    /// - `PreInc` means that progress position is
    ///   incremented before the associated work.
    ///     - For example incrementing position from 2 to 3 means that work
    ///       of step 2 has been completed and work of step 3 is about to begin.
    ///
    /// # Examples
    ///
    /// Here first step has been completed and second is about to begin
    /// so completion percentage is 33%.
    ///
    /// ```rust
    /// use ml_progress::progress_builder;
    ///
    /// let progress = progress_builder!("[" percent "] " pos "/" total)
    ///     .total(Some(3))
    ///     .pre_inc()
    ///     .build()?;
    /// progress.inc(1);
    /// progress.inc(1);
    /// progress.finish_at_current_pos();
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// ```text
    /// [ 33%] 2/3
    /// ```
    pub fn pre_inc(self) -> Self {
        Self {
            pre_inc: true,
            ..self
        }
    }

    /// Sets thousands separator, default is space.
    ///
    /// See [custom configuration] for an example.
    ///
    /// [custom configuration]: crate#custom-configuration
    pub fn thousands_separator(self, separator: &str) -> Self {
        Self {
            thousands_separator: separator.to_owned(),
            ..self
        }
    }

    /// Sets progress total, default is `None`.
    ///
    /// See [custom configuration] for an example.
    ///
    /// [custom configuration]: crate#custom-configuration
    pub fn total<T: TryInto<u64>>(self, total: Option<T>) -> Self {
        let total = if let Some(total) = total {
            match total.try_into() {
                Ok(total) => Ok(Some(total)),
                Err(_) => Err(Error::TotalIsOutOfRange),
            }
        } else {
            Ok(None)
        };

        Self { total, ..self }
    }
}

// ======================================================================
// FUNCTIONS - PUBLIC

/// Returns given value as binary prefix with corresponding value.
///
/// Uses 1024-based prefixes `Ki`, `Mi`, `Gi`, ..., `Yi`.
///
/// # Examples
///
/// ```rust
/// assert_eq!(ml_progress::binary_prefix(2048.0), (2.0, "Ki"));
/// ```
pub fn binary_prefix(mut value: f64) -> (f64, &'static str) {
    let mut scale = 0;
    while value.abs() >= 1024.0 && scale < BINARY_PREFIXES.len() - 1 {
        value /= 1024.0;
        scale += 1;
    }
    (value, BINARY_PREFIXES[scale])
}

/// Returns given value as decimal prefix with corresponding value.
///
/// Uses 1000-based prefixes `k`, `M`, `G`, ..., `Y`.
///
/// # Examples
///
/// ```rust
/// assert_eq!(ml_progress::decimal_prefix(2000.0), (2.0, "k"));
/// ```
pub fn decimal_prefix(mut value: f64) -> (f64, &'static str) {
    let mut scale = 0;
    while value.abs() >= 1000.0 && scale < DECIMAL_PREFIXES.len() - 1 {
        value /= 1000.0;
        scale += 1;
    }
    (value, DECIMAL_PREFIXES[scale])
}

/// Returns given duration in approximate format: amount and unit.
///
/// - Amount is the number of full units, i.e. it's not rounded.
/// - Unit can be `h` (hours), `m` (minutes) or `s` (seconds)
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// assert_eq!(ml_progress::duration_approx(Duration::from_secs(234)), (3, "m"));
/// ```
pub fn duration_approx(duration: Duration) -> (u64, &'static str) {
    let secs = duration.as_secs();
    if secs < 60 {
        (secs, "s")
    } else if secs < 3600 {
        (secs / 60, "m")
    } else {
        (secs / 3600, "h")
    }
}

/// Returns given duration as hours, minutes and seconds.
///
/// Returned value is the number of full seconds, i.e. it's not rounded.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// assert_eq!(ml_progress::duration_hms(Duration::from_secs(234)), (0, 3, 54));
/// ```
pub fn duration_hms(duration: Duration) -> (u64, u64, u64) {
    let secs = duration.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    (h, m, s)
}

/// Formats integer with digits in groups of three.
///
/// # Examples
///
/// ```rust
/// assert_eq!(ml_progress::group_digits(12345, " "), "12 345");
/// ```
pub fn group_digits(mut value: u64, separator: &str) -> String {
    // `u64` can have at most 7 3-digit groups
    let mut groups = [0; 7];
    let mut pos = 0;
    while pos == 0 || value > 0 {
        groups[pos] = value % 1000;
        value /= 1000;
        pos += 1;
    }

    let mut result = String::with_capacity(pos * 3 + (pos - 1) * separator.len());
    pos -= 1;
    result.push_str(&format!("{}", groups[pos]));
    while pos > 0 {
        pos -= 1;
        result.push_str(separator);
        result.push_str(&format!("{:03}", groups[pos]));
    }
    result
}

// ======================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // binary_prefix

    #[test]
    fn binary_prefix_misc() {
        assert_eq!(binary_prefix(0.0), (0.0, ""));

        assert_eq!(binary_prefix(2560.0), (2.5, "Ki"));
        assert_eq!(binary_prefix(2621440.0), (2.5, "Mi"));

        assert_eq!(binary_prefix(-2560.0), (-2.5, "Ki"));
        assert_eq!(binary_prefix(-2621440.0), (-2.5, "Mi"));
    }

    #[test]
    fn binary_prefix_overflow() {
        assert_eq!(binary_prefix(91.0f64.exp2()), (2048.0, "Yi"));
    }

    // ============================================================
    // decimal_prefix

    #[test]
    fn decimal_prefix_misc() {
        assert_eq!(decimal_prefix(0.0), (0.0, ""));

        assert_eq!(decimal_prefix(2500.0), (2.5, "k"));
        assert_eq!(decimal_prefix(2500000.0), (2.5, "M"));

        assert_eq!(decimal_prefix(-2500.0), (-2.5, "k"));
        assert_eq!(decimal_prefix(-2500000.0), (-2.5, "M"));
    }

    #[test]
    fn decimal_prefix_overflow() {
        // TODO: This shouldn't use exact comparison.
        assert_eq!(decimal_prefix(2.0e27), (2000.0, "Y"));
    }

    // ============================================================
    // duration_approx

    #[test]
    fn duration_approx_no_rounding() {
        assert_eq!(duration_approx(Duration::from_millis(1800)), (1, "s"));
    }

    // ============================================================
    // duration_hms

    #[test]
    fn duration_hms_no_rounding() {
        assert_eq!(duration_hms(Duration::from_millis(1800)), (0, 0, 1));
    }

    // ============================================================
    // group_digits

    #[test]
    fn group_digits_0() {
        assert_eq!(group_digits(0, " "), "0");
    }

    #[test]
    fn group_digits_max() {
        assert_eq!(group_digits(u64::MAX, " "), "18 446 744 073 709 551 615");
    }

    #[test]
    fn group_digits_long_separator() {
        assert_eq!(group_digits(12_345, "abc"), "12abc345");
    }

    #[test]
    fn group_digits_has_zero_padding() {
        assert_eq!(group_digits(1_002_034, " "), "1 002 034");
    }

    #[test]
    fn group_digits_no_zero_padding() {
        assert_eq!(group_digits(1_234_567, " "), "1 234 567");
    }
}
