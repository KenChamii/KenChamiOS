package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.model.MediaStatus

/** The fields of a library entry that the user edits. */
data class EntryInput(
    val status: MediaStatus,
    val progressTotal: Int?,
    val rating: Int?,
    val notes: String,
    val tags: List<String>,
    val startedAt: Long?,
    val finishedAt: Long?,
    val platform: String?,
) {
    fun validate() {
        require(rating == null || rating in 1..10) { "the rating must be 1 to 10" }
        require(progressTotal == null || progressTotal >= 0) { "the total cannot be negative" }
    }
}
