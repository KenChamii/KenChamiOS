package com.kenchami.kenchamios

import android.content.Intent
import android.net.Uri
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SharedTextTest {
    private fun share(text: CharSequence?, type: String = "text/plain", action: String = Intent.ACTION_SEND) =
        Intent(action).setType(type).apply { text?.let { putExtra(Intent.EXTRA_TEXT, it) } }

    @Test
    fun plainTextBecomesOneCleanLine() {
        assertEquals("Read this https://example.com/a", sharedText(share("  Read this\n\thttps://example.com/a \u0000")))
    }

    @Test
    fun longTextIsCut() {
        assertEquals(MAX_SHARED_CHARS, sharedText(share("x".repeat(100_000)))?.length)
    }

    @Test
    fun onlyASendOfPlainTextIsRead() {
        assertNull(sharedText(null))
        assertNull(sharedText(share("hi", type = "image/png")))
        assertNull(sharedText(share("hi", action = Intent.ACTION_VIEW)))
        assertNull(sharedText(share(null)))
        assertNull(sharedText(share("   ")))
    }

    @Test
    fun streamsAndFileUrisAreIgnored() {
        val stream = share(null).putExtra(Intent.EXTRA_STREAM, Uri.parse("content://evil/file"))

        assertNull(sharedText(stream))
        assertNull(sharedText(share("file:///data/data/com.kenchami.kenchamios/databases/kenchamios.db")))
        assertNull(sharedText(share("CONTENT://media/external/1")))
    }
}
