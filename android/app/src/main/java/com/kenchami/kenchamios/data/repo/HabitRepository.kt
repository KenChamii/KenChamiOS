package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.HabitSummary
import com.kenchami.kenchamios.core.summarizeHabit
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.HabitCheckEntity
import com.kenchami.kenchamios.data.db.entity.HabitEntity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import java.time.LocalDate
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class HabitRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val habits = database.habitDao()

    /** Live habits in the order they were added, summarized for [today]. */
    fun observe(today: LocalDate): Flow<List<HabitSummary>> = combine(habits.observeAll(), habits.observeChecks()) { all, checks ->
        val days = checks.groupBy({ it.habitId }, { it.day })
        all.map { summarizeHabit(it.id, it.name, days[it.id].orEmpty(), today) }
    }

    suspend fun create(name: String): String {
        val cleanName = name.trim()
        require(cleanName.isNotEmpty()) { "habit name is required" }
        return database.withTransaction {
            val id = stamper.newId()
            habits.insert(HabitEntity(id, stamper.nowMs(), stamper.stamp(), stamper.deviceId(), false, cleanName))
            activity.record("habit.created", "habit", id, "Started habit $cleanName")
            id
        }
    }

    /** Soft deletes the habit and its check-ins. */
    suspend fun delete(id: String) {
        database.withTransaction {
            val device = stamper.deviceId()
            check(habits.softDelete(id, stamper.stamp(), device) > 0) { "habit not found" }
            habits.softDeleteChecks(id, stamper.stamp(), device)
            activity.record("habit.deleted", "habit", id, "Deleted habit")
        }
    }

    /** Checks the habit for [day], or clears the check when it is already there; returns the new state. */
    suspend fun toggle(id: String, day: LocalDate): Boolean = database.withTransaction {
        val habit = checkNotNull(habits.get(id)) { "habit not found" }
        val device = stamper.deviceId()
        val cleared = habits.clearCheck(id, day.toString(), stamper.stamp(), device) > 0
        if (cleared) {
            activity.record("habit.unchecked", "habit", id, "Unchecked ${habit.name} for $day")
        } else {
            habits.insertCheck(HabitCheckEntity(stamper.newId(), stamper.nowMs(), stamper.stamp(), device, false, id, day.toString()))
            activity.record("habit.checked", "habit", id, "Checked ${habit.name} for $day")
        }
        !cleared
    }
}
