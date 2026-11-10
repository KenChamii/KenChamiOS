package com.kenchami.kenchamios.timer

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.TimerRepository
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TimerControllerTest {
    private lateinit var env: TestDatabase
    private lateinit var timer: TimerController
    private lateinit var taskId: String
    private lateinit var projectId: String

    @Before
    fun setUp() = runBlocking {
        env = TestDatabase()
        val tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
        projectId = ProjectRepository(env.database, env.stamper, env.activity).create("Demo")
        taskId = tasks.create(projectId, "Do it")
        timer = newController()
    }

    @After
    fun tearDown() = env.close()

    private fun newController() = TimerController(
        TimerRepository(env.database, env.stamper, env.activity),
        TaskRepository(env.database, env.stamper, env.activity, env.clock),
        env.clock,
    )

    private fun focusEntries(): List<Triple<Long, Long, Int>> = env.database.openHelper.writableDatabase
        .query("SELECT started_at, ended_at, duration_s FROM time_entries WHERE kind = 'focus' ORDER BY started_at")
        .use { cursor ->
            generateSequence {
                if (cursor.moveToNext()) Triple(cursor.getLong(0), cursor.getLong(1), cursor.getInt(2)) else null
            }.toList()
        }

    private fun advance(ms: Long) {
        env.clock.ms += ms
    }

    @Test
    fun startCountsDownAndRejectsASecondTimer() = runBlocking<Unit> {
        timer.start(taskId)

        val state = checkNotNull(timer.state.value)
        assertEquals(TimerPhase.Focus, state.phase)
        assertEquals(25 * 60, state.remainingS(env.clock.ms))
        assertThrows(IllegalStateException::class.java) { runBlocking { timer.start(taskId) } }

        advance(10 * MINUTE)
        assertNull(timer.tick())
        assertEquals(15 * 60, checkNotNull(timer.state.value).remainingS(env.clock.ms))
    }

    @Test
    fun finishedFocusLogsAnEntryAndStartsTheBreak() = runBlocking<Unit> {
        val start = env.clock.ms
        timer.start(taskId)

        advance(25 * MINUTE)
        val change = timer.tick()

        assertEquals(PhaseChange(TimerPhase.Focus, TimerPhase.Break), change)
        assertEquals(listOf(Triple(start, start + 25 * MINUTE, 25 * 60)), focusEntries())
        assertEquals(1, env.count("SELECT COUNT(*) FROM time_entries WHERE task_id = '$taskId' AND project_id = '$projectId'"))
        assertEquals(1, env.count("SELECT COUNT(*) FROM activity WHERE kind = 'time.logged'"))
    }

    @Test
    fun longBreakFollowsTheFourthFocusAndBreaksAreNotLogged() = runBlocking<Unit> {
        timer.start(taskId)

        val phases = List(8) {
            advance(checkNotNull(timer.state.value).phaseLengthS * 1000L)
            checkNotNull(timer.tick()).next
        }

        assertEquals(
            listOf(
                TimerPhase.Break,
                TimerPhase.Focus,
                TimerPhase.Break,
                TimerPhase.Focus,
                TimerPhase.Break,
                TimerPhase.Focus,
                TimerPhase.LongBreak,
                TimerPhase.Focus,
            ),
            phases,
        )
        assertEquals(4, focusEntries().size)
        assertEquals(4, env.count("SELECT COUNT(*) FROM time_entries"))
    }

    @Test
    fun pauseAndResumeMoveTheDeadline() = runBlocking<Unit> {
        timer.start(taskId)
        advance(10 * MINUTE)
        timer.pause()

        assertEquals(TimerStatus.Paused, checkNotNull(timer.state.value).status)
        advance(60 * MINUTE)
        assertNull(timer.tick())
        assertEquals(15 * 60, checkNotNull(timer.state.value).remainingS(env.clock.ms))

        timer.resume()
        advance(14 * MINUTE)
        assertNull(timer.tick())
        advance(MINUTE)
        assertEquals(PhaseChange(TimerPhase.Focus, TimerPhase.Break), timer.tick())
    }

    @Test
    fun aRestartedAppContinuesTheSavedTimer() = runBlocking<Unit> {
        timer.start(taskId)
        advance(5 * MINUTE)
        timer.pause()

        val after = newController()
        assertNull(after.state.value)
        after.recover()

        assertEquals(TimerStatus.Paused, checkNotNull(after.state.value).status)
        assertEquals(20 * 60, checkNotNull(after.state.value).remainingS(env.clock.ms))
    }

    @Test
    fun aRunningTimerIsRecoveredAndEndsLate() = runBlocking<Unit> {
        val start = env.clock.ms
        timer.start(taskId)
        advance(40 * MINUTE)

        val after = newController()
        after.recover()

        assertEquals(PhaseChange(TimerPhase.Focus, TimerPhase.Break), after.tick())
        assertEquals(listOf(Triple(start, start + 25 * MINUTE, 25 * 60)), focusEntries())
    }

    @Test
    fun stopClearsTheTimerForGood() = runBlocking<Unit> {
        timer.start(taskId)
        advance(MINUTE)

        timer.stop()

        assertNull(timer.state.value)
        assertNull(newController().also { it.recover() }.state.value)
        assertEquals(0, focusEntries().size)
        timer.pause()
        assertNull(timer.state.value)
    }

    @Test
    fun startRejectsAnUnknownTask() {
        assertThrows(IllegalStateException::class.java) { runBlocking { timer.start("missing") } }
    }

    private companion object {
        const val MINUTE = 60_000L
    }
}
