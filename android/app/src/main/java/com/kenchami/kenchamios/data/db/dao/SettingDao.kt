package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.SettingEntity
import kotlinx.coroutines.flow.Flow

/** A setting without the sync columns. */
data class SettingValue(val key: String, val valueJson: String)

@Dao
interface SettingDao {
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertIfMissing(setting: SettingEntity)

    /** Replaces the row with the same key. */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(setting: SettingEntity)

    @Query("SELECT value_json FROM settings WHERE key = :key AND deleted = 0")
    suspend fun valueOf(key: String): String?

    @Query("SELECT key, value_json AS valueJson FROM settings WHERE key LIKE :prefix || '%' AND deleted = 0")
    suspend fun withPrefix(prefix: String): List<SettingValue>

    @Query("SELECT key, value_json AS valueJson FROM settings WHERE key LIKE :prefix || '%' AND deleted = 0")
    fun observeWithPrefix(prefix: String): Flow<List<SettingValue>>

    @Query("DELETE FROM settings WHERE key LIKE :prefix || '%'")
    suspend fun deleteWithPrefix(prefix: String)
}
