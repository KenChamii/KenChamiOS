package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.repo.ProjectRepository
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import mockwebserver3.MockResponse
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

class MemorySecretStore : SecretStore {
    private var value: String? = null

    override fun get() = value

    override fun set(value: String) {
        this.value = value
    }

    override fun clear() {
        value = null
    }
}

@RunWith(RobolectricTestRunner::class)
class SyncRunnerTest {
    private val env = TestDatabase()
    private val state = SyncState(env.database.settingDao(), env.stamper)
    private val secrets = MemorySecretStore()
    private var found: Endpoint? = null
    private var pullReply: () -> MockResponse = { TestDesktop.json(fromDesktop(SyncFixtures.obj("pull-valid")).toString()) }
    private val desktop = TestDesktop { request ->
        if (request.method == "POST") TestDesktop.json("{\"applied\":1,\"ignored\":0}") else pullReply()
    }
    private lateinit var runner: SyncRunner

    @Before
    fun setUp() {
        val store = SyncStore(env.database, SyncFixtures.schema, env.hlc, env.clock, env.stamper)
        runner = SyncRunner(store, state, secrets, desktop.apis(), { found }, env.stamper, env.clock)
    }

    @After
    fun tearDown() {
        desktop.close()
        env.close()
    }

    // the rows of the shared fixture are written by the desktop in these tests
    private fun fromDesktop(page: JSONObject): JSONObject {
        val changes = page.getJSONArray("changes")
        for (i in 0 until changes.length()) changes.getJSONObject(i).getJSONObject("row").put("device_id", "dev-desktop")
        return page
    }

    private suspend fun pair(port: Int = desktop.port) {
        state.savePairing(DesktopLink(desktop.host, port, desktop.fingerprint, "dev-desktop"))
        secrets.set("secret-token")
    }

    @Test
    fun withoutAPairingNothingHappens() = runBlocking {
        assertEquals(SyncResult.NotPaired, runner.sync())
        assertTrue(desktop.requests.isEmpty())
    }

    @Test
    fun aRoundPushesLocalChangesPullsDesktopChangesAndMovesTheCursors() = runBlocking {
        ProjectRepository(env.database, env.stamper, env.activity).create("From the phone")
        pair()

        val result = runner.sync()

        assertEquals(SyncResult.Done(sent = 2, received = 2), result)
        val push = desktop.requests.first { it.method == "POST" }
        assertEquals("Bearer ${env.stamper.deviceId()}.secret-token", push.headers["Authorization"])
        val changes = JSONObject(push.body?.utf8() ?: "").getJSONArray("changes")
        val tables = List(changes.length()) { changes.getJSONObject(it).getString("table") }
        assertEquals(listOf("projects", "activity"), tables)
        assertEquals(42, state.pullSeq())
        assertTrue(state.sentSeq() > 0)
        assertEquals(2, env.count("SELECT COUNT(*) FROM tasks"))
        val snapshot = state.observe().first()
        assertTrue(snapshot.lastSyncAt != null && snapshot.lastError == null)
    }

    @Test
    fun theNextRoundStartsWhereTheLastOneStopped() = runBlocking {
        pair()
        runner.sync()
        desktop.requests.clear()

        runner.sync()

        val pull = desktop.requests.single { it.method == "GET" }
        assertEquals("/sync/pull?since=42&limit=$MAX_CHANGES", pull.target)
        assertTrue("nothing new to push", desktop.requests.none { it.method == "POST" })
    }

    @Test
    fun aRevokedPhoneForgetsThePairing() = runBlocking {
        pair()
        pullReply = { TestDesktop.json("{\"error\":\"not paired\"}", 401) }

        val result = runner.sync()

        assertTrue(result is SyncResult.Failed)
        assertNull(state.link())
        assertNull(secrets.get())
    }

    @Test
    fun aChangedCertificateStopsTheSyncButKeepsThePairing() = runBlocking {
        state.savePairing(DesktopLink(desktop.host, desktop.port, "0".repeat(64), "dev-desktop"))
        secrets.set("secret-token")

        val result = runner.sync()

        assertTrue(result is SyncResult.Failed)
        assertNotNull(state.link())
        assertTrue(desktop.requests.isEmpty())
    }

    @Test
    fun whenTheDesktopMovedItIsFoundAgainAndTheAddressIsUpdated() = runBlocking {
        pair(port = 1)
        found = Endpoint(desktop.host, desktop.port)

        val result = runner.sync()

        assertTrue(result is SyncResult.Done)
        assertEquals(desktop.port, state.link()?.port)
    }

    @Test
    fun withoutAnyDesktopTheFailureIsRecorded() = runBlocking {
        pair(port = 1)

        val result = runner.sync()

        assertTrue(result is SyncResult.Failed)
        assertNotNull(state.observe().first().lastError)
    }

    @Test
    fun invalidDataFromTheDesktopIsNotStored() = runBlocking {
        val bad = fromDesktop(SyncFixtures.obj("pull-valid"))
        bad.getJSONArray("changes").getJSONObject(0).getJSONObject("row").put("status", "blocked")
        pullReply = { TestDesktop.json(bad.toString()) }
        pair()

        val result = runner.sync()

        assertTrue(result is SyncResult.Failed)
        assertEquals(0, env.count("SELECT COUNT(*) FROM tasks"))
        assertEquals(0, state.pullSeq())
    }
}
