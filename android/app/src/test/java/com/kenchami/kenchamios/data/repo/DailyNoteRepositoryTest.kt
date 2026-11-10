package com.kenchami.kenchamios.data.repo

import android.database.sqlite.SQLiteConstraintException
import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.db.entity.DailyNoteEntity
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDate

@RunWith(RobolectricTestRunner::class)
class DailyNoteRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var notes: DailyNoteRepository
    private val day = LocalDate.of(2026, 11, 10)

    @Before
    fun setUp() {
        env = TestDatabase()
        notes = DailyNoteRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun appendingAddsAnEmptyLineBetween() {
        assertEquals("Done", appendToNote("", "Done"))
        assertEquals("Morning\n\nEvening", appendToNote("Morning\n\n", "Evening"))
    }

    @Test
    fun theFirstSaveCreatesTheNoteAndLaterSavesReplaceIt() = runBlocking<Unit> {
        assertEquals("", notes.body(day))

        notes.save(day, "First")
        notes.save(day, "Second")

        assertEquals("Second", notes.body(day))
        assertEquals(1, env.count("SELECT COUNT(*) FROM daily_notes"))
        val id = checkNotNull(env.database.dailyNoteDao().get(day.toString())).id
        assertEquals(listOf("daily_note.created"), env.database.activityDao().kindsFor(id))
        assertEquals(1, env.count("SELECT COUNT(*) FROM change_log WHERE table_name = 'daily_notes'"))
    }

    @Test
    fun appendGoesBelowTheNoteAndRefusesBlankText() = runBlocking<Unit> {
        notes.append(day, " Went well ")
        notes.append(day, "Tomorrow: docs")

        assertEquals("Went well\n\nTomorrow: docs", notes.body(day))
        assertThrows(IllegalArgumentException::class.java) { runBlocking { notes.append(day, "  ") } }
    }

    @Test
    fun theCalendarDotsAreTheDaysWithTextInTheRange() = runBlocking<Unit> {
        notes.save(day, "Text")
        notes.save(day.plusDays(1), "   ")
        notes.save(day.plusMonths(1), "Later")

        assertEquals(setOf("2026-11-10"), notes.observeDaysWithText(day.withDayOfMonth(1), day.withDayOfMonth(30)).first())
    }

    @Test
    fun aDayHasOneLiveNoteButADeletedOneDoesNotCount() = runBlocking<Unit> {
        notes.save(day, "Kept")
        val dao = env.database.dailyNoteDao()
        val deleted = DailyNoteEntity("old", 1, 1, "d", true, day.toString(), "Gone")
        dao.insert(deleted)

        assertThrows(SQLiteConstraintException::class.java) {
            runBlocking { dao.insert(deleted.copy(id = "twin", deleted = false)) }
        }
        assertEquals("Kept", notes.body(day))
    }
}
