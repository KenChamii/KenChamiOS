package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.core.capacity
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.plannedTask
import com.kenchami.kenchamios.ui.notes.DailyNotesContent
import com.kenchami.kenchamios.ui.notes.DailyNotesState
import com.kenchami.kenchamios.ui.notes.NotesActions
import com.kenchami.kenchamios.ui.planner.AvailableHoursSetting
import com.kenchami.kenchamios.ui.planner.PlanActions
import com.kenchami.kenchamios.ui.planner.PlanDayContent
import com.kenchami.kenchamios.ui.planner.PlanDayState
import com.kenchami.kenchamios.ui.planner.ReviewActions
import com.kenchami.kenchamios.ui.planner.ReviewDayContent
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDate
import java.time.YearMonth

@RunWith(RobolectricTestRunner::class)
class PlannerScreensTest {
    @get:Rule
    val compose = createComposeRule()

    private val today = LocalDate.of(2026, 11, 10)
    private val bounds = DayBounds(start = 10_000, end = 20_000)
    private val events = mutableListOf<String>()

    private fun planState(planned: List<PlannedTask>, candidates: List<PlannedTask> = emptyList()) =
        PlanDayState(today, bounds, planned, candidates, capacity(planned.map { it.durationMin }, 360))

    private fun showPlan(state: PlanDayState, query: String = "") = compose.setContent {
        PlanDayContent(
            state,
            query,
            PlanActions(
                onSearch = { events += "search $it" },
                onAdd = { events += "add ${it.title}" },
                onEstimate = { task, minutes -> events += "estimate ${task.title} $minutes" },
                onRemove = { events += "remove ${it.title}" },
            ),
            PaddingValues(0.dp),
        )
    }

    @Test
    fun thePlanShowsTheTotalAgainstTheHoursAndWarnsWhenItDoesNotFit() {
        showPlan(planState(listOf(plannedTask("Write", 240), plannedTask("Test", 180), plannedTask("Call", null))))

        compose.onNodeWithText("7h 00m of 6h 00m").assertIsDisplayed()
        compose.onNodeWithText("1 without an estimate").assertIsDisplayed()
        compose.onNodeWithText("1h 00m more than you have", substring = true).assertIsDisplayed()
    }

    @Test
    fun aPlanThatFitsHasNoWarning() {
        showPlan(planState(listOf(plannedTask("Write", 90))))

        compose.onNodeWithText("1h 30m of 6h 00m").assertIsDisplayed()
        compose.onNodeWithText("more than you have", substring = true).assertDoesNotExist()
    }

    @Test
    fun plannedTasksTakeAnEstimateOrLeaveToday() {
        showPlan(planState(listOf(plannedTask("Write", 30))))

        compose.onNodeWithContentDescription("Estimate for Write").performClick()
        compose.onNodeWithText("1h 00m").performClick()
        compose.onNodeWithContentDescription("Take Write off today").performClick()

        assertEquals(listOf("estimate Write 60", "remove Write"), events)
    }

    @Test
    fun suggestionsSayWhyAndGoOnTodayWithATap() {
        val overdue = plannedTask("Fix bug", null, TaskStatus.Doing, dueAt = 5_000, plannedOn = null)
        showPlan(planState(emptyList(), listOf(overdue)))

        compose.onNodeWithText("Nothing planned yet. Add tasks below.").assertIsDisplayed()
        compose.onNodeWithTag("plan-day").performScrollToNode(hasText("Fix bug"))
        compose.onNodeWithText("Demo - Overdue - In progress").assertIsDisplayed()
        compose.onNodeWithContentDescription("Plan Fix bug for today").performClick()
        compose.onNodeWithText("Search open tasks").performTextInput("fix")

        assertEquals(listOf("add Fix bug", "search fix"), events)
    }

    @Test
    fun anEmptySearchSaysNothingMatches() {
        showPlan(planState(emptyList()), query = "zzz")

        compose.onNodeWithTag("plan-day").performScrollToNode(hasText("No open task matches."))
        compose.onNodeWithText("No open task matches.").assertIsDisplayed()
    }

