package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.Stamper
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

private const val PUSH_BATCH = 100

/** Builds the HTTPS client for a desktop; the real one pins its certificate. */
fun interface SyncApiFactory {
    fun create(host: String, port: Int, fingerprint: String): SyncApi
}

val PinnedApiFactory = SyncApiFactory { host, port, fingerprint -> SyncApi(PinnedTls.client(fingerprint), host, port) }

/** Finds the desktop on the network again when its address changed. */
fun interface DesktopLocator {
    suspend fun find(): Endpoint?
}

data class Endpoint(val host: String, val port: Int)

sealed interface SyncResult {
    data class Done(val sent: Int, val received: Int) : SyncResult

    data object NotPaired : SyncResult

    data class Failed(val message: String) : SyncResult
}

/** One sync round: push what changed here, then pull what changed on the desktop. */
@Singleton
class SyncRunner @Inject constructor(
    private val store: SyncStore,
    private val state: SyncState,
    private val secrets: SecretStore,
    private val apis: SyncApiFactory,
    private val locator: DesktopLocator,
    private val stamper: Stamper,
    private val clock: Clock,
) {
    private val lock = Mutex()

    suspend fun sync(): SyncResult = lock.withLock {
        val link = state.link()
        val token = secrets.get()
        if (link == null || token == null) SyncResult.NotPaired else attempt(link, "${stamper.deviceId()}.$token")
    }

    private suspend fun attempt(link: DesktopLink, credential: String): SyncResult = try {
        roundWithRediscovery(link, credential).also { state.markSynced(clock.millis()) }
    } catch (failure: SyncFailure.AuthLost) {
        unpair()
        fail(failure)
    } catch (failure: SyncFailure) {
        fail(failure)
    } catch (rejected: RejectedException) {
        fail(rejected)
    }

    private suspend fun fail(error: Exception): SyncResult.Failed {
        val message = error.message ?: "Sync failed."
        state.markFailed(message)
        return SyncResult.Failed(message)
    }

    private suspend fun unpair() {
        secrets.clear()
        state.clear()
    }

    private suspend fun roundWithRediscovery(link: DesktopLink, credential: String): SyncResult.Done = try {
        round(link, credential)
    } catch (unreachable: SyncFailure.Unreachable) {
        val found = locator.find()
        if (found == null || (found.host == link.host && found.port == link.port)) throw unreachable
        state.updateAddress(found.host, found.port)
        round(link.copy(host = found.host, port = found.port), credential)
    }

    private suspend fun round(link: DesktopLink, credential: String): SyncResult.Done {
        val api = apis.create(link.host, link.port, link.fingerprint)
        val sent = push(api, link, credential)
        val received = pull(api, credential)
        store.purgeSentTombstones(state.sentSeq())
        return SyncResult.Done(sent, received)
    }

    private suspend fun push(api: SyncApi, link: DesktopLink, credential: String): Int {
        var sent = 0
        var cursor = state.sentSeq()
        do {
            val page = store.readPage(cursor, PUSH_BATCH, link.desktopDeviceId)
            if (page.changes.isNotEmpty()) {
                api.push(credential, page.changes)
                sent += page.changes.size
            }
            cursor = page.next
            state.setSentSeq(cursor)
        } while (page.hasMore)
        return sent
    }

    private suspend fun pull(api: SyncApi, credential: String): Int {
        var received = 0
        var cursor = state.pullSeq()
        do {
            val page = api.pull(credential, cursor, MAX_CHANGES)
            received += store.apply(page.changes).applied
            cursor = page.next
            state.setPullSeq(cursor)
        } while (page.hasMore)
        return received
    }
}
