package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.core.cleanTags
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.db.entity.MediaEntryEntity
import com.kenchami.kenchamios.data.db.entity.MediaItemEntity
import com.kenchami.kenchamios.data.db.entity.MediaProgressLogEntity
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class MediaRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val media = database.mediaDao()

    fun observeLibrary(): Flow<List<LibraryEntry>> = media.observeLibrary()

    fun observeEntry(id: String): Flow<LibraryEntry?> = media.observeEntry(id)

    /** Adds a manual item and its library entry in one step and returns the entry id. */
    suspend fun add(type: MediaType, title: String, status: MediaStatus, total: Int? = null, year: Int? = null): String {
        val cleanTitle = title.trim()
        require(cleanTitle.isNotEmpty()) { "a title is required" }
        require(total == null || total >= 0) { "the total cannot be negative" }
        return database.withTransaction {
            val now = stamper.nowMs()
            val device = stamper.deviceId()
            val itemId = stamper.newId()
            val entryId = stamper.newId()
            media.insertItem(
                MediaItemEntity(
                    id = itemId,
                    createdAt = now,
                    updatedAt = stamper.stamp(),
                    deviceId = device,
                    deleted = false,
                    type = type,
                    title = cleanTitle,
                    coverUrl = null,
                    coverCachePath = null,
                    year = year,
                    genres = emptyList(),
                    synopsis = "",
                    source = MediaSource.Manual,
                    externalId = null,
                ),
            )
            media.insertEntry(
                MediaEntryEntity(
                    id = entryId,
                    createdAt = now,
                    updatedAt = stamper.stamp(),
                    deviceId = device,
                    deleted = false,
                    mediaItemId = itemId,
                    status = status,
                    progress = 0,
                    progressTotal = total,
                    rating = null,
                    notes = "",
                    tags = emptyList(),
                    startedAt = now.takeIf { status == MediaStatus.InProgress },
                    finishedAt = now.takeIf { status == MediaStatus.Completed },
                    platform = null,
                ),
            )
            activity.record("media.added", "media_entry", entryId, "Added $cleanTitle")
            entryId
        }
    }

    suspend fun update(id: String, input: EntryInput) {
        input.validate()
        database.withTransaction {
            val current = checkNotNull(media.getLibraryEntry(id)) { "library entry not found" }
            val entry = checkNotNull(media.getEntry(id)) { "library entry not found" }
            require(input.progressTotal == null || current.progress <= input.progressTotal) {
                "the total is below the current progress"
            }
            val now = stamper.nowMs()
            val moved = input.status != current.status
            media.updateEntry(
                entry.copy(
                    status = input.status,
                    progressTotal = input.progressTotal,
                    rating = input.rating,
                    notes = input.notes.trim(),
                    tags = cleanTags(input.tags),
                    startedAt = input.startedAt ?: now.takeIf { moved && input.status == MediaStatus.InProgress },
                    finishedAt = input.finishedAt ?: now.takeIf { moved && input.status == MediaStatus.Completed },
                    platform = input.platform?.trim()?.takeIf { it.isNotEmpty() },
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                ),
            )
            activity.record("media.updated", "media_entry", id, "Updated ${current.title}")
        }
    }

    /** Adds one to the progress counter; an entry at its total stays where it is. */
    suspend fun increment(id: String) {
        val current = checkNotNull(media.getLibraryEntry(id)) { "library entry not found" }
        if (current.canIncrement) setProgress(id, current.progress + 1)
    }

    /** Takes one off the progress counter, never below zero. */
    suspend fun decrement(id: String) {
        val current = checkNotNull(media.getLibraryEntry(id)) { "library entry not found" }
        if (current.progress > 0) setProgress(id, current.progress - 1)
    }

    /** Sets the progress counter, logs the change and moves the status along with it. */
    suspend fun setProgress(id: String, value: Int) {
        database.withTransaction {
            val current = checkNotNull(media.getLibraryEntry(id)) { "library entry not found" }
            val entry = checkNotNull(media.getEntry(id)) { "library entry not found" }
            require(value >= 0) { "progress cannot be negative" }
            require(current.progressTotal == null || value <= current.progressTotal) { "progress cannot go past the total" }
            if (value == current.progress) return@withTransaction

            val now = stamper.nowMs()
            val device = stamper.deviceId()
            val reachedTotal = current.progressTotal == value
            val firstStep = current.status == MediaStatus.Planned && value > current.progress
            media.updateEntry(
                entry.copy(
                    progress = value,
                    status = when {
                        reachedTotal -> MediaStatus.Completed
                        firstStep -> MediaStatus.InProgress
                        else -> current.status
                    },
                    startedAt = if (!reachedTotal && firstStep) current.startedAt ?: now else current.startedAt,
                    finishedAt = if (reachedTotal) current.finishedAt ?: now else current.finishedAt,
                    updatedAt = stamper.stamp(),
                    deviceId = device,
                ),
            )
            media.insertLog(
                MediaProgressLogEntity(
                    id = stamper.newId(),
                    createdAt = now,
                    updatedAt = stamper.stamp(),
                    deviceId = device,
                    deleted = false,
                    entryId = id,
                    delta = value - current.progress,
                    valueAfter = value,
                    loggedAt = now,
                ),
            )
            activity.record("media.progress", "media_entry", id, "${current.title}: ${progressUnit(current.mediaType)} $value")
        }
    }

    /** Soft deletes the entry and the item it tracks. */
    suspend fun delete(id: String) {
        database.withTransaction {
            val current = checkNotNull(media.getLibraryEntry(id)) { "library entry not found" }
            val device = stamper.deviceId()
            check(media.softDeleteEntry(id, stamper.stamp(), device) > 0) { "library entry not found" }
            check(media.softDeleteItemOfEntry(id, stamper.stamp(), device) > 0) { "library entry not found" }
            activity.record("media.deleted", "media_entry", id, "Removed ${current.title}")
        }
    }

    private fun progressUnit(type: MediaType): String = when (type) {
        MediaType.Anime -> "episode"
        MediaType.Manga -> "chapter"
        MediaType.Game -> "hours"
        MediaType.Movie -> "watched"
        MediaType.Show -> "episode"
        MediaType.Album, MediaType.Artist -> "listens"
    }
}
