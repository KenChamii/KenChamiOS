package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.TestClock
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.repo.EntryInput
import com.kenchami.kenchamios.libraryEntry
import com.kenchami.kenchamios.projectSummary
import com.kenchami.kenchamios.ui.media.EntryForm
import com.kenchami.kenchamios.ui.quickadd.IdeaForm
import com.kenchami.kenchamios.ui.quickadd.MediaForm
import com.kenchami.kenchamios.ui.quickadd.NewTask
import com.kenchami.kenchamios.ui.quickadd.PromptForm
import com.kenchami.kenchamios.ui.quickadd.TaskForm
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class FormsTest {
    @get:Rule
    val compose = createComposeRule()

    @Test
    fun taskFormNeedsATitleThenSubmitsWithProjectPriorityAndRepeat() {
        var submitted: NewTask? = null
        compose.setContent {
            TaskForm(listOf(projectSummary("p1", "Alpha"), projectSummary("p2", "Beta")), TestClock(), "") { submitted = it }
        }

        compose.onNodeWithText("Add task").performScrollTo().assertIsNotEnabled()
        compose.onNodeWithText("Task").performTextInput("Ship it")
        compose.onNodeWithText("Beta").performScrollTo().performClick()
        compose.onNodeWithText("High").performScrollTo().performClick()
        compose.onNodeWithText("Every week").performScrollTo().performClick()
        compose.onNodeWithText("Add task").performScrollTo().assertIsEnabled().performClick()

        assertEquals(NewTask("p2", "Ship it", Priority.High, null, "weekly:1"), submitted)
    }

    @Test
    fun taskFormTodayDueIsLocalMidnight() {
        var submitted: NewTask? = null
        compose.setContent { TaskForm(listOf(projectSummary()), TestClock(ms = 1_700_000_000_000), "") { submitted = it } }

        compose.onNodeWithText("Task").performTextInput("A")
        compose.onNodeWithText("Today").performScrollTo().performClick()
        compose.onNodeWithText("Add task").performScrollTo().performClick()

        assertEquals(1_699_920_000_000, submitted?.dueAt)
    }

    @Test
    fun taskFormFilesIntoTheInboxWithoutAProject() {
        var submitted: NewTask? = null
        compose.setContent { TaskForm(emptyList(), TestClock(), "Shared link") { submitted = it } }

        compose.onNodeWithText("Add task").performScrollTo().performClick()

        assertEquals(NewTask(Inbox.ID, "Shared link", Priority.Medium, null, null), submitted)
    }

    @Test
    fun taskFormReadsTheLineAndPreviewsWhatItFound() {
        var submitted: NewTask? = null
        compose.setContent { TaskForm(listOf(projectSummary("p1", "Home")), TestClock(ms = 1_700_000_000_000), "") { submitted = it } }

        compose.onNodeWithText("Task").performTextInput("Pay rent friday !high #home every month")

        compose.onNodeWithText("Due", substring = true).assertIsDisplayed()
        compose.onNodeWithText("High priority - Every month - Home", substring = true).assertIsDisplayed()
        compose.onNodeWithText("Add task").performScrollTo().performClick()
        // the test clock is Tuesday 2023-11-14 in UTC, friday is the 17th
        assertEquals(NewTask("p1", "Pay rent", Priority.High, 1_700_179_200_000, "monthly:1"), submitted)
    }

    @Test
    fun taskFormFlagsAnUnknownProjectTag() {
        compose.setContent { TaskForm(listOf(projectSummary("p1", "Home")), TestClock(), "Call #nowhere") {} }

        compose.onNodeWithText("No project named #nowhere").assertIsDisplayed()
    }

    @Test
    fun ideaFormSubmitsNameAndDescription() {
        var submitted: Pair<String, String>? = null
        compose.setContent { IdeaForm { name, description -> submitted = name to description } }

        compose.onNodeWithText("Add idea").assertIsNotEnabled()
        compose.onNodeWithText("Idea").performTextInput("Budget app")
        compose.onNodeWithText("Description").performTextInput("track money")
        compose.onNodeWithText("Add idea").performClick()

        assertEquals("Budget app" to "track money", submitted)
    }

    @Test
    fun promptFormCanLinkAProject() {
        var submitted: Triple<String, String, String?>? = null
        compose.setContent {
            PromptForm(listOf(projectSummary("p1", "Alpha"))) { title, body, project -> submitted = Triple(title, body, project) }
        }

        compose.onNodeWithText("Title").performTextInput("Review")
        compose.onNodeWithText("Prompt").performTextInput("Look at {{ project.name }}")
        compose.onNodeWithText("Alpha").performClick()
        compose.onNodeWithText("Add prompt").performClick()

        assertEquals(Triple("Review", "Look at {{ project.name }}", "p1"), submitted)
    }

    @Test
    fun mediaFormRefusesATotalThatIsNotANumber() {
        compose.setContent { MediaForm { _, _, _, _ -> } }

        compose.onNodeWithText("Title").performTextInput("Berserk")
        compose.onNodeWithText("Total episodes, chapters or hours (optional)").performTextInput("abc")
        compose.onNodeWithText("Add to library").assertIsNotEnabled()
    }

    @Test
    fun mediaFormSubmitsWithAValidTotal() {
        var submitted: List<Any?>? = null
        compose.setContent { MediaForm { type, title, status, total -> submitted = listOf(type, title, status, total) } }

        compose.onNodeWithText("Title").performTextInput("Berserk")
        compose.onNodeWithText("Manga").performClick()
        compose.onNodeWithText("In progress").performClick()
        compose.onNodeWithText("Total episodes, chapters or hours (optional)").performTextInput("364")
        compose.onNodeWithText("Add to library").assertIsEnabled().performClick()

        assertEquals(listOf(MediaType.Manga, "Berserk", MediaStatus.InProgress, 364), submitted)
    }

    @Test
    fun entryFormSavesTheEditedFields() {
        var saved: EntryInput? = null
        compose.setContent {
            EntryForm(libraryEntry(progress = 3), { saved = it }, {}, {}, PaddingValues(0.dp))
        }

        compose.onNodeWithText("Completed").performClick()
        compose.onNodeWithText("Rating (1 to 10)").performTextInput("9")
        compose.onNodeWithText("Tags, separated by commas").performTextInput("fav, seasonal")
        compose.onNodeWithText("Save").performScrollTo().performClick()

        val input = checkNotNull(saved)
        assertEquals(MediaStatus.Completed, input.status)
        assertEquals(9, input.rating)
        assertEquals(12, input.progressTotal)
        assertEquals(listOf("fav", "seasonal"), input.tags.map(String::trim))
    }

    @Test
    fun entryFormRefusesARatingOutsideOneToTen() {
        var saved: EntryInput? = null
        compose.setContent { EntryForm(libraryEntry(), { saved = it }, {}, {}, PaddingValues(0.dp)) }

        compose.onNodeWithText("Rating (1 to 10)").performTextInput("11")

        compose.onNodeWithText("Save").assertIsNotEnabled()
        assertNull(saved)
    }

    @Test
    fun entryFormProgressButtonsFollowTheCounter() {
        var added = 0
        var subtracted = 0
        compose.setContent {
            EntryForm(libraryEntry(progress = 12), {}, { added++ }, { subtracted++ }, PaddingValues(0.dp))
        }

        compose.onNodeWithText("+1").assertIsNotEnabled()
        compose.onNodeWithText("-1").performClick()

        assertEquals(0, added)
        assertEquals(1, subtracted)
    }
}
