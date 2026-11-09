package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Upsert
import com.kenchami.kenchamios.data.db.entity.TimeEntryEntity
import com.kenchami.kenchamios.data.db.entity.TimerStateEntity

@Dao
interface TimerDao {
    @Query("SELECT * FROM timer_state WHERE id = :deviceId AND deleted = 0")
    suspend fun get(deviceId: String): TimerStateEntity?

    @Query("SELECT created_at FROM timer_state WHERE id = :deviceId")
    suspend fun createdAt(deviceId: String): Long?

    @Upsert
    suspend fun save(state: TimerStateEntity)

    @Query("UPDATE timer_state SET deleted = 1, updated_at = :stamp WHERE id = :deviceId")
    suspend fun clear(deviceId: String, stamp: Long)

    @Insert
    suspend fun insertTimeEntry(entry: TimeEntryEntity)
}
