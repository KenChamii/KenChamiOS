package com.kenchami.kenchamios.data.repo

import android.database.sqlite.SQLiteConstraintException
import com.kenchami.kenchamios.data.TestDatabase
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
class HabitRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var habits: HabitRepository
    private val today = LocalDate.of(2026, 11, 10)

    @Before
    fun setUp() {
        env = TestDatabase()
        habits = HabitRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun checkingInBuildsAStreakAndTogglingClearsIt() = runBlocking<Unit> {
        val id = habits.create(" Read ")
        habits.toggle(id, today.minusDays(2))
        habits.toggle(id, today.minusDays(1))

        assertEquals(true, habits.toggle(id, today))
        assertEquals(3, habits.observe(today).first().single().streak)

        assertEquals(false, habits.toggle(id, today))
        val habit = habits.observe(today).first().single()
        assertEquals("Read", habit.name)
        assertEquals(2, habit.streak)
        assertEquals(false, habit.checkedToday)
        assertEquals(
            listOf("habit.created", "habit.checked", "habit.checked", "habit.checked", "habit.unchecked"),
            env.database.activityDao().kindsFor(id),
        )
    }

    @Test
    fun aDayHasOneLiveCheckButADeletedOneDoesNotCount() = runBlocking<Unit> {
        val id = habits.create("Run")
        habits.toggle(id, today)
        habits.toggle(id, today)
        habits.toggle(id, today)

        assertEquals(1, env.count("SELECT COUNT(*) FROM habit_checks WHERE deleted = 0"))
        assertEquals(2, env.count("SELECT COUNT(*) FROM habit_checks"))
        assertThrows(SQLiteConstraintException::class.java) {
            env.database.openHelper.writableDatabase.execSQL(
                "INSERT INTO habit_checks (id, created_at, updated_at, device_id, deleted, habit_id, day) " +
                    "VALUES ('copy', 1, 1, 'd', 0, '$id', '$today')",
            )
        }
    }

    @Test
    fun deleteHidesTheHabitWithItsChecks() = runBlocking<Unit> {
        val id = habits.create("Run")
        habits.toggle(id, today)

        habits.delete(id)

        assertEquals(emptyList<Any>(), habits.observe(today).first())
        assertEquals(0, env.count("SELECT COUNT(*) FROM habit_checks WHERE deleted = 0"))
        assertThrows(IllegalStateException::class.java) { runBlocking { habits.toggle(id, today) } }
    }

    @Test
    fun createRejectsABlankName() {
        assertThrows(IllegalArgumentException::class.java) { runBlocking { habits.create("  ") } }
    }
}
