package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.kenchami.kenchamios.data.db.entity.SettingEntity

@Dao
interface SettingDao {
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertIfMissing(setting: SettingEntity)

    @Query("SELECT value_json FROM settings WHERE key = :key AND deleted = 0")
    suspend fun valueOf(key: String): String?
}
