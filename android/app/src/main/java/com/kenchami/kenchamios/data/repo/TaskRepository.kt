package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.OrderKey
import com.kenchami.kenchamios.core.RepeatRule
import com.kenchami.kenchamios.core.nextDueAt
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.Flow
import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TaskRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
    private val clock: Clock,
) {
    private val tasks = database.taskDao()
    private val projects = database.projectDao()
    private val children = database.projectChildDao()

    fun observeTopLevel(projectId: String): Flow<List<TaskEntity>> = tasks.observeTopLevel(projectId)

    /** Open tasks due before [until], overdue ones first. */
    fun observeDue(until: Long): Flow<List<DueTask>> = tasks.observeDue(until)

    suspend fun get(id: String): TaskEntity? = tasks.get(id)

    /** New tasks start in To do, at the end of their siblings; the inbox is created on its first task. */
    suspend fun create(
        projectId: String,
        title: String,
        priority: Priority = Priority.Medium,
        dueAt: Long? = null,
        recurrence: String? = null,
    ): String {
        val cleanTitle = title.trim()
        require(cleanTitle.isNotEmpty()) { "task title is required" }
        require(recurrence == null || RepeatRule.parse(recurrence) != null) { "invalid repeat rule" }
        return database.withTransaction {
            if (projectId == Inbox.ID) ensureInbox(projects, stamper, activity)
            val id = stamper.newId()
            tasks.insert(
                TaskEntity(
                    id = id,
                    createdAt = stamper.nowMs(),
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                    deleted = false,
                    projectId = projectId,
                    parentId = null,
                    title = cleanTitle,
                    bodyMd = "",
                    acceptanceMd = "",
                    relatedFiles = emptyList(),
                    status = TaskStatus.Todo,
                    priority = priority,
                    dueAt = dueAt,
                    orderKey = keyAfterLast(projectId, null, TaskStatus.Todo),
                    tags = emptyList(),
                    recurrence = recurrence,
                    plannedOn = null,
                    scheduledAt = null,
                    durationMin = null,
                ),
            )
            activity.record("task.created", "task", id, "Added task $cleanTitle")
            id
        }
    }

    /**
     * Puts the task last in the column of [status]. Completing a repeating task adds its next occurrence
     * in the same transaction and returns its id.
     */
    suspend fun setStatus(id: String, status: TaskStatus): String? = database.withTransaction {
        val task = checkNotNull(tasks.get(id)) { "task not found" }
        val last = tasks.lastKey(task.projectId, task.parentId, status, id)
        tasks.update(
            task.copy(
                status = status,
                orderKey = OrderKey.between(last.orEmpty(), null),
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
            ),
        )
        activity.record("task.moved", "task", id, "Moved task to ${status.wire}")
        val completed = status == TaskStatus.Done && task.status != TaskStatus.Done
        if (completed && task.recurrence != null) addNextOccurrence(task) else null
    }

    private suspend fun addNextOccurrence(task: TaskEntity): String {
        val id = stamper.newId()
        tasks.insert(
            task.copy(
                id = id,
                createdAt = stamper.nowMs(),
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
                status = TaskStatus.Todo,
                dueAt = nextDueAt(task.recurrence, task.dueAt, clock.millis(), clock.zone),
                orderKey = keyAfterLast(task.projectId, task.parentId, TaskStatus.Todo),
                // the plan belongs to the finished occurrence, the next one is planned on its own
                plannedOn = null,
                scheduledAt = null,
                durationMin = null,
            ),
        )
        activity.record("task.created", "task", id, "Added task ${task.title}")
        return id
    }

    /** Moves a top-level task with its subtasks to another project, last in its column. */
    suspend fun moveToProject(id: String, projectId: String) {
        database.withTransaction {
            val task = checkNotNull(tasks.get(id)) { "task not found" }
            require(task.parentId == null) { "a subtask moves with its parent" }
            if (projectId == Inbox.ID) ensureInbox(projects, stamper, activity)
            val target = checkNotNull(projects.get(projectId)) { "project not found" }
            val device = stamper.deviceId()
            tasks.update(
                task.copy(
                    projectId = projectId,
                    orderKey = keyAfterLast(projectId, null, task.status),
                    updatedAt = stamper.stamp(),
                    deviceId = device,
                ),
            )
            tasks.moveSubtasks(id, projectId, stamper.stamp(), device)
            activity.record("task.moved", "task", id, "Moved ${task.title} to ${target.name}")
        }
    }

    /** Soft deletes the task with its subtasks and checklist items. */
    suspend fun delete(id: String) {
        database.withTransaction {
            val device = stamper.deviceId()
            val changed = tasks.softDeleteTree(id, stamper.stamp(), device)
            check(changed > 0) { "task not found" }
            children.softDeleteChecklistsOfTask(id, stamper.stamp(), device)
            activity.record("task.deleted", "task", id, "Deleted task")
        }
    }

    private suspend fun keyAfterLast(projectId: String, parentId: String?, status: TaskStatus): String =
        OrderKey.between(tasks.lastKey(projectId, parentId, status, "").orEmpty(), null)
}
