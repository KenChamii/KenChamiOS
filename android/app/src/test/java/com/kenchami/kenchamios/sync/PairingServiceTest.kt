package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class PairingServiceTest {
    private val env = TestDatabase()
    private val state = SyncState(env.database.settingDao(), env.stamper)
    private val secrets = MemorySecretStore()
    private val desktop = TestDesktop { TestDesktop.json(SyncFixtures.text("pair-response")) }
    private val pairing = PairingService(state, secrets, desktop.apis(), env.stamper, "Test phone")

    @After
    fun tearDown() {
        desktop.close()
        env.close()
    }

    private fun payload(fingerprint: String = desktop.fingerprint) = PairingPayload(desktop.host, desktop.port, fingerprint, "7K3M9QX2TV")

    @Test
    fun pairingStoresTheLinkAndTheCredential() = runBlocking {
        pairing.pair(payload())

        assertEquals("x".repeat(43), secrets.get())
        val link = state.link()
        assertEquals(DesktopLink(desktop.host, desktop.port, desktop.fingerprint, "0190f0a2-7a1c-7c3e-9b52-1d3f6a8e4c11"), link)
        assertEquals(0, state.sentSeq())
        assertEquals(0, state.pullSeq())
    }

    @Test
    fun theCodeIsNeverSentToAServerWithAnotherCertificate() {
        assertThrows(SyncFailure.PinMismatch::class.java) { runBlocking { pairing.pair(payload("0".repeat(64))) } }

        assertTrue(desktop.requests.isEmpty())
        assertNull(secrets.get())
    }

    @Test
    fun unpairingForgetsEverything() = runBlocking {
        pairing.pair(payload())

        pairing.unpair()

        assertNull(state.link())
        assertNull(secrets.get())
    }
}
