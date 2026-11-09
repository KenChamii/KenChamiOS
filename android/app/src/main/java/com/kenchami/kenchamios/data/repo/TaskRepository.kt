package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.OrderKey
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TaskRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val tasks = database.taskDao()
    private val children = database.projectChildDao()

    fun observeTopLevel(projectId: String): Flow<List<TaskEntity>> = tasks.observeTopLevel(projectId)

    /** Open tasks due before [until], overdue ones first. */
    fun observeDue(until: Long): Flow<List<DueTask>> = tasks.observeDue(until)

    suspend fun get(id: String): TaskEntity? = tasks.get(id)

    /** New tasks start in To do, at the end of their siblings. */
    suspend fun create(projectId: String, title: String, priority: Priority = Priority.Medium, dueAt: Long? = null): String {
        val cleanTitle = title.trim()
        require(cleanTitle.isNotEmpty()) { "task title is required" }
        return database.withTransaction {
            val id = stamper.newId()
            val last = tasks.lastKey(projectId, null, TaskStatus.Todo, "")
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
                    orderKey = OrderKey.between(last.orEmpty(), null),
                    tags = emptyList(),
                ),
            )
            activity.record("task.created", "task", id, "Added task $cleanTitle")
            id
        }
    }

    /** Puts the task last in the column of [status]. */
    suspend fun setStatus(id: String, status: TaskStatus) {
        database.withTransaction {
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
}
