package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus

@Entity(
    tableName = "tasks",
    foreignKeys = [
        ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"]),
        ForeignKey(TaskEntity::class, parentColumns = ["id"], childColumns = ["parent_id"]),
    ],
    indices = [Index("project_id", "status", "order_key"), Index("parent_id"), Index("due_at"), Index("planned_on")],
)
data class TaskEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String,
    @ColumnInfo(name = "parent_id") val parentId: String?,
    val title: String,
    @ColumnInfo(name = "body_md") val bodyMd: String,
    @ColumnInfo(name = "acceptance_md") val acceptanceMd: String,
    @ColumnInfo(name = "related_files") val relatedFiles: List<String>,
    val status: TaskStatus,
    val priority: Priority,
    @ColumnInfo(name = "due_at") val dueAt: Long?,
    @ColumnInfo(name = "order_key") val orderKey: String,
    val tags: List<String>,
    /** `unit:interval` like `weekly:2`; null for a one-off task */
    val recurrence: String?,
    /** local day the task is planned for, `YYYY-MM-DD` */
    @ColumnInfo(name = "planned_on") val plannedOn: String?,
    /** start of the time block in unix ms, only set together with [plannedOn] */
    @ColumnInfo(name = "scheduled_at") val scheduledAt: Long?,
    /** estimate or block length, 1 to 1440 minutes */
    @ColumnInfo(name = "duration_min") val durationMin: Int?,
)

@Entity(
    tableName = "checklist_items",
    foreignKeys = [ForeignKey(TaskEntity::class, parentColumns = ["id"], childColumns = ["task_id"])],
    indices = [Index("task_id", "order_key")],
)
data class ChecklistItemEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "task_id") val taskId: String,
    val text: String,
    val done: Boolean,
    @ColumnInfo(name = "order_key") val orderKey: String,
)
