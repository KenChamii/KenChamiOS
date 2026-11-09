package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.TimeEntryEntity
import com.kenchami.kenchamios.data.db.entity.TimerStateEntity
import com.kenchami.kenchamios.data.model.TimeEntryKind
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.timer.FinishedPhase
import com.kenchami.kenchamios.timer.TimerState
import javax.inject.Inject
import javax.inject.Singleton

/** Stores the timer of this device and the focus time it logs. */
@Singleton
class TimerRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val timers = database.timerDao()

    /** The timer saved for this device, if one is running or paused. */
    suspend fun load(): TimerState? = timers.get(stamper.deviceId())?.let {
        TimerState(
            projectId = it.projectId,
            taskId = it.taskId,
            phase = it.phase,
            status = it.status,
            round = it.round,
            phaseStartedAt = it.phaseStartedAt,
            phaseLengthS = it.phaseLengthS,
            elapsedMs = it.elapsedMs,
            runningSince = it.runningSince,
        )
    }

    suspend fun save(state: TimerState) {
        database.withTransaction { upsert(state) }
    }

    /** Saves the next phase together with the focus time that just ended; breaks are not logged. */
    suspend fun saveFinished(next: TimerState, finished: FinishedPhase) {
        database.withTransaction {
            if (finished.phase == TimerPhase.Focus) logFocus(next, finished)
            upsert(next)
        }
    }

    suspend fun clear() {
        timers.clear(stamper.deviceId(), stamper.stamp())
    }

    private suspend fun logFocus(timer: TimerState, finished: FinishedPhase) {
        val id = stamper.newId()
        timers.insertTimeEntry(
            TimeEntryEntity(
                id = id,
                createdAt = stamper.nowMs(),
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
                deleted = false,
                projectId = timer.projectId,
                taskId = timer.taskId,
                kind = TimeEntryKind.Focus,
                startedAt = finished.startedAt,
                endedAt = finished.endedAt,
                durationS = finished.durationS,
                note = "",
            ),
        )
        activity.record("time.logged", "time_entry", id, "Logged ${finished.durationS / 60} min")
    }

    private suspend fun upsert(state: TimerState) {
        val deviceId = stamper.deviceId()
        timers.save(
            TimerStateEntity(
                id = deviceId,
                createdAt = timers.createdAt(deviceId) ?: stamper.nowMs(),
                updatedAt = stamper.stamp(),
                deviceId = deviceId,
                deleted = false,
                projectId = state.projectId,
                taskId = state.taskId,
                phase = state.phase,
                status = state.status,
                round = state.round,
                phaseStartedAt = state.phaseStartedAt,
                phaseLengthS = state.phaseLengthS,
                elapsedMs = state.elapsedMs,
                runningSince = state.runningSince,
            ),
        )
    }
}
