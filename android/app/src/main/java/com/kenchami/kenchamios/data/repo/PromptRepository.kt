package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.cleanTags
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class PromptRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val prompts = database.promptDao()
    private val projects = database.projectDao()

    /** Favorites first, then the most used. */
    fun observeAll(): Flow<List<PromptEntity>> = prompts.observeAll()

    suspend fun create(title: String, body: String, tags: List<String> = emptyList(), projectId: String? = null) {
        val cleanTitle = title.trim()
        require(cleanTitle.isNotEmpty()) { "prompt title is required" }
        database.withTransaction {
            val id = stamper.newId()
            prompts.insert(
                PromptEntity(
                    id = id,
                    createdAt = stamper.nowMs(),
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                    deleted = false,
                    projectId = projectId,
                    title = cleanTitle,
                    bodyMd = body,
                    tags = cleanTags(tags),
                    favorite = false,
                    useCount = 0,
                    lastUsedAt = null,
                ),
            )
            activity.record("prompt.created", "prompt", id, "Added prompt $cleanTitle")
        }
    }

    suspend fun setFavorite(id: String, favorite: Boolean) {
        database.withTransaction {
            val prompt = checkNotNull(prompts.get(id)) { "prompt not found" }
            prompts.update(prompt.copy(favorite = favorite, updatedAt = stamper.stamp(), deviceId = stamper.deviceId()))
            val summary = if (favorite) "Marked prompt as favorite" else "Removed prompt from favorites"
            activity.record("prompt.favorite", "prompt", id, summary)
        }
    }

    /** Counts one copy of the prompt and returns its text with the project variables filled in. */
    suspend fun copy(id: String): String = database.withTransaction {
        val prompt = checkNotNull(prompts.get(id)) { "prompt not found" }
        prompts.update(
            prompt.copy(
                useCount = prompt.useCount + 1,
                lastUsedAt = stamper.nowMs(),
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
            ),
        )
        activity.record("prompt.used", "prompt", id, "Copied prompt")
        fillProjectVariables(prompt.bodyMd, prompt.projectId?.let { projects.get(it) })
    }

    suspend fun delete(id: String) {
        database.withTransaction {
            val changed = prompts.softDelete(id, stamper.stamp(), stamper.deviceId())
            check(changed > 0) { "prompt not found" }
            activity.record("prompt.deleted", "prompt", id, "Deleted prompt")
        }
    }
}
