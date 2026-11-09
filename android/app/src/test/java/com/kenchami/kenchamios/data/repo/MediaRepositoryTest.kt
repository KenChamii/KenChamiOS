package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import kotlinx.coroutines.flow.first
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
class MediaRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var media: MediaRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        media = MediaRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    private suspend fun entry(id: String) = checkNotNull(media.observeEntry(id).first())

    private fun TestDatabase.progressDeltas(): List<Int> =
        database.openHelper.writableDatabase.query("SELECT delta FROM media_progress_log ORDER BY rowid").use { cursor ->
            generateSequence { if (cursor.moveToNext()) cursor.getInt(0) else null }.toList()
        }

    private fun input(status: MediaStatus, total: Int? = null, rating: Int? = null) =
        EntryInput(status, total, rating, " good ", listOf("fav", "fav"), null, null, "  ")

    @Test
    fun addCreatesTheItemAndEntryWithCleanedFields() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "  Frieren ", MediaStatus.Planned, total = 28, year = 2023)

        val added = entry(id)
        assertEquals("Frieren", added.title)
        assertEquals(MediaStatus.Planned, added.status)
        assertEquals(0, added.progress)
        assertEquals(28, added.progressTotal)
        assertEquals(2023, added.year)
        assertEquals(1, media.observeLibrary().first().size)
        assertEquals(listOf("media.added"), env.database.activityDao().kindsFor(id))
    }

    @Test
    fun addStartsOrFinishesTheDatesFromTheStatus() = runBlocking<Unit> {
        val watching = entry(media.add(MediaType.Anime, "A", MediaStatus.InProgress))
        val finished = entry(media.add(MediaType.Anime, "B", MediaStatus.Completed))

        assertEquals(env.clock.ms, watching.startedAt)
        assertNull(watching.finishedAt)
        assertEquals(env.clock.ms, finished.finishedAt)
    }

    @Test
    fun addRejectsABlankTitleAndANegativeTotal() {
        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { media.add(MediaType.Game, " ", MediaStatus.Planned) }
        }
        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { media.add(MediaType.Game, "G", MediaStatus.Planned, total = -1) }
        }
    }

    @Test
    fun firstStepStartsAPlannedEntryAndLogsTheChange() = runBlocking<Unit> {
        val id = media.add(MediaType.Manga, "Berserk", MediaStatus.Planned, total = 100)

        media.increment(id)

        val stepped = entry(id)
        assertEquals(1, stepped.progress)
        assertEquals(MediaStatus.InProgress, stepped.status)
        assertEquals(env.clock.ms, stepped.startedAt)
        assertEquals(1, env.count("SELECT COUNT(*) FROM media_progress_log WHERE delta = 1 AND value_after = 1"))
        assertEquals(listOf("media.added", "media.progress"), env.database.activityDao().kindsFor(id))
    }

    @Test
    fun reachingTheTotalCompletesTheEntryAndStopsThere() = runBlocking<Unit> {
        val id = media.add(MediaType.Movie, "Heat", MediaStatus.Planned, total = 1)

        media.increment(id)
        media.increment(id)

        val done = entry(id)
        assertEquals(1, done.progress)
        assertEquals(MediaStatus.Completed, done.status)
        assertEquals(env.clock.ms, done.finishedAt)
        assertEquals(1, env.count("SELECT COUNT(*) FROM media_progress_log"))
    }

    @Test
    fun decrementTakesOneOffButStopsAtZero() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "A", MediaStatus.InProgress)
        media.setProgress(id, 2)

        repeat(3) { media.decrement(id) }

        assertEquals(0, entry(id).progress)
        assertEquals(listOf(2, -1, -1), env.progressDeltas())
    }

    @Test
    fun progressWithoutATotalKeepsCounting() = runBlocking<Unit> {
        val id = media.add(MediaType.Game, "Hades", MediaStatus.InProgress)

        repeat(3) { media.increment(id) }

        assertEquals(3, entry(id).progress)
        assertEquals(MediaStatus.InProgress, entry(id).status)
    }

    @Test
    fun setProgressRejectsValuesOutsideTheRange() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "A", MediaStatus.Planned, total = 12)

        assertThrows(IllegalArgumentException::class.java) { runBlocking { media.setProgress(id, 13) } }
        assertThrows(IllegalArgumentException::class.java) { runBlocking { media.setProgress(id, -1) } }
    }

    @Test
    fun updateKeepsTheRatingNotesAndTags() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "A", MediaStatus.Planned, total = 12)

        media.update(id, input(MediaStatus.Completed, total = 12, rating = 9))

        val updated = entry(id)
        assertEquals(9, updated.rating)
        assertEquals("good", updated.notes)
        assertEquals(listOf("fav"), updated.tags)
        assertNull(updated.platform)
        assertEquals(MediaStatus.Completed, updated.status)
        assertEquals(env.clock.ms, updated.finishedAt)
    }

    @Test
    fun updateRejectsABadRatingOrATotalBelowTheProgress() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "A", MediaStatus.InProgress)
        media.setProgress(id, 5)

        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { media.update(id, input(MediaStatus.InProgress, rating = 11)) }
        }
        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { media.update(id, input(MediaStatus.InProgress, total = 4)) }
        }
    }

    @Test
    fun deleteHidesTheEntryAndItsItem() = runBlocking<Unit> {
        val id = media.add(MediaType.Anime, "A", MediaStatus.Planned)

        media.delete(id)

        assertEquals(emptyList<String>(), media.observeLibrary().first().map { it.title })
        assertEquals(0, env.count("SELECT COUNT(*) FROM media_items WHERE deleted = 0"))
        assertEquals(listOf("media.added", "media.deleted"), env.database.activityDao().kindsFor(id))
    }

    @Test
    fun continueRowListsActiveShowsNewestFirstAndSkipsMovies() = runBlocking<Unit> {
        val anime = media.add(MediaType.Anime, "Anime", MediaStatus.InProgress)
        env.clock.ms += 1_000
        val game = media.add(MediaType.Game, "Game", MediaStatus.InProgress)
        media.add(MediaType.Movie, "Movie", MediaStatus.InProgress)
        media.add(MediaType.Manga, "Planned", MediaStatus.Planned)
        media.setProgress(anime, 12)

        val items = continueItems(media.observeLibrary().first())

        assertEquals(listOf("Anime", "Game"), items.map { it.title })
        assertEquals("Episode 13", nextLabel(items[0]))
        assertEquals("Just started", nextLabel(items[1]))
        assertEquals(game, items[1].id)
    }
}
