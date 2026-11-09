package com.kenchami.kenchamios.sync

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent

/** Workers are not Hilt components, so they reach the sync runner through this. */
@EntryPoint
@InstallIn(SingletonComponent::class)
interface SyncWorkerEntryPoint {
    fun runner(): SyncRunner
}

class SyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        EntryPointAccessors.fromApplication(applicationContext, SyncWorkerEntryPoint::class.java).runner().sync()
        // a failed round is recorded for the sync screen and the next schedule tries again
        return Result.success()
    }
}
