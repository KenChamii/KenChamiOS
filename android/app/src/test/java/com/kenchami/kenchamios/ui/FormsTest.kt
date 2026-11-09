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
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.repo.EntryInput
import com.kenchami.kenchamios.libraryEntry
import com.kenchami.kenchamios.projectSummary
import com.kenchami.kenchamios.ui.media.EntryForm
import com.kenchami.kenchamios.ui.quickadd.IdeaForm
import com.kenchami.kenchamios.ui.quickadd.MediaForm
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
    fun taskFormNeedsATitleThenSubmitsWithProjectAndPriority() {
        var submitted: List<Any?>? = null
        val clock = TestClock()
        compose.setContent {
            TaskForm(listOf(projectSummary("p1", "Alpha"), projectSummary("p2", "Beta")), clock, onAddIdea = {
            }) { project, title, priority, due ->
                submitted = listOf(project, title, priority, due)
            }
        }

        compose.onNodeWithText("Add task").assertIsNotEnabled()
        compose.onNodeWithText("Task").performTextInput("Ship it")
        compose.onNodeWithText("Beta").performClick()
        compose.onNodeWithText("High").performClick()
        compose.onNodeWithText("Add task").assertIsEnabled().performClick()

        assertEquals(listOf("p2", "Ship it", Priority.High, null), submitted)
    }

    @Test
    fun taskFormTodayDueIsLocalMidnight() {
        var due: Long? = null
        val clock = TestClock(ms = 1_700_000_000_000)
        compose.setContent {
            TaskForm(listOf(projectSummary()), clock, onAddIdea = {}) { _, _, _, dueAt -> due = dueAt }
        }

        compose.onNodeWithText("Task").performTextInput("A")
        compose.onNodeWithText("Today").performClick()
        compose.onNodeWithText("Add task").performClick()

        assertEquals(1_699_920_000_000, due)
    }

    @Test
    fun taskFormAsksForAnIdeaFirstWithoutProjects() {
        var wantsIdea = false
        compose.setContent { TaskForm(emptyList(), TestClock(), onAddIdea = { wantsIdea = true }) { _, _, _, _ -> } }

        compose.onNodeWithText("A task needs a project first.").assertIsDisplayed()
        compose.onNodeWithText("Add an idea").performClick()

        assertEquals(true, wantsIdea)
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
