package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

// one live note per day is kept by triggers, Room has no partial unique index
@Entity(tableName = "daily_notes", indices = [Index("day")])
data class DailyNoteEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    /** local calendar day, `YYYY-MM-DD` */
    val day: String,
    /** markdown */
    val body: String,
)
