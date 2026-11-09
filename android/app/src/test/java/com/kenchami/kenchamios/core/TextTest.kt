package com.kenchami.kenchamios.core

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TextTest {
    @Test
    fun slugifyJoinsWords() {
        assertEquals("my-app-v2", slugify("  My App!  v2 "))
    }

    @Test
    fun slugifyFallsBackWhenEmpty() {
        assertEquals("project", slugify("!!!"))
    }

    @Test
    fun cleanTagsTrimsAndDedupes() {
        assertEquals(listOf("rust", "ui"), cleanTags(listOf(" rust ", "", "rust", "ui")))
    }

    @Test
    fun stringListUsesTheDesktopJsonForm() {
        assertEquals("""["a","b c"]""", encodeStringList(listOf("a", "b c")))
        assertEquals(listOf("a", "b \"c\""), decodeStringList(encodeStringList(listOf("a", "b \"c\""))))
    }

    @Test
    fun stringIsAJsonString() {
        assertEquals("\"abc\"", encodeString("abc"))
        assertEquals("abc", decodeString("\"abc\""))
    }
}
