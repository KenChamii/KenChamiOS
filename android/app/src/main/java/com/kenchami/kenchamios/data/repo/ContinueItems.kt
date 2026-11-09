package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType

private const val MAX_ITEMS = 6
private val CONTINUABLE = setOf(MediaType.Anime, MediaType.Manga, MediaType.Show, MediaType.Game)

/** What is in progress, most recently touched first. Movies and music have nothing to continue. */
fun continueItems(entries: List<LibraryEntry>): List<LibraryEntry> = entries
    .filter { it.status == MediaStatus.InProgress && it.mediaType in CONTINUABLE }
    .sortedByDescending { it.updatedAt }
    .take(MAX_ITEMS)

/** "Episode 13", "Chapter 141", or the hours so far for a game. */
fun nextLabel(entry: LibraryEntry): String = when (entry.mediaType) {
    MediaType.Manga -> "Chapter ${entry.progress + 1}"
    MediaType.Game -> if (entry.progress == 0) "Just started" else "${entry.progress} h played"
    else -> "Episode ${entry.progress + 1}"
}
