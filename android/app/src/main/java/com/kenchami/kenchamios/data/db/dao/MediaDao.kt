package com.kenchami.kenchamios.data.db.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Update
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.db.entity.MediaEntryEntity
import com.kenchami.kenchamios.data.db.entity.MediaItemEntity
import com.kenchami.kenchamios.data.db.entity.MediaProgressLogEntity
import kotlinx.coroutines.flow.Flow

private const val SELECT_ENTRY =
    """SELECT e.id, e.updated_at, e.media_item_id, i.type, i.title, i.year, i.genres, i.synopsis,
        i.source, e.status, e.progress, e.progress_total, e.rating, e.notes, e.tags,
        e.started_at, e.finished_at, e.platform
     FROM media_entries e JOIN media_items i ON i.id = e.media_item_id """

@Dao
interface MediaDao {
    @Insert
    suspend fun insertItem(item: MediaItemEntity)

    @Insert
    suspend fun insertEntry(entry: MediaEntryEntity)

    @Insert
    suspend fun insertLog(log: MediaProgressLogEntity)

    @Update
    suspend fun updateEntry(entry: MediaEntryEntity)

    @Query("SELECT * FROM media_entries WHERE id = :id AND deleted = 0")
    suspend fun getEntry(id: String): MediaEntryEntity?

    @Query(SELECT_ENTRY + "WHERE e.deleted = 0 AND i.deleted = 0 ORDER BY e.updated_at DESC")
    fun observeLibrary(): Flow<List<LibraryEntry>>

    @Query(SELECT_ENTRY + "WHERE e.id = :id AND e.deleted = 0 AND i.deleted = 0")
    fun observeEntry(id: String): Flow<LibraryEntry?>

    @Query(SELECT_ENTRY + "WHERE e.id = :id AND e.deleted = 0 AND i.deleted = 0")
    suspend fun getLibraryEntry(id: String): LibraryEntry?

    @Query("UPDATE media_entries SET deleted = 1, updated_at = :stamp, device_id = :deviceId WHERE id = :id AND deleted = 0")
    suspend fun softDeleteEntry(id: String, stamp: Long, deviceId: String): Int

    @Query(
        """UPDATE media_items SET deleted = 1, updated_at = :stamp, device_id = :deviceId
          WHERE id = (SELECT media_item_id FROM media_entries WHERE id = :entryId) AND deleted = 0""",
    )
    suspend fun softDeleteItemOfEntry(entryId: String, stamp: Long, deviceId: String): Int
}
