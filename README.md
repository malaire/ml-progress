# ml-progress

Progress indicator for terminal/console.

- single line
- no ANSI escape codes, just `\r`
- background thread for timely updates
- opinionated syntax

**Early version - this hasn't yet been tested properly.**

## Usage

1. Either
    - create [`Progress`] with [`progress!`] macro
      (if you don't need custom configuration) or
    - create [`ProgressBuilder`] with [`progress_builder!`] macro,
      set custom options, and then create [`Progress`] with [`build`].
2. During prosessing update state with [`inc`] and [`message`].
3. Finish using one of [`finish`], [`finish_and_clear`]
   or [`finish_at_current_pos`].

## Examples

### Default items

```rust
use ml_progress::progress;

let progress = progress!(10)?;
for _ in 0..10 {
    // std::thread::sleep(std::time::Duration::from_millis(500));
    progress.inc(1);
}
progress.finish();

# Ok::<(), ml_progress::Error>(())
```

```text
##############################-------------------- 6/10 (2s)
```

### Custom items

```rust
use ml_progress::progress;

let progress = progress!(
    10;
    "[" percent "] " pos "/" total " " bar_fill " (" eta_hms ")"
)?;
for _ in 0..10 {
    // std::thread::sleep(std::time::Duration::from_millis(500));
    progress.inc(1);
}
progress.finish();

# Ok::<(), ml_progress::Error>(())
```

```text
[ 60%] 6/10 ########################----------------- (0:02)
```

### Custom configuration

```rust
use ml_progress::progress_builder;

let progress = progress_builder!(
    "[" percent "] " pos_group "/" total_group " " bar_fill " (" eta_hms ")"
)
.total(Some(10000))
.thousands_separator(",")
.build()?;

for _ in 0..10 {
    // std::thread::sleep(std::time::Duration::from_millis(500));
    progress.inc(1000);
}
progress.finish();

# Ok::<(), ml_progress::Error>(())
```

```text
[ 60%] 6,000/10,000 ###################-------------- (0:02)
```

## Items

Items are used with [`progress!`] and [`progress_builder!`] macros.

- Each item is either a single token (e.g. `bar_fill`)
  or multiple tokens surrounded by parentheses (e.g. `(eta "{:2}{:1}")`).
- Items are given without separators in between, except whitespace.
- At most one `*_fill` item is allowed.

### Summary

Summary of all possible items. See below for further details.

```ignore
"foo"                   // "foo"

bar_fill                // "######----"

eta                     // "5m"         ; same as (eta "{}{}")
(eta FORMAT NONE)       // u64, &str

eta_hms                 // "5:23"

message_fill            // "foo"

percent                 // " 23%"       ; same as (percent "{:3.0}%")
(percent FORMAT NONE)   // f64

pos                     // "1234567"    ; same as (pos     "{}"     )
pos_group               // "1 234 567"  ; same as (pos     "{:#}"   )
pos_bin                 // "1.18 Mi"    ; same as (pos_bin "{:#} {}")
pos_dec                 // "1.23 M"     ; same as (pos_dec "{:#} {}")
(pos     FORMAT)        // u64
(pos_bin FORMAT)        // f64, prefix
(pos_dec FORMAT)        // f64, prefix

speed                   // "1234567"    ; same as (speed     "{:#}"   )
speed_int               // "1234567"    ; same as (speed_int "{}"     )
speed_group             // "1 234 567"  ; same as (speed_int "{:#}"   )
speed_bin               // "1.18 Mi"    ; same as (speed_bin "{:#} {}")
speed_dec               // "1.23 M"     ; same as (speed_dec "{:#} {}")
(speed     FORMAT NONE) // f64
(speed_int FORMAT NONE) // u64
(speed_bin FORMAT NONE) // f64, prefix
(speed_dec FORMAT NONE) // f64, prefix

total                   // "1234567"    ; same as (total     "{}"     )
total_group             // "1 234 567"  ; same as (total     "{:#}"   )
total_bin               // "1.18 Mi"    ; same as (total_bin "{:#} {}")
total_dec               // "1.23 M"     ; same as (total_dec "{:#} {}")
(total     FORMAT NONE) // u64
(total_bin FORMAT NONE) // f64, prefix
(total_dec FORMAT NONE) // f64, prefix

(|state| EXPR)
```

`FORMAT`
- Normal Rust format string with special handling of `#` "alternate" flag (see below).

`NONE`
- Literal value to be shown when value in question is `None`.
    - See corresponding functions of [`State`] about when values are `None`.
- This is optional and can be left out. Default value is empty string.

### Prefixes

- Items ending with `_bin` use 1024-based binary prefixes (`Ki`, `Mi`, `Gi`, ...).
- Items ending with `_dec` use 1000-based decimal prefixes (`k`, `M`, `G`, ...).

If value is below 1024 or 1000 then prefix is empty string.

### Alternate format

When `#` "alternate" flag is used in `FORMAT`,
following applies based on type:

- `u64` is formatted with digits in groups of three.
    - Thousands separator can be set with [`ProgressBuilder::thousands_separator`],
      default is space.
    - e.g. `1234567` with `"{:#}"` is shown as `"1 234 567"` with default separator
- `f64`
    - Format `precision` (default: 4) is considered "fit width" and value is
      shown with maximum number of decimals so that it fits this width,
      or with no decimals if fit is not possible.
    - With `pos_bin`, `pos_dec`, `total_bin` and `total_dec`:
      Amounts with empty prefix are shown with no decimals.
    - Examples
        - `1.2345` with `"{:#}"` is shown as `"1.23"`
        - `12.345` with `"{:#}"` is shown as `"12.3"`
        - `123.45` with `"{:#}"` is shown as `"123"`
        - `1234.5` with `"{:#}"` is shown as `"1235"`
        - `12345` with `"{:#}"` is shown as `"12345"`
- `prefix`
    - `"{:#}"` shows empty prefix as-is
      and other prefixes with prepended space.

### Literal

```ignore
"foo"                   // "foo"
```
Shows given literal string.

### `bar_fill`

```ignore
bar_fill                // "######----"
```
Shows progress bar which fills remaining space on the line.

- Spaces are shown instead if `total` is `None`.

### `eta`

```ignore
eta                     // "5m"         ; same as (eta "{}{}")
(eta FORMAT)            // u64, &str
(eta FORMAT NONE)
```
Shows estimated time remaining in approximate format: amount and unit,
or `NONE` if estimate is not available.

- Amount is the number of full units, i.e. it's not rounded.
- Unit can be `h` (hours), `m` (minutes) or `s` (seconds)

### `eta_hms`

```ignore
eta_hms                 // "12:34:56"   "0:56"
```
Shows estimated time remaining as hours/minutes/seconds,
or empty string if estimate is not available.

- Depending on magnitude format is one of
  H:MM:SS, MM:SS or M:SS.
- Value is the number of full seconds, i.e. it's not rounded.

### `message_fill`

```ignore
message_fill            // "foo"
```
Shows the message set with [`Progress::message`][`message`],
filling the remaining space on the line.

### `percent`

```ignore
percent                 // " 23%"       ; same as (percent "{:3.0}%")
(percent FORMAT)        // f64
(percent FORMAT NONE)
```
Shows percentual completion or `NONE` if `total` is `None`.

### `pos`

```ignore
pos                     // "1234567"    ; same as (pos     "{}"     )
pos_group               // "1 234 567"  ; same as (pos     "{:#}"   )
pos_bin                 // "1.18 Mi"    ; same as (pos_bin "{:#} {}")
pos_dec                 // "1.23 M"     ; same as (pos_dec "{:#} {}")

(pos     FORMAT)        // u64
(pos_bin FORMAT)        // f64, prefix
(pos_dec FORMAT)        // f64, prefix
```
Shows position.
- `pos` - as integer
- `pos_group` - as integer, with digits in groups of three
- `pos_bin` - as floating-point amount with binary prefix
- `pos_dec` - as floating-point amount with decimal prefix

### `speed`

```ignore
speed                   // "1234567"    ; same as (speed     "{:#}"   )
speed_int               // "1234567"    ; same as (speed_int "{}"     )
speed_group             // "1 234 567"  ; same as (speed_int "{:#}"   )
speed_bin               // "1.18 Mi"    ; same as (speed_bin "{:#} {}")
speed_dec               // "1.23 M"     ; same as (speed_dec "{:#} {}")
(speed     FORMAT)      // f64
(speed     FORMAT NONE)
(speed_int FORMAT)      // u64
(speed_int FORMAT NONE)
(speed_bin FORMAT)      // f64, prefix
(speed_bin FORMAT NONE)
(speed_dec FORMAT)      // f64, prefix
(speed_dec FORMAT NONE)
```
Shows speed as steps per second or `NONE` if speed is not available.
- `speed` - as floating-point
- `speed_int` - as integer
- `speed_group` - as integer, with digits in groups of three
- `speed_bin` - as floating-point amount with binary prefix
- `speed_dec` - as floating-point amount with decimal prefix

### `total`

```ignore
total                   // "1234567"    ; same as (total     "{}"     )
total_group             // "1 234 567"  ; same as (total     "{:#}"   )
total_bin               // "1.18 Mi"    ; same as (total_bin "{:#} {}")
total_dec               // "1.23 M"     ; same as (total_dec "{:#} {}")

(total     FORMAT)      // u64
(total     FORMAT NONE)
(total_bin FORMAT)      // f64, prefix
(total_bin FORMAT NONE)
(total_dec FORMAT)      // f64, prefix
(total_dec FORMAT NONE)
```
Shows total or `NONE` if `total` is `None`.
- `total` - as integer
- `total_group` - as integer, with digits in groups of three
- `total_bin` - as floating-point amount with binary prefix
- `total_dec` - as floating-point amount with decimal prefix

### Custom item

```ignore
(|state| EXPR)
```
Shows return value of given function
which takes [`State`] as input and returns `String`.

```ignore
(|s| custom_eta(s))     // "12h 34m 56s"

```

```no_run
use ml_progress::State;

fn custom_eta(state: &State) -> String {
    if let Some(eta) = state.eta() {
        let (h, m, s) = ml_progress::duration_hms(eta);
        if h > 0 {
            format!("{}h {}m {}s", h, m, s)
        } else if m > 0 {
            format!("{}m {}s", m, s)
        } else {
            format!("{}s", s)
        }
    } else {
        "".to_string()
    }
}
```

[`Progress`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html
[`finish`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html#method.finish
[`finish_and_clear`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html#method.finish_and_clear
[`finish_at_current_pos`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html#method.finish_at_current_pos
[`inc`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html#method.inc
[`message`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.Progress.html#method.message

[`ProgressBuilder`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.ProgressBuilder.html
[`build`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.ProgressBuilder.html#method.build
[`ProgressBuilder::thousands_separator`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.ProgressBuilder.html#method.thousands_separator

[`State`]: https://docs.rs/ml-progress/0.1.0/ml_progress/struct.State.html

[`progress!`]: https://docs.rs/ml-progress/0.1.0/ml_progress/macro.progress.html
[`progress_builder!`]: https://docs.rs/ml-progress/0.1.0/ml_progress/macro.progress_builder.html
