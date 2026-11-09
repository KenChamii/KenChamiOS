package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(tableName = "habits")
data class HabitEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    val name: String,
)

// one live check per habit and day is kept by triggers, Room has no partial unique index
@Entity(
    tableName = "habit_checks",
    foreignKeys = [ForeignKey(HabitEntity::class, parentColumns = ["id"], childColumns = ["habit_id"])],
    indices = [Index("habit_id", "day")],
)
data class HabitCheckEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "habit_id") val habitId: String,
    /** local calendar day, `YYYY-MM-DD` */
    val day: String,
)
