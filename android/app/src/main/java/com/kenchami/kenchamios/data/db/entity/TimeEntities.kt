package com.kenchami.kenchamios.data.db.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.kenchami.kenchamios.data.model.TimeEntryKind
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus

@Entity(
    tableName = "time_entries",
    foreignKeys = [
        ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"]),
        ForeignKey(TaskEntity::class, parentColumns = ["id"], childColumns = ["task_id"]),
    ],
    indices = [Index("project_id", "started_at"), Index("task_id"), Index("started_at")],
)
data class TimeEntryEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String,
    @ColumnInfo(name = "task_id") val taskId: String?,
    val kind: TimeEntryKind,
    @ColumnInfo(name = "started_at") val startedAt: Long,
    @ColumnInfo(name = "ended_at") val endedAt: Long,
    @ColumnInfo(name = "duration_s") val durationS: Int,
    val note: String,
)

// one row per device, its id is the device id; the timer is not synced
@Entity(
    tableName = "timer_state",
    foreignKeys = [
        ForeignKey(ProjectEntity::class, parentColumns = ["id"], childColumns = ["project_id"]),
        ForeignKey(TaskEntity::class, parentColumns = ["id"], childColumns = ["task_id"]),
    ],
    indices = [Index("project_id"), Index("task_id")],
)
data class TimerStateEntity(
    @PrimaryKey val id: String,
    @ColumnInfo(name = "created_at") val createdAt: Long,
    @ColumnInfo(name = "updated_at") val updatedAt: Long,
    @ColumnInfo(name = "device_id") val deviceId: String,
    val deleted: Boolean,
    @ColumnInfo(name = "project_id") val projectId: String,
    @ColumnInfo(name = "task_id") val taskId: String,
    val phase: TimerPhase,
    val status: TimerStatus,
    val round: Int,
    @ColumnInfo(name = "phase_started_at") val phaseStartedAt: Long,
    @ColumnInfo(name = "phase_length_s") val phaseLengthS: Int,
    @ColumnInfo(name = "elapsed_ms") val elapsedMs: Long,
    @ColumnInfo(name = "running_since") val runningSince: Long?,
)
