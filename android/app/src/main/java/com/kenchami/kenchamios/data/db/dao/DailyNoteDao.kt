package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.DailyNoteEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface DailyNoteDao {
    @Insert
    suspend fun insert(note: DailyNoteEntity)

    @Query("SELECT * FROM daily_notes WHERE day = :day AND deleted = 0")
    suspend fun get(day: String): DailyNoteEntity?

    @Query("UPDATE daily_notes SET body = :body, updated_at = :stamp, device_id = :deviceId WHERE id = :id")
    suspend fun setBody(id: String, body: String, stamp: Long, deviceId: String)

    @Query("SELECT day FROM daily_notes WHERE deleted = 0 AND trim(body) != '' AND day >= :from AND day <= :to ORDER BY day")
    fun observeDaysWithText(from: String, to: String): Flow<List<String>>
}
