package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.DEFAULT_AVAILABLE_MIN
import com.kenchami.kenchamios.core.MAX_DURATION_MIN
import com.kenchami.kenchamios.core.isValidAvailableMinutes
import com.kenchami.kenchamios.core.likePattern
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.model.Inbox
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.time.LocalDate
import javax.inject.Inject
import javax.inject.Singleton

// the same key as the desktop, the setting stays on each device
private const val AVAILABLE_KEY = "planner.available_minutes"

/** When a task is planned: a local day, an optional time block on it and an estimate. */
data class TaskPlan(val plannedOn: LocalDate?, val scheduledAt: Long?, val durationMin: Int?) {
    companion object {
        val NONE = TaskPlan(null, null, null)
    }
}

@Singleton
class PlannerRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
    private val settings: LocalSettings,
) {
    private val tasks = database.taskDao()

    /** Tasks planned for [day], time blocks first by start. */
    fun observePlanned(day: LocalDate): Flow<List<PlannedTask>> = tasks.observePlanned(day.toString())

    /**
     * Open tasks worth planning for [today]: due before [until], in the Inbox, in progress or left from an earlier day.
     * With a [query], open tasks whose title matches it instead.
     */
    fun observeCandidates(today: LocalDate, until: Long, query: String): Flow<List<PlannedTask>> =
        tasks.observeCandidates(today.toString(), until, Inbox.ID, query.trim().takeIf { it.isNotEmpty() }?.let(::likePattern))

    /** Replaces the task's plan; [TaskPlan.NONE] takes it off the planner. */
    suspend fun setPlan(id: String, plan: TaskPlan) {
        require(plan.scheduledAt == null || plan.plannedOn != null) { "a scheduled task needs a planned day" }
        require(plan.scheduledAt == null || plan.scheduledAt >= 0) { "the time block start is not valid" }
        require(plan.durationMin == null || plan.durationMin in 1..MAX_DURATION_MIN) { "the duration must be 1 to 1440 minutes" }
        database.withTransaction {
            val task = checkNotNull(tasks.get(id)) { "task not found" }
            val day = plan.plannedOn?.toString()
            tasks.setPlan(id, day, plan.scheduledAt, plan.durationMin, stamper.stamp(), stamper.deviceId())
            when {
                day == null -> activity.record("task.unplanned", "task", id, "Unplanned ${task.title}")
                plan.scheduledAt == null -> activity.record("task.planned", "task", id, "Planned ${task.title} for $day")
                else -> activity.record("task.planned", "task", id, "Scheduled ${task.title} on $day")
            }
        }
    }

    /** Minutes of planned work that fit in a day; 6 hours until the user changes it. */
    fun observeAvailableMinutes(): Flow<Int> = settings.observePrefix(AVAILABLE_KEY).map { values ->
        values.firstOrNull { it.key == AVAILABLE_KEY }?.valueJson?.toIntOrNull()?.takeIf(::isValidAvailableMinutes)
            ?: DEFAULT_AVAILABLE_MIN
    }

    suspend fun setAvailableMinutes(minutes: Int) {
        require(isValidAvailableMinutes(minutes)) { "the hours you have must be between half an hour and 24 hours" }
        settings.put(AVAILABLE_KEY, minutes.toString())
    }
}
