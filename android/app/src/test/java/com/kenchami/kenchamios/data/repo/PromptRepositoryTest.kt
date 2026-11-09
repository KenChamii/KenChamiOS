package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class PromptRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var prompts: PromptRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        prompts = PromptRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    private suspend fun titles() = prompts.observeAll().first().map { it.title }

    @Test
    fun createTrimsAndRecordsActivity() = runBlocking<Unit> {
        prompts.create(" Code review ", "body", tags = listOf(" review ", "review"))

        val prompt = prompts.observeAll().first().single()
        assertEquals("Code review", prompt.title)
        assertEquals(listOf("review"), prompt.tags)
        assertEquals(0, prompt.useCount)
        assertEquals(listOf("prompt.created"), env.database.activityDao().kindsFor(prompt.id))
    }

    @Test
    fun createRejectsABlankTitle() {
        assertThrows(IllegalArgumentException::class.java) { runBlocking { prompts.create("  ", "x") } }
    }

    @Test
    fun copyFillsTheProjectVariablesAndCountsTheUse() = runBlocking<Unit> {
        val projects = ProjectRepository(env.database, env.stamper, env.activity)
        val project = projects.create("Sample", "A small app")
        prompts.create("Review", "Review {{ project.name }} ({{project.stage}}): {{ task.title }}", projectId = project)
        val id = prompts.observeAll().first().single().id

        val text = prompts.copy(id)

        assertEquals("Review Sample (idea): {{ task.title }}", text)
        val used = prompts.observeAll().first().single()
        assertEquals(1, used.useCount)
        assertEquals(env.clock.ms, used.lastUsedAt)
        assertEquals(listOf("prompt.created", "prompt.used"), env.database.activityDao().kindsFor(id))
    }

    @Test
    fun copyOfAGlobalPromptIsUnchanged() = runBlocking<Unit> {
        prompts.create("Explain", "Explain {{ project.name }}")
        val id = prompts.observeAll().first().single().id

        assertEquals("Explain {{ project.name }}", prompts.copy(id))
    }

    @Test
    fun favoritesComeFirstThenTheMostUsed() = runBlocking<Unit> {
        prompts.create("A", "")
        prompts.create("B", "")
        prompts.create("C", "")
        val byTitle = prompts.observeAll().first().associate { it.title to it.id }

        prompts.copy(byTitle.getValue("B"))
        prompts.copy(byTitle.getValue("B"))
        prompts.copy(byTitle.getValue("C"))
        prompts.setFavorite(byTitle.getValue("A"), true)

        assertEquals(listOf("A", "B", "C"), titles())
        val kinds = env.database.activityDao().kindsFor(byTitle.getValue("A"))
        assertEquals(listOf("prompt.created", "prompt.favorite"), kinds)
    }

    @Test
    fun deleteHidesThePrompt() = runBlocking<Unit> {
        prompts.create("A", "")
        val id = prompts.observeAll().first().single().id

        prompts.delete(id)

        assertEquals(emptyList<String>(), titles())
        assertEquals(listOf("prompt.created", "prompt.deleted"), env.database.activityDao().kindsFor(id))
    }
}
