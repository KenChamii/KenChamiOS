package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.slugify
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.model.Stage
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProjectRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val projects = database.projectDao()
    private val tasks = database.taskDao()
    private val prompts = database.promptDao()
    private val savePoints = database.savePointDao()
    private val children = database.projectChildDao()

    fun observeSummaries(search: String?): Flow<List<ProjectSummary>> =
        projects.observeSummaries(search?.trim()?.takeIf { it.isNotEmpty() }?.let { "%$it%" })

    fun observeSummary(id: String): Flow<ProjectSummary?> = projects.observeSummary(id)

    /** Adds a project at stage idea and returns its id. */
    suspend fun create(name: String, description: String = ""): String {
        val cleanName = name.trim()
        require(cleanName.isNotEmpty()) { "project name is required" }
        return database.withTransaction {
            val id = stamper.newId()
            projects.insert(
                ProjectEntity(
                    id = id,
                    createdAt = stamper.nowMs(),
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                    deleted = false,
                    name = cleanName,
                    slug = uniqueSlug(cleanName),
                    stage = Stage.Idea,
                    description = description.trim(),
                    repoPath = null,
                    color = null,
                    tags = emptyList(),
                    archived = false,
                    finishedAt = null,
                ),
            )
            activity.record("project.created", "project", id, "Created project $cleanName")
            id
        }
    }

    suspend fun setStage(id: String, stage: Stage) {
        database.withTransaction {
            val project = checkNotNull(projects.get(id)) { "project not found" }
            projects.update(
                project.copy(
                    stage = stage,
                    finishedAt = if (stage == Stage.Done) stamper.nowMs() else null,
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                ),
            )
            activity.record("project.stage_changed", "project", id, "Moved to ${stage.wire}")
        }
    }

    /** Soft deletes the project and everything that belongs to it. */
    suspend fun delete(id: String) {
        database.withTransaction {
            val project = checkNotNull(projects.get(id)) { "project not found" }
            val device = stamper.deviceId()
            projects.update(project.copy(deleted = true, updatedAt = stamper.stamp(), deviceId = device))
            children.softDeleteDocs(id, stamper.stamp(), device)
            prompts.softDeleteByProject(id, stamper.stamp(), device)
            savePoints.softDeleteByProject(id, stamper.stamp(), device)
            children.softDeleteTimeEntries(id, stamper.stamp(), device)
            tasks.softDeleteByProject(id, stamper.stamp(), device)
            children.softDeleteChecklistsOfProject(id, stamper.stamp(), device)
            activity.record("project.deleted", "project", id, "Deleted project")
        }
    }

    private suspend fun uniqueSlug(name: String): String {
        val base = slugify(name)
        var candidate = base
        var suffix = 2
        while (projects.slugTaken(candidate)) {
            candidate = "$base-$suffix"
            suffix++
        }
        return candidate
    }
}
