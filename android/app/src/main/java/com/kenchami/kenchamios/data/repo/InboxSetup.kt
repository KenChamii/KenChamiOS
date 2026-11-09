package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.dao.ProjectDao
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Stage

/** Creates the inbox the first time it is needed, or brings it back if it was deleted; call it inside a transaction. */
internal suspend fun ensureInbox(projects: ProjectDao, stamper: Stamper, activity: ActivityRecorder) {
    val existing = projects.getAny(Inbox.ID)
    if (existing != null && !existing.deleted) return
    if (existing == null) {
        projects.insert(
            ProjectEntity(
                id = Inbox.ID,
                createdAt = stamper.nowMs(),
                updatedAt = stamper.stamp(),
                deviceId = stamper.deviceId(),
                deleted = false,
                name = Inbox.NAME,
                slug = Inbox.SLUG,
                stage = Stage.Idea,
                description = "",
                repoPath = null,
                color = null,
                tags = emptyList(),
                archived = false,
                finishedAt = null,
            ),
        )
    } else {
        projects.update(existing.copy(deleted = false, updatedAt = stamper.stamp(), deviceId = stamper.deviceId()))
    }
    activity.record("project.created", "project", Inbox.ID, "Created the inbox")
}
