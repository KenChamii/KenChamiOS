package com.kenchami.kenchamios.sync

import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SyncApiTest {
    private var reply = TestDesktop.json("{}")
    private val desktop = TestDesktop { reply }

    @After
    fun close() = desktop.close()

    @Test
    fun aServerWithTheRightCertificateIsTrustedAndPullIsParsed() = runBlocking {
        reply = TestDesktop.json(SyncFixtures.text("pull-valid"))

        val page = desktop.api().pull("dev.token", 7, 50)

        assertEquals(2, page.changes.size)
        assertEquals(42, page.next)
        assertEquals(false, page.hasMore)
        val request = desktop.requests.single()
        assertEquals("/sync/pull?since=7&limit=50", request.target)
        assertEquals("Bearer dev.token", request.headers["Authorization"])
    }

    @Test
    fun aServerWithAnotherCertificateIsRefusedBeforeAnythingIsSent() {
        val wrongPin = "0".repeat(64)

        val failure = assertThrows(SyncFailure.PinMismatch::class.java) {
            runBlocking { desktop.api(wrongPin).pull("dev.token", 0, 10) }
        }

        assertTrue(failure.cause != null)
        assertTrue("no request reached the server", desktop.requests.isEmpty())
    }

    @Test
    fun pushSendsTheChangesAsJson() = runBlocking {
        reply = TestDesktop.json("{\"applied\":11,\"ignored\":0}")

        val outcome = desktop.api().push("dev.token", SyncFixtures.changes("push-valid"))

        assertEquals(ApplyOutcome(11, 0), outcome)
        val sent = JSONObject(desktop.requests.single().body?.utf8() ?: "")
        assertEquals(11, sent.getJSONArray("changes").length())
        assertEquals("POST", desktop.requests.single().method)
    }

    @Test
    fun pairSendsTheCodeAndReturnsTheCredential() = runBlocking {
        reply = TestDesktop.json(SyncFixtures.text("pair-response"))

        val result = desktop.api().pair("7K3M9QX2TV", "phone-1", "Pixel")

        assertEquals("x".repeat(43), result.token)
        assertEquals("0190f0a2-7a1c-7c3e-9b52-1d3f6a8e4c11", result.desktopDeviceId)
        val sent = JSONObject(desktop.requests.single().body?.utf8() ?: "")
        assertEquals("7K3M9QX2TV", sent.getString("code"))
    }

    @Test
    fun statusCodesBecomeSpecificFailures() {
        val expected = mapOf(
            401 to SyncFailure.AuthLost::class.java,
            403 to SyncFailure.WrongCode::class.java,
            404 to SyncFailure.PairingClosed::class.java,
            429 to SyncFailure.Throttled::class.java,
            422 to SyncFailure.Refused::class.java,
            500 to SyncFailure.BadAnswer::class.java,
        )
        for ((code, type) in expected) {
            reply = TestDesktop.json("{\"error\":\"nope\"}", code)
            assertThrows("status $code", type) { runBlocking { desktop.api().pull("dev.token", 0, 10) } }
        }
    }

    @Test
    fun aMalformedAnswerIsABadAnswer() {
        reply = TestDesktop.json("{\"changes\":\"not a list\"}")

        assertThrows(SyncFailure.BadAnswer::class.java) { runBlocking { desktop.api().pull("dev.token", 0, 10) } }
    }

    @Test
    fun aClosedPortIsUnreachable() {
        val api = SyncApi(PinnedTls.client(desktop.fingerprint), desktop.host, 1)

        assertThrows(SyncFailure.Unreachable::class.java) { runBlocking { api.pull("dev.token", 0, 10) } }
    }

    @Test
    fun thePinMustBeSixtyFourLowercaseHexCharacters() {
        for (bad in listOf("", "abc", "G".repeat(64), "A".repeat(64))) {
            assertThrows(IllegalArgumentException::class.java) { PinnedTrustManager(bad) }
        }
    }
}
