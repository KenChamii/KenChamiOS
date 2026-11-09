package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.dueTask
import com.kenchami.kenchamios.libraryEntry
import com.kenchami.kenchamios.timer.TimerConfig
import com.kenchami.kenchamios.timer.TimerState
import com.kenchami.kenchamios.todayState
import com.kenchami.kenchamios.ui.quickadd.QuickAddTab
import com.kenchami.kenchamios.ui.today.FocusCard
import com.kenchami.kenchamios.ui.today.SetupStep
import com.kenchami.kenchamios.ui.today.TimerActions
import com.kenchami.kenchamios.ui.today.TodayContent
import com.kenchami.kenchamios.ui.today.TodayNav
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

    private val noTimer = TimerActions({}, {}, {})
    private val quickAdds = mutableListOf<QuickAddTab>()
    private var syncOpened = false
    private val nav = TodayNav({}, {}, { quickAdds += it }, { syncOpened = true })

    // the hero pushes the lists below the first screenful
    private fun scrollTo(text: String) = compose.onNodeWithTag("today").performScrollToNode(hasText(text))

    private fun show(
        state: TodayState,
        focus: FocusCard? = null,
        timer: TimerActions = noTimer,
        onFinishTask: (String) -> Unit = {},
        onAddOne: (String) -> Unit = {},
    ) = compose.setContent { TodayContent(state, focus, timer, onFinishTask, onAddOne, nav, PaddingValues(0.dp)) }

    @Test
    fun greetsWithTheDate() {
        show(todayState())

        compose.onNodeWithText("Good morning").assertIsDisplayed()
        compose.onNodeWithText("Tuesday, 10 November").assertIsDisplayed()
    }

    @Test
    fun showsDueAndOverdueTasksAndTheActiveProject() {
        show(todayState(due = listOf(dueTask("Write docs", 12_000), dueTask("Old task", 5_000))))

        scrollTo("Write docs")
        compose.onNodeWithText("Write docs").assertIsDisplayed()
        scrollTo("Demo - overdue")
        compose.onNodeWithText("Demo - overdue").assertIsDisplayed()
        scrollTo("Build - 1/3 tasks")
        compose.onNodeWithText("Build - 1/3 tasks").assertIsDisplayed()
    }

    @Test
    fun tickingATaskReportsItAndKeepsTheTick() {
        var finished: String? = null
        show(todayState(due = listOf(dueTask("Write docs", 12_000))), onFinishTask = { finished = it })

        scrollTo("Write docs")
        compose.onNode(isToggleable()).performClick()

        assertEquals("t-Write docs", finished)
        compose.onNode(isToggleable()).assertIsOn().assertIsNotEnabled()
    }

    @Test
    fun continueRowShowsTheNextEpisodeAndAddsOne() {
        var added: String? = null
        show(todayState(entries = listOf(libraryEntry(progress = 3))), onAddOne = { added = it })

        scrollTo("Anime - Episode 4")
        compose.onNodeWithText("Anime - Episode 4").assertIsDisplayed()
        compose.onNodeWithText("+1").performClick()

        assertEquals("e1", added)
    }

    @Test
    fun emptyDayOffersToAddATaskAndHidesEmptyBlocks() {
        show(todayState(projects = emptyList()))

        scrollTo("Add a task")
        compose.onNodeWithText("Nothing due today.").assertIsDisplayed()
        compose.onNodeWithText("Add a task").performClick()

        assertEquals(listOf(QuickAddTab.Task), quickAdds)
        compose.onNodeWithText("Continue").assertDoesNotExist()
        // only the hero count, no section
        compose.onAllNodesWithText("Active projects").assertCountEquals(1)
        compose.onNodeWithText("Recent activity").assertDoesNotExist()
    }

    @Test
    fun firstRunChecklistLeadsToEachAction() {
        show(todayState(projects = emptyList(), setup = SetupStep.entries))

        compose.onNodeWithText("Get started").assertIsDisplayed()
        compose.onNodeWithText("due today").assertDoesNotExist()
        compose.onNodeWithText("Create your first project").performClick()
        compose.onNodeWithText("Add a task to it").performClick()
        compose.onNodeWithText("Add a show, game or album").performClick()
        compose.onNodeWithText("Pair with the desktop app").performClick()

        assertEquals(listOf(QuickAddTab.Idea, QuickAddTab.Task, QuickAddTab.Media), quickAdds)
        assertEquals(true, syncOpened)
    }

    @Test
    fun runningTimerCardOffersPauseAndStop() {
        var paused = false
        var stopped = false
        val timer = TimerState.start("p1", "t1", TimerConfig.DEFAULT, now = System.currentTimeMillis())
        show(todayState(), FocusCard(timer, "Write docs"), TimerActions({ paused = true }, {}, { stopped = true }))

        scrollTo("Pause")
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
        show(todayState(), FocusCard(timer, "Write docs"), TimerActions({}, { resumed = true }, {}))

        assertEquals(TimerStatus.Paused, timer.status)
        scrollTo("Resume")
        compose.onNodeWithText("Resume").performClick()

        assertEquals(true, resumed)
    }
}
