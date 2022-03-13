// ======================================================================
// MACROS - PUBLIC

/// Creates [`Progress`] with default configuration.
///
/// # Usage
///
/// This macro takes total and/or items as arguments
/// and returns `Result<`[`Progress`]`,`[`Error`]`>` using default configuration.
///
/// - If total is `Some`, it's given as expression which returns `impl TryInto<u64>`.
/// - If total is `None`, it's given as `None` or empty.
/// - If items are given, they must be preceded by `;`
///   after which actual items are given using special syntax
///   documented at [items](crate#items).
/// - Default items: `bar_fill " " pos "/" total " (" eta ")"`
///
/// ```ignore
/// progress!(EXPR)          // total is `Some(EXPR)`, default items
/// progress!(EXPR ; ITEMS)  // total is `Some(EXPR)`, given items
/// progress!(None ; ITEMS)  // total is `None`, given items
/// progress!(     ; ITEMS)  // - same
/// ```
///
/// # Examples
///
/// See crate index for [usage](crate#usage) and [examples](crate#examples)
/// in larger context. Following examples are only about this macro.
///
/// ```rust
/// use ml_progress::progress;
///
/// // Total is `Some(10)`, default items.
/// let progress = progress!(10)?;
///
/// // Total is `Some(10)`, given items.
/// let progress = progress!(10; pos "/" total " " bar_fill)?;
///
/// // Total is `None`, given items.
/// let progress = progress!(None; pos "/" total " " bar_fill)?;
/// let progress = progress!(    ; pos "/" total " " bar_fill)?;
///
/// # Ok::<(), ml_progress::Error>(())
/// ```
///
/// [`Error`]: crate::Error
/// [`Progress`]: crate::Progress
#[macro_export]
macro_rules! progress {
    ( $(None)? ; $($item:tt)+ ) => {
        $crate::ProgressBuilder::new($crate::items!($($item)+)).build()
    };

    ( $total:expr $( ; $($item:tt)* )? ) => {
        $crate::ProgressBuilder::new($crate::items!($($($item)*)?))
            .total(Some($total))
            .build()
    };
}

/// Creates [`ProgressBuilder`] to configure [`Progress`].
///
/// # Usage
///
/// This macro takes optional items as arguments and returns [`ProgressBuilder`].
///
/// - Items are given using special syntax documented at [items](crate#items).
/// - Default items: `bar_fill " " pos "/" total " (" eta ")"`
///
/// ```ignore
/// progress_builder!()       // default items
/// progress_builder!(ITEMS)  // given items
/// ```
///
/// # Examples
///
/// See crate index for [usage](crate#usage) and [examples](crate#examples)
/// in larger context. Following examples are only about this macro.
///
/// ```rust
/// use ml_progress::progress_builder;
///
/// // Default items.
/// let builder = progress_builder!();
///
/// // Given items.
/// let builder = progress_builder!(pos "/" total " " bar_fill);
///
/// ```
///
/// [`Progress`]: crate::Progress
/// [`ProgressBuilder`]: crate::ProgressBuilder
#[macro_export]
macro_rules! progress_builder {
    ( $($item:tt)* ) => {
        $crate::ProgressBuilder::new($crate::items!($($item)*))
    };
}

/// _Internal_ Creates `Vec<`[`Item`]`>`.
///
/// This is used internally by [`progress`] and [`progress_builder`] macros.
///
/// [`Item`]: crate::internal::Item
#[macro_export]
macro_rules! items {
    ( $($item:tt)* ) => {
        vec![ $( $crate::item!($item) ),* ]
    };
}

