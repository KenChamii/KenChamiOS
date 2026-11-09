package com.kenchami.kenchamios.data

import com.kenchami.kenchamios.data.db.dao.ActivityDao
import com.kenchami.kenchamios.data.db.entity.ActivityEntity
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ActivityRecorder @Inject constructor(private val stamper: Stamper, private val activity: ActivityDao) {
    /** Adds a timeline row; call it inside the transaction of the write it describes. */
    suspend fun record(kind: String, entityType: String, entityId: String, summary: String) {
        val now = stamper.nowMs()
        activity.insert(
            ActivityEntity(
                id = stamper.newId(),
                createdAt = now,
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
                deleted = false,
                kind = kind,
                entityType = entityType,
                entityId = entityId,
                summary = summary,
                at = now,
            ),
        )
    }
}
