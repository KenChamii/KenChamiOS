package com.kenchami.kenchamios

import android.content.Intent

const val MAX_SHARED_CHARS = 500

private val CONTROL_CHARS = Regex("""\p{Cntrl}+""")
private val SPACES = Regex("""\s+""")
private val FILE_URI = Regex("""^(file|content):""", RegexOption.IGNORE_CASE)

/**
 * The text another app shared, cleaned to one line for the quick add field, or null.
 * The activity is exported, so only plain text is read: no streams, no file or content URIs, and at most [MAX_SHARED_CHARS].
 */
fun sharedText(intent: Intent?): String? {
    if (intent?.action != Intent.ACTION_SEND || intent.type != "text/plain") return null
    val raw = intent.getCharSequenceExtra(Intent.EXTRA_TEXT)?.toString() ?: return null
    val line = raw.take(MAX_SHARED_CHARS * 2).replace(CONTROL_CHARS, " ").replace(SPACES, " ").trim().take(MAX_SHARED_CHARS)
    return line.takeIf { it.isNotEmpty() && !FILE_URI.containsMatchIn(it) }
}
