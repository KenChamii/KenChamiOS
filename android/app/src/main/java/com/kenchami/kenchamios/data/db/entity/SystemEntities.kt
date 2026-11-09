package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(tableName = "settings", indices = [Index("key", unique = true)])
data class SettingEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    val key: String,
    @ColumnInfo(name = "value_json") val valueJson: String,
)

@Entity(tableName = "activity", indices = [Index("at")])
data class ActivityEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    val kind: String,
    @ColumnInfo(name = "entity_type") val entityType: String,
    @ColumnInfo(name = "entity_id") val entityId: String,
    val summary: String,
    val at: Long,
)

/** One entry per changed row, filled by triggers; the sync reads it to find what to send. */
@Entity(tableName = "change_log", indices = [Index("table_name", "row_id", unique = true)])
data class ChangeLogEntity(
    @PrimaryKey(autoGenerate = true) val seq: Long,
    @ColumnInfo(name = "table_name") val tableName: String,
    @ColumnInfo(name = "row_id") val rowId: String,
    val hlc: Long,
)
