package com.kenchami.kenchamios.timer

import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class TimerMachineTest {
    private val config = TimerConfig(focusS = 25 * 60, breakS = 5 * 60, longBreakS = 15 * 60)

    private fun started(now: Long = 0) = TimerState.start("p", "t", config, now)

    @Test
    fun countsDownWhileRunning() {
        val timer = started()
        assertEquals(25 * 60, timer.remainingS(0))
        assertEquals(24 * 60, timer.remainingS(MINUTE))
        assertEquals("partial seconds round up", 25 * 60, timer.remainingS(500))
    }

    @Test
    fun focusEndsIntoAShortBreak() {
        val timer = started()
        assertNull(timer.advance(config, 25 * MINUTE - 1))

        val (next, finished) = checkNotNull(timer.advance(config, 25 * MINUTE))

        assertEquals(FinishedPhase(TimerPhase.Focus, 0, 25 * MINUTE, 25 * 60), finished)
        assertEquals(TimerPhase.Break, next.phase)
        assertEquals(1, next.round)
        assertEquals(5 * 60, next.phaseLengthS)
        assertEquals(25 * MINUTE, next.phaseStartedAt)
        assertEquals(5 * 60, next.remainingS(25 * MINUTE))
    }

    @Test
    fun everyFourthFocusIsFollowedByALongBreak() {
        var timer = started()
        var now = 0L
        val afterFocus = mutableListOf<TimerPhase>()
        repeat(8) {
            now += timer.phaseLengthS * 1000L
            val (next, finished) = checkNotNull(timer.advance(config, now))
            if (finished.phase == TimerPhase.Focus) afterFocus += next.phase
            timer = next
        }

        assertEquals(
            listOf(TimerPhase.Break, TimerPhase.Break, TimerPhase.Break, TimerPhase.LongBreak),
            afterFocus,
        )
        assertEquals(TimerPhase.Focus, timer.phase)
        assertEquals(4, timer.round)
    }

    @Test
    fun pauseStopsTheClockAndResumeContinues() {
        val paused = started().pause(10 * MINUTE)
        assertEquals(TimerStatus.Paused, paused.status)
        assertEquals(15 * 60, paused.remainingS(50 * MINUTE))
        assertNull(paused.advance(config, 90 * MINUTE))

        val resumed = paused.resume(60 * MINUTE)
        assertEquals(10 * 60, resumed.remainingS(65 * MINUTE))
        assertNull(resumed.advance(config, 74 * MINUTE))
        assertNotNull(resumed.advance(config, 75 * MINUTE))
    }

    @Test
    fun pausingAndResumingTwiceChangeNothing() {
        assertEquals(started(), started().resume(MINUTE))

        val paused = started().pause(MINUTE)
        assertEquals(paused, paused.pause(5 * MINUTE))
    }

    @Test
    fun aLateCheckEndsThePhaseWhenItWasDue() {
        val (next, finished) = checkNotNull(started().advance(config, 40 * MINUTE))

        assertEquals(25 * MINUTE, finished.endedAt)
        assertEquals(40 * MINUTE, next.phaseStartedAt)
        assertEquals(5 * 60, next.remainingS(40 * MINUTE))
    }

    @Test
    fun configChangesApplyToTheNextPhase() {
        val longer = config.copy(breakS = 10 * 60)

        val (next, _) = checkNotNull(started().advance(longer, 25 * MINUTE))

        assertEquals(10 * 60, next.phaseLengthS)
    }

    @Test
    fun aClockThatGoesBackAddsNoTime() {
        val timer = started(now = 10 * MINUTE)
        assertEquals(25 * 60, timer.remainingS(5 * MINUTE))
        assertNull(timer.advance(config, 5 * MINUTE))
    }

    private companion object {
        const val MINUTE = 60_000L
    }
}
