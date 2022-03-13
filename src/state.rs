use std::{
    borrow::Cow,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use terminal_size::Width;

use crate::{
    internal::{FillItem, Item},
    Error, DEFAULT_DRAW_DELAY, DEFAULT_DRAW_INTERVAL, MIN_ETA_ELAPSED, MIN_SPEED_ELAPSED,
};

// ======================================================================
// State - PUBLIC

/// Current state of [`Progress`].
///
/// This is used with [custom item] and returned by [`Progress::state`].
///
/// See [custom item] for an example.
///
/// [custom item]: crate#custom-item
/// [`Progress`]: crate::Progress
/// [`Progress::state`]: crate::Progress::state
pub struct State {
    pos: u64,
    total: Option<u64>,
    percent: Option<f64>,
    pre_inc: bool,
    thousands_separator: String,
    message: Cow<'static, str>,

    start_time: Instant,
    speed: Option<f64>,
    eta_instant: Option<Instant>,

    items: Vec<Item>,

    prev_draw: Option<Instant>,
    next_draw: Option<Instant>,
    is_finished: bool,
}

impl State {
    /// Returns estimated time remaining or `None` if estimate is not available.
    ///
    /// Estimate is based on completed steps and time of latest completion.
    ///
    /// Estimate is available if
    /// - [`total`] is `Some` and
    /// - at least one step and at most [`total`] steps have been completed and
    /// - at least 100 ms has elapsed since [`Progress`] creation.
    ///
    /// See [custom item] for an example.
    ///
    /// [custom item]: crate#custom-item
    /// [`Progress`]: crate::Progress
    /// [`total`]: State::total
    pub fn eta(&self) -> Option<Duration> {
        if self.is_finished {
            Some(Duration::ZERO)
        } else if let Some(eta) = self.eta_instant {
            eta.checked_duration_since(Instant::now())
        } else {
            None
        }
    }

    /// Returns percentual completion or `None` if [`total`] is `None`.
    ///
    /// Returned value can be over 100 if [`position`]
    /// is incremented beyond [`total`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// let progress = progress!(10)?;
    /// progress.inc(6);
    /// assert_eq!(progress.state().lock().percent(), Some(60.0));
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// [`position`]: State::pos
    /// [`total`]: State::total
    pub fn percent(&self) -> Option<f64> {
        self.percent
    }

    /// Returns position.
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
    pub fn pos(&self) -> u64 {
        self.pos
    }

    /// Returns speed in steps per second
    /// or `None` if speed is not available.
    ///
    /// Speed is average from when [`Progress`] was created until latest [`inc`].
    ///
    /// Speed is available if
    /// - at least one step has been completed and
    /// - at least 100 ms has elapsed since [`Progress`] creation.
    ///
    /// [`Progress`]: crate::Progress
    /// [`inc`]: crate::Progress::inc
    pub fn speed(&self) -> Option<f64> {
        self.speed
    }

    /// Returns thousands separator.
    ///
    /// Separator can be set with [`ProgressBuilder::thousands_separator`],
    /// default is space.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress_builder;
    ///
    /// let progress = progress_builder!().thousands_separator(",").build()?;
    /// assert_eq!(progress.state().lock().thousands_separator(), ",");
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    ///
    /// [`ProgressBuilder::thousands_separator`]: crate::ProgressBuilder::thousands_separator
    pub fn thousands_separator(&self) -> &str {
        &self.thousands_separator
    }

    /// Returns total.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ml_progress::progress;
    ///
    /// let progress = progress!(10)?;
    /// assert_eq!(progress.state().lock().total(), Some(10));
    /// # Ok::<(), ml_progress::Error>(())
    /// ```
    pub fn total(&self) -> Option<u64> {
        self.total
    }
}

// ======================================================================
// State - CRATE

impl State {
    pub(crate) fn finish(&mut self, drawer: &JoinHandle<()>) {
        if !self.is_finished {
            if let Some(total) = self.total {
                self.pos = total;
            } else {
                self.total = Some(self.pos);
            }
            self.percent = Some(100.0);
            self.eta_instant = None;
            self.is_finished = true;
            drawer.thread().unpark();

            self.draw();
            if terminal_size::terminal_size().is_some() {
                eprintln!();
            }
        }
    }

    pub(crate) fn finish_and_clear(&mut self, drawer: &JoinHandle<()>) {
        if !self.is_finished {
            self.is_finished = true;
            drawer.thread().unpark();

            if let Some((Width(width), _)) = terminal_size::terminal_size() {
                let width = width as usize;
                eprint!("\r{:width$.width$}\r", "");
            }
        }
    }

    pub(crate) fn finish_at_current_pos(&mut self, drawer: &JoinHandle<()>) {
        if !self.is_finished {
            self.is_finished = true;
            drawer.thread().unpark();

            self.draw();
            if terminal_size::terminal_size().is_some() {
                eprintln!();
            }
        }
    }

    // Only for `Progress::drop`.
    //
    // - Finishes without any additional output.
    // - Can leave drawn state out-of-sync with internal state.
    pub(crate) fn finish_quietly(&mut self, drawer: &JoinHandle<()>) {
        if !self.is_finished {
            self.is_finished = true;
            drawer.thread().unpark();
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.is_finished
    }

    pub(crate) fn inc(&mut self, steps: u64, drawer: &JoinHandle<()>) {
        let now = Instant::now();
        let elapsed = now - self.start_time;

        self.pos += steps;

        let completed = if self.pre_inc {
            self.pos.saturating_sub(1)
        } else {
            self.pos
        };

        if elapsed >= MIN_SPEED_ELAPSED && completed > 0 {
            self.speed = Some(completed as f64 / elapsed.as_secs_f64());
        }

        if let Some(total) = self.total {
            self.percent = Some(completed as f64 / total as f64 * 100.0);

            if completed > total {
                self.eta_instant = None;
            } else if elapsed >= MIN_ETA_ELAPSED && completed > 0 {
                let duration = elapsed.mul_f64(total as f64 / completed as f64);
                self.eta_instant = Some(self.start_time + duration);
            }
        }

        self.queue_draw(now, drawer);
    }

    pub(crate) fn message(
        &mut self,
        message: impl Into<Cow<'static, str>>,
        drawer: &JoinHandle<()>,
    ) {
        self.message = message.into();
        self.queue_draw(Instant::now(), drawer);
    }

    pub(crate) fn new(
        total: Option<u64>,
        pre_inc: bool,
        thousands_separator: String,
        items: Vec<Item>,
    ) -> Result<Self, Error> {
        let mut fill_item_count = 0;
        for item in &items {
            if let Item::Fill(_) = item {
                fill_item_count += 1;
            }
        }

        if fill_item_count > 1 {
            Err(Error::MultipleFillItems)
        } else {
            let now = Instant::now();

            Ok(Self {
                pos: 0,
                total,
                percent: if total.is_none() { None } else { Some(0.0) },
                pre_inc,
                thousands_separator,
                message: Cow::Borrowed(""),

                start_time: now,
                speed: None,
                eta_instant: None,

                items,

                prev_draw: None,
                next_draw: Some(now + DEFAULT_DRAW_DELAY),
                is_finished: false,
            })
        }
    }

    // Returns
    // - `OK(())` - was drawn
    // - `Err(None)` - not drawn, no draw scheduled
    // - `Err(Some(..))` - not drawn, draw is scheduled after returned duration
    pub(crate) fn try_draw(&mut self) -> Result<(), Option<Duration>> {
        assert!(!self.is_finished);

        if let Some(next_draw) = self.next_draw {
            let now = Instant::now();
            if next_draw > now {
                Err(Some(next_draw - now))
            } else {
                self.draw();
                self.prev_draw = Some(now);
                self.next_draw = None;
                Ok(())
            }
        } else {
            Err(None)
        }
    }
}

// ======================================================================
// State - PRIVATE

impl State {
    fn draw(&mut self) {
        if let Some((Width(width), _)) = terminal_size::terminal_size() {
            let width = width as usize;

            let mut pre_fill = String::with_capacity(width);
            let mut fill = None;
            let mut post_fill = String::with_capacity(width);

            for item in &self.items {
                let active = if fill.is_none() {
                    &mut pre_fill
                } else {
                    &mut post_fill
                };

                match item {
                    Item::Fill(item) => fill = Some(item),
                    Item::Fn(f) => active.push_str(&f(self)),
                    Item::Literal(s) => active.push_str(s),
                }
            }

            let fill_width =
                width.saturating_sub(pre_fill.chars().count() + post_fill.chars().count());

            let mut line = String::with_capacity(width);
            line.push_str(&pre_fill);
            match fill {
                Some(&FillItem::Bar) => {
                    if let Some(percent) = self.percent {
                        let done_width =
                            ((fill_width as f64 * percent / 100.0) as usize).min(fill_width);
                        line.push_str(&"#".repeat(done_width));
                        line.push_str(&"-".repeat(fill_width - done_width));
                    } else {
                        line.push_str(&" ".repeat(fill_width));
                    }
                }

                Some(FillItem::Message) => {
                    line.push_str(&format!("{:fill_width$.fill_width$}", self.message))
                }

                None => (),
            }
            line.push_str(&post_fill);

            eprint!("\r{:width$.width$}", line);
        }
    }

    fn queue_draw(&mut self, now: Instant, drawer: &JoinHandle<()>) {
        if !self.is_finished && self.next_draw.is_none() {
            let mut next_draw = now + DEFAULT_DRAW_DELAY;
            if let Some(prev_draw) = self.prev_draw {
                next_draw = next_draw.max(prev_draw + DEFAULT_DRAW_INTERVAL);
            }
            self.next_draw = Some(next_draw);

            drawer.thread().unpark();
        }
    }
}
