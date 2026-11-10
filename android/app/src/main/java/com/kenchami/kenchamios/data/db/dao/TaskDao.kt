package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Update
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.Flow

private const val MAX_DUE_TASKS = 50

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

    @Query("UPDATE tasks SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE (id = :id OR parent_id = :id) AND deleted = 0")
    suspend fun softDeleteTree(id: String, stamp: Long, deviceId: String): Int

    @Query(
        "UPDATE tasks SET project_id = :projectId, updated_at = :stamp, device_id = :deviceId WHERE parent_id = :parentId AND deleted = 0",
    )
    suspend fun moveSubtasks(parentId: String, projectId: String, stamp: Long, deviceId: String)

    @Query("UPDATE tasks SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteByProject(projectId: String, stamp: Long, deviceId: String)
}
