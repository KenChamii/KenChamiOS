package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.SavePointEntity
import com.kenchami.kenchamios.data.model.SavePointSource
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SavePointRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val savePoints = database.savePointDao()
    private val projects = database.projectDao()

    /** Newest first. */
    fun observe(projectId: String): Flow<List<SavePointEntity>> = savePoints.observe(projectId)

    /** Stores a manual save point at the project's current stage. */
    suspend fun create(projectId: String, note: String, doneMd: String = "", nextMd: String = "") {
        val cleanNote = note.trim()
        require(cleanNote.isNotEmpty()) { "save point note is required" }
        database.withTransaction {
            val project = checkNotNull(projects.get(projectId)) { "project not found" }
            val id = stamper.newId()
            savePoints.insert(
                SavePointEntity(
                    id = id,
                    createdAt = stamper.nowMs(),
                    updatedAt = stamper.stamp(),
                    deviceId = stamper.deviceId(),
                    deleted = false,
                    projectId = projectId,
                    stage = project.stage,
                    source = SavePointSource.Manual,
                    note = cleanNote,
                    doneMd = doneMd.trim(),
                    nextMd = nextMd.trim(),
                    sessionSummaryMd = "",
                    gitBranch = null,
                    gitCommit = null,
                ),
            )
            activity.record("save_point.created", "save_point", id, "Save point: $cleanNote")
        }
    }

    suspend fun delete(id: String) {
        database.withTransaction {
            val changed = savePoints.softDelete(id, stamper.stamp(), stamper.deviceId())
            check(changed > 0) { "save point not found" }
            activity.record("save_point.deleted", "save_point", id, "Deleted save point")
        }
    }
}
