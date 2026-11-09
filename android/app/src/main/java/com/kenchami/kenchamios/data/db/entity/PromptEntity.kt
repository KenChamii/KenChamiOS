package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "prompts",
    foreignKeys = [ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"])],
    indices = [Index("project_id")],
)
data class PromptEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String?,
    val title: String,
    @ColumnInfo(name = "body_md") val bodyMd: String,
    val tags: List<String>,
    val favorite: Boolean,
    @ColumnInfo(name = "use_count") val useCount: Int,
    @ColumnInfo(name = "last_used_at") val lastUsedAt: Long?,
)