/// _Internal_ Creates one [`Item`].
///
/// This is used internally by [`items`] macro.
///
/// [`Item`]: crate::internal::Item
#[macro_export]
macro_rules! item {
    // ============================================================
    // BAR

    ( bar_fill ) => {
        $crate::internal::Item::Fill($crate::internal::FillItem::Bar)
    };

    // ============================================================
    // ETA

    (  eta                  ) => { $crate::item!(( eta "{}{}"  "" )) };
    (( eta $format:literal )) => { $crate::item!(( eta $format "" )) };

    (( eta $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(eta) = s.eta() {
                let (amount, unit) = $crate::duration_approx(eta);
                format!(
                    $format,
                    $crate::internal::FormatInteger::new(
                        amount,
                        s.thousands_separator()
                    ),
                    unit,
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // ETA HMS

    ( eta_hms ) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(eta) = s.eta() {
                let (h,m,s) = $crate::duration_hms(eta);
                if h > 0 {
                    format!("{}:{:02}:{:02}", h, m, s)
                } else {
                    format!("{}:{:02}", m, s)
                }
            } else {
                "".to_string()
            }
        }))
    };

    // ============================================================
    // MESSAGE

    ( message_fill ) => {
        $crate::internal::Item::Fill($crate::internal::FillItem::Message)
    };

    // ============================================================
    // PERCENT

    (  percent                  ) => { $crate::item!(( percent "{:3.0}%" "" )) };
    (( percent $format:literal )) => { $crate::item!(( percent $format   "" )) };

    (( percent $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(percent) = s.percent() {
                format!($format, $crate::internal::FormatFloat::new(percent, false))
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // POS

    ( pos       ) => { $crate::item!(( pos "{}"   )) };
    ( pos_group ) => { $crate::item!(( pos "{:#}" )) };

    (( pos $format:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            format!(
                $format,
                $crate::internal::FormatInteger::new(s.pos(), s.thousands_separator())
            )
        }))
    };

    // ============================================================
    // POS_BIN

    ( pos_bin ) => { $crate::item!(( pos_bin "{:#} {}" )) };

    (( pos_bin $format:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            let (amount, prefix) = $crate::binary_prefix(s.pos() as f64);
            format!(
                $format,
                $crate::internal::FormatFloat::new(amount, prefix == ""),
                $crate::internal::FormatPrefix::new(prefix),
            )
        }))
    };

    // ============================================================
    // POS_DEC

    ( pos_dec ) => { $crate::item!(( pos_dec "{:#} {}" )) };

    (( pos_dec $format:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            let (amount, prefix) = $crate::decimal_prefix(s.pos() as f64);
            format!(
                $format,
                $crate::internal::FormatFloat::new(amount, prefix == ""),
                $crate::internal::FormatPrefix::new(prefix),
            )
        }))
    };

    // ============================================================
    // SPEED

    (  speed                  ) => { $crate::item!(( speed "{:#}"  "" )) };
    (( speed $format:literal )) => { $crate::item!(( speed $format "" )) };

    (( speed $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(speed) = s.speed() {
                format!($format, $crate::internal::FormatFloat::new(speed, false))
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // SPEED_GROUP / SPEED_INT

    (  speed_int                  ) => { $crate::item!(( speed_int "{}"    "" )) };
    (  speed_group                ) => { $crate::item!(( speed_int "{:#}"  "" )) };
    (( speed_int $format:literal )) => { $crate::item!(( speed_int $format "" )) };

    (( speed_int $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(speed) = s.speed() {
                format!(
                    $format,
                    $crate::internal::FormatInteger::new(
                        speed.round() as u64,
                        s.thousands_separator(),
                    ),
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // SPEED_BIN

    (  speed_bin                  ) => { $crate::item!(( speed_bin "{:#} {}" "" )) };
    (( speed_bin $format:literal )) => { $crate::item!(( speed_bin $format   "" )) };

    (( speed_bin $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(speed) = s.speed() {
                let (amount, prefix) = $crate::binary_prefix(speed);
                format!(
                    $format,
                    $crate::internal::FormatFloat::new(amount, false),
                    $crate::internal::FormatPrefix::new(prefix),
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // SPEED_DEC

    (  speed_dec                  ) => { $crate::item!(( speed_dec "{:#} {}" "" )) };
    (( speed_dec $format:literal )) => { $crate::item!(( speed_dec $format   "" )) };

    (( speed_dec $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(speed) = s.speed() {
                let (amount, prefix) = $crate::decimal_prefix(speed);
                format!(
                    $format,
                    $crate::internal::FormatFloat::new(amount, false),
                    $crate::internal::FormatPrefix::new(prefix),
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // TOTAL

    (  total                  ) => { $crate::item!(( total "{}"    "" )) };
    (  total_group            ) => { $crate::item!(( total "{:#}"  "" )) };
    (( total $format:literal )) => { $crate::item!(( total $format "" )) };

    (( total $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(total) = s.total() {
                format!(
                    $format,
                    $crate::internal::FormatInteger::new(total, s.thousands_separator())
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // TOTAL_BIN

    (  total_bin                  ) => { $crate::item!(( total_bin "{:#} {}" "" )) };
    (( total_bin $format:literal )) => { $crate::item!(( total_bin $format   "" )) };

    (( total_bin $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(total) = s.total() {
                let (amount, prefix) = $crate::binary_prefix(total as f64);
                format!(
                    $format,
                    $crate::internal::FormatFloat::new(amount, prefix == ""),
                    $crate::internal::FormatPrefix::new(prefix),
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // TOTAL_DEC

    (  total_dec                  ) => { $crate::item!(( total_dec "{:#} {}" "" )) };
    (( total_dec $format:literal )) => { $crate::item!(( total_dec $format   "" )) };

    (( total_dec $format:literal $none:literal )) => {
        $crate::internal::Item::Fn(Box::new(|s| {
            if let Some(total) = s.total() {
                let (amount, prefix) = $crate::decimal_prefix(total as f64);
                format!(
                    $format,
                    $crate::internal::FormatFloat::new(amount, prefix == ""),
                    $crate::internal::FormatPrefix::new(prefix),
                )
            } else {
                $none.to_string()
            }
        }))
    };

    // ============================================================
    // OTHER

    (( $expr:expr )) => {
        $crate::internal::Item::Fn(Box::new($expr))
    };

    ( $literal:literal ) => {
        $crate::internal::Item::Literal(format!("{}", $literal))
    };
}
