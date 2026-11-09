package com.kenchami.kenchamios.sync

import mockwebserver3.Dispatcher
import mockwebserver3.MockResponse
import mockwebserver3.MockWebServer
import mockwebserver3.RecordedRequest
import okhttp3.tls.HandshakeCertificates
import okhttp3.tls.HeldCertificate
import java.security.MessageDigest

/** A stand-in for the desktop: an HTTPS server with a self-signed certificate made for the test. */
class TestDesktop(private val answer: (RecordedRequest) -> MockResponse = { MockResponse.Builder().code(404).build() }) {
    private val certificate = HeldCertificate.Builder().commonName("kenchamios.local").build()
    private val server = MockWebServer()
    val requests = mutableListOf<RecordedRequest>()

    val fingerprint: String = MessageDigest.getInstance("SHA-256").digest(certificate.certificate.encoded)
        .joinToString("") { "%02x".format(it) }

    init {
        server.useHttps(HandshakeCertificates.Builder().heldCertificate(certificate).build().sslSocketFactory())
        server.dispatcher = object : Dispatcher() {
            override fun dispatch(request: RecordedRequest): MockResponse {
                synchronized(requests) { requests += request }
                return answer(request)
            }
        }
        server.start()
    }

    val host: String get() = server.hostName
    val port: Int get() = server.port

    fun api(fingerprint: String = this.fingerprint) = SyncApi(PinnedTls.client(fingerprint), host, port)

    fun apis() = SyncApiFactory { host, port, pin -> SyncApi(PinnedTls.client(pin), host, port) }

    fun close() = server.close()

    companion object {
        fun json(body: String, code: Int = 200): MockResponse =
            MockResponse.Builder().code(code).addHeader("Content-Type", "application/json").body(body).build()
    }
}
