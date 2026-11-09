package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Query

/** The soft delete cascade of a project and a task: rows the phone does not edit, only removes. */
@Dao
interface ProjectChildDao {
    @Query("UPDATE project_docs SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteDocs(projectId: String, stamp: Long, deviceId: String)

    @Query("UPDATE time_entries SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteTimeEntries(projectId: String, stamp: Long, deviceId: String)

    @Query(
        """UPDATE checklist_items SET deleted = 1, updated_at = :stamp, device_id = :deviceId
          WHERE deleted = 0 AND task_id IN (SELECT id FROM tasks WHERE project_id = :projectId)""",
    )
    suspend fun softDeleteChecklistsOfProject(projectId: String, stamp: Long, deviceId: String)

    @Query(
        """UPDATE checklist_items SET deleted = 1, updated_at = :stamp, device_id = :deviceId
          WHERE deleted = 0 AND task_id IN (SELECT id FROM tasks WHERE id = :taskId OR parent_id = :taskId)""",
    )
    suspend fun softDeleteChecklistsOfTask(taskId: String, stamp: Long, deviceId: String)
}
