package com.kenchami.kenchamios.data.repo

import androidx.room.withTransaction
import com.kenchami.kenchamios.data.ActivityRecorder
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.entity.DailyNoteEntity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.time.LocalDate
import javax.inject.Inject
import javax.inject.Singleton

/** [addition] below [body], separated by an empty line. */
fun appendToNote(body: String, addition: String): String {
    val kept = body.trimEnd()
    return if (kept.isEmpty()) addition else "$kept\n\n$addition"
}

/** One markdown note per local day. */
@Singleton
class DailyNoteRepository @Inject constructor(
    private val database: AppDatabase,
    private val stamper: Stamper,
    private val activity: ActivityRecorder,
) {
    private val notes = database.dailyNoteDao()

    /** The day's note text, empty while nothing was written for it. */
    suspend fun body(day: LocalDate): String = notes.get(day.toString())?.body.orEmpty()

    /** Days from [from] to [to] (both included, `YYYY-MM-DD`) whose note has text, for the calendar dots. */
    fun observeDaysWithText(from: LocalDate, to: LocalDate): Flow<Set<String>> =
        notes.observeDaysWithText(from.toString(), to.toString()).map { it.toSet() }

    /** Replaces the day's note, creating it on the first save. */
    suspend fun save(day: LocalDate, body: String) {
        database.withTransaction { write(day, body) }
    }

    /** Adds [text] at the end of the day's note, after an empty line. */
    suspend fun append(day: LocalDate, text: String) {
        val addition = text.trim()
        require(addition.isNotEmpty()) { "there is nothing to add to the note" }
        database.withTransaction { write(day, appendToNote(body(day), addition)) }
    }

    private suspend fun write(day: LocalDate, body: String) {
        val existing = notes.get(day.toString())
        if (existing != null) {
            notes.setBody(existing.id, body, stamper.stamp(), stamper.deviceId())
            return
        }
        val id = stamper.newId()
        notes.insert(DailyNoteEntity(id, stamper.nowMs(), stamper.stamp(), stamper.deviceId(), false, day.toString(), body))
        activity.record("daily_note.created", "daily_note", id, "Started the note for $day")
    }
}
