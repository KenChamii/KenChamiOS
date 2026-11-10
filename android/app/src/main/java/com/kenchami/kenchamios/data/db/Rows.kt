package com.kenchami.kenchamios.data.db

import androidx.room.ColumnInfo
import androidx.room.Embedded
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TrashKind

/** A project with the numbers the list shows. */
data class ProjectSummary(
    @Embedded val project: ProjectEntity,
    @ColumnInfo(name = "tasks_total") val tasksTotal: Int,
    @ColumnInfo(name = "tasks_done") val tasksDone: Int,
    @ColumnInfo(name = "last_activity_at") val lastActivityAt: Long,
)

/** A task that is due today or overdue, with the project it belongs to. */
data class DueTask(
    val id: String,
    val title: String,
    @ColumnInfo(name = "project_id") val projectId: String,
    @ColumnInfo(name = "project_name") val projectName: String,
    @ColumnInfo(name = "due_at") val dueAt: Long,
    val priority: Priority,
    val status: TaskStatus,
)

/** A library entry together with the item it tracks. */
data class LibraryEntry(
    val id: String,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "media_item_id") val mediaItemId: String,
    @ColumnInfo(name = "type") val mediaType: MediaType,
    val title: String,
    val year: Int?,
    val genres: List<String>,
    val synopsis: String,
    val source: MediaSource,
    val status: MediaStatus,
    val progress: Int,
    @ColumnInfo(name = "progress_total") val progressTotal: Int?,
    val rating: Int?,
    val notes: String,
    val tags: List<String>,
    @ColumnInfo(name = "started_at") val startedAt: Long?,
    @ColumnInfo(name = "finished_at") val finishedAt: Long?,
    val platform: String?,
) {
    /** False once the counter is at its total. */
    val canIncrement: Boolean get() = progressTotal == null || progress < progressTotal
}

/** A deleted row the trash lists; [detail] is the project name or the media type. */
data class TrashRow(
    val kind: TrashKind,
    val id: String,
    val title: String,
    val detail: String?,
    @ColumnInfo(name = "deleted_at") val deletedAt: Long,
)
