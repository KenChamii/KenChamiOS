package com.kenchami.kenchamios.data

import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.core.Uuid7Generator
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.util.UUID

@RunWith(RobolectricTestRunner::class)
class StamperTest {
    private lateinit var env: TestDatabase

    @Before
    fun setUp() {
        env = TestDatabase()
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun deviceIdIsAUuidStoredOnceAsAJsonSetting() = runBlocking<Unit> {
        val id = env.stamper.deviceId()

        assertEquals(7, UUID.fromString(id).version())
        assertEquals("\"$id\"", env.database.settingDao().valueOf("device_id"))
        assertEquals(1, env.count("SELECT COUNT(*) FROM settings WHERE key = 'device_id'"))
    }

    @Test
    fun anotherStamperOnTheSameDatabaseReadsTheStoredId() = runBlocking<Unit> {
        val first = env.stamper.deviceId()
        val second = Stamper(env.clock, Hlc(env.clock), Uuid7Generator(env.clock), env.database.settingDao())

        assertEquals(first, second.deviceId())
    }

    @Test
    fun stampsAlwaysGrow() {
        val first = env.stamper.stamp()
        assertTrue(env.stamper.stamp() > first)
    }
}
