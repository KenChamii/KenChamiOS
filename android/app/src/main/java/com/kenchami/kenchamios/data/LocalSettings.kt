package com.kenchami.kenchamios.data

import com.kenchami.kenchamios.data.db.dao.SettingDao
import com.kenchami.kenchamios.data.db.dao.SettingValue
import com.kenchami.kenchamios.data.db.entity.SettingEntity
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

/** Reads and writes rows of the settings table, which never syncs and has no change log. */
@Singleton
class LocalSettings @Inject constructor(private val settings: SettingDao, private val stamper: Stamper) {
    suspend fun get(key: String): String? = settings.valueOf(key)

    suspend fun withPrefix(prefix: String): List<SettingValue> = settings.withPrefix(prefix)

    fun observePrefix(prefix: String): Flow<List<SettingValue>> = settings.observeWithPrefix(prefix)

    suspend fun put(key: String, json: String) {
        settings.put(SettingEntity(stamper.newId(), stamper.nowMs(), stamper.stamp(), stamper.deviceId(), false, key, json))
    }

    suspend fun delete(key: String) = settings.delete(key)
}
