package com.kenchami.kenchamios.timer

import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus

private const val ROUNDS_BEFORE_LONG_BREAK = 4

/** Phase lengths in seconds. */
data class TimerConfig(val focusS: Int, val breakS: Int, val longBreakS: Int) {
    fun lengthS(phase: TimerPhase): Int = when (phase) {
        TimerPhase.Focus -> focusS
        TimerPhase.Break -> breakS
        TimerPhase.LongBreak -> longBreakS
    }

    companion object {
        val DEFAULT = TimerConfig(focusS = 25 * 60, breakS = 5 * 60, longBreakS = 15 * 60)
    }
}

/** What a finished phase leaves behind. */
data class FinishedPhase(val phase: TimerPhase, val startedAt: Long, val endedAt: Long, val durationS: Int)

/**
 * A timer that is running or paused. Every method takes `now` (unix ms), so tests drive it without waiting.
 * The rules are those of the desktop timer.
 */
data class TimerState(
    val projectId: String,
    val taskId: String,
    val phase: TimerPhase,
    val status: TimerStatus,
    /** focus phases finished so far */
    val round: Int,
    val phaseStartedAt: Long,
    val phaseLengthS: Int,
    /** time counted before the current running stretch */
    val elapsedMs: Long,
    val runningSince: Long?,
) {
    fun elapsedMs(now: Long): Long {
        // a clock that went backwards counts as no time passed
        val since = runningSince?.let { maxOf(0, now - it) } ?: 0
        return elapsedMs + since
    }

    fun remainingMs(now: Long): Long = maxOf(0, phaseLengthS * 1000L - elapsedMs(now))

    /** Rounded up, so the display reaches 0 only when the phase is over. */
    fun remainingS(now: Long): Int = ((remainingMs(now) + 999) / 1000).toInt()

    fun isDue(now: Long): Boolean = status == TimerStatus.Running && remainingMs(now) == 0L

    fun pause(now: Long): TimerState = if (status == TimerStatus.Running) {
        copy(elapsedMs = elapsedMs(now), runningSince = null, status = TimerStatus.Paused)
    } else {
        this
    }

    fun resume(now: Long): TimerState = if (status == TimerStatus.Paused) copy(runningSince = now, status = TimerStatus.Running) else this

    /**
     * Ends the phase when its time is up and starts the next one right away.
     * A late check (after a restart) still ends it at the time it was due.
     */
    fun advance(config: TimerConfig, now: Long): Pair<TimerState, FinishedPhase>? {
        if (!isDue(now)) return null
        val overshoot = elapsedMs(now) - phaseLengthS * 1000L
        val finished = FinishedPhase(phase, phaseStartedAt, endedAt = now - overshoot, durationS = phaseLengthS)
        val round = if (finished.phase == TimerPhase.Focus) round + 1 else round
        val next = nextPhase(finished.phase, round)
        return copy(
            phase = next,
            round = round,
            phaseLengthS = config.lengthS(next),
            phaseStartedAt = now,
            elapsedMs = 0,
            runningSince = now,
        ) to finished
    }

    // the round counter already includes the focus phase that just ended
    private fun nextPhase(finished: TimerPhase, round: Int): TimerPhase = when {
        finished == TimerPhase.Focus && round % ROUNDS_BEFORE_LONG_BREAK == 0 -> TimerPhase.LongBreak
        finished == TimerPhase.Focus -> TimerPhase.Break
        else -> TimerPhase.Focus
    }

    companion object {
        fun start(projectId: String, taskId: String, config: TimerConfig, now: Long) = TimerState(
            projectId = projectId,
            taskId = taskId,
            phase = TimerPhase.Focus,
            status = TimerStatus.Running,
            round = 0,
            phaseStartedAt = now,
            phaseLengthS = config.focusS,
            elapsedMs = 0,
            runningSince = now,
        )
    }
}
