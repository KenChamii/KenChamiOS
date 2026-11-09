use serde::Serialize;

const ROUNDS_BEFORE_LONG_BREAK: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Phase {
    Focus,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Status {
    Running,
    Paused,
}

/// Phase lengths in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerConfig {
    pub focus_s: u32,
    pub break_s: u32,
    pub long_break_s: u32,
}

impl TimerConfig {
    fn length_s(self, phase: Phase) -> u32 {
        match phase {
            Phase::Focus => self.focus_s,
            Phase::Break => self.break_s,
            Phase::LongBreak => self.long_break_s,
        }
    }
}

/// A timer that is running or paused. Every method takes `now` (unix ms) so
/// tests can drive it without waiting.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct TimerState {
    pub project_id: String,
    pub task_id: String,
    pub phase: Phase,
    pub status: Status,
    /// focus phases finished so far
    pub round: u32,
    pub phase_started_at: i64,
    pub phase_length_s: u32,
    /// time counted before the current running stretch
    pub elapsed_ms: i64,
    pub running_since: Option<i64>,
}

/// What a finished phase leaves behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinishedPhase {
    pub phase: Phase,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_s: u32,
}

/// What the ui and the tray show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimerView {
    pub project_id: String,
    pub task_id: String,
    pub phase: Phase,
    pub status: Status,
    pub round: u32,
    pub length_s: u32,
    pub remaining_s: u32,
}

impl TimerState {
    pub fn start(project_id: String, task_id: String, config: TimerConfig, now: i64) -> Self {
        Self {
            project_id,
            task_id,
            phase: Phase::Focus,
            status: Status::Running,
            round: 0,
            phase_started_at: now,
            phase_length_s: config.focus_s,
            elapsed_ms: 0,
            running_since: Some(now),
        }
    }

    pub fn elapsed_ms(&self, now: i64) -> i64 {
        // a clock that went backwards counts as no time passed
        let since = self.running_since.map_or(0, |since| (now - since).max(0));
        self.elapsed_ms + since
    }

    pub fn remaining_s(&self, now: i64) -> u32 {
        // rounded up, so the display reaches 0 only when the phase is over
        u32::try_from((self.remaining_ms(now) + 999) / 1000).unwrap_or(0)
    }

    pub fn pause(&mut self, now: i64) {
        if self.status == Status::Running {
            self.elapsed_ms = self.elapsed_ms(now);
            self.running_since = None;
            self.status = Status::Paused;
        }
    }

    pub fn resume(&mut self, now: i64) {
        if self.status == Status::Paused {
            self.running_since = Some(now);
            self.status = Status::Running;
        }
    }

    pub fn is_due(&self, now: i64) -> bool {
        self.status == Status::Running && self.remaining_ms(now) == 0
    }

    fn remaining_ms(&self, now: i64) -> i64 {
        (i64::from(self.phase_length_s) * 1000 - self.elapsed_ms(now)).max(0)
    }

    /// Ends the phase when its time is up and starts the next one right away.
    /// A late check (after a restart) still ends it at the time it was due.
    pub fn advance(&mut self, config: TimerConfig, now: i64) -> Option<FinishedPhase> {
        if !self.is_due(now) {
            return None;
        }
        let overshoot = self.elapsed_ms(now) - i64::from(self.phase_length_s) * 1000;
        let finished = FinishedPhase {
            phase: self.phase,
            started_at: self.phase_started_at,
            ended_at: now - overshoot,
            duration_s: self.phase_length_s,
        };
        if finished.phase == Phase::Focus {
            self.round += 1;
        }
        self.phase = self.next_phase(finished.phase);
        self.phase_length_s = config.length_s(self.phase);
        self.phase_started_at = now;
        self.elapsed_ms = 0;
        self.running_since = Some(now);
        Some(finished)
    }

    // the round counter already includes the focus phase that just ended
    fn next_phase(&self, finished: Phase) -> Phase {
        match finished {
            Phase::Focus if self.round.is_multiple_of(ROUNDS_BEFORE_LONG_BREAK) => Phase::LongBreak,
            Phase::Focus => Phase::Break,
            Phase::Break | Phase::LongBreak => Phase::Focus,
        }
    }

