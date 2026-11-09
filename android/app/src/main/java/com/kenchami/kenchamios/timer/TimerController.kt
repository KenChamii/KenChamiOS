package com.kenchami.kenchamios.timer

import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.TimerRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

/** The phase that ended and the one that started in its place. */
data class PhaseChange(val finished: TimerPhase, val next: TimerPhase)

/** The one timer of this device. The screens and the foreground service both go through it. */
@Singleton
class TimerController @Inject constructor(
    private val repository: TimerRepository,
    private val tasks: TaskRepository,
    private val clock: Clock,
) {
    private val lock = Mutex()
    private val current = MutableStateFlow<TimerState?>(null)
    private val config = TimerConfig.DEFAULT

    val state: StateFlow<TimerState?> = current

    /** Picks up the timer that was running when the app last closed. */
    suspend fun recover() {
        lock.withLock { if (current.value == null) current.value = repository.load() }
    }

    suspend fun start(taskId: String) {
        lock.withLock {
            check(current.value == null) { "a timer is already running" }
            val task = checkNotNull(tasks.get(taskId)) { "task not found" }
            val started = TimerState.start(task.projectId, task.id, config, clock.millis())
            repository.save(started)
            current.value = started
        }
    }

    suspend fun pause() = change { it.pause(clock.millis()) }

    suspend fun resume() = change { it.resume(clock.millis()) }

    /** Drops the timer; the unfinished phase is not logged. */
    suspend fun stop() {
        lock.withLock {
            repository.clear()
            current.value = null
        }
    }

    /** Ends the current phase when it is due, logging a finished focus phase. */
    suspend fun tick(): PhaseChange? = lock.withLock {
        val running = current.value ?: return@withLock null
        val (next, finished) = running.advance(config, clock.millis()) ?: return@withLock null
        repository.saveFinished(next, finished)
        current.value = next
        PhaseChange(finished.phase, next.phase)
    }

    private suspend fun change(apply: (TimerState) -> TimerState) {
        lock.withLock {
            val running = current.value ?: return@withLock
            val updated = apply(running)
            repository.save(updated)
            current.value = updated
        }
    }
}
