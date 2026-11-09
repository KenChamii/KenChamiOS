package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.dueTask
import com.kenchami.kenchamios.libraryEntry
import com.kenchami.kenchamios.projectSummary
import com.kenchami.kenchamios.timer.TimerConfig
import com.kenchami.kenchamios.timer.TimerState
import com.kenchami.kenchamios.ui.today.FocusCard
import com.kenchami.kenchamios.ui.today.TimerActions
import com.kenchami.kenchamios.ui.today.TodayContent
import com.kenchami.kenchamios.ui.today.TodayState
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TodayScreenTest {
    @get:Rule
    val compose = createComposeRule()

    private val bounds = DayBounds(start = 10_000, end = 20_000)
    private val noTimer = TimerActions({}, {}, {})

    // the hero pushes the lists below the first screenful
    private fun scrollTo(text: String) = compose.onNodeWithTag("today").performScrollToNode(hasText(text))

    private fun state(due: List<DueTask> = emptyList(), entries: List<LibraryEntry> = emptyList()) =
        TodayState(bounds, now = 15_000, due = due, continueItems = entries, projects = listOf(projectSummary()), activity = emptyList())

    @Test
    fun showsDueAndOverdueTasksAndTheActiveProject() {
        val due = listOf(dueTask("Write docs", 12_000), dueTask("Old task", 5_000))
        compose.setContent { TodayContent(state(due), null, noTimer, {}, {}, {}, {}, PaddingValues(0.dp)) }

        compose.onNodeWithText("Write docs").assertIsDisplayed()
        compose.onNodeWithText("Demo - overdue").assertIsDisplayed()
        scrollTo("Build - 1/3 tasks")
        compose.onNodeWithText("Build - 1/3 tasks").assertIsDisplayed()
    }

    @Test
    fun checkingATaskReportsIt() {
        var finished: String? = null
        compose.setContent {
            TodayContent(state(listOf(dueTask("Write docs", 12_000))), null, noTimer, { finished = it }, {}, {}, {}, PaddingValues(0.dp))
        }

        compose.onNode(isToggleable()).performClick()

        assertEquals("t-Write docs", finished)
    }

    @Test
    fun continueRowShowsTheNextEpisodeAndAddsOne() {
        var added: String? = null
        compose.setContent {
            TodayContent(state(entries = listOf(libraryEntry(progress = 3))), null, noTimer, {
            }, { added = it }, {}, {}, PaddingValues(0.dp))
        }

        compose.onNodeWithText("Anime - Episode 4").assertIsDisplayed()
        scrollTo("+1")
        compose.onNodeWithText("+1").performClick()

        assertEquals("e1", added)
    }

    @Test
    fun emptyDayShowsTheEmptyMessages() {
        compose.setContent { TodayContent(state(), null, noTimer, {}, {}, {}, {}, PaddingValues(0.dp)) }

        scrollTo("Nothing in progress.")
        compose.onNodeWithText("Nothing due or overdue.").assertIsDisplayed()
        compose.onNodeWithText("Nothing in progress.").assertIsDisplayed()
    }

    @Test
    fun runningTimerCardOffersPauseAndStop() {
        var paused = false
        var stopped = false
        val timer = TimerState.start("p1", "t1", TimerConfig.DEFAULT, now = System.currentTimeMillis())
        compose.setContent {
            TodayContent(
                state(),
                FocusCard(timer, "Write docs"),
                TimerActions({
                    paused = true
                }, {}, { stopped = true }),
                {},
                {},
                {},
                {},
                PaddingValues(0.dp),
            )
        }

        compose.onNodeWithText("Focus").assertIsDisplayed()
        compose.onNodeWithText("Write docs").assertIsDisplayed()
        compose.onNodeWithText("Pause").performClick()
        compose.onNodeWithText("Stop").performClick()

        assertEquals(true, paused)
        assertEquals(true, stopped)
    }

    @Test
    fun pausedTimerCardOffersResume() {
        var resumed = false
        val timer = TimerState.start("p1", "t1", TimerConfig.DEFAULT, now = System.currentTimeMillis()).pause(System.currentTimeMillis())
        compose.setContent {
            TodayContent(
                state(),
                FocusCard(timer, "Write docs"),
                TimerActions({
                }, { resumed = true }, {}),
                {},
                {},
                {},
                {},
                PaddingValues(0.dp),
            )
        }

        assertEquals(TimerStatus.Paused, timer.status)
        compose.onNodeWithText("Resume").performClick()

        assertEquals(true, resumed)
    }
}