    pub fn view(&self, now: i64) -> TimerView {
        TimerView {
            project_id: self.project_id.clone(),
            task_id: self.task_id.clone(),
            phase: self.phase,
            status: self.status,
            round: self.round,
            length_s: self.phase_length_s,
            remaining_s: self.remaining_s(now),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: TimerConfig = TimerConfig {
        focus_s: 25 * 60,
        break_s: 5 * 60,
        long_break_s: 15 * 60,
    };
    const MINUTE: i64 = 60_000;

    fn started() -> TimerState {
        TimerState::start("p".into(), "t".into(), CONFIG, 0)
    }

    #[test]
    fn counts_down_while_running() {
        let timer = started();
        assert_eq!(timer.remaining_s(0), 25 * 60);
        assert_eq!(timer.remaining_s(MINUTE), 24 * 60);
        assert_eq!(timer.remaining_s(500), 25 * 60, "partial seconds round up");
    }

    #[test]
    fn focus_ends_into_a_short_break() {
        let mut timer = started();
        assert_eq!(timer.advance(CONFIG, 25 * MINUTE - 1), None);

        let finished = timer.advance(CONFIG, 25 * MINUTE).expect("focus is over");

        assert_eq!(
            finished,
            FinishedPhase {
                phase: Phase::Focus,
                started_at: 0,
                ended_at: 25 * MINUTE,
                duration_s: 25 * 60,
            }
        );
        assert_eq!(timer.phase, Phase::Break);
        assert_eq!(timer.round, 1);
        assert_eq!(timer.phase_length_s, 5 * 60);
        assert_eq!(timer.phase_started_at, 25 * MINUTE);
        assert_eq!(timer.remaining_s(25 * MINUTE), 5 * 60);
    }

    #[test]
    fn every_fourth_focus_is_followed_by_a_long_break() {
        let mut timer = started();
        let mut now = 0;
        let mut after_focus = Vec::new();
        for _ in 0..8 {
            now += i64::from(timer.phase_length_s) * 1000;
            let finished = timer.advance(CONFIG, now).expect("phase over");
            if finished.phase == Phase::Focus {
                after_focus.push(timer.phase);
            }
        }
        assert_eq!(
            after_focus,
            [Phase::Break, Phase::Break, Phase::Break, Phase::LongBreak]
        );
        assert_eq!(timer.phase, Phase::Focus);
        assert_eq!(timer.round, 4);
    }

    #[test]
    fn pause_stops_the_clock_and_resume_continues() {
        let mut timer = started();
        timer.pause(10 * MINUTE);
        assert_eq!(timer.status, Status::Paused);
        assert_eq!(timer.remaining_s(50 * MINUTE), 15 * 60);
        assert_eq!(timer.advance(CONFIG, 90 * MINUTE), None);

        timer.resume(60 * MINUTE);
        assert_eq!(timer.remaining_s(65 * MINUTE), 10 * 60);
        assert_eq!(timer.advance(CONFIG, 74 * MINUTE), None);
        assert!(timer.advance(CONFIG, 75 * MINUTE).is_some());
    }

    #[test]
    fn pause_and_resume_twice_change_nothing() {
        let mut timer = started();
        timer.resume(MINUTE);
        assert_eq!(timer, started());

        timer.pause(MINUTE);
        let paused = timer.clone();
        timer.pause(5 * MINUTE);
        assert_eq!(timer, paused);
    }

    #[test]
    fn a_late_check_ends_the_phase_when_it_was_due() {
        let mut timer = started();

        let finished = timer.advance(CONFIG, 40 * MINUTE).expect("long overdue");

        assert_eq!(finished.ended_at, 25 * MINUTE);
        assert_eq!(timer.phase_started_at, 40 * MINUTE);
        assert_eq!(timer.remaining_s(40 * MINUTE), 5 * 60);
    }

    #[test]
    fn config_changes_apply_to_the_next_phase() {
        let mut timer = started();
        let longer = TimerConfig {
            break_s: 10 * 60,
            ..CONFIG
        };

        timer.advance(longer, 25 * MINUTE);

        assert_eq!(timer.phase_length_s, 10 * 60);
    }

    #[test]
    fn a_clock_that_goes_back_adds_no_time() {
        let mut timer = TimerState::start("p".into(), "t".into(), CONFIG, 10 * MINUTE);
        assert_eq!(timer.remaining_s(5 * MINUTE), 25 * 60);
        assert_eq!(timer.advance(CONFIG, 5 * MINUTE), None);
    }

    #[test]
    fn view_reports_the_state() {
        let view = started().view(MINUTE);
        assert_eq!(view.phase, Phase::Focus);
        assert_eq!(view.status, Status::Running);
        assert_eq!((view.length_s, view.remaining_s), (25 * 60, 24 * 60));
    }
}
