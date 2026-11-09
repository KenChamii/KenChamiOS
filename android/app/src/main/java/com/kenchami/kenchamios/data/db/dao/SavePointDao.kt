package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.SavePointEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface SavePointDao {
    @Insert
    suspend fun insert(savePoint: SavePointEntity)

    // newest first
    @Query("SELECT * FROM save_points WHERE project_id = :projectId AND deleted = 0 ORDER BY created_at DESC, id DESC")
    fun observe(projectId: String): Flow<List<SavePointEntity>>

    @Query("UPDATE save_points SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE id = :id AND deleted = 0")
    suspend fun softDelete(id: String, stamp: Long, deviceId: String): Int

    @Query("UPDATE save_points SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE project_id = :projectId AND deleted = 0")
    suspend fun softDeleteByProject(projectId: String, stamp: Long, deviceId: String)
}
