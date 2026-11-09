package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.db.dao.ChangeLogDao
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/** While the phone is paired: keeps the 15 minute schedule and asks for a sync soon after every local change. */
@Singleton
class SyncTrigger @Inject constructor(
    private val changeLog: ChangeLogDao,
    private val state: SyncState,
    private val scheduler: SyncScheduler,
    @AppScope private val scope: CoroutineScope,
) {
    private var job: Job? = null

    /** Safe to call from every activity start; only the first call begins watching. */
    @OptIn(ExperimentalCoroutinesApi::class)
    fun start() {
        if (job?.isActive == true) return
        job = scope.launch {
            state.observe().map { it.link != null }.distinctUntilChanged().transformLatest { paired ->
                if (paired) {
                    scheduler.schedulePeriodic()
                    emitAll(changeLog.observeLatest())
                } else {
                    scheduler.cancelAll()
                }
            }.collect { scheduler.syncSoon() }
        }
    }
}
