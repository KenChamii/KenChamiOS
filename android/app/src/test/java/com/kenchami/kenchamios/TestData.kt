package com.kenchami.kenchamios

import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus

fun dueTask(title: String, dueAt: Long) = DueTask("t-$title", title, "p1", "Demo", dueAt, Priority.Medium, TaskStatus.Todo)

fun libraryEntry(
    id: String = "e1",
    title: String = "Frieren",
    type: MediaType = MediaType.Anime,
    status: MediaStatus = MediaStatus.InProgress,
    progress: Int = 3,
    total: Int? = 12,
) = LibraryEntry(
    id = id,
    updatedAt = 1,
    mediaItemId = "i-$id",
    mediaType = type,
    title = title,
    year = null,
    genres = emptyList(),
    synopsis = "",
    source = MediaSource.Manual,
    status = status,
    progress = progress,
    progressTotal = total,
    rating = null,
    notes = "",
    tags = emptyList(),
    startedAt = null,
    finishedAt = null,
    platform = null,
)

fun projectSummary(id: String = "p1", name: String = "Demo", stage: Stage = Stage.Build, done: Int = 1, total: Int = 3) = ProjectSummary(
    project = ProjectEntity(
        id = id, createdAt = 1, updatedAt = 1, deviceId = "d", deleted = false, name = name, slug = name.lowercase(),
        stage = stage, description = "", repoPath = null, color = null, tags = emptyList(), archived = false, finishedAt = null,
    ),
    tasksTotal = total,
    tasksDone = done,
    lastActivityAt = 1,
)

fun prompt(id: String = "pr1", title: String = "Review", favorite: Boolean = false, uses: Int = 0) = PromptEntity(
    id = id, createdAt = 1, updatedAt = 1, deviceId = "d", deleted = false, projectId = null, title = title,
    bodyMd = "Review the diff", tags = emptyList(), favorite = favorite, useCount = uses, lastUsedAt = null,
)
