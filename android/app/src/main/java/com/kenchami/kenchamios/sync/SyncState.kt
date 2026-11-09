package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.core.decodeString
import com.kenchami.kenchamios.core.encodeString
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.dao.SettingDao
import com.kenchami.kenchamios.data.db.entity.SettingEntity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

private const val PREFIX = "sync."
private const val HOST = "sync.host"
private const val PORT = "sync.port"
private const val FINGERPRINT = "sync.fingerprint"
private const val DESKTOP_ID = "sync.desktop_id"
private const val SENT_SEQ = "sync.sent_seq"
private const val PULL_SEQ = "sync.pull_seq"
private const val LAST_SYNC_AT = "sync.last_sync_at"
private const val LAST_ERROR = "sync.last_error"

/** What the sync screen shows. */
data class SyncSnapshot(val link: DesktopLink?, val lastSyncAt: Long?, val lastError: String?)

/** The pairing and the sync cursors, kept in the local settings table (never synced). The credential is not here. */
@Singleton
class SyncState @Inject constructor(private val settings: SettingDao, private val stamper: Stamper) {
    fun observe(): Flow<SyncSnapshot> = settings.observeWithPrefix(PREFIX).map { rows ->
        val values = rows.associate { it.key to it.valueJson }
        SyncSnapshot(link(values), values[LAST_SYNC_AT]?.toLongOrNull(), values[LAST_ERROR]?.let(::decodeString))
    }

    suspend fun link(): DesktopLink? = link(settings.withPrefix(PREFIX).associate { it.key to it.valueJson })

    private fun link(values: Map<String, String>): DesktopLink? {
        val host = values[HOST]?.let(::decodeString) ?: return null
        val port = values[PORT]?.toIntOrNull() ?: return null
        val fingerprint = values[FINGERPRINT]?.let(::decodeString) ?: return null
        val desktop = values[DESKTOP_ID]?.let(::decodeString) ?: return null
        return DesktopLink(host, port, fingerprint, desktop)
    }

    /** Stores a new pairing and starts the cursors from zero, so the first sync sends and receives everything. */
    suspend fun savePairing(link: DesktopLink) {
        clear()
        putText(HOST, link.host)
        putText(FINGERPRINT, link.fingerprint)
        putText(DESKTOP_ID, link.desktopDeviceId)
        putRaw(PORT, link.port.toString())
        putRaw(SENT_SEQ, "0")
        putRaw(PULL_SEQ, "0")
    }

    /** The desktop moved to another address; the pin and the cursors stay. */
    suspend fun updateAddress(host: String, port: Int) {
        putText(HOST, host)
        putRaw(PORT, port.toString())
    }

    suspend fun clear() = settings.deleteWithPrefix(PREFIX)

    suspend fun sentSeq(): Long = settings.valueOf(SENT_SEQ)?.toLongOrNull() ?: 0

    suspend fun setSentSeq(value: Long) = putRaw(SENT_SEQ, value.toString())

    suspend fun pullSeq(): Long = settings.valueOf(PULL_SEQ)?.toLongOrNull() ?: 0

    suspend fun setPullSeq(value: Long) = putRaw(PULL_SEQ, value.toString())

    suspend fun markSynced(at: Long) {
        putRaw(LAST_SYNC_AT, at.toString())
        settings.deleteWithPrefix(LAST_ERROR)
    }

    suspend fun markFailed(message: String) = putText(LAST_ERROR, message)

    private suspend fun putText(key: String, value: String) = putRaw(key, encodeString(value))

    private suspend fun putRaw(key: String, json: String) {
        val now = stamper.nowMs()
        settings.put(SettingEntity(stamper.newId(), now, stamper.stamp(), stamper.deviceId(), false, key, json))
    }
}
