package com.kenchami.kenchamios.data.repo

import android.database.sqlite.SQLiteConstraintException
import androidx.room.withTransaction
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.TrashRow
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.TrashKind
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TrashRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val trash = database.trashDao()

    /** Deleted projects, tasks, prompts, save points, media and habits, newest first; rows deleted with their parent are left out. */
    fun observe(): Flow<List<TrashRow>> = trash.observe(Inbox.ID)

    /** Brings a deleted row back with whatever was deleted along with it; a task comes back only while its project is live. */
    suspend fun restore(kind: TrashKind, id: String) {
        try {
            database.withTransaction {
                val since = checkNotNull(stampOf(kind, id)) { "deleted ${kind.wire} not found" }
                val device = stamper.deviceId()
                restoreRows(kind, id, since, device)
                activity.record("${kind.wire}.restored", entityType(kind), id, "Restored from trash")
            }
        } catch (_: SQLiteConstraintException) {
            throw IllegalStateException("a live copy of this item already exists")
        }
    }

    private suspend fun stampOf(kind: TrashKind, id: String): Long? = when (kind) {
        TrashKind.Project -> trash.projectStamp(id)
        TrashKind.Task -> trash.taskStamp(id)
        TrashKind.Prompt -> trash.promptStamp(id)
        TrashKind.SavePoint -> trash.savePointStamp(id)
        TrashKind.Media -> trash.mediaStamp(id)
        TrashKind.Habit -> trash.habitStamp(id)
    }

    private suspend fun restoreRows(kind: TrashKind, id: String, since: Long, device: String) {
        when (kind) {
            TrashKind.Project -> {
                trash.restoreProject(id, since, stamper.stamp(), device)
                trash.restoreDocsOfProject(id, since, stamper.stamp(), device)
                trash.restorePromptsOfProject(id, since, stamper.stamp(), device)
                trash.restoreSavePointsOfProject(id, since, stamper.stamp(), device)
                trash.restoreTimeEntriesOfProject(id, since, stamper.stamp(), device)
                trash.restoreChecklistsOfProject(id, since, stamper.stamp(), device)
                trash.restoreTasksOfProject(id, since, stamper.stamp(), device)
            }
            TrashKind.Task -> {
                trash.restoreChecklistsOfTask(id, since, stamper.stamp(), device)
                trash.restoreTaskTree(id, since, stamper.stamp(), device)
            }
            TrashKind.Prompt -> trash.restorePrompt(id, since, stamper.stamp(), device)
            TrashKind.SavePoint -> trash.restoreSavePoint(id, since, stamper.stamp(), device)
            TrashKind.Media -> {
                trash.restoreItemOfEntry(id, since, stamper.stamp(), device)
                trash.restoreEntry(id, since, stamper.stamp(), device)
            }
            TrashKind.Habit -> {
                trash.restoreHabit(id, since, stamper.stamp(), device)
                trash.restoreChecksOfHabit(id, since, stamper.stamp(), device)
            }
        }
    }

    // the entity type the delete wrote in the activity log
    private fun entityType(kind: TrashKind) = when (kind) {
        TrashKind.Media -> "media_entry"
        else -> kind.wire
    }
}
