package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.kenchami.kenchamios.data.model.DocKind
import com.kenchami.kenchamios.data.model.SavePointSource
import com.kenchami.kenchamios.data.model.Stage

@Entity(tableName = "projects")
data class ProjectEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    val name: String,
    val slug: String,
    val stage: Stage,
    val description: String,
    @ColumnInfo(name = "repo_path") val repoPath: String?,
    val color: String?,
    val tags: List<String>,
    val archived: Boolean,
    @ColumnInfo(name = "finished_at") val finishedAt: Long?,
)

@Entity(
    tableName = "project_docs",
    foreignKeys = [ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"])],
    indices = [Index("project_id", "kind")],
)
data class ProjectDocEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String,
    val kind: DocKind,
    val title: String,
    @ColumnInfo(name = "body_md") val bodyMd: String,
)

@Entity(
    tableName = "save_points",
    foreignKeys = [ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"])],
    indices = [Index("project_id", "created_at")],
)
data class SavePointEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String,
    val stage: Stage,
    val source: SavePointSource,
    val note: String,
    @ColumnInfo(name = "done_md") val doneMd: String,
    @ColumnInfo(name = "next_md") val nextMd: String,
    @ColumnInfo(name = "session_summary_md") val sessionSummaryMd: String,
    @ColumnInfo(name = "git_branch") val gitBranch: String?,
    @ColumnInfo(name = "git_commit") val gitCommit: String?,
)
