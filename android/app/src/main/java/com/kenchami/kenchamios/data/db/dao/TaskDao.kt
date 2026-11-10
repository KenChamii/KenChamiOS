package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Update
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.Flow

private const val MAX_DUE_TASKS = 50
private const val MAX_CANDIDATES = 50
private const val PLANNED_SELECT = """SELECT t.id, t.title, t.project_id, p.name AS project_name, t.status, t.due_at,
        t.planned_on, t.scheduled_at, t.duration_min
   FROM tasks t JOIN projects p ON p.id = t.project_id
  WHERE t.deleted = 0 AND p.deleted = 0 AND p.archived = 0"""

@Dao
interface TaskDao {
    @Insert
    suspend fun insert(task: TaskEntity)

    @Update
    suspend fun update(task: TaskEntity)

    @Query("SELECT * FROM tasks WHERE id = :id AND deleted = 0")
    suspend fun get(id: String): TaskEntity?

    @Query("SELECT * FROM tasks WHERE project_id = :projectId AND parent_id IS NULL AND deleted = 0 ORDER BY order_key")
    fun observeTopLevel(projectId: String): Flow<List<TaskEntity>>

    // greatest key among the siblings in a column, without the task that is being moved
    @Query(
        """SELECT MAX(order_key) FROM tasks
          WHERE project_id = :projectId AND parent_id IS :parentId AND status = :status
            AND deleted = 0 AND id <> :excludeId""",
    )
    suspend fun lastKey(projectId: String, parentId: String?, status: TaskStatus, excludeId: String): String?

    @Query(
        """SELECT t.id, t.title, t.project_id, p.name AS project_name, t.due_at, t.priority, t.status
           FROM tasks t JOIN projects p ON p.id = t.project_id
          WHERE t.deleted = 0 AND t.status != 'done' AND t.due_at IS NOT NULL AND t.due_at < :until
            AND p.deleted = 0 AND p.archived = 0
          ORDER BY t.due_at, t.id
          LIMIT $MAX_DUE_TASKS""",
    )
    fun observeDue(until: Long): Flow<List<DueTask>>

    @Query(
        """$PLANNED_SELECT AND t.planned_on = :day
          ORDER BY t.scheduled_at IS NULL, t.scheduled_at, t.order_key""",
    )
    fun observePlanned(day: String): Flow<List<PlannedTask>>

    // the same picks as the desktop's plan my day: due, in the Inbox, in progress or left from an earlier day
    @Query(
        """$PLANNED_SELECT AND t.status != 'done'
            AND (t.planned_on IS NULL OR t.planned_on < :today)
            AND CASE WHEN :pattern IS NULL
                     THEN t.due_at < :until OR t.project_id = :inboxId OR t.status IN ('doing', 'review')
                          OR t.planned_on < :today
                     ELSE t.title LIKE :pattern ESCAPE '\' END
          ORDER BY t.due_at IS NULL, t.due_at, t.created_at, t.id
          LIMIT $MAX_CANDIDATES""",
    )
    fun observeCandidates(today: String, until: Long, inboxId: String, pattern: String?): Flow<List<PlannedTask>>

    @Query(
        """UPDATE tasks SET planned_on = :plannedOn, scheduled_at = :scheduledAt, duration_min = :durationMin,
                updated_at = :stamp, device_id = :deviceId
          WHERE id = :id AND deleted = 0""",
    )
    suspend fun setPlan(id: String, plannedOn: String?, scheduledAt: Long?, durationMin: Int?, stamp: Long, deviceId: String): Int

    @Query("UPDATE tasks SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE (id = :id OR parent_id = :id) AND deleted = 0")
    suspend fun softDeleteTree(id: String, stamp: Long, deviceId: String): Int

    @Query(
        "UPDATE tasks SET project_id = :projectId, updated_at = :stamp, device_id = :deviceId WHERE parent_id = :parentId AND deleted = 0",
    )
    suspend fun moveSubtasks(parentId: String, projectId: String, stamp: Long, deviceId: String)

    @Query("UPDATE tasks SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteByProject(projectId: String, stamp: Long, deviceId: String)
}
