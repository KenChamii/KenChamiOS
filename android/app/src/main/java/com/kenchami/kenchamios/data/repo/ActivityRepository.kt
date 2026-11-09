package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.ActivityEntity
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ActivityRepository @Inject constructor(database: AppDatabase) {
    private val activity = database.activityDao()

    /** The newest activity first. */
    fun observeRecent(limit: Int): Flow<List<ActivityEntity>> = activity.observeRecent(limit)
}
