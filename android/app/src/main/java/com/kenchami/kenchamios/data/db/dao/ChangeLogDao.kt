package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface ChangeLogDao {
    /** Moves whenever any synced row is written; the sync trigger watches it. */
    @Query("SELECT COALESCE(MAX(seq), 0) FROM change_log")
    fun observeLatest(): Flow<Long>
}
