package com.kenchami.kenchamios.ui.media

import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.libraryEntry
import org.junit.Test
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue

class MediaQueuesTest {
    private val library = listOf(
        libraryEntry("old", status = MediaStatus.InProgress).copy(updatedAt = 1),
        libraryEntry("new", status = MediaStatus.InProgress).copy(updatedAt = 9),
        libraryEntry("plan-b", status = MediaStatus.Planned).copy(updatedAt = 5),
        libraryEntry("plan-a", status = MediaStatus.Planned).copy(updatedAt = 2),
        libraryEntry("game", type = MediaType.Game, status = MediaStatus.InProgress),
        libraryEntry("done", status = MediaStatus.Completed),
    )

    @Test
    fun continueStartsWithTheTitleTouchedLast() {
        assertEquals(listOf("new", "old"), mediaQueues(library, MediaType.Anime).continueWith.map { it.id })
    }

    @Test
    fun upNextStartsWithThePlannedTitleTouchedLongestAgo() {
        assertEquals(listOf("plan-a", "plan-b"), mediaQueues(library, MediaType.Anime).upNext.map { it.id })
    }

    @Test
    fun noTypeMeansAllTypes() {
        assertEquals(3, mediaQueues(library, null).continueWith.size)
    }

    @Test
    fun aTypeWithNothingQueuedIsEmpty() {
        assertTrue(mediaQueues(library, MediaType.Movie).isEmpty)
    }

    @Test
    fun upNextIsCapped() {
        val many = (1..20).map { libraryEntry("p$it", status = MediaStatus.Planned) }
        assertEquals(UP_NEXT_LIMIT, mediaQueues(many, null).upNext.size)
    }

    @Test
    fun nextUnitNamesTheNextEpisodeOrChapter() {
        assertEquals("Episode 4", nextUnit(libraryEntry(progress = 3)))
        assertEquals("Chapter 1", nextUnit(libraryEntry(type = MediaType.Manga, progress = 0)))
    }

    @Test
    fun nextUnitIsEmptyAtTheEndAndForTypesWithoutUnits() {
        assertNull(nextUnit(libraryEntry(progress = 12, total = 12)))
        assertNull(nextUnit(libraryEntry(type = MediaType.Game)))
    }
}