    private fun showReview(planned: List<PlannedTask>?) = compose.setContent {
        ReviewDayContent(
            planned,
            ReviewActions(
                onDone = { events += "done ${it.title}" },
                onTomorrow = { events += "tomorrow ${it.title}" },
                onDrop = { events += "drop ${it.title}" },
                onAddNote = { events += "note $it" },
                onOpenNote = { events += "open note" },
            ),
        )
    }

    @Test
    fun theReviewSettlesEachOpenTaskAndAddsANote() {
        showReview(listOf(plannedTask("Write"), plannedTask("Shipped", status = TaskStatus.Done)))

        compose.onNodeWithContentDescription("Mark Write done").performClick()
        compose.onNodeWithContentDescription("Move Write to tomorrow").performClick()
        compose.onNodeWithContentDescription("Drop Write from the plan").performClick()
        compose.onNodeWithContentDescription("Mark Shipped done").assertDoesNotExist()
        compose.onNodeWithText("Add to today's note").assertIsNotEnabled()
        compose.onNodeWithText("How did today go?").performScrollTo().performTextInput("Good day")
        compose.onNodeWithText("Good day").assertExists()
        compose.onNodeWithText("Add to today's note").performScrollTo().performClick()
        compose.onNodeWithText("Added").performScrollTo().assertIsDisplayed()
        compose.onNodeWithText("Open today's note").performScrollTo().performClick()

        assertEquals(listOf("done Write", "tomorrow Write", "drop Write", "note Good day", "open note"), events)
    }

    @Test
    fun anEmptyReviewSaysNothingWasPlanned() {
        showReview(emptyList())

        compose.onNodeWithText("Nothing was planned for today.").assertIsDisplayed()
    }

    private fun showNotes(draft: String?) = compose.setContent {
        DailyNotesContent(
            DailyNotesState(today, YearMonth.of(2026, 11), today, setOf("2026-11-03", "2026-11-10")),
            draft,
            NotesActions({ events += "select $it" }, { events += "month $it" }, { events += "edit $it" }),
            PaddingValues(0.dp),
        )
    }

    @Test
    fun theCalendarMarksDaysWithNotesAndPicksADay() {
        showNotes("Morning")

        compose.onNodeWithText("November 2026").assertIsDisplayed()
        compose.onNodeWithContentDescription("2026-11-03, has a note").assertIsDisplayed()
        compose.onNodeWithContentDescription("2026-11-12").performClick()
        compose.onNodeWithContentDescription("Next month").performClick()
        compose.onNodeWithContentDescription("Previous month").performClick()

        assertEquals(listOf("select 2026-11-12", "month 1", "month -1"), events)
    }

    @Test
    fun theEditorShowsTheNoteAndReportsTyping() {
        showNotes("Morning")

        compose.onNodeWithText("Tuesday, 10 November").assertIsDisplayed()
        compose.onNodeWithText("Morning").performTextReplacement("Evening")

        assertEquals(listOf("edit Evening"), events)
    }

    @Test
    fun noEditorWhileTheNoteLoads() {
        showNotes(null)

        compose.onNodeWithText("Note").assertDoesNotExist()
    }

    @Test
    fun availableHoursSaveAValidNumberOnly() {
        var saved: Int? = null
        compose.setContent { AvailableHoursSetting(360) { saved = it } }

        compose.onNodeWithText("6").assertIsDisplayed()
        compose.onNodeWithText("Save").assertIsNotEnabled()
        compose.onNodeWithText("6").performTextReplacement("40")
        compose.onNodeWithText("Use half an hour to 24 hours, like 6 or 7.5.").assertIsDisplayed()
        compose.onNodeWithText("Save").assertIsNotEnabled()
        compose.onNodeWithText("40").performTextReplacement("7.5")
        compose.onNodeWithText("Save").performClick()

        assertEquals(450, saved)
    }
}
