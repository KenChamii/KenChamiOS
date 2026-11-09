package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.libraryEntry
import com.kenchami.kenchamios.projectSummary
import com.kenchami.kenchamios.prompt
import com.kenchami.kenchamios.ui.media.MediaContent
import com.kenchami.kenchamios.ui.media.MediaFilters
import com.kenchami.kenchamios.ui.media.MediaState
import com.kenchami.kenchamios.ui.projects.ProjectsContent
import com.kenchami.kenchamios.ui.prompts.PromptsContent
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ListScreensTest {
    @get:Rule
    val compose = createComposeRule()

    private val noPadding = PaddingValues(0.dp)

    @Test
    fun projectsShowStageAndProgressAndOpenOnTap() {
        var opened: String? = null
        compose.setContent {
            ProjectsContent("", {}, listOf(projectSummary("p9", "Alpha")), now = 1, onOpenProject = { opened = it }, padding = noPadding)
        }

        compose.onNodeWithText("Alpha").performClick()
        compose.onNodeWithText("Build - 1/3 tasks - just now").assertIsDisplayed()

        assertEquals("p9", opened)
    }

    @Test
    fun projectsReportTypedSearch() {
        var query = ""
        compose.setContent { ProjectsContent("", { query = it }, emptyList(), 1, {}, noPadding) }

        compose.onNodeWithText("Search").performTextInput("alp")

        assertEquals("alp", query)
        compose.onNodeWithText("No projects yet. Use + to add an idea.").assertIsDisplayed()
    }

    @Test
    fun tappingAPromptCopiesIt() {
        var copied: String? = null
        compose.setContent {
            PromptsContent(listOf(prompt(title = "Review", uses = 2)), { copied = it.id }, {}, {}, noPadding)
        }

        compose.onNodeWithText("Used 2 times").assertIsDisplayed()
        compose.onNodeWithText("Review").performClick()

        assertEquals("pr1", copied)
    }

    @Test
    fun promptStarTogglesTheFavorite() {
        var toggled: Boolean? = null
        compose.setContent {
            PromptsContent(listOf(prompt(favorite = true)), {}, { toggled = it.favorite }, {}, noPadding)
        }

        compose.onNodeWithContentDescription("Remove from favorites").performClick()

        assertEquals(true, toggled)
    }

    @Test
    fun emptyPromptLibraryExplainsHowToAdd() {
        compose.setContent { PromptsContent(emptyList(), {}, {}, {}, noPadding) }

        compose.onNodeWithText("No prompts yet. Use + to add one.").assertIsDisplayed()
    }

    @Test
    fun mediaRowsShowProgressAndOnlyOfferPlusOneBelowTheTotal() {
        var added: String? = null
        val entries = listOf(libraryEntry("open", "Open", progress = 3), libraryEntry("full", "Full", progress = 12))
        compose.setContent {
            MediaContent(MediaState(MediaFilters(), entries), {}, {}, { added = it }, {}, noPadding)
        }

        compose.onNodeWithText("Anime - In progress - 3/12").assertIsDisplayed()
        compose.onNodeWithText("+1").performClick()

        assertEquals("open", added)
    }

    @Test
    fun mediaFiltersReportTheChosenTypeAndStatus() {
        var type: MediaType? = null
        var status: MediaStatus? = null
        compose.setContent {
            MediaContent(MediaState(MediaFilters(), emptyList()), { type = it }, { status = it }, {}, {}, noPadding)
        }

        compose.onNodeWithText("Manga").performClick()
        compose.onNodeWithText("Completed").performClick()

        assertEquals(MediaType.Manga, type)
        assertEquals(MediaStatus.Completed, status)
    }
}
