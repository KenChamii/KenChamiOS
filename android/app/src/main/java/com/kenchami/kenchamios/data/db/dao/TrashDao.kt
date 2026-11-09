package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Query
import com.kenchami.kenchamios.data.db.TrashRow
import kotlinx.coroutines.flow.Flow

private const val MAX_ITEMS = 200

/**
 * Deleted rows and the restore cascade, the same queries as the desktop `db/trash.rs`.
 * A restore brings back the rows deleted at or after the item's own delete stamp: they went with the same action.
 */
@Dao
interface TrashDao {
    // updated_at is an hlc, its upper bits are the wall clock in ms
    @Query(
        """SELECT 'project' AS kind, p.id, p.name AS title, NULL AS detail, p.updated_at >> 16 AS deleted_at
             FROM projects p
            WHERE p.deleted = 1 AND p.id != :inboxId
           UNION ALL
           SELECT 'task', t.id, t.title, p.name, t.updated_at >> 16
             FROM tasks t JOIN projects p ON p.id = t.project_id
            WHERE t.deleted = 1 AND p.deleted = 0
              AND (t.parent_id IS NULL OR EXISTS (SELECT 1 FROM tasks pt WHERE pt.id = t.parent_id AND pt.deleted = 0))
           UNION ALL
           SELECT 'prompt', r.id, r.title, p.name, r.updated_at >> 16
             FROM prompts r LEFT JOIN projects p ON p.id = r.project_id
            WHERE r.deleted = 1 AND (r.project_id IS NULL OR p.deleted = 0)
           UNION ALL
           SELECT 'save_point', s.id, CASE WHEN s.note != '' THEN s.note ELSE 'Save point' END, p.name, s.updated_at >> 16
             FROM save_points s JOIN projects p ON p.id = s.project_id
            WHERE s.deleted = 1 AND p.deleted = 0
           UNION ALL
           SELECT 'media', e.id, i.title, i.type, e.updated_at >> 16
             FROM media_entries e JOIN media_items i ON i.id = e.media_item_id
            WHERE e.deleted = 1
           UNION ALL
           SELECT 'habit', h.id, h.name, NULL, h.updated_at >> 16
             FROM habits h
            WHERE h.deleted = 1
           ORDER BY deleted_at DESC, id
           LIMIT $MAX_ITEMS""",
    )
    fun observe(inboxId: String): Flow<List<TrashRow>>

    @Query("SELECT updated_at FROM projects WHERE id = :id AND deleted = 1")
    suspend fun projectStamp(id: String): Long?

    @Query(
        """SELECT t.updated_at FROM tasks t JOIN projects p ON p.id = t.project_id
          WHERE t.id = :id AND t.deleted = 1 AND p.deleted = 0""",
    )
    suspend fun taskStamp(id: String): Long?

    @Query("SELECT updated_at FROM prompts WHERE id = :id AND deleted = 1")
    suspend fun promptStamp(id: String): Long?

    @Query("SELECT updated_at FROM save_points WHERE id = :id AND deleted = 1")
    suspend fun savePointStamp(id: String): Long?

    @Query("SELECT updated_at FROM media_entries WHERE id = :id AND deleted = 1")
    suspend fun mediaStamp(id: String): Long?

    @Query("SELECT updated_at FROM habits WHERE id = :id AND deleted = 1")
    suspend fun habitStamp(id: String): Long?

    @Query(
        """UPDATE projects SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreProject(id: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE project_docs SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE project_id = :projectId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreDocsOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE prompts SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE project_id = :projectId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restorePromptsOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE save_points SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE project_id = :projectId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreSavePointsOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE time_entries SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE project_id = :projectId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreTimeEntriesOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE checklist_items SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE task_id IN (SELECT id FROM tasks WHERE project_id = :projectId)
            AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreChecklistsOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE tasks SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE project_id = :projectId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreTasksOfProject(projectId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE checklist_items SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE task_id IN (SELECT id FROM tasks WHERE id = :taskId OR parent_id = :taskId)
            AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreChecklistsOfTask(taskId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE tasks SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE (id = :taskId OR parent_id = :taskId) AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreTaskTree(taskId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE prompts SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restorePrompt(id: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE save_points SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreSavePoint(id: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE media_items SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = (SELECT media_item_id FROM media_entries WHERE id = :entryId)
            AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreItemOfEntry(entryId: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE media_entries SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreEntry(id: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE habits SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreHabit(id: String, since: Long, stamp: Long, deviceId: String)

    @Query(
        """UPDATE habit_checks SET deleted = 0, updated_at = :stamp, device_id = :deviceId
          WHERE habit_id = :habitId AND deleted = 1 AND updated_at >= :since""",
    )
    suspend fun restoreChecksOfHabit(habitId: String, since: Long, stamp: Long, deviceId: String)
}
