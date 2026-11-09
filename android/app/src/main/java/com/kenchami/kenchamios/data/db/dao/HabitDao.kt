package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.HabitCheckEntity
import com.kenchami.kenchamios.data.db.entity.HabitEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface HabitDao {
    @Insert
    suspend fun insert(habit: HabitEntity)

    @Insert
    suspend fun insertCheck(check: HabitCheckEntity)

    @Query("SELECT * FROM habits WHERE id = :id AND deleted = 0")
    suspend fun get(id: String): HabitEntity?

    @Query("SELECT * FROM habits WHERE deleted = 0 ORDER BY created_at, id")
    fun observeAll(): Flow<List<HabitEntity>>

    @Query("SELECT * FROM habit_checks WHERE deleted = 0")
    fun observeChecks(): Flow<List<HabitCheckEntity>>

    @Query("UPDATE habits SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE id = :id AND deleted = 0")
    suspend fun softDelete(id: String, stamp: Long, deviceId: String): Int

    @Query("UPDATE habit_checks SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE habit_id = :habitId AND deleted = 0")
    suspend fun softDeleteChecks(habitId: String, stamp: Long, deviceId: String)

    @Query(
        """UPDATE habit_checks SET deleted = 1, updated_at = :stamp, device_id = :deviceId
          WHERE habit_id = :habitId AND day = :day AND deleted = 0""",
    )
    suspend fun clearCheck(habitId: String, day: String, stamp: Long, deviceId: String): Int
}
