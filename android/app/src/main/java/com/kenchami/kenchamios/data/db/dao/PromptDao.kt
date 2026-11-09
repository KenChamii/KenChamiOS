package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Update
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface PromptDao {
    @Insert
    suspend fun insert(prompt: PromptEntity)

    @Update
    suspend fun update(prompt: PromptEntity)

    @Query("SELECT * FROM prompts WHERE id = :id AND deleted = 0")
    suspend fun get(id: String): PromptEntity?

    // favorites first, then the most used
    @Query(
        """SELECT * FROM prompts WHERE deleted = 0
          ORDER BY favorite DESC, use_count DESC, COALESCE(last_used_at, 0) DESC, title""",
    )
    fun observeAll(): Flow<List<PromptEntity>>

    @Query("UPDATE prompts SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE id = :id AND deleted = 0")
    suspend fun softDelete(id: String, stamp: Long, deviceId: String): Int

    @Query("UPDATE prompts SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteByProject(projectId: String, stamp: Long, deviceId: String)
}
