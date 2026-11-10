package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Update
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.model.Inbox
import kotlinx.coroutines.flow.Flow

// activity on a project's tasks, docs and save points counts as activity on the project
private const val SUMMARY =
    """SELECT p.*,
        (SELECT COUNT(*) FROM tasks t
          WHERE t.project_id = p.id AND t.parent_id IS NULL AND t.deleted = 0) AS tasks_total,
        (SELECT COUNT(*) FROM tasks t
          WHERE t.project_id = p.id AND t.parent_id IS NULL AND t.deleted = 0
            AND t.status = 'done') AS tasks_done,
        COALESCE((SELECT MAX(a.at) FROM activity a
                   WHERE a.entity_id = p.id
                      OR a.entity_id IN (SELECT id FROM tasks WHERE project_id = p.id)
                      OR a.entity_id IN (SELECT id FROM project_docs WHERE project_id = p.id)
                      OR a.entity_id IN (SELECT id FROM save_points WHERE project_id = p.id)),
                 p.created_at) AS last_activity_at
     FROM projects p """

@Dao
interface ProjectDao {
    @Insert
    suspend fun insert(project: ProjectEntity)

    @Update
    suspend fun update(project: ProjectEntity)

    @Query("SELECT * FROM projects WHERE id = :id AND deleted = 0")
    suspend fun get(id: String): ProjectEntity?

    /** Also a deleted one, so the inbox can be brought back under its fixed id. */
    @Query("SELECT * FROM projects WHERE id = :id")
    suspend fun getAny(id: String): ProjectEntity?

    // the inbox has its own screen
    @Query(
        SUMMARY + """WHERE p.deleted = 0
           AND p.archived = 0
           AND p.id != '${Inbox.ID}'
           AND (:search IS NULL OR p.name LIKE :search OR p.description LIKE :search)
         ORDER BY last_activity_at DESC""",
    )
    fun observeSummaries(search: String?): Flow<List<ProjectSummary>>

    @Query(SUMMARY + "WHERE p.id = :id AND p.deleted = 0")
    fun observeSummary(id: String): Flow<ProjectSummary?>

    @Query("SELECT EXISTS (SELECT 1 FROM projects WHERE slug = :slug AND deleted = 0)")
    suspend fun slugTaken(slug: String): Boolean
}
