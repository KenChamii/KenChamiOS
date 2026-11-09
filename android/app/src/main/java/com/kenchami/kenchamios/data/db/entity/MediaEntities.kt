package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType

@Entity(tableName = "media_items", indices = [Index("source", "external_id")])
data class MediaItemEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    val type: MediaType,
    val title: String,
    @ColumnInfo(name = "cover_url") val coverUrl: String?,
    @ColumnInfo(name = "cover_cache_path") val coverCachePath: String?,
    val year: Int?,
    val genres: List<String>,
    val synopsis: String,
    val source: MediaSource,
    @ColumnInfo(name = "external_id") val externalId: String?,
)

@Entity(
    tableName = "media_entries",
    foreignKeys = [ForeignKey(MediaItemEntity::class, parentColumns = ["id"], childColumns = ["media_item_id"])],
    indices = [Index("media_item_id", unique = true), Index("finished_at")],
)
data class MediaEntryEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "media_item_id") val mediaItemId: String,
    val status: MediaStatus,
    val progress: Int,
    @ColumnInfo(name = "progress_total") val progressTotal: Int?,
    val rating: Int?,
    val notes: String,
    val tags: List<String>,
    @ColumnInfo(name = "started_at") val startedAt: Long?,
    @ColumnInfo(name = "finished_at") val finishedAt: Long?,
    val platform: String?,
)

@Entity(
    tableName = "media_progress_log",
    foreignKeys = [ForeignKey(MediaEntryEntity::class, parentColumns = ["id"], childColumns = ["entry_id"])],
    indices = [Index("entry_id", "logged_at"), Index("logged_at")],
)
data class MediaProgressLogEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "entry_id") val entryId: String,
    val delta: Int,
    @ColumnInfo(name = "value_after") val valueAfter: Int,
    @ColumnInfo(name = "logged_at") val loggedAt: Long,
)
