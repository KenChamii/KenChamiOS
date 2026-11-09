package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.ActivityEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface ActivityDao {
    @Insert
    suspend fun insert(activity: ActivityEntity)

    @Query("SELECT * FROM activity WHERE deleted = 0 ORDER BY at DESC, rowid DESC LIMIT :limit")
    fun observeRecent(limit: Int): Flow<List<ActivityEntity>>

    @Query("SELECT kind FROM activity WHERE entity_id = :entityId ORDER BY at, rowid")
    suspend fun kindsFor(entityId: String): List<String>
}
