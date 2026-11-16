package com.kenchami.kenchamios.ui.media

import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType

const val UP_NEXT_LIMIT = 8

data class MediaQueues(val continueWith: List<LibraryEntry> = emptyList(), val upNext: List<LibraryEntry> = emptyList()) {
    val isEmpty: Boolean get() = continueWith.isEmpty() && upNext.isEmpty()
}

/** Started titles, the one touched last first, and planned titles, the one touched longest ago first. */
fun mediaQueues(library: List<LibraryEntry>, type: MediaType?): MediaQueues {
    val ofType = library.filter { type == null || it.mediaType == type }
    return MediaQueues(
        continueWith = ofType.filter { it.status == MediaStatus.InProgress }.sortedByDescending { it.updatedAt },
        // entries have no creation time on the phone, so the last change stands in for it
        upNext = ofType.filter { it.status == MediaStatus.Planned }.sortedBy { it.updatedAt }.take(UP_NEXT_LIMIT),
    )
}

/** What to open next in a started title, for example "Episode 5"; null for types without units. */
fun nextUnit(entry: LibraryEntry): String? {
    val unit = when (entry.mediaType) {
        MediaType.Anime, MediaType.Show -> "Episode"
        MediaType.Manga -> "Chapter"
        else -> return null
    }
    val next = entry.progress + 1
    return if (entry.progressTotal != null && next > entry.progressTotal) null else "$unit $next"
}
