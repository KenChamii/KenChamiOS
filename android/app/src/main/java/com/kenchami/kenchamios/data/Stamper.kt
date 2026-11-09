package com.kenchami.kenchamios.data

import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.core.Uuid7Generator
import com.kenchami.kenchamios.core.decodeString
import com.kenchami.kenchamios.core.encodeString
import com.kenchami.kenchamios.data.db.dao.SettingDao
import com.kenchami.kenchamios.data.db.entity.SettingEntity
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

private const val DEVICE_ID_KEY = "device_id"

/** Everything a write needs to fill the sync columns: ids, times, the clock value and the device id. */
@Singleton
class Stamper @Inject constructor(
    private val clock: Clock,
    private val hlc: Hlc,
    private val uuids: Uuid7Generator,
    private val settings: SettingDao,
) {
    private val deviceIdLock = Mutex()
    private var deviceId: String? = null

    fun newId(): String = uuids.next()

    /** Unix ms, for `created_at` and the other plain timestamps. */
    fun nowMs(): Long = clock.millis()

    /** The next `updated_at` value. */
    fun stamp(): Long = hlc.now()

    /** The id of this device, created and stored on the first call. */
    suspend fun deviceId(): String = deviceId ?: deviceIdLock.withLock { deviceId ?: loadDeviceId().also { deviceId = it } }

    private suspend fun loadDeviceId(): String {
        val candidate = newId()
        settings.insertIfMissing(
            SettingEntity(newId(), nowMs(), stamp(), candidate, false, DEVICE_ID_KEY, encodeString(candidate)),
        )
        return decodeString(checkNotNull(settings.valueOf(DEVICE_ID_KEY)) { "device id was not stored" })
    }
}
